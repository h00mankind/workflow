//! Outside sources: git repositories that the library keeps as verbatim
//! copies at a pinned commit, one plugin folder per source.
//!
//! `fetch` gets a source's newest commit into a cache and remembers it.
//! `diff` shows what that commit would change in `plugins/<source>/`, and
//! remembers the commit it showed. `accept` writes the copy of that commit
//! and moves the pin. Only `accept` changes the library. It swaps the
//! folder in one rename, and on an error it puts the old copy and the old
//! pin back.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{Context as _, Result, bail};
use toml_edit::DocumentMut;

use crate::plugins;

pub const FILE: &str = "sources.toml";
const CACHE: &str = ".cache/sources";
/// The commit the last `fetch` found, kept in the cache repository.
const FETCHED: &str = "refs/workflow/fetched";
/// The commit the last `diff` showed. `accept` takes this commit.
const SHOWN: &str = "refs/workflow/shown";
const LICENSE_FILES: &[&str] = &["LICENSE", "LICENSE.md", "LICENSE.txt"];
const SYMLINK_MODE: &str = "120000";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Source {
    pub name: String,
    pub repository: String,
    /// The folder in the repository to copy from: a complete plugin, or a
    /// folder that holds skills at any depth.
    pub path: String,
    /// `path` is a complete plugin, copied as it is.
    pub plugin: bool,
    /// The branch to follow. The repository's default branch when absent.
    pub branch: Option<String>,
    /// The pin: the commit the library copy came from.
    pub commit: Option<String>,
    /// The skills to copy. Every skill under `path` when absent.
    pub skills: Option<Vec<String>>,
}

#[derive(Debug, Default)]
pub struct Diff {
    pub from: Option<String>,
    pub to: String,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub changed: Vec<String>,
    /// Skills whose library copy is not the same as the pinned commit.
    pub edited_here: Vec<String>,
    /// Skills that `sources.toml` names and the source does not have.
    pub missing: Vec<String>,
    /// Every skill of the copy at `to`.
    pub skills: Vec<String>,
    /// The source's commits from the pin to the fetched commit.
    pub log: String,
    pub patch: String,
}

impl Diff {
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.changed.is_empty() && self.from.as_deref() == Some(&self.to)
    }
}

pub fn load(library: &Path) -> Result<Vec<Source>> {
    let path = library.join(FILE);
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?;
    parse(&text)
}

#[cfg(test)]
fn find(library: &Path, name: &str) -> Result<Source> {
    load(library)?
        .into_iter()
        .find(|source| source.name == name)
        .with_context(|| format!("{FILE} has no source named {name}"))
}

fn parse(text: &str) -> Result<Vec<Source>> {
    let document: DocumentMut = text.parse().with_context(|| format!("{FILE} is not valid TOML"))?;
    let mut sources = Vec::new();
    for (name, item) in document.iter() {
        let table = item.as_table().with_context(|| format!("`{name}` in {FILE} is not a table"))?;
        let text = |key: &str| table.get(key).and_then(|value| value.as_str()).map(str::to_string);
        if !is_plain_name(name) {
            bail!("`{name}` in {FILE} is not a plain folder name");
        }
        let skills = match table.get("skills") {
            None => None,
            Some(list) => Some(
                list.as_array()
                    .with_context(|| format!("`skills` of {name} is not a list"))?
                    .iter()
                    .filter_map(|skill| skill.as_str().map(str::to_string))
                    .collect(),
            ),
        };
        sources.push(Source {
            name: name.to_string(),
            repository: text("repository").with_context(|| format!("{name} has no `repository`"))?,
            path: text("path").unwrap_or_default().trim_matches('/').to_string(),
            plugin: table.get("plugin").and_then(|value| value.as_bool()).unwrap_or(false),
            branch: text("branch"),
            commit: text("commit"),
            skills,
        });
    }
    Ok(sources)
}

