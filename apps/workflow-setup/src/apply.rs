//! Makes the disk match `manifest.toml`.
//!
//! `plan_all` compares the manifest to the disk and returns the drift.
//! `apply` makes a backup, then writes. It keeps a record of what it
//! installed and never changes or removes an item it does not own. A link is
//! owned when the record has it, or when it points into the library. In a
//! Codex config file, the engine owns the entries of its own marketplace.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context as _, Result, bail};
use serde::{Deserialize, Serialize};
use toml_edit::DocumentMut;

use crate::{
    adapters::{Harness, LINK_DIRS},
    library,
    manifest::{self, Scope},
    plugins::{self, MARKETPLACE},
};

const CODEX_CONFIG: &str = ".codex/config.toml";

/// The folders and tools the engine works with. Tests give it their own.
pub struct Env {
    pub library: PathBuf,
    pub home: PathBuf,
    /// Where the ownership record and the backups are kept.
    pub data: PathBuf,
    /// Runs the `codex` command line with these arguments.
    pub codex: Box<dyn Fn(&[&str]) -> Result<()>>,
}

impl Env {
    pub fn new(library: PathBuf) -> Env {
        let home = crate::scan::home();
        Env {
            library: library.canonicalize().unwrap_or(library),
            data: home.join("Library/Application Support/workflow-setup"),
            home,
            codex: Box::new(|args| {
                let output = Command::new("codex").args(args).output().context("cannot run codex")?;
                if !output.status.success() {
                    bail!("codex {}: {}", args.join(" "), String::from_utf8_lossy(&output.stderr).trim());
                }
                Ok(())
            }),
        }
    }

    fn record_file(&self) -> PathBuf {
        self.data.join("installed.json")
    }

    fn codex_cache(&self, plugin: &str) -> PathBuf {
        self.home.join(".codex/plugins/cache").join(MARKETPLACE).join(plugin)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    /// Nothing is at `path`.
    Link { path: PathBuf, target: PathBuf },
    /// An owned link is at `path` and points at `from`.
    Relink { path: PathBuf, from: PathBuf, target: PathBuf },
    /// An owned link is at `path` and the manifest does not assign it.
    Remove { path: PathBuf, from: PathBuf },
    /// Something not owned is at `path`. `apply` leaves it. `foreign_link`
    /// is its target when it is a link, which `apply` with `adopt` replaces.
    Conflict { path: PathBuf, target: PathBuf, foreign_link: Option<PathBuf> },
    /// Codex does not have this library as a marketplace.
    CodexMarketplace,
    /// The copy of the plugin that Codex keeps is missing or old.
    CodexInstall { plugin: String },
    /// Codex has a copy of a plugin that no scope assigns.
    CodexUninstall { plugin: String },
    /// A Codex config file must enable or disable the plugin, or, with
    /// `None`, must not name it.
    CodexEnable { file: PathBuf, plugin: String, enabled: Option<bool> },
}

/// The drift of one scope.
#[derive(Debug)]
pub struct Plan {
    pub scope: Scope,
    pub changes: Vec<Change>,
    /// Skill names that two assigned plugins supply where skills install
    /// without a prefix. The first plugin wins.
    pub collisions: Vec<library::Collision>,
    /// Every link the manifest assigns in this scope, with its target.
    links: BTreeMap<PathBuf, PathBuf>,
    /// True when the manifest assigns something here that is not a link.
    keeps_record: bool,
}

impl Plan {
    pub fn in_sync(&self) -> bool {
        self.changes.is_empty()
    }

    /// True when a link that something else made is in the way.
    pub fn can_adopt(&self) -> bool {
        self.changes.iter().any(|change| matches!(change, Change::Conflict { foreign_link: Some(_), .. }))
    }

