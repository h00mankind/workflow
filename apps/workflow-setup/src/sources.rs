//! Outside sources: git repositories whose skills the library keeps as
//! verbatim copies at a pinned commit.
//!
//! `fetch` gets a source's newest commit into a cache and remembers it.
//! `diff` shows what that commit would change in `skills/<source>/`.
//! `accept` writes the copy and moves the pin. Only `accept` changes the
//! library, and it accepts the commit that `diff` showed.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{Context as _, Result, bail};
use toml_edit::DocumentMut;

pub const FILE: &str = "sources.toml";
const CACHE: &str = ".cache/sources";
/// The commit the last `fetch` found, kept in the cache repository.
const FETCHED: &str = "refs/workflow/fetched";
const LICENSE_FILES: &[&str] = &["LICENSE", "LICENSE.md", "LICENSE.txt"];
const SYMLINK_MODE: &str = "120000";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Source {
    pub name: String,
    pub repository: String,
    /// The folder in the repository that holds the skills, at any depth.
    pub path: String,
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
    let current = library.join("skills").join(&source.name);

    let mut diff = Diff { from: source.commit.clone(), to: to.clone(), missing, ..Default::default() };
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
    Ok(diff)
}

/// Replaces `skills/<source>/` with the copy at the fetched commit and moves
/// the pin to it. Returns the new pin.
pub fn accept(library: &Path, source: &Source) -> Result<String> {
    let cache = cache_repo(library, source);
    let to = fetched(&cache, source)?;
    let (next, missing) = export(library, source, &to, "next")?;
    if !missing.is_empty() {
        bail!("{} does not have: {}. Correct `skills` in {FILE}.", source.name, missing.join(", "));
    }
    let current = library.join("skills").join(&source.name);
    let previous = stage_dir(library, source, "previous");
    remove_dir(&previous)?;
    if current.is_dir() {
        fs::rename(&current, &previous).with_context(|| format!("cannot move {}", current.display()))?;
    }
    fs::create_dir_all(library.join("skills"))?;
    fs::rename(&next, &current).with_context(|| format!("cannot write {}", current.display()))?;
    set_pin(library, &source.name, &to)?;
    Ok(to)
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

/// Writes the source's skills at `commit` to a stage folder in the cache, in
/// the layout of `skills/<source>/`. Returns the folder and the named skills
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
    let missing = match &source.skills {
        Some(wanted) => {
            let wanted: BTreeSet<&String> = wanted.iter().collect();
            let missing = wanted.iter().filter(|name| !dirs.contains_key(**name)).map(|name| name.to_string()).collect();
            dirs.retain(|name, _| wanted.contains(name));
            missing
        }
        None => Vec::new(),
    };

    for (name, dir) in &dirs {
        let prefix = format!("{dir}/");
        // A link in a skill folder can point at any file on this machine.
        if let Some((_, link)) = entries.iter().find(|(mode, path)| *mode == SYMLINK_MODE && path.starts_with(&prefix)) {
            bail!("{}: {link} is a symbolic link. The library does not copy links.", source.name);
        }
        let target = out.join(name);
        fs::create_dir_all(&target)?;
        let mut archive = Command::new("git")
            .arg("--git-dir")
            .arg(&cache)
            .args(["archive", "--format=tar", &format!("{commit}:{dir}")])
            .stdout(Stdio::piped())
            .spawn()
            .context("cannot run git")?;
        let unpacked = Command::new("tar")
            .arg("-xf")
            .arg("-")
            .arg("-C")
            .arg(&target)
            .stdin(archive.stdout.take().context("git archive has no output")?)
            .status()
            .context("cannot run tar")?;
        if !archive.wait()?.success() || !unpacked.success() {
            bail!("cannot copy {dir} from {}", source.name);
        }
    }

    for license in LICENSE_FILES {
        if let Ok(text) = git(&cache, &["show", &format!("{commit}:{license}")]) {
            fs::write(out.join("LICENSE"), text + "\n")?;
            break;
        }
    }
    Ok((out, missing))
}

// ---------- compare ----------

/// Skill folders in `to` and not in `from`, in `from` and not in `to`, and
/// in both with different files.
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

/// Top-level entry name to its files' contents, by relative path.
fn tree(dir: &Path) -> BTreeMap<String, BTreeMap<String, Vec<u8>>> {
    let mut out = BTreeMap::new();
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == ".DS_Store" {
            continue;
        }
        let mut files = BTreeMap::new();
        read_files(&entry.path(), "", &mut files);
        out.insert(name, files);
    }
    out
}

fn read_files(path: &Path, rel: &str, out: &mut BTreeMap<String, Vec<u8>>) {
    if path.is_dir() {
        for entry in fs::read_dir(path).into_iter().flatten().flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name != ".DS_Store" {
                read_files(&entry.path(), &format!("{rel}/{name}"), out);
            }
        }
    } else {
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
mod tests {
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

    fn commit(repo: &Path, files: &[(&str, &str)], delete: &[&str]) {
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
    fn fixture(test: &str, skills: &str) -> (PathBuf, PathBuf) {
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
        assert_eq!(diff.added, ["LICENSE", "one", "two"]);
        assert!(diff.patch.contains("+one v1"));
        assert!(!library.join("skills").exists(), "diff wrote to the library");

        assert_eq!(accept(&library, &source).unwrap(), first);
        assert_eq!(read(&library, "skills/up/one/SKILL.md"), "one v1");
        assert_eq!(read(&library, "skills/up/two/extra.md"), "extra");
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
        assert_eq!(read(&library, "skills/up/one/SKILL.md"), "one v1", "diff wrote to the library");

        assert_eq!(accept(&library, &source).unwrap(), second);
        assert_eq!(read(&library, "skills/up/one/SKILL.md"), "one v2");
        assert!(!library.join("skills/up/two").exists());

        // A second accept changes nothing.
        let source = find(&library, "up").unwrap();
        assert!(super::diff(&library, &source).unwrap().is_empty());
        assert_eq!(accept(&library, &source).unwrap(), second);
        assert_eq!(read(&library, "skills/up/three/SKILL.md"), "three v1");
    }

    #[test]
    fn accept_takes_the_commit_that_diff_showed() {
        let (upstream, library) = fixture("pinned", "");
        let source = find(&library, "up").unwrap();
        let first = fetch(&library, &source).unwrap();
        commit(&upstream, &[("skills/a/one/SKILL.md", "one v2")], &[]);
        // Not fetched again: the newer commit must not enter.
        assert_eq!(accept(&library, &source).unwrap(), first);
        assert_eq!(read(&library, "skills/up/one/SKILL.md"), "one v1");
    }

    #[test]
    fn a_skill_list_limits_the_copy_and_reports_local_edits() {
        let (_, library) = fixture("list", "skills = [\"two\", \"gone\"]\n");
        let source = find(&library, "up").unwrap();
        fetch(&library, &source).unwrap();
        let diff = diff(&library, &source).unwrap();
        assert_eq!(diff.added, ["LICENSE", "two"]);
        assert_eq!(diff.missing, ["gone"]);
        assert!(accept(&library, &source).is_err(), "accept took a list with a missing skill");

        fs::write(library.join(FILE), read(&library, FILE).replace(", \"gone\"", "")).unwrap();
        let source = find(&library, "up").unwrap();
        accept(&library, &source).unwrap();
        assert!(!library.join("skills/up/one").exists());

        fs::write(library.join("skills/up/two/SKILL.md"), "edited here").unwrap();
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
        assert!(!library.join("skills").exists());
    }
}