/// Gets the source's newest commit into the cache. Returns that commit.
pub fn fetch(library: &Path, source: &Source) -> Result<String> {
    let cache = cache_repo(library, source);
    if cache.is_dir() {
        git(&cache, &["remote", "set-url", "origin", &source.repository])?;
        git(&cache, &["fetch", "--quiet", "--prune", "origin", "+refs/heads/*:refs/heads/*"])?;
    } else {
        fs::create_dir_all(library.join(CACHE))?;
        let status = Command::new("git")
            .args(["clone", "--quiet", "--bare", &source.repository])
            .arg(&cache)
            .status()
            .context("cannot run git")?;
        if !status.success() {
            bail!("cannot clone {}", source.repository);
        }
    }
    let newest = match &source.branch {
        Some(branch) => format!("refs/heads/{branch}"),
        None => "HEAD".to_string(),
    };
    let commit = git(&cache, &["rev-parse", "--verify", &format!("{newest}^{{commit}}")])?;
    git(&cache, &["update-ref", FETCHED, &commit])?;
    Ok(commit)
}

/// What `accept` would change in the library. Changes nothing.
pub fn diff(library: &Path, source: &Source) -> Result<Diff> {
    let cache = cache_repo(library, source);
    let to = fetched(&cache, source)?;
    let (next, missing) = export(library, source, &to, "next")?;
    let current = plugins::dir(library, &source.name);

    let skills = fs::read_dir(next.join("skills")).into_iter().flatten().flatten().map(|entry| entry.file_name().to_string_lossy().into_owned());
    let mut skills: Vec<String> = skills.filter(|name| !name.starts_with('.')).collect();
    skills.sort();
    let mut diff = Diff { from: source.commit.clone(), to: to.clone(), missing, skills, ..Default::default() };
    (diff.added, diff.removed, diff.changed) = compare(&current, &next);

    if let Some(pin) = source.commit.as_deref().filter(|pin| has_commit(&cache, pin)) {
        let (pinned, _) = export(library, source, pin, "pinned")?;
        let (added, removed, changed) = compare(&pinned, &current);
        diff.edited_here = added.into_iter().chain(removed).chain(changed).collect();
        diff.edited_here.sort();
        if pin != to {
            let range = format!("{pin}..{to}");
            let mut args = vec!["log", "--oneline", "--no-decorate", &range, "--"];
            if !source.path.is_empty() {
                args.push(&source.path);
            }
            diff.log = git(&cache, &args)?;
        }
    }

    // Before the first accept there is no copy: compare with an empty folder.
    let empty = stage_dir(library, source, "empty");
    fs::create_dir_all(&empty)?;
    let before = if current.is_dir() { &current } else { &empty };
    let patch = Command::new("git")
        .current_dir(library)
        .args(["diff", "--no-index", "--no-color", "--"])
        .arg(before.strip_prefix(library).unwrap_or(before))
        .arg(next.strip_prefix(library).unwrap_or(&next))
        .output()
        .context("cannot run git")?;
    diff.patch = String::from_utf8_lossy(&patch.stdout).into_owned();
    git(&cache, &["update-ref", SHOWN, &to])?;
    Ok(diff)
}