    /// The changes for display: one group per kind of change and place,
    /// with the names it holds.
    pub fn groups(&self, home: &Path) -> Vec<(String, Vec<String>)> {
        let mut groups: Vec<(String, Vec<String>)> = Vec::new();
        for change in &self.changes {
            let place = |path: &Path| {
                let dir = path.parent().unwrap_or(&self.scope.root);
                dir.strip_prefix(&self.scope.root).unwrap_or(dir).display().to_string()
            };
            let name = |path: &Path| path.file_name().unwrap_or_default().to_string_lossy().into_owned();
            let from = |from: &Path| tilde(home, from.parent().unwrap_or(from));
            let (label, name) = match change {
                Change::Link { path, .. } => (format!("link     {}", place(path)), name(path)),
                Change::Relink { path, from: was, .. } => (format!("relink   {}  (now links into {})", place(path), from(was)), name(path)),
                Change::Remove { path, from: was } => (format!("remove   {}  (links into {}, not assigned)", place(path), from(was)), name(path)),
                Change::Conflict { path, foreign_link: Some(was), .. } => (
                    format!("conflict {}  (a link into {}, not made by workflow-setup; adopt replaces it)", place(path), from(was)),
                    name(path),
                ),
                Change::Conflict { path, foreign_link: None, .. } => {
                    (format!("conflict {}  (a real file or folder; move it away by hand)", place(path)), name(path))
                }
                Change::CodexMarketplace => ("codex    add this library as the marketplace".to_string(), MARKETPLACE.to_string()),
                Change::CodexInstall { plugin } => ("codex    copy to the Codex plugin cache".to_string(), plugin.clone()),
                Change::CodexUninstall { plugin } => ("codex    remove from the Codex plugin cache (not assigned)".to_string(), plugin.clone()),
                Change::CodexEnable { file, plugin, enabled } => {
                    let verb = match enabled {
                        Some(true) => "enable in",
                        Some(false) => "disable in",
                        None => "remove the entry from",
                    };
                    (format!("codex    {verb} {}", tilde(home, file)), plugin.clone())
                }
            };
            match groups.iter_mut().find(|(group, _)| *group == label) {
                Some((_, names)) => names.push(name),
                None => groups.push((label, vec![name])),
            }
        }
        groups
    }
}

/// A path as the user reads it: from `~` when it is in the home folder.
pub fn tilde(home: &Path, path: &Path) -> String {
    match path.strip_prefix(home) {
        Ok(rest) if rest.as_os_str().is_empty() => "~".to_string(),
        Ok(rest) => format!("~/{}", rest.display()),
        Err(_) => path.display().to_string(),
    }
}

/// The plugins that the manifest assigns to the scope at `root`, for one
/// harness or more.
pub fn assigned(env: &Env, root: &Path) -> Result<BTreeSet<String>> {
    let manifest = manifest::load(&env.library, &env.home)?;
    Ok(manifest
        .scopes
        .iter()
        .filter(|scope| scope.root == root)
        .flat_map(|scope| Harness::ALL.into_iter().flat_map(|harness| scope.plugins_for(harness)))
        .collect())
}

#[derive(Default, Serialize, Deserialize)]
struct Record {
    /// Scope root to the links installed there, each with its target.
    scopes: BTreeMap<PathBuf, BTreeMap<PathBuf, PathBuf>>,
    /// Plugin to the hash of its content when Codex last copied it.
    #[serde(default)]
    codex: BTreeMap<String, u64>,
}

impl Record {
    fn load(env: &Env) -> Record {
        fs::read_to_string(env.record_file())
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    fn save(&self, env: &Env) -> Result<()> {
        fs::create_dir_all(&env.data)?;
        let path = env.record_file();
        fs::write(&path, serde_json::to_string_pretty(self)?).with_context(|| format!("cannot write {}", path.display()))
    }
}

/// The drift of every scope: global scope first, then each project in the
/// manifest, then each project that left the manifest and still has
/// installed items.
pub fn plan_all(env: &Env) -> Result<Vec<Plan>> {
    let manifest = manifest::load(&env.library, &env.home)?;
    let record = Record::load(env);
    let mut scopes = vec![Scope::empty(env.home.clone(), &env.home)];
    for scope in manifest.scopes {
        match scope.global {
            true => scopes[0] = scope,
            false => scopes.push(scope),
        }
    }
    for root in record.scopes.keys() {
        if !scopes.iter().any(|scope| scope.root == *root) {
            scopes.push(Scope::empty(root.clone(), &env.home));
        }
    }
    let known = library::plugins(&env.library);
    for scope in &scopes {
        for plugin in Harness::ALL.into_iter().flat_map(|harness| scope.plugins_for(harness)) {
            if !known.contains(&plugin) {
                bail!("{}: {} names the plugin {plugin}, which the library does not have", scope.label, manifest::FILE);
            }
        }
    }
    // Codex keeps one copy of a plugin for all scopes.
    let codex_plugins: BTreeSet<String> = scopes.iter().flat_map(|scope| scope.plugins_for(Harness::Codex)).collect();
    scopes.iter().map(|scope| plan(env, &record, scope, &codex_plugins)).collect()
}

fn plan(env: &Env, record: &Record, scope: &Scope, codex_plugins: &BTreeSet<String>) -> Result<Plan> {
    let mut links: BTreeMap<PathBuf, PathBuf> = BTreeMap::new();
    let mut collisions = Vec::new();
    for &harness in &scope.harnesses {
        let assigned = scope.plugins_for(harness);
        if let Some(dir) = harness.plugin_link_dir() {
            for plugin in &assigned {
                links.insert(scope.root.join(dir).join(plugin), plugins::dir(&env.library, plugin));
            }
        }
        if let Some(dir) = harness.skill_link_dir() {
            let skills: Vec<_> = library::skills(&env.library).into_iter().filter(|skill| assigned.contains(&skill.origin)).collect();
            let (winners, collided) = library::resolve(&skills);
            for (name, item) in winners {
                links.insert(scope.root.join(dir).join(name), item.dir);
            }
            collisions.extend(collided);
        }
    }

    let owned = record.scopes.get(&scope.root);
    let is_owned = |path: &Path, link: &Path| link.starts_with(&env.library) || owned.is_some_and(|owned| owned.get(path).is_some_and(|target| target == link));

    let mut changes = Vec::new();
    for (path, target) in &links {
        let Ok(meta) = path.symlink_metadata() else {
            changes.push(Change::Link { path: path.clone(), target: target.clone() });
            continue;
        };
        let link = meta.is_symlink().then(|| fs::read_link(path).unwrap_or_default());
        match link {
            Some(link) if link == *target => {}
            Some(link) if is_owned(path, &link) => changes.push(Change::Relink { path: path.clone(), from: link, target: target.clone() }),
            foreign_link => changes.push(Change::Conflict { path: path.clone(), target: target.clone(), foreign_link }),
        }
    }
    for dir in LINK_DIRS {
        let mut entries: Vec<PathBuf> = fs::read_dir(scope.root.join(dir)).into_iter().flatten().flatten().map(|entry| entry.path()).collect();
        entries.sort();
        for path in entries {
            if links.contains_key(&path) {
                continue;
            }
            if let Ok(link) = fs::read_link(&path) {
                if is_owned(&path, &link) {
                    changes.push(Change::Remove { path, from: link });
                }
            }
        }
    }

    let assigned = scope.plugins_for(Harness::Codex);
    let file = scope.root.join(CODEX_CONFIG);
    let config = read_toml(&file)?;
    let flags = codex_flags(&config);
    if scope.global {
        let registered = config
            .get("marketplaces")
            .and_then(|all| all.get(MARKETPLACE))
            .and_then(|entry| entry.get("source"))
            .and_then(|source| source.as_str())
            .is_some_and(|source| Path::new(source) == env.library);
        if !codex_plugins.is_empty() && !registered {
            changes.push(Change::CodexMarketplace);
        }
        for plugin in codex_plugins {
            let installed = env.codex_cache(plugin).is_dir() && record.codex.get(plugin) == Some(&hash_dir(&plugins::dir(&env.library, plugin)));
            if !installed {
                changes.push(Change::CodexInstall { plugin: plugin.clone() });
            }
            // An install enables the plugin for every project.
            let enabled = Some(assigned.contains(plugin));
            if !installed || flags.get(plugin).copied() != enabled {
                changes.push(Change::CodexEnable { file: file.clone(), plugin: plugin.clone(), enabled });
            }
        }
        let stale: BTreeSet<&String> = record.codex.keys().chain(flags.keys()).filter(|plugin| !codex_plugins.contains(*plugin)).collect();
        for plugin in stale {
            match record.codex.contains_key(plugin) {
                true => changes.push(Change::CodexUninstall { plugin: plugin.clone() }),
                false => changes.push(Change::CodexEnable { file: file.clone(), plugin: plugin.clone(), enabled: None }),
            }
        }
    } else {
        for plugin in &assigned {
            if flags.get(plugin) != Some(&true) {
                changes.push(Change::CodexEnable { file: file.clone(), plugin: plugin.clone(), enabled: Some(true) });
            }
        }
        for plugin in flags.keys().filter(|plugin| !assigned.contains(plugin)) {
            changes.push(Change::CodexEnable { file: file.clone(), plugin: plugin.clone(), enabled: None });
        }
    }

    Ok(Plan { scope: scope.clone(), changes, collisions, links, keeps_record: !assigned.is_empty() })
}

/// What `apply` did in one scope.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Applied {
    pub written: usize,
    pub removed: usize,
    pub left: usize,
}

/// Makes the disk match the plans. With `adopt`, a link that something else
/// made is replaced too. A real file or folder is never replaced. Returns
/// what it did and the backup folder, if it changed the disk.
pub fn apply(env: &Env, plans: &[Plan], adopt: bool) -> Result<(Vec<Applied>, Option<PathBuf>)> {
    plugins::sync(&env.library)?;

    // (path, the link target it had) for each link that goes away, and each
    // config file that changes.
    let mut old_links: Vec<(&Path, &Path)> = Vec::new();
    let mut old_files: BTreeSet<PathBuf> = BTreeSet::new();
    for change in plans.iter().flat_map(|plan| &plan.changes) {
        match change {
            Change::Relink { path, from, .. } | Change::Remove { path, from } => old_links.push((path, from)),
            Change::Conflict { path, foreign_link: Some(from), .. } if adopt => old_links.push((path, from)),
            Change::CodexEnable { file, .. } => {
                old_files.insert(file.clone());
            }
            Change::CodexMarketplace | Change::CodexInstall { .. } | Change::CodexUninstall { .. } => {
                old_files.insert(env.home.join(CODEX_CONFIG));
            }
            Change::Link { .. } | Change::Conflict { .. } => {}
        }
    }
    old_files.retain(|file| file.is_file());
    let backup_dir = if old_links.is_empty() && old_files.is_empty() {
        None
    } else {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map(|since| since.as_millis()).unwrap_or_default();
        let dir = env.data.join("backups").join(stamp.to_string());
        fs::create_dir_all(&dir)?;
        fs::write(dir.join("links.json"), serde_json::to_string_pretty(&old_links)?).with_context(|| format!("cannot write the backup in {}", dir.display()))?;
        for file in &old_files {
            let name = file.to_string_lossy().trim_start_matches('/').replace('/', "__");
            fs::copy(file, dir.join(name)).with_context(|| format!("cannot back up {}", file.display()))?;
        }
        Some(dir)
    };

    let mut record = Record::load(env);
    let mut results = Vec::new();
    for plan in plans {
        let mut applied = Applied::default();
        // Start from what is right on disk now; a conflict takes its link out.
        let mut owned = plan.links.clone();
        for change in &plan.changes {
            match change {
                Change::Link { path, target } => {
                    link(path, target)?;
                    applied.written += 1;
                }
                Change::Relink { path, target, .. } => {
                    relink(path, target)?;
                    applied.written += 1;
                }
                Change::Conflict { path, target, foreign_link: Some(_) } if adopt => {
                    relink(path, target)?;
                    applied.written += 1;
                }
                Change::Remove { path, .. } => {
                    fs::remove_file(path).with_context(|| format!("cannot unlink {}", path.display()))?;
                    applied.removed += 1;
                }
                Change::Conflict { path, .. } => {
                    owned.remove(path);
                    applied.left += 1;
                }
                Change::CodexMarketplace => {
                    // A marketplace of this name with another source goes first.
                    (env.codex)(&["plugin", "marketplace", "remove", MARKETPLACE]).ok();
                    (env.codex)(&["plugin", "marketplace", "add", &env.library.to_string_lossy()])?;
                    applied.written += 1;
                }
                Change::CodexInstall { plugin } => {
                    (env.codex)(&["plugin", "add", &format!("{plugin}@{MARKETPLACE}")])?;
                    record.codex.insert(plugin.clone(), hash_dir(&plugins::dir(&env.library, plugin)));
                    applied.written += 1;
                }
                Change::CodexUninstall { plugin } => {
                    (env.codex)(&["plugin", "remove", &format!("{plugin}@{MARKETPLACE}")])?;
                    record.codex.remove(plugin);
                    applied.removed += 1;
                }
                Change::CodexEnable { file, plugin, enabled } => {
                    set_codex_flag(file, plugin, *enabled)?;
                    match enabled {
                        Some(_) => applied.written += 1,
                        None => applied.removed += 1,
                    }
                }
            }
        }
        if owned.is_empty() && !plan.keeps_record {
            record.scopes.remove(&plan.scope.root);
        } else {
            record.scopes.insert(plan.scope.root.clone(), owned);
        }
        results.push(applied);
    }
    record.save(env)?;
    Ok((results, backup_dir))
}

fn relink(path: &Path, target: &Path) -> Result<()> {
    fs::remove_file(path).with_context(|| format!("cannot unlink {}", path.display()))?;
    link(path, target)
}

fn link(path: &Path, target: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("cannot create {}", parent.display()))?;
    }
    std::os::unix::fs::symlink(target, path).with_context(|| format!("cannot link {}", path.display()))
}

