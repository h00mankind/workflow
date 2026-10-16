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
    adapters::mcp::Format as McpFormat,
    agents::{self, Agent, Kind},
    library,
    manifest::{self, Entry, Scope},
    mcp,
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
    /// Runs the `claude` command line with these arguments.
    pub claude: Box<dyn Fn(&[&str]) -> Result<()>>,
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
            claude: Box::new(|args| {
                let output = Command::new("claude").args(args).output().context("cannot run claude")?;
                if !output.status.success() {
                    bail!("claude mcp failed: {}", String::from_utf8_lossy(&output.stderr).trim());
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
    /// Writes an MCP entry, or removes it with `None`.
    McpSet { file: PathBuf, format: McpFormat, server: String, entry: Option<serde_json::Value>, previous: Option<u64> },
    /// An MCP entry or file that the engine must leave unchanged.
    McpConflict { file: PathBuf, server: String, reason: String },
}

/// The drift of one scope.
#[derive(Debug)]
pub struct Plan {
    pub scope: Scope,
    pub changes: Vec<Change>,
    /// Link paths that two assigned plugins want with different targets.
    /// The first plugin in the library's order wins.
    pub clashes: Vec<Clash>,
    /// Skills that a plugin agent of the scope gets twice.
    pub doubles: Vec<Double>,
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
                Change::McpSet { file, server, entry, .. } => {
                    let verb = if entry.is_some() { "write in" } else { "remove from" };
                    (format!("mcp      {verb} {}", tilde(home, file)), server.clone())
                }
                Change::McpConflict { file, server, reason } => (format!("conflict {}  ({reason})", tilde(home, file)), server.clone()),
            };
            match groups.iter_mut().find(|(group, _)| *group == label) {
                Some((_, names)) => names.push(name),
                None => groups.push((label, vec![name])),
            }
        }
        groups
    }
}

/// Two or more plugins want the same link path with different targets.
/// Only the winner gets the path.
#[derive(Debug, PartialEq, Eq)]
pub struct Clash {
    pub path: PathBuf,
    pub winner: String,
    pub losers: Vec<String>,
}

/// A Skills agent links the skills of a plugin into a folder that a plugin
/// agent also reads, and the plugin agent has that plugin whole. The plugin
/// agent sees each skill twice: `<plugin>:<skill>` and `<skill>`.
#[derive(Debug, PartialEq, Eq)]
pub struct Double {
    /// The plugin agent, by its manifest key.
    pub reader: &'static str,
    /// The Skills agents that make the links, by their manifest keys.
    pub linkers: Vec<&'static str>,
    pub dir: PathBuf,
    pub plugin: String,
    pub skills: Vec<String>,
}

impl Double {
    /// The warning for `status`. `dir` is the folder as the user reads it.
    pub fn message(&self, dir: &str) -> String {
        let (reader, plugin, linkers) = (self.reader, &self.plugin, self.linkers.join(" and "));
        match self.skills.as_slice() {
            [skill] => format!("{reader} also reads {dir}, so it gets {skill} twice: {plugin}:{skill} and {skill}. {linkers} links it there."),
            skills => format!(
                "{reader} also reads {dir}, so it gets {} skills of {plugin} twice, as {plugin}:<skill> and <skill>. {linkers} links them there: {}.",
                skills.len(),
                skills.join(" ")
            ),
        }
    }
}

/// The doubles of a scope. The `project` and `global` folders of a plugin
/// agent in the agent table are the folders where it reads plain skills.
fn doubles(library: &Path, scope: &Scope) -> Vec<Double> {
    let skills = library::skills(library);
    let dir_of = |agent: &Agent| agent.skill_dir(&scope.root, scope.global);
    let mut doubles: Vec<Double> = Vec::new();
    for &reader in scope.agents.iter().filter(|agent| agent.kind != Kind::Skills) {
        let Some(dir) = dir_of(reader) else { continue };
        let whole = scope.plugins_for(reader);
        for &linker in scope.agents.iter().filter(|agent| agent.kind == Kind::Skills && dir_of(agent).as_ref() == Some(&dir)) {
            for entry in scope.entries_for(linker).into_iter().filter(|entry| whole.iter().any(|plugin| plugin == entry.plugin())) {
                let names: Vec<String> = match &entry {
                    Entry::Skill { skill, .. } => vec![skill.clone()],
                    Entry::Plugin(plugin) => skills.iter().filter(|skill| skill.origin == *plugin).map(|skill| skill.name.clone()).collect(),
                };
                let at = match doubles.iter().position(|double| double.reader == reader.key() && double.plugin == entry.plugin()) {
                    Some(at) => at,
                    None => {
                        let double = Double { reader: reader.key(), linkers: Vec::new(), dir: dir.clone(), plugin: entry.plugin().to_string(), skills: Vec::new() };
                        doubles.push(double);
                        doubles.len() - 1
                    }
                };
                let double = &mut doubles[at];
                if !double.linkers.contains(&linker.key()) {
                    double.linkers.push(linker.key());
                }
                for name in names {
                    if !double.skills.contains(&name) {
                        double.skills.push(name);
                    }
                }
            }
        }
    }
    doubles
}