/// Replaces `plugins/<source>/` with the copy at the commit that the last
/// `diff` showed, and moves the pin to it. Returns the new pin. On an error
/// after the swap, the old copy and the old `sources.toml` come back.
pub fn accept(library: &Path, source: &Source) -> Result<String> {
    let cache = cache_repo(library, source);
    if !cache.is_dir() {
        bail!("{} is not fetched. Run `sources fetch {}` first.", source.name, source.name);
    }
    let to = git(&cache, &["rev-parse", "--verify", "--quiet", SHOWN]).map_err(|_| anyhow::anyhow!("run `sources diff {}` first. accept takes the commit that diff showed.", source.name))?;
    let (next, missing) = export(library, source, &to, "next")?;
    if !missing.is_empty() {
        bail!("{} does not have: {}. Correct `skills` in {FILE}.", source.name, missing.join(", "));
    }
    let current = plugins::dir(library, &source.name);
    let previous = stage_dir(library, source, "previous");
    remove_dir(&previous)?;
    let pins = fs::read_to_string(library.join(FILE)).with_context(|| format!("cannot read {FILE}"))?;
    let had_copy = current.is_dir();
    if had_copy {
        fs::rename(&current, &previous).with_context(|| format!("cannot move {}", current.display()))?;
    }
    let swap = || -> Result<()> {
        fs::create_dir_all(library.join("plugins"))?;
        fs::rename(&next, &current).with_context(|| format!("cannot write {}", current.display()))?;
        set_pin(library, &source.name, &to)?;
        plugins::sync(library)
    };
    let restore = || -> Result<()> {
        remove_dir(&current)?;
        if had_copy {
            fs::rename(&previous, &current).with_context(|| format!("cannot put {} back. The old copy is in {}.", current.display(), previous.display()))?;
        }
        if fs::read_to_string(library.join(FILE)).ok().as_deref() != Some(pins.as_str()) {
            fs::write(library.join(FILE), &pins).with_context(|| format!("cannot put {FILE} back"))?;
        }
        plugins::sync(library).ok();
        Ok(())
    };
    if let Err(error) = swap() {
        let outcome = match restore() {
            Ok(()) => "The library is as it was.".to_string(),
            Err(restore_error) => format!("The restore also failed, so the library is not as it was: {restore_error:#}"),
        };
        return Err(error.context(format!("accept of {} failed. {outcome}", source.name)));
    }
    Ok(to)
}

/// The `sources.toml` table for a new source. `spec` is a GitHub
/// `owner/repo`, a git URL, or a local path. The name is `name`, else the
/// last part of the repository.
pub fn new_source(spec: &str, name: Option<&str>, skills: &[&str], home: &Path) -> Result<Source> {
    let is_url = ["https://", "http://", "ssh://", "git@", "file://"].iter().any(|scheme| spec.starts_with(scheme));
    let repository = if is_url {
        spec.to_string()
    } else if spec.starts_with(['/', '.', '~']) {
        let path = match spec.strip_prefix("~/") {
            Some(rest) => home.join(rest),
            None => std::path::absolute(spec)?,
        };
        if !path.is_dir() {
            bail!("{} is not a folder", path.display());
        }
        path.to_string_lossy().into_owned()
    } else {
        match spec.split_once('/') {
            Some((owner, repo)) if is_plain_name(owner) && is_plain_name(repo) => format!("https://github.com/{owner}/{repo}"),
            _ => bail!("`{spec}` is not owner/repo, a git URL, or a local path"),
        }
    };
    let last = repository.trim_end_matches('/').rsplit(['/', ':']).next().unwrap_or_default().trim_end_matches(".git");
    let name = name.unwrap_or(last).to_string();
    if !is_plain_name(&name) {
        bail!("`{name}` is not a plain folder name. Give one with --name.");
    }
    let skills = (!skills.is_empty()).then(|| skills.iter().map(|skill| skill.to_string()).collect());
    Ok(Source { name, repository, path: String::new(), plugin: false, branch: None, commit: None, skills })
}

/// An error when the name of `source` belongs to another source or to a
/// plugin of the library. True when `sources.toml` has this source already.
pub fn check_new(library: &Path, source: &Source) -> Result<bool> {
    if let Some(known) = load(library)?.into_iter().find(|known| known.name == source.name) {
        if known.repository == source.repository {
            return Ok(true);
        }
        bail!("{FILE} has a source named {} from {}. Give another name with --name.", known.name, known.repository);
    }
    if plugins::dir(library, &source.name).exists() {
        bail!("the library has a plugin named {} that is not a source. Give another name with --name.", source.name);
    }
    Ok(false)
}