// ---------- Codex config ----------

fn read_toml(file: &Path) -> Result<DocumentMut> {
    let text = match fs::read_to_string(file) {
        Ok(text) => text,
        Err(_) => return Ok(DocumentMut::new()),
    };
    text.parse().with_context(|| format!("{} is not valid TOML", file.display()))
}

/// Plugin to its `enabled` value, for the plugins of this library's marketplace.
fn codex_flags(config: &DocumentMut) -> BTreeMap<String, bool> {
    let mut flags = BTreeMap::new();
    let Some(all) = config.get("plugins").and_then(|plugins| plugins.as_table_like()) else { return flags };
    for (key, entry) in all.iter() {
        if let Some(plugin) = key.strip_suffix(&format!("@{MARKETPLACE}")) {
            flags.insert(plugin.to_string(), entry.get("enabled").and_then(|enabled| enabled.as_bool()).unwrap_or(true));
        }
    }
    flags
}

/// Sets `[plugins."<plugin>@workflow"] enabled` in a Codex config file, or
/// removes that table. Keeps the rest of the file as it is.
pub fn set_codex_flag(file: &Path, plugin: &str, enabled: Option<bool>) -> Result<()> {
    let mut config = read_toml(file)?;
    let key = format!("{plugin}@{MARKETPLACE}");
    match enabled {
        Some(enabled) => {
            let all = config.entry("plugins").or_insert(toml_edit::table());
            let all = all.as_table_mut().with_context(|| format!("`plugins` in {} is not a table", file.display()))?;
            all.set_implicit(true);
            all.entry(&key).or_insert(toml_edit::table())["enabled"] = toml_edit::value(enabled);
        }
        None => {
            if let Some(all) = config.get_mut("plugins").and_then(|plugins| plugins.as_table_mut()) {
                all.remove(&key);
                if all.is_empty() {
                    config.remove("plugins");
                }
            }
        }
    }
    let text = config.to_string();
    if text.trim().is_empty() {
        return match fs::remove_file(file) {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(error).with_context(|| format!("cannot remove {}", file.display())),
            _ => Ok(()),
        };
    }
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(file, text).with_context(|| format!("cannot write {}", file.display()))
}