/// One link that the manifest wants, before clashes are resolved.
struct Want {
    /// The plugin's place in the library's order of precedence.
    rank: usize,
    plugin: String,
    target: PathBuf,
}

/// Every link that the manifest assigns in `scope`, keyed by its path. Two
/// agents that read the same folder share one link. Where two plugins want
/// the same path, the first in the library's order wins and the rest are
/// clashes. The kind of each agent decides what a whole plugin becomes.
fn desired_links(library: &Path, scope: &Scope) -> (BTreeMap<PathBuf, PathBuf>, Vec<Clash>) {
    let order = library::plugins(library);
    let skills = library::skills(library);
    let mut wants: BTreeMap<PathBuf, Vec<Want>> = BTreeMap::new();
    for &agent in &scope.agents {
        let Some(dir) = agent.skill_dir(&scope.root, scope.global) else { continue };
        for entry in scope.entries_for(agent) {
            let plugin = entry.plugin();
            let rank = order.iter().position(|known| known == plugin).unwrap_or(usize::MAX);
            let mut want = |name: &str, target: PathBuf| wants.entry(dir.join(name)).or_default().push(Want { rank, plugin: plugin.to_string(), target });
            match (agent.kind, &entry) {
                (_, Entry::Skill { skill, .. }) => want(skill, plugins::dir(library, plugin).join("skills").join(skill)),
                (Kind::ClaudePlugin, Entry::Plugin(_)) => want(plugin, plugins::dir(library, plugin)),
                (Kind::CodexPlugin, Entry::Plugin(_)) => {}
                (Kind::Skills, Entry::Plugin(_)) => {
                    for skill in skills.iter().filter(|skill| skill.origin == plugin) {
                        want(&skill.name, skill.dir.clone());
                    }
                }
            }
        }
    }
    let mut links = BTreeMap::new();
    let mut clashes = Vec::new();
    for (path, mut candidates) in wants {
        candidates.sort_by_key(|want| want.rank);
        let winner = &candidates[0];
        let mut losers: Vec<String> = Vec::new();
        for loser in candidates.iter().filter(|want| want.target != winner.target) {
            if !losers.contains(&loser.plugin) {
                losers.push(loser.plugin.clone());
            }
        }
        if !losers.is_empty() {
            clashes.push(Clash { path: path.clone(), winner: winner.plugin.clone(), losers });
        }
        links.insert(path, winner.target.clone());
    }
    (links, clashes)
}