/// Writes `source` as a new table at the end of `sources.toml`. Comments
/// stay. A source with the same name and repository is already there, so
/// nothing changes.
pub fn write_source(library: &Path, source: &Source) -> Result<()> {
    if check_new(library, source)? {
        return Ok(());
    }
    let path = library.join(FILE);
    let text = fs::read_to_string(&path).unwrap_or_default();
    let mut document: DocumentMut = text.parse().with_context(|| format!("{FILE} is not valid TOML"))?;
    let mut table = toml_edit::Table::new();
    table["repository"] = toml_edit::value(&source.repository);
    if let Some(skills) = &source.skills {
        table["skills"] = toml_edit::value(skills.iter().collect::<toml_edit::Array>());
    }
    document.insert(&source.name, toml_edit::Item::Table(table));
    fs::write(&path, document.to_string()).with_context(|| format!("cannot write {}", path.display()))
}

// ---------- cache ----------

fn cache_repo(library: &Path, source: &Source) -> PathBuf {
    library.join(CACHE).join(format!("{}.git", source.name))
}

fn stage_dir(library: &Path, source: &Source, stage: &str) -> PathBuf {
    library.join(CACHE).join(format!("{}.{stage}", source.name))
}

fn fetched(cache: &Path, source: &Source) -> Result<String> {
    if !cache.is_dir() {
        bail!("{} is not fetched. Run `sources fetch {}` first.", source.name, source.name);
    }
    git(cache, &["rev-parse", "--verify", FETCHED]).with_context(|| format!("run `sources fetch {}` first", source.name))
}

fn has_commit(cache: &Path, commit: &str) -> bool {
    git(cache, &["cat-file", "-e", &format!("{commit}^{{commit}}")]).is_ok()
}