/// A hash of every file in `dir`, by path and content.
fn hash_dir(dir: &Path) -> u64 {
    fn walk(path: &Path, files: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(path).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, files);
            } else if entry.file_name() != ".DS_Store" {
                files.push(path);
            }
        }
    }
    let mut files = Vec::new();
    walk(dir, &mut files);
    files.sort();
    let mut hasher = DefaultHasher::new();
    for file in files {
        file.strip_prefix(dir).unwrap_or(&file).hash(&mut hasher);
        fs::read(&file).unwrap_or_default().hash(&mut hasher);
    }
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFEST: &str = r#"
[global]
plugins = ["h00man"]
claude = ["pstack"]

[project."~/app"]
plugins = ["h00man", "pstack"]
harnesses = ["claude", "codex", "opencode"]
"#;

    /// A library with two plugins, a home folder, and a `codex` that does to
    /// the home folder what the real one does.
    fn fixture(test: &str) -> Env {
        let root = std::env::temp_dir().join(format!("workflow-setup-apply-{test}-{}", std::process::id()));
        fs::remove_dir_all(&root).ok();
        let (library, home) = (root.join("library"), root.join("home"));
        for skill in ["h00man/skills/plan", "h00man/skills/unslop", "pstack/skills/unslop", "pstack/skills/swarm"] {
            let dir = library.join("plugins").join(skill);
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join("SKILL.md"), skill).unwrap();
        }
        fs::create_dir_all(home.join("app")).unwrap();
        fs::write(library.join(manifest::FILE), MANIFEST).unwrap();
        let codex = {
            let (library, home) = (library.clone(), home.clone());
            move |args: &[&str]| {
                let config = home.join(CODEX_CONFIG);
                let plugin = args.last().unwrap().trim_end_matches("@workflow");
                let cache = home.join(".codex/plugins/cache/workflow").join(plugin);
                match args[..args.len() - 1] {
                    ["plugin", "marketplace", "add"] => {
                        fs::create_dir_all(config.parent().unwrap())?;
                        let text = fs::read_to_string(&config).unwrap_or_default();
                        fs::write(&config, format!("{text}[marketplaces.workflow]\nsource_type = \"local\"\nsource = \"{}\"\n", library.display()))?;
                    }
                    ["plugin", "marketplace", "remove"] => {}
                    ["plugin", "add"] => {
                        fs::create_dir_all(cache.join("0.0.0"))?;
                        set_codex_flag(&config, plugin, Some(true))?;
                    }
                    ["plugin", "remove"] => {
                        fs::remove_dir_all(cache)?;
                        set_codex_flag(&config, plugin, None)?;
                    }
                    _ => bail!("the test codex does not know {args:?}"),
                }
                Ok(())
            }
        };
        Env { library, home, data: root.join("data"), codex: Box::new(codex) }
    }

    fn run(env: &Env, adopt: bool) -> Vec<Applied> {
        apply(env, &plan_all(env).unwrap(), adopt).unwrap().0
    }

    fn target(env: &Env, path: &str) -> String {
        let link = fs::read_link(env.home.join(path)).unwrap_or_default();
        link.strip_prefix(&env.library).unwrap_or(&link).to_string_lossy().into_owned()
    }

    fn read(env: &Env, path: &str) -> String {
        fs::read_to_string(env.home.join(path)).unwrap_or_default()
    }

    fn backups(env: &Env) -> usize {
        fs::read_dir(env.data.join("backups")).map(Iterator::count).unwrap_or(0)
    }

    fn in_sync(env: &Env) -> bool {
        plan_all(env).unwrap().iter().all(Plan::in_sync)
    }

    #[test]
    fn apply_installs_the_manifest_and_a_second_run_changes_nothing() {
        let env = fixture("twice");
        run(&env, false);
        // Claude Code gets one link per plugin.
        assert_eq!(target(&env, ".claude/skills/h00man"), "plugins/h00man");
        assert_eq!(target(&env, ".claude/skills/pstack"), "plugins/pstack");
        assert_eq!(target(&env, "app/.claude/skills/pstack"), "plugins/pstack");
        // opencode gets one link per skill, and the first plugin wins a name.
        assert_eq!(target(&env, "app/.opencode/skill/unslop"), "plugins/h00man/skills/unslop");
        assert_eq!(target(&env, "app/.opencode/skill/swarm"), "plugins/pstack/skills/swarm");
        assert_eq!(plan_all(&env).unwrap()[1].collisions.len(), 1);
        // Codex has both plugins. pstack is on for the project only.
        assert!(env.home.join(".codex/plugins/cache/workflow/pstack").is_dir());
        let global = read(&env, CODEX_CONFIG);
        assert!(global.contains("[plugins.\"h00man@workflow\"]\nenabled = true"), "{global}");
        assert!(global.contains("[plugins.\"pstack@workflow\"]\nenabled = false"), "{global}");
        assert!(read(&env, "app/.codex/config.toml").contains("[plugins.\"pstack@workflow\"]\nenabled = true"));
        // The generated manifests and the marketplace are in the library.
        assert!(env.library.join("plugins/h00man/.codex-plugin/plugin.json").is_file());
        assert!(fs::read_to_string(env.library.join(".agents/plugins/marketplace.json")).unwrap().contains("./plugins/pstack"));

        assert!(in_sync(&env));
        let before = backups(&env);
        assert_eq!(run(&env, false), [Applied::default(), Applied::default()]);
        assert_eq!(backups(&env), before, "a run that changed nothing made a backup");
    }

    #[test]
    fn a_changed_plugin_is_copied_to_codex_again() {
        let env = fixture("changed");
        run(&env, false);
        fs::write(env.library.join("plugins/pstack/skills/swarm/SKILL.md"), "edited").unwrap();
        let global = &plan_all(&env).unwrap()[0];
        assert_eq!(
            global.changes,
            [
                Change::CodexInstall { plugin: "pstack".into() },
                Change::CodexEnable { file: env.home.join(CODEX_CONFIG), plugin: "pstack".into(), enabled: Some(false) },
            ]
        );
        run(&env, false);
        assert!(read(&env, CODEX_CONFIG).contains("[plugins.\"pstack@workflow\"]\nenabled = false"));
        assert!(in_sync(&env));
    }

    #[test]
    fn apply_removes_only_what_it_owns() {
        let env = fixture("owned");
        let skills = env.home.join(".claude/skills");
        fs::create_dir_all(skills.join("by-hand")).unwrap();
        std::os::unix::fs::symlink("/somewhere/else", skills.join("from-a-plugin")).unwrap();
        fs::create_dir_all(env.home.join(".codex")).unwrap();
        fs::write(env.home.join(CODEX_CONFIG), "model = \"x\"\n\n[plugins.\"other@elsewhere\"]\nenabled = true\n").unwrap();
        run(&env, false);

        // The manifest stops assigning pstack and the project.
        fs::write(env.library.join(manifest::FILE), "[global]\nplugins = [\"h00man\"]\n").unwrap();
        run(&env, false);
        assert!(skills.join("h00man").exists());
        assert!(skills.join("pstack").symlink_metadata().is_err());
        assert!(skills.join("by-hand").is_dir(), "apply removed a folder it does not own");
        assert!(skills.join("from-a-plugin").symlink_metadata().is_ok(), "apply removed a link it does not own");
        assert!(env.home.join("app/.opencode/skill/plan").symlink_metadata().is_err());
        assert!(!env.home.join("app/.codex/config.toml").exists());
        assert!(!env.home.join(".codex/plugins/cache/workflow/pstack").exists());
        let global = read(&env, CODEX_CONFIG);
        assert!(global.contains("model = \"x\"") && global.contains("other@elsewhere") && !global.contains("pstack@workflow"), "{global}");
        assert!(in_sync(&env));
    }

    #[test]
    fn a_link_into_the_library_is_owned_and_a_foreign_item_is_a_conflict() {
        let env = fixture("conflict");
        let skills = env.home.join(".claude/skills");
        fs::create_dir_all(skills.join("h00man")).unwrap();
        // A link from an older layout: into the library, and broken.
        std::os::unix::fs::symlink(env.library.join("skills/poteto/swarm"), skills.join("swarm")).unwrap();
        std::os::unix::fs::symlink("/a/plugin/pstack", skills.join("pstack")).unwrap();

        let global = &plan_all(&env).unwrap()[0];
        let conflicts: Vec<_> = global
            .changes
            .iter()
            .filter_map(|change| match change {
                Change::Conflict { path, .. } => Some(path.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(conflicts, [skills.join("h00man"), skills.join("pstack")]);

        run(&env, false);
        assert!(skills.join("swarm").symlink_metadata().is_err(), "the old link into the library stayed");
        assert_eq!(target(&env, ".claude/skills/pstack"), "/a/plugin/pstack");

        // With adopt, the foreign link is replaced. The real folder stays.
        run(&env, true);
        assert_eq!(target(&env, ".claude/skills/pstack"), "plugins/pstack");
        assert!(skills.join("h00man").is_dir() && !skills.join("h00man").is_symlink());
        let mut all: Vec<_> = fs::read_dir(env.data.join("backups")).unwrap().flatten().map(|entry| entry.path()).collect();
        all.sort();
        assert!(fs::read_to_string(all.pop().unwrap().join("links.json")).unwrap().contains("/a/plugin/pstack"));
    }

    #[test]
    fn a_plugin_that_the_library_does_not_have_is_an_error() {
        let env = fixture("missing");
        fs::write(env.library.join(manifest::FILE), "[global]\nplugins = [\"nope\"]\n").unwrap();
        assert!(plan_all(&env).is_err());
        assert!(!env.home.join(".claude").exists());
    }
}