/// Every folder in a scope where the engine makes links or made them before.
fn link_dirs(scope: &Scope) -> BTreeSet<PathBuf> {
    let table = agents::all().iter().filter_map(|agent| agent.skill_dir(&scope.root, scope.global));
    table.chain(agents::LEGACY_DIRS.iter().map(|dir| scope.root.join(dir))).collect()
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
/// agent or more.
pub fn assigned(env: &Env, root: &Path) -> Result<BTreeSet<String>> {
    let manifest = manifest::load(&env.library, &env.home)?;
    Ok(manifest
        .scopes
        .iter()
        .filter(|scope| scope.root == root)
        .flat_map(|scope| scope.agents.iter().flat_map(|agent| scope.plugins_for(agent)))
        .collect())
}

#[derive(Default, Serialize, Deserialize)]
pub struct Record {
    /// Scope root to the links installed there, each with its target.
    scopes: BTreeMap<PathBuf, BTreeMap<PathBuf, PathBuf>>,
    /// Plugin to the hash of its content when Codex last copied it.
    #[serde(default)]
    codex: BTreeMap<String, u64>,
    /// Config file to the MCP entries written there, each with its value hash.
    #[serde(default)]
    mcp: BTreeMap<PathBuf, BTreeMap<String, u64>>,
}

impl Record {
    /// True when the engine owns the link at `path` in the scope at `root`:
    /// the link points into the library, or the record has it.
    pub fn owns(&self, env: &Env, root: &Path, path: &Path, link: &Path) -> bool {
        link.starts_with(&env.library) || self.scopes.get(root).is_some_and(|owned| owned.get(path).is_some_and(|target| target == link))
    }

    pub fn load(env: &Env) -> Record {
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
    plan_manifest(env, manifest::load(&env.library, &env.home)?)
}

/// `plan_all` for a manifest that is not on the disk yet, such as the
/// result of an edit in a dry run.
pub fn plan_manifest(env: &Env, manifest: manifest::Manifest) -> Result<Vec<Plan>> {
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
    let servers = mcp::load(&env.library)?;
    for scope in &scopes {
        for entry in scope.agents.iter().flat_map(|agent| scope.entries_for(agent)) {
            check_entry(&env.library, &entry).with_context(|| format!("{}: {}", scope.label, manifest::FILE))?;
        }
        for server in &scope.mcp {
            if !servers.contains_key(server) {
                bail!("{}: {} names the MCP server {server}, which the library does not have", scope.label, manifest::FILE);
            }
        }
    }
    // Codex keeps one copy of a plugin for all scopes.
    let codex = agents::of_kind(Kind::CodexPlugin);
    let codex_plugins: BTreeSet<String> = scopes.iter().flat_map(|scope| scope.plugins_for(codex)).collect();
    scopes.iter().map(|scope| {
        let mut plan = plan(env, &record, scope, &codex_plugins)?;
        plan_mcp(&record, &servers, &mut plan)?;
        Ok(plan)
    }).collect()
}

/// An error when the library does not have the plugin or the skill.
pub fn check_entry(library: &Path, entry: &Entry) -> Result<()> {
    let plugin = entry.plugin();
    if !library::plugins(library).iter().any(|known| known == plugin) {
        bail!("the library has no plugin {plugin}");
    }
    if let Entry::Skill { skill, .. } = entry {
        if !library::skills(library).iter().any(|item| item.origin == plugin && item.name == *skill) {
            bail!("the plugin {plugin} has no skill {skill}");
        }
    }
    Ok(())
}

fn plan_mcp(record: &Record, servers: &BTreeMap<String, mcp::Server>, plan: &mut Plan) -> Result<()> {
    for config in crate::adapters::mcp::configs(&plan.scope) {
        let desired: BTreeSet<String> = if config.assigned { plan.scope.mcp.iter().cloned().collect() } else { BTreeSet::new() };
        let owned = record.mcp.get(&config.file).cloned().unwrap_or_default();
        let names: BTreeSet<String> = desired.iter().chain(owned.keys()).cloned().collect();
        if names.is_empty() {
            continue;
        }
        plan.keeps_record |= !desired.is_empty();
        let (entries, blocked) = config.format.read(&config.file)?;
        for server in names {
            let previous = entries.get(&server).map(hash_entry);
            let reason = match &blocked {
                Some(reason) => Some(reason.clone()),
                None if previous.is_some() && !owned.contains_key(&server) => Some("the entry is not owned".to_string()),
                None if previous.is_some() && previous != owned.get(&server).copied() => Some("the owned entry was edited outside workflow-setup".to_string()),
                None => None,
            };
            if let Some(reason) = reason {
                plan.changes.push(Change::McpConflict { file: config.file.clone(), server, reason });
                continue;
            }
            let entry = desired.contains(&server).then(|| config.format.entry(&servers[&server]));
            if entry.as_ref().map(hash_entry) != previous || (entry.is_none() && owned.contains_key(&server)) {
                plan.changes.push(Change::McpSet { file: config.file.clone(), format: config.format, server, entry, previous });
            }
        }
    }
    Ok(())
}

fn hash_entry(entry: &serde_json::Value) -> u64 {
    fn sorted(value: &serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Object(values) => {
                let values: BTreeMap<_, _> = values.iter().map(|(key, value)| (key.clone(), sorted(value))).collect();
                serde_json::Value::Object(values.into_iter().collect())
            }
            serde_json::Value::Array(values) => serde_json::Value::Array(values.iter().map(sorted).collect()),
            _ => value.clone(),
        }
    }
    let mut hasher = DefaultHasher::new();
    sorted(entry).to_string().hash(&mut hasher);
    hasher.finish()
}

fn plan(env: &Env, record: &Record, scope: &Scope, codex_plugins: &BTreeSet<String>) -> Result<Plan> {
    let (links, clashes) = desired_links(&env.library, scope);

    let is_owned = |path: &Path, link: &Path| record.owns(env, &scope.root, path, link);

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
    for dir in link_dirs(scope) {
        let mut entries: Vec<PathBuf> = fs::read_dir(dir).into_iter().flatten().flatten().map(|entry| entry.path()).collect();
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

    let assigned = scope.plugins_for(agents::of_kind(Kind::CodexPlugin));
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

    Ok(Plan { scope: scope.clone(), changes, clashes, doubles: doubles(&env.library, scope), links, keeps_record: !assigned.is_empty() })
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
            Change::McpSet { file, entry, previous, .. } if entry.is_some() || previous.is_some() => {
                old_files.insert(file.clone());
            }
            Change::CodexMarketplace | Change::CodexInstall { .. } | Change::CodexUninstall { .. } => {
                old_files.insert(env.home.join(CODEX_CONFIG));
            }
            Change::Link { .. } | Change::Conflict { .. } | Change::McpConflict { .. } | Change::McpSet { .. } => {}
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
                Change::McpConflict { .. } => applied.left += 1,
                Change::McpSet { file, format, server, entry, previous } => {
                    let (entries, blocked) = format.read(file)?;
                    if blocked.is_some() || entries.get(server).map(hash_entry) != *previous {
                        applied.left += 1;
                        continue;
                    }
                    format.set(file, server, entry.as_ref(), previous.is_some(), env.claude.as_ref())?;
                    let (entries, _) = format.read(file)?;
                    match entry {
                        Some(entry) => {
                            let hash = hash_entry(entry);
                            if entries.get(server).map(hash_entry) != Some(hash) {
                                bail!("MCP server {server} was not written in {}", file.display());
                            }
                            record.mcp.entry(file.clone()).or_default().insert(server.clone(), hash);
                            applied.written += 1;
                        }
                        None => {
                            if entries.contains_key(server) {
                                bail!("MCP server {server} was not removed from {}", file.display());
                            }
                            if let Some(owned) = record.mcp.get_mut(file) {
                                owned.remove(server);
                                if owned.is_empty() {
                                    record.mcp.remove(file);
                                }
                            }
                            applied.removed += usize::from(previous.is_some());
                        }
                    }
                }
            }
        }
        let owns_mcp = crate::adapters::mcp::configs(&plan.scope).iter().any(|config| record.mcp.contains_key(&config.file));
        if owned.is_empty() && !plan.keeps_record && !owns_mcp {
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

/// The plugins of this library that the Codex config of the scope at `root`
/// names, each with its `enabled` value.
pub fn codex_plugins(root: &Path) -> Result<BTreeMap<String, bool>> {
    Ok(codex_flags(&read_toml(&root.join(CODEX_CONFIG))?))
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
pub(crate) mod tests {
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
    pub(crate) fn fixture(test: &str) -> Env {
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
        Env { library, home, data: root.join("data"), codex: Box::new(codex), claude: Box::new(|args| bail!("unexpected claude call: {args:?}")) }
    }

    pub(crate) fn run(env: &Env, adopt: bool) -> Vec<Applied> {
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
        // opencode gets one link per skill in .opencode/skills, and the first plugin wins a name.
        assert_eq!(target(&env, "app/.opencode/skills/unslop"), "plugins/h00man/skills/unslop");
        assert_eq!(target(&env, "app/.opencode/skills/swarm"), "plugins/pstack/skills/swarm");
        assert_eq!(plan_all(&env).unwrap()[1].clashes.len(), 1);
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
        assert!(env.home.join("app/.agents/skills/plan").symlink_metadata().is_err());
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

    #[test]
    fn old_opencode_links_move_to_the_folder_that_the_agent_table_gives() {
        let env = fixture("migrate");
        let old = env.home.join("app/.opencode/skill");
        fs::create_dir_all(&old).unwrap();
        for skill in ["plan", "swarm"] {
            let origin = if skill == "plan" { "h00man" } else { "pstack" };
            std::os::unix::fs::symlink(env.library.join("plugins").join(origin).join("skills").join(skill), old.join(skill)).unwrap();
        }
        let removes = plan_all(&env).unwrap()[1].changes.iter().filter(|change| matches!(change, Change::Remove { path, .. } if path.starts_with(&old))).count();
        assert_eq!(removes, 2);
        run(&env, false);
        assert_eq!(fs::read_dir(&old).unwrap().count(), 0, "an old opencode link stayed");
        assert_eq!(target(&env, "app/.opencode/skills/plan"), "plugins/h00man/skills/plan");
        assert_eq!(target(&env, "app/.opencode/skills/swarm"), "plugins/pstack/skills/swarm");
        assert!(!env.home.join("app/.agents").exists(), "Codex reads .agents/skills and would get each skill twice");
        assert!(in_sync(&env));
    }

    #[test]
    fn agents_that_share_a_folder_share_one_link_and_a_clash_is_reported_once() {
        let env = fixture("shared");
        fs::write(env.library.join(manifest::FILE), "[project.\"~/app\"]\nplugins = [\"pstack\", \"h00man\"]\nharnesses = [\"cline\", \"cursor\", \"amp\", \"zed\"]\n").unwrap();
        let plan = &plan_all(&env).unwrap()[1];
        let dir = env.home.join("app/.agents/skills");
        let mut linked: Vec<_> = plan.changes.iter().filter_map(|change| match change {
            Change::Link { path, .. } => Some(path.strip_prefix(&dir).unwrap().to_string_lossy().into_owned()),
            _ => None,
        }).collect();
        linked.sort();
        assert_eq!(linked, ["plan", "swarm", "unslop"]);
        assert_eq!(plan.clashes, [Clash { path: dir.join("unslop"), winner: "h00man".into(), losers: vec!["pstack".into()] }]);
        run(&env, false);
        assert_eq!(target(&env, "app/.agents/skills/unslop"), "plugins/h00man/skills/unslop", "the clash took the wrong winner");
        assert!(in_sync(&env));
    }

    #[test]
    fn a_skills_agent_that_links_into_a_folder_codex_reads_is_a_warning() {
        let env = fixture("doubles");
        fs::write(env.library.join(manifest::FILE), "[project.\"~/app\"]\nplugins = [\"h00man\"]\nharnesses = [\"codex\", \"cursor\"]\n").unwrap();
        let plan = &plan_all(&env).unwrap()[1];
        let dir = env.home.join("app/.agents/skills");
        let double = Double { reader: "codex", linkers: vec!["cursor"], dir: dir.clone(), plugin: "h00man".into(), skills: vec!["plan".into(), "unslop".into()] };
        assert_eq!(plan.doubles, [double]);
        assert!(plan.changes.contains(&Change::Link { path: dir.join("plan"), target: env.library.join("plugins/h00man/skills/plan") }), "the warning dropped a link");

        fs::write(env.library.join(manifest::FILE), "[project.\"~/app\"]\ncodex = [\"h00man\"]\ncursor = [\"h00man/plan\"]\nharnesses = [\"codex\", \"cursor\"]\n").unwrap();
        let message = plan_all(&env).unwrap()[1].doubles.iter().map(|double| double.message(".agents/skills")).collect::<Vec<_>>();
        assert_eq!(message, ["codex also reads .agents/skills, so it gets plan twice: h00man:plan and plan. cursor links it there."]);

        fs::write(env.library.join(manifest::FILE), "[project.\"~/app\"]\nplugins = [\"h00man\"]\nharnesses = [\"codex\", \"opencode\"]\n").unwrap();
        assert!(plan_all(&env).unwrap()[1].doubles.is_empty());
    }

    #[test]
    fn a_single_skill_is_one_link_without_a_prefix_for_every_kind() {
        let env = fixture("single");
        fs::write(env.library.join(manifest::FILE), "[global]\nplugins = [\"pstack/swarm\"]\n[project.\"~/app\"]\nplugins = [\"h00man/plan\"]\nharnesses = [\"claude\", \"codex\", \"cursor\"]\n").unwrap();
        run(&env, false);
        assert_eq!(target(&env, ".claude/skills/swarm"), "plugins/pstack/skills/swarm");
        assert_eq!(target(&env, ".agents/skills/swarm"), "plugins/pstack/skills/swarm", "Codex did not get the skill in its global folder");
        assert_eq!(target(&env, "app/.claude/skills/plan"), "plugins/h00man/skills/plan");
        assert_eq!(target(&env, "app/.agents/skills/plan"), "plugins/h00man/skills/plan");
        assert!(env.home.join(".claude/skills/pstack").symlink_metadata().is_err(), "a single skill installed its whole plugin");
        assert!(!env.home.join(".codex/plugins/cache/workflow/pstack").exists(), "a single skill installed its plugin in Codex");
        assert!(!read(&env, CODEX_CONFIG).contains("pstack@workflow"));
        assert!(in_sync(&env));

        for bad in ["h00man/nope", "nope/plan"] {
            fs::write(env.library.join(manifest::FILE), format!("[global]\nplugins = [\"{bad}\"]\n")).unwrap();
            assert!(plan_all(&env).is_err(), "{bad} passed");
        }
    }

    #[test]
    fn the_repo_manifest_parses_and_is_in_sync_after_one_apply() {
        let env = fixture("repo-manifest");
        let dir = env.library.join("plugins/matt/skills/teach");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("SKILL.md"), "teach").unwrap();
        fs::write(env.library.join(manifest::FILE), include_str!("../../../manifest.toml")).unwrap();
        run(&env, false);
        assert_eq!(target(&env, "Documents/h00man.code/workflow/.opencode/skills/teach"), "plugins/matt/skills/teach");
        assert!(!env.home.join("Documents/h00man.code/workflow/.agents").exists());
        assert!(plan_all(&env).unwrap().iter().all(|plan| plan.doubles.is_empty()));
        assert_eq!(target(&env, ".claude/skills/pstack"), "plugins/pstack");
        assert!(in_sync(&env));
    }

    fn mcp_fixture(test: &str) -> Env {
        let mut env = fixture(test);
        fs::create_dir_all(env.library.join("mcp")).unwrap();
        fs::write(env.library.join("mcp/local.toml"), "command = \"npx\"\nargs = [\"-y\", \"some-mcp\"]\nenv = [\"GITHUB_TOKEN\"]\n").unwrap();
        fs::write(env.library.join("mcp/remote.toml"), "url = \"https://example.com/mcp\"\nbearer_token_env = \"EXAMPLE_TOKEN\"\n").unwrap();
        fs::write(env.library.join(manifest::FILE), "[global]\nmcp = [\"local\", \"remote\"]\n[project.\"~/app\"]\nmcp = [\"local\", \"remote\"]\nharnesses = [\"claude\", \"codex\", \"opencode\"]\n").unwrap();
        let file = env.home.join(".claude.json");
        env.claude = Box::new(move |args| {
            let mut doc: serde_json::Value = serde_json::from_str(&fs::read_to_string(&file).unwrap_or_else(|_| "{\"mcpServers\":{}}".into()))?;
            match args {
                ["mcp", "add-json", "--scope", "user", name, json] => {
                    assert!(doc["mcpServers"].get(*name).is_none(), "claude add-json cannot replace an existing server");
                    let mut entry: serde_json::Value = serde_json::from_str(json)?;
                    if entry.get("command").is_some() {
                        entry["type"] = serde_json::json!("stdio");
                    }
                    doc["mcpServers"][*name] = entry;
                }
                ["mcp", "remove", "--scope", "user", name] => {
                    assert!(doc["mcpServers"].as_object_mut().unwrap().remove(*name).is_some());
                }
                _ => bail!("the test claude does not know {args:?}"),
            }
            fs::write(&file, serde_json::to_string_pretty(&doc)?)?;
            Ok(())
        });
        env
    }

    fn mcp_entries(env: &Env, path: &str, format: McpFormat) -> BTreeMap<String, serde_json::Value> {
        format.read(&env.home.join(path)).unwrap().0
    }

    const MCP_FILES: [(&str, McpFormat); 5] = [
        (".claude.json", McpFormat::ClaudeUser),
        (CODEX_CONFIG, McpFormat::Codex),
        ("app/.mcp.json", McpFormat::Json),
        ("app/.codex/config.toml", McpFormat::Codex),
        ("app/.agents/mcp.json", McpFormat::Json),
    ];

    #[test]
    fn mcp_installs_for_both_harnesses_in_both_scopes_and_is_idempotent() {
        let env = mcp_fixture("mcp-twice");
        let plans = plan_all(&env).unwrap();
        assert_eq!(plans.iter().map(|plan| plan.changes.len()).collect::<Vec<_>>(), [4, 6]);
        assert!(plans[0].groups(&env.home).iter().any(|(label, names)| label.contains(".claude.json") && names == &["local", "remote"]));
        assert_eq!(run(&env, false), [Applied { written: 4, ..Applied::default() }, Applied { written: 6, ..Applied::default() }]);
        for (file, format) in MCP_FILES {
            let entries = mcp_entries(&env, file, format);
            assert_eq!(entries.len(), 2, "{file}");
            if format == McpFormat::Codex {
                assert_eq!(entries["local"], serde_json::json!({"command": "npx", "args": ["-y", "some-mcp"], "env_vars": ["GITHUB_TOKEN"]}));
                assert_eq!(entries["remote"], serde_json::json!({"url": "https://example.com/mcp", "bearer_token_env_var": "EXAMPLE_TOKEN"}));
            } else {
                assert_eq!(entries["local"], serde_json::json!({"command": "npx", "args": ["-y", "some-mcp"], "env": {"GITHUB_TOKEN": "${GITHUB_TOKEN}"}}));
                assert_eq!(entries["remote"], serde_json::json!({"type": "http", "url": "https://example.com/mcp", "headers": {"Authorization": "Bearer ${EXAMPLE_TOKEN}"}}));
            }
            let record = Record::load(&env);
            assert_eq!(record.mcp[&env.home.join(file)]["local"], hash_entry(&entries["local"]));
        }
        assert!(!env.home.join(".agents/mcp.json").exists());
        assert!(!env.home.join("app/.opencode").exists());
        assert!(in_sync(&env));
        let before = backups(&env);
        let (applied, backup) = apply(&env, &plan_all(&env).unwrap(), false).unwrap();
        assert_eq!(applied, [Applied::default(), Applied::default()]);
        assert_eq!(backup, None);
        assert_eq!(backups(&env), before);
    }

    #[test]
    fn mcp_updates_owned_entries_and_removes_servers_and_scopes_that_leave_the_manifest() {
        let env = mcp_fixture("mcp-remove");
        run(&env, false);
        fs::write(env.library.join("mcp/local.toml"), "command = \"new-command\"\n").unwrap();
        assert_eq!(run(&env, false).iter().map(|applied| applied.written).sum::<usize>(), 5);
        for (file, format) in MCP_FILES {
            assert_eq!(mcp_entries(&env, file, format)["local"]["command"], "new-command");
        }
        fs::write(env.library.join(manifest::FILE), "[global]\nmcp = [\"local\"]\n").unwrap();
        assert_eq!(run(&env, false).iter().map(|applied| applied.removed).sum::<usize>(), 8);
        for (file, format) in MCP_FILES {
            let entries = mcp_entries(&env, file, format);
            assert!(!entries.contains_key("remote"));
            assert_eq!(entries.contains_key("local"), !file.starts_with("app/"));
        }
        let record = Record::load(&env);
        assert!(!record.scopes.contains_key(&env.home.join("app")));
        assert_eq!(record.mcp.len(), 2);
        assert!(in_sync(&env));
    }

    #[test]
    fn mcp_leaves_foreign_entries_even_when_they_match_and_adopt_is_set() {
        let env = mcp_fixture("mcp-foreign");
        let servers = mcp::load(&env.library).unwrap();
        for (file, format) in MCP_FILES {
            let path = env.home.join(file);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            if format == McpFormat::ClaudeUser {
                fs::write(&path, serde_json::json!({"mcpServers": {"local": format.entry(&servers["local"])}, "other": true}).to_string()).unwrap();
            } else {
                format.write(&path, "local", Some(&format.entry(&servers["local"]))).unwrap();
            }
        }
        let plans = plan_all(&env).unwrap();
        assert_eq!(plans.iter().flat_map(|plan| &plan.changes).filter(|change| matches!(change, Change::McpConflict { .. })).count(), 5);
        assert_eq!(run(&env, true).iter().map(|applied| applied.left).sum::<usize>(), 5);
        let record = Record::load(&env);
        for (file, format) in MCP_FILES {
            assert_eq!(mcp_entries(&env, file, format)["local"], format.entry(&servers["local"]));
            assert!(!record.mcp[&env.home.join(file)].contains_key("local"));
        }
        assert!(read(&env, ".claude.json").contains("\"other\": true"));
    }

    #[test]
    fn mcp_leaves_externally_edited_entries_on_update_and_removal() {
        let env = mcp_fixture("mcp-edited");
        run(&env, false);
        for (file, format) in MCP_FILES {
            let path = env.home.join(file);
            if format == McpFormat::ClaudeUser {
                let mut doc: serde_json::Value = serde_json::from_str(&read(&env, file)).unwrap();
                doc["mcpServers"]["local"]["command"] = serde_json::json!("by-hand");
                fs::write(path, doc.to_string()).unwrap();
            } else {
                let mut entry = mcp_entries(&env, file, format)["local"].clone();
                entry["command"] = serde_json::json!("by-hand");
                format.write(&path, "local", Some(&entry)).unwrap();
            }
        }
        let before: Vec<_> = MCP_FILES.iter().map(|(file, _)| read(&env, file)).collect();
        assert_eq!(run(&env, true).iter().map(|applied| applied.left).sum::<usize>(), 5);
        assert_eq!(MCP_FILES.iter().map(|(file, _)| read(&env, file)).collect::<Vec<_>>(), before);
        fs::write(env.library.join(manifest::FILE), "").unwrap();
        assert_eq!(run(&env, false).iter().map(|applied| applied.left).sum::<usize>(), 5);
        for (file, format) in MCP_FILES {
            let entries = mcp_entries(&env, file, format);
            assert_eq!(entries.len(), 1);
            assert_eq!(entries["local"]["command"], "by-hand");
        }
    }

    #[test]
    fn mcp_rejects_unknown_servers_and_invalid_definitions_before_writing() {
        let env = mcp_fixture("mcp-invalid");
        fs::write(env.library.join(manifest::FILE), "[global]\nmcp = [\"missing\"]\n").unwrap();
        assert!(plan_all(&env).unwrap_err().to_string().contains("MCP server missing"));
        fs::write(env.library.join(manifest::FILE), "[global]\nmcp = [\"local\"]\n").unwrap();
        for text in [
            "command = \"npx\"\nurl = \"https://example.com/mcp\"\n",
            "env = [\"TOKEN\"]\n",
            "command = \"npx\"\nenv = [\"TOKEN=secret\"]\n",
            "command = \"npx\"\nenv = { TOKEN = \"secret\" }\n",
            "url = \"https://example.com/mcp\"\nbearer_token_env = \"Bearer secret\"\n",
            "url = \"https://example.com/mcp\"\nbearer_token = \"secret\"\n",
        ] {
            fs::write(env.library.join("mcp/local.toml"), text).unwrap();
            assert!(plan_all(&env).is_err(), "accepted {text}");
        }
        assert!(!env.home.join(".claude.json").exists());
        assert!(!env.home.join(CODEX_CONFIG).exists());
    }

    #[test]
    fn mcp_leaves_json_comments_and_backs_up_each_changed_file_once() {
        let env = mcp_fixture("mcp-comments");
        let comments = "{ // kept\n\"mcpServers\": {} }\n";
        fs::write(env.home.join("app/.mcp.json"), comments).unwrap();
        fs::create_dir_all(env.home.join("app/.agents")).unwrap();
        fs::write(env.home.join("app/.agents/mcp.json"), comments).unwrap();
        let originals = [
            (".claude.json", "{\"mcpServers\":{},\"other\":true}\n"),
            (CODEX_CONFIG, "# keep\nmodel = 'test'\n"),
            ("app/.codex/config.toml", "# keep project\nmodel = 'test'\n"),
        ];
        for (file, text) in originals {
            let path = env.home.join(file);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
        let (applied, backup) = apply(&env, &plan_all(&env).unwrap(), false).unwrap();
        assert_eq!(applied[1].left, 4);
        assert_eq!(read(&env, "app/.mcp.json"), comments);
        assert_eq!(read(&env, "app/.agents/mcp.json"), comments);
        let backup = backup.unwrap();
        assert_eq!(fs::read_dir(&backup).unwrap().count(), 4);
        for (file, text) in originals {
            let name = env.home.join(file).to_string_lossy().trim_start_matches('/').replace('/', "__");
            assert_eq!(fs::read_to_string(backup.join(name)).unwrap(), text);
        }
        let before = backups(&env);
        assert_eq!(apply(&env, &plan_all(&env).unwrap(), false).unwrap().1, None);
        assert_eq!(backups(&env), before);
    }

    #[test]
    fn mcp_keeps_codex_text_outside_the_owned_table_byte_for_byte() {
        let env = mcp_fixture("mcp-preserve");
        let prefix = "# before\nmodel  =  'test' # inline\n\n[other]\nweird = [ 1,  2 ]\n\n[mcp_servers.foreign]\ncommand = 'leave-me'\nargs = [ 'one' ] # kept\n";
        for file in [CODEX_CONFIG, "app/.codex/config.toml"] {
            let path = env.home.join(file);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, prefix).unwrap();
        }
        run(&env, false);
        let suffix = "\n[after]\nvalue = { odd = 'spacing' } # last\n";
        for file in [CODEX_CONFIG, "app/.codex/config.toml"] {
            let text = read(&env, file);
            assert!(text.starts_with(prefix), "{text}");
            fs::write(env.home.join(file), format!("{text}{suffix}")).unwrap();
        }
        fs::write(env.library.join("mcp/local.toml"), "command = \"updated\"\n").unwrap();
        run(&env, false);
        for file in [CODEX_CONFIG, "app/.codex/config.toml"] {
            let text = read(&env, file);
            assert!(text.starts_with(prefix) && text.ends_with(suffix), "{text}");
        }
        fs::write(env.library.join(manifest::FILE), "").unwrap();
        run(&env, false);
        for file in [CODEX_CONFIG, "app/.codex/config.toml"] {
            assert_eq!(read(&env, file), format!("{prefix}{suffix}"));
        }
    }

    #[test]
    fn mcp_skips_opencode_and_only_installs_for_the_scopes_harnesses() {
        let env = mcp_fixture("mcp-harnesses");
        fs::write(env.library.join(manifest::FILE), "[project.\"~/app\"]\nmcp = [\"local\"]\nharnesses = [\"opencode\"]\n").unwrap();
        assert!(in_sync(&env));
        fs::write(env.library.join(manifest::FILE), "[project.\"~/app\"]\nmcp = [\"remote\"]\nharnesses = [\"codex\"]\n").unwrap();
        run(&env, false);
        assert!(!env.home.join("app/.mcp.json").exists());
        assert_eq!(mcp_entries(&env, "app/.agents/mcp.json", McpFormat::Json).len(), 1);
        assert_eq!(mcp_entries(&env, "app/.codex/config.toml", McpFormat::Codex).len(), 1);
        fs::write(env.library.join(manifest::FILE), "[project.\"~/app\"]\nmcp = [\"local\"]\nharnesses = [\"claude\"]\n").unwrap();
        run(&env, false);
        assert_eq!(mcp_entries(&env, "app/.mcp.json", McpFormat::Json).len(), 1);
        assert!(mcp_entries(&env, "app/.codex/config.toml", McpFormat::Codex).is_empty());
        assert!(in_sync(&env));
    }
}