fn git(repo: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("--git-dir")
        .arg(repo)
        .args(args)
        .output()
        .context("cannot run git")?;
    if !output.status.success() {
        bail!("git {}: {}", args.join(" "), String::from_utf8_lossy(&output.stderr).trim());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Writes the source at `commit` to a stage folder in the cache, in the
/// layout of `plugins/<source>/`. Returns the folder and the named skills
/// that the source does not have.
fn export(library: &Path, source: &Source, commit: &str, stage: &str) -> Result<(PathBuf, Vec<String>)> {
    let cache = cache_repo(library, source);
    let out = stage_dir(library, source, stage);
    remove_dir(&out)?;
    fs::create_dir_all(&out)?;

    let mut args = vec!["ls-tree", "-r", "-z", commit, "--"];
    if !source.path.is_empty() {
        args.push(&source.path);
    }
    let listing = git(&cache, &args)?;
    // "<mode> <type> <hash>\t<path>", NUL-separated.
    let entries: Vec<(&str, &str)> = listing
        .split('\0')
        .filter_map(|entry| entry.split_once('\t'))
        .map(|(meta, path)| (meta.split(' ').next().unwrap_or_default(), path))
        .collect();
    // A link in a copied folder can point at any file on this machine.
    let refuse_links = |dir: &str| match entries.iter().find(|(mode, path)| *mode == SYMLINK_MODE && path.starts_with(&format!("{dir}/"))) {
        Some((_, link)) => bail!("{}: {link} is a symbolic link. The library does not copy links.", source.name),
        None => Ok(()),
    };

    let mut missing = Vec::new();
    if source.plugin {
        let manifest = |file: &str| entries.iter().any(|(_, path)| *path == format!("{}/{file}", source.path));
        if !manifest(plugins::CLAUDE_MANIFEST) && !manifest(plugins::CODEX_MANIFEST) {
            bail!("{}: `{}` is not a plugin. It has no plugin.json.", source.name, source.path);
        }
        refuse_links(&source.path)?;
        unpack(&cache, commit, &source.path, &out)?;
    } else {
        let mut dirs: BTreeMap<String, String> = BTreeMap::new();
        for (_, path) in &entries {
            let Some(dir) = path.strip_suffix("/SKILL.md") else { continue };
            let name = dir.rsplit('/').next().unwrap_or(dir).to_string();
            if !is_plain_name(&name) {
                bail!("{}: `{dir}` is not a plain skill folder name", source.name);
            }
            if let Some(other) = dirs.insert(name.clone(), dir.to_string()) {
                bail!("{} has two skills named {name}: {other} and {dir}", source.name);
            }
        }
        if let Some(wanted) = &source.skills {
            let wanted: BTreeSet<&String> = wanted.iter().collect();
            missing = wanted.iter().filter(|name| !dirs.contains_key(**name)).map(|name| name.to_string()).collect();
            dirs.retain(|name, _| wanted.contains(name));
        }
        for (name, dir) in &dirs {
            refuse_links(dir)?;
            unpack(&cache, commit, dir, &out.join("skills").join(name))?;
        }
        plugins::write_manifests(&out, &source.name, &format!("Skills from {}, pinned in sources.toml.", source.repository))?;
    }

    if !out.join("LICENSE").exists() {
        for license in LICENSE_FILES {
            if let Ok(text) = git(&cache, &["show", &format!("{commit}:{license}")]) {
                fs::write(out.join("LICENSE"), text + "\n")?;
                break;
            }
        }
    }
    Ok((out, missing))
}

/// Copies the folder `dir` of the repository at `commit` to `target`.
fn unpack(cache: &Path, commit: &str, dir: &str, target: &Path) -> Result<()> {
    fs::create_dir_all(target)?;
    let mut archive = Command::new("git")
        .arg("--git-dir")
        .arg(cache)
        .args(["archive", "--format=tar", &format!("{commit}:{dir}")])
        .stdout(Stdio::piped())
        .spawn()
        .context("cannot run git")?;
    let unpacked = Command::new("tar")
        .arg("-xf")
        .arg("-")
        .arg("-C")
        .arg(target)
        .stdin(archive.stdout.take().context("git archive has no output")?)
        .status()
        .context("cannot run tar")?;
    if !archive.wait()?.success() || !unpacked.success() {
        bail!("cannot copy {dir}");
    }
    Ok(())
}

// ---------- compare ----------

/// The parts in `to` and not in `from`, in `from` and not in `to`, and in
/// both with different files. A part is one skill, or one other top-level
/// entry of the plugin folder.
fn compare(from: &Path, to: &Path) -> (Vec<String>, Vec<String>, Vec<String>) {
    let (from, to) = (tree(from), tree(to));
    let added = to.keys().filter(|name| !from.contains_key(*name)).cloned().collect();
    let removed = from.keys().filter(|name| !to.contains_key(*name)).cloned().collect();
    let changed = to
        .iter()
        .filter(|(name, files)| from.get(*name).is_some_and(|other| other != *files))
        .map(|(name, _)| name.clone())
        .collect();
    (added, removed, changed)
}

/// Part name to its files' contents, by path.
fn tree(dir: &Path) -> BTreeMap<String, BTreeMap<String, Vec<u8>>> {
    let mut files = BTreeMap::new();
    read_files(dir, "", &mut files);
    let mut out: BTreeMap<String, BTreeMap<String, Vec<u8>>> = BTreeMap::new();
    for (path, bytes) in files {
        let mut parts = path.split('/');
        let part = match (parts.next(), parts.next()) {
            (Some("skills"), Some(skill)) => skill,
            (Some(top), _) => top,
            (None, _) => continue,
        };
        out.entry(part.to_string()).or_default().insert(path, bytes);
    }
    out
}

fn read_files(path: &Path, rel: &str, out: &mut BTreeMap<String, Vec<u8>>) {
    if path.is_dir() {
        for entry in fs::read_dir(path).into_iter().flatten().flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name != ".DS_Store" {
                let rel = if rel.is_empty() { name } else { format!("{rel}/{name}") };
                read_files(&entry.path(), &rel, out);
            }
        }
    } else if !rel.is_empty() {
        out.insert(rel.to_string(), fs::read(path).unwrap_or_default());
    }
}

// ---------- write ----------

fn set_pin(library: &Path, name: &str, commit: &str) -> Result<()> {
    let path = library.join(FILE);
    let text = fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?;
    let mut document: DocumentMut = text.parse().with_context(|| format!("{FILE} is not valid TOML"))?;
    document[name]["commit"] = toml_edit::value(commit);
    fs::write(&path, document.to_string()).with_context(|| format!("cannot write {}", path.display()))
}

fn remove_dir(dir: &Path) -> Result<()> {
    match fs::remove_dir_all(dir) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
            Err(error).with_context(|| format!("cannot remove {}", dir.display()))
        }
        _ => Ok(()),
    }
}

fn is_plain_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('.')
        && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    fn run(dir: &Path, args: &[&str]) {
        let status = Command::new("git")
            .current_dir(dir)
            .args(["-c", "user.name=test", "-c", "user.email=test@example.com", "-c", "commit.gpgsign=false"])
            .args(args)
            .stdout(Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?}");
    }

    pub(crate) fn commit(repo: &Path, files: &[(&str, &str)], delete: &[&str]) {
        for (path, text) in files {
            let path = repo.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
        for path in delete {
            fs::remove_dir_all(repo.join(path)).unwrap();
        }
        run(repo, &["add", "-A"]);
        run(repo, &["commit", "--quiet", "-m", "change"]);
    }

    /// A source repository and a library that names it, in a fresh folder.
    pub(crate) fn fixture(test: &str, skills: &str) -> (PathBuf, PathBuf) {
        let root = std::env::temp_dir().join(format!("workflow-setup-{test}-{}", std::process::id()));
        fs::remove_dir_all(&root).ok();
        let (upstream, library) = (root.join("upstream"), root.join("library"));
        fs::create_dir_all(&upstream).unwrap();
        fs::create_dir_all(&library).unwrap();
        run(&upstream, &["init", "--quiet", "-b", "main"]);
        commit(
            &upstream,
            &[
                ("LICENSE", "MIT"),
                ("skills/a/one/SKILL.md", "one v1"),
                ("skills/b/two/SKILL.md", "two v1"),
                ("skills/b/two/extra.md", "extra"),
            ],
            &[],
        );
        fs::write(
            library.join(FILE),
            format!("# kept\n[up]\nrepository = \"{}\"\npath = \"skills\"\n{skills}", upstream.display()),
        )
        .unwrap();
        (upstream, library)
    }

    fn read(library: &Path, path: &str) -> String {
        fs::read_to_string(library.join(path)).unwrap_or_default()
    }

    #[test]
    fn nothing_enters_the_library_before_accept() {
        let (upstream, library) = fixture("accept", "");
        let source = find(&library, "up").unwrap();
        let first = fetch(&library, &source).unwrap();

        let diff = diff(&library, &source).unwrap();
        assert_eq!(diff.added, [".claude-plugin", ".codex-plugin", "LICENSE", "one", "two"]);
        assert!(diff.patch.contains("+one v1"));
        assert!(!library.join("plugins").exists(), "diff wrote to the library");

        assert_eq!(accept(&library, &source).unwrap(), first);
        assert_eq!(read(&library, "plugins/up/skills/one/SKILL.md"), "one v1");
        assert_eq!(read(&library, "plugins/up/skills/two/extra.md"), "extra");
        let source = find(&library, "up").unwrap();
        assert_eq!(source.commit.as_deref(), Some(first.as_str()));
        assert!(read(&library, FILE).starts_with("# kept\n"), "accept dropped a comment");

        // The source changes one skill, removes one, and adds one.
        commit(
            &upstream,
            &[("skills/a/one/SKILL.md", "one v2"), ("skills/a/three/SKILL.md", "three v1")],
            &["skills/b/two"],
        );
        let second = fetch(&library, &source).unwrap();
        let diff = super::diff(&library, &source).unwrap();
        assert_eq!((diff.added, diff.removed, diff.changed), (vec!["three".to_string()], vec!["two".to_string()], vec!["one".to_string()]));
        assert_eq!(read(&library, "plugins/up/skills/one/SKILL.md"), "one v1", "diff wrote to the library");

        assert_eq!(accept(&library, &source).unwrap(), second);
        assert_eq!(read(&library, "plugins/up/skills/one/SKILL.md"), "one v2");
        assert!(!library.join("plugins/up/skills/two").exists());

        // A second accept changes nothing.
        let source = find(&library, "up").unwrap();
        assert!(super::diff(&library, &source).unwrap().is_empty());
        assert_eq!(accept(&library, &source).unwrap(), second);
        assert_eq!(read(&library, "plugins/up/skills/three/SKILL.md"), "three v1");
    }

    #[test]
    fn accept_takes_the_commit_that_diff_showed() {
        let (upstream, library) = fixture("pinned", "");
        let source = find(&library, "up").unwrap();
        let first = fetch(&library, &source).unwrap();
        assert!(accept(&library, &source).is_err(), "accept took a commit that no diff showed");
        assert_eq!(diff(&library, &source).unwrap().to, first);
        commit(&upstream, &[("skills/a/one/SKILL.md", "one v2")], &[]);
        let second = fetch(&library, &source).unwrap();
        // Fetched again but not shown: the newer commit must not enter.
        assert_eq!(accept(&library, &source).unwrap(), first);
        assert_eq!(read(&library, "plugins/up/skills/one/SKILL.md"), "one v1");
        assert_eq!(diff(&library, &source).unwrap().to, second);
        assert_eq!(accept(&library, &source).unwrap(), second);
        assert_eq!(read(&library, "plugins/up/skills/one/SKILL.md"), "one v2");
    }

    #[test]
    fn a_failed_accept_puts_the_old_copy_and_the_old_pin_back() {
        use std::os::unix::fs::PermissionsExt;
        let (upstream, library) = fixture("atomic", "");
        let source = find(&library, "up").unwrap();
        let first = fetch(&library, &source).unwrap();
        diff(&library, &source).unwrap();
        accept(&library, &source).unwrap();

        commit(&upstream, &[("skills/a/one/SKILL.md", "one v2")], &[]);
        let source = find(&library, "up").unwrap();
        fetch(&library, &source).unwrap();
        diff(&library, &source).unwrap();
        let pins = library.join(FILE);
        fs::set_permissions(&pins, fs::Permissions::from_mode(0o444)).unwrap();
        let result = accept(&library, &source);
        fs::set_permissions(&pins, fs::Permissions::from_mode(0o644)).unwrap();

        let message = format!("{:#}", result.expect_err("accept wrote a read-only sources.toml"));
        assert!(message.contains("The library is as it was.") && message.contains("ermission denied"), "{message}");
        assert_eq!(read(&library, "plugins/up/skills/one/SKILL.md"), "one v1", "the old copy did not come back");
        assert_eq!(find(&library, "up").unwrap().commit.as_deref(), Some(first.as_str()));
    }

    #[test]
    fn add_writes_a_new_source_one_time() {
        let home = Path::new("/h");
        let github = new_source("mattpocock/skills", None, &["teach"], home).unwrap();
        assert_eq!((github.name.as_str(), github.repository.as_str()), ("skills", "https://github.com/mattpocock/skills"));
        assert_eq!(github.skills, Some(vec!["teach".to_string()]));
        let url = new_source("git@github.com:owner/tools.git", Some("mine"), &[], home).unwrap();
        assert_eq!((url.name.as_str(), url.skills), ("mine", None));
        assert_eq!(new_source("https://example.com/x/kit.git", None, &[], home).unwrap().name, "kit");
        assert!(new_source("not a spec", None, &[], home).is_err());
        assert!(new_source("/no/such/folder", None, &[], home).is_err());

        let (upstream, library) = fixture("add", "");
        let local = new_source(upstream.to_str().unwrap(), Some("other"), &["one"], home).unwrap();
        write_source(&library, &local).unwrap();
        let text = read(&library, FILE);
        write_source(&library, &local).unwrap();
        assert_eq!(read(&library, FILE), text, "a second add changed sources.toml");
        assert!(text.starts_with("# kept\n") && text.contains("[other]\nrepository = "));
        assert_eq!(find(&library, "other").unwrap().skills, Some(vec!["one".to_string()]));
        assert!(write_source(&library, &Source { repository: "/elsewhere".into(), ..local }).is_err(), "a name took a second repository");

        let source = find(&library, "other").unwrap();
        fetch(&library, &source).unwrap();
        assert_eq!(diff(&library, &source).unwrap().skills, ["one"]);
    }

    #[test]
    fn a_skill_list_limits_the_copy_and_reports_local_edits() {
        let (_, library) = fixture("list", "skills = [\"two\", \"gone\"]\n");
        let source = find(&library, "up").unwrap();
        fetch(&library, &source).unwrap();
        let diff = diff(&library, &source).unwrap();
        assert_eq!(diff.added, [".claude-plugin", ".codex-plugin", "LICENSE", "two"]);
        assert_eq!(diff.missing, ["gone"]);
        assert!(accept(&library, &source).is_err(), "accept took a list with a missing skill");

        fs::write(library.join(FILE), read(&library, FILE).replace(", \"gone\"", "")).unwrap();
        let source = find(&library, "up").unwrap();
        accept(&library, &source).unwrap();
        assert!(!library.join("plugins/up/skills/one").exists());

        fs::write(library.join("plugins/up/skills/two/SKILL.md"), "edited here").unwrap();
        let source = find(&library, "up").unwrap();
        let diff = super::diff(&library, &source).unwrap();
        assert_eq!(diff.edited_here, ["two"]);
        assert_eq!(diff.changed, ["two"]);
    }

    #[test]
    fn a_symbolic_link_in_a_skill_is_refused() {
        let (upstream, library) = fixture("link", "");
        std::os::unix::fs::symlink("/etc/hosts", upstream.join("skills/a/one/hosts")).unwrap();
        commit(&upstream, &[], &[]);
        let source = find(&library, "up").unwrap();
        fetch(&library, &source).unwrap();
        assert!(diff(&library, &source).is_err());
        assert!(accept(&library, &source).is_err());
        assert!(!library.join("plugins").exists());
    }

    #[test]
    fn a_complete_plugin_is_copied_as_it_is() {
        let (upstream, library) = fixture("plugin", "");
        commit(&upstream, &[("pack/.claude-plugin/plugin.json", "{\"name\":\"up\"}"), ("pack/skills/one/SKILL.md", "one"), ("pack/agents/helper.md", "helper"), ("pack/hooks/start.sh", "#!/bin/sh")], &[]);
        fs::write(library.join(FILE), read(&library, FILE).replace("path = \"skills\"", "path = \"pack\"\nplugin = true")).unwrap();
        let source = find(&library, "up").unwrap();
        fetch(&library, &source).unwrap();
        assert_eq!(diff(&library, &source).unwrap().added, [".claude-plugin", "LICENSE", "agents", "hooks", "one"]);
        accept(&library, &source).unwrap();
        assert_eq!(read(&library, "plugins/up/.claude-plugin/plugin.json"), "{\"name\":\"up\"}", "accept wrote over the plugin's own manifest");
        assert_eq!(read(&library, "plugins/up/agents/helper.md"), "helper");
        assert!(!library.join("plugins/up/.codex-plugin").exists());

        // A folder that is not a plugin is refused.
        fs::write(library.join(FILE), read(&library, FILE).replace("path = \"pack\"", "path = \"skills\"")).unwrap();
        assert!(diff(&library, &find(&library, "up").unwrap()).is_err());
    }
}
