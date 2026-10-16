//! `manifest.toml`: which plugins and MCP servers go to each scope.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context as _, Result, bail};
use toml_edit::{DocumentMut, Item as Toml, Table};

use crate::agents::{self, Agent, Kind};

pub const FILE: &str = "manifest.toml";
pub const GLOBAL: &str = "global";
/// The keys of a scope that are not agent names.
const SCOPE_KEYS: [&str; 3] = ["harnesses", "plugins", "mcp"];

/// One item of a list in a scope: a whole plugin, or one skill of a plugin.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Entry {
    /// `<plugin>`
    Plugin(String),
    /// `<plugin>/<skill>`. It installs as one link without a prefix, for every agent.
    Skill { plugin: String, skill: String },
}

impl Entry {
    pub fn parse(text: &str) -> Result<Entry> {
        let plain = |name: &str| !name.is_empty() && !name.starts_with('.') && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'));
        match text.split_once('/') {
            None if plain(text) => Ok(Entry::Plugin(text.to_string())),
            Some((plugin, skill)) if plain(plugin) && plain(skill) => Ok(Entry::Skill { plugin: plugin.to_string(), skill: skill.to_string() }),
            _ => bail!("`{text}` is not a plugin name or <plugin>/<skill>"),
        }
    }

    pub fn plugin(&self) -> &str {
        match self {
            Entry::Plugin(plugin) | Entry::Skill { plugin, .. } => plugin,
        }
    }
}

impl std::fmt::Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Entry::Plugin(plugin) => write!(f, "{plugin}"),
            Entry::Skill { plugin, skill } => write!(f, "{plugin}/{skill}"),
        }
    }
}

#[derive(Debug, Default)]
pub struct Manifest {
    pub scopes: Vec<Scope>,
}

/// One place that receives plugins and servers: the home folder, or one project.
#[derive(Clone, Debug)]
pub struct Scope {
    /// `global`, or the project path as the manifest gives it.
    pub label: String,
    pub root: PathBuf,
    pub global: bool,
    /// The agents of the scope, from `harnesses`.
    pub agents: Vec<&'static Agent>,
    /// Plugins and skills for every agent of the scope.
    pub plugins: Vec<Entry>,
    /// Servers for every agent of the scope that supports MCP.
    pub mcp: Vec<String>,
    /// More plugins and skills, for one agent only.
    pub only: BTreeMap<&'static Agent, Vec<Entry>>,
}

impl Scope {
    /// A scope that receives nothing. `apply` uses it to remove what it
    /// installed in a project that left the manifest.
    pub fn empty(root: PathBuf, home: &Path) -> Scope {
        let global = root == home;
        Scope {
            label: if global { GLOBAL.to_string() } else { root.to_string_lossy().into_owned() },
            root,
            global,
            agents: Vec::new(),
            plugins: Vec::new(),
            mcp: Vec::new(),
            only: BTreeMap::new(),
        }
    }

    /// The plugins and skills this scope assigns to `agent`, each one time.
    pub fn entries_for(&self, agent: &Agent) -> Vec<Entry> {
        let mut entries = Vec::new();
        if self.agents.contains(&agent) {
            for entry in self.plugins.iter().chain(self.only.get(agent).into_iter().flatten()) {
                if !entries.contains(entry) {
                    entries.push(entry.clone());
                }
            }
        }
        entries
    }

    /// The whole plugins this scope assigns to `agent`.
    pub fn plugins_for(&self, agent: &Agent) -> Vec<String> {
        let entries = self.entries_for(agent).into_iter();
        entries.filter_map(|entry| if let Entry::Plugin(plugin) = entry { Some(plugin) } else { None }).collect()
    }

    /// True when an agent of this kind is in the scope.
    pub fn has(&self, kind: Kind) -> bool {
        self.agents.iter().any(|agent| agent.kind == kind)
    }
}

pub fn load(library: &Path, home: &Path) -> Result<Manifest> {
    let path = library.join(FILE);
    if !path.is_file() {
        return Ok(Manifest::default());
    }
    let text = fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?;
    parse(&text, home)
}

pub fn parse(text: &str, home: &Path) -> Result<Manifest> {
    let document: DocumentMut = text.parse().with_context(|| format!("{FILE} is not valid TOML"))?;
    let mut manifest = Manifest::default();
    if let Some(table) = document.get(GLOBAL).and_then(Toml::as_table) {
        manifest.scopes.push(scope(GLOBAL, home.to_path_buf(), true, table)?);
    }
    if let Some(projects) = document.get("project").and_then(Toml::as_table) {
        for (path, table) in projects.iter() {
            let table = table.as_table().with_context(|| format!("project `{path}` is not a table"))?;
            let root = match path.strip_prefix("~/") {
                Some(rest) => home.join(rest),
                None => PathBuf::from(path),
            };
            if !root.is_absolute() {
                bail!("project `{path}` in {FILE} must be an absolute path or start with ~/");
            }
            manifest.scopes.push(scope(path, root, false, table)?);
        }
    }
    Ok(manifest)
}

fn scope(label: &str, root: PathBuf, global: bool, table: &Table) -> Result<Scope> {
    let list = |key: &str| match table.get(key) {
        Some(items) => strings(items, key),
        None => Ok(Vec::new()),
    };
    let entries = |key: &str| -> Result<Vec<Entry>> {
        list(key)?.iter().map(|text| Entry::parse(text).with_context(|| format!("{label}: `{key}`"))).collect()
    };
    let mut scope_agents: Vec<&'static Agent> = Vec::new();
    match table.get("harnesses") {
        Some(keys) => {
            for key in strings(keys, "harnesses")? {
                let agent = agents::find(&key).with_context(|| format!("{label}: `harnesses` names {key}"))?;
                if !scope_agents.contains(&agent) {
                    scope_agents.push(agent);
                }
            }
        }
        None => scope_agents = agents::default(),
    }
    let mut only: BTreeMap<&'static Agent, Vec<Entry>> = BTreeMap::new();
    for (key, _) in table.iter().filter(|(key, _)| !SCOPE_KEYS.contains(key)) {
        let agent = agents::find(key).with_context(|| format!("{label}: `{key}` is not a key of a scope"))?;
        let extra = entries(key)?;
        if !extra.is_empty() && !scope_agents.contains(&agent) {
            bail!("{label}: `{key}` has plugins, but `harnesses` does not name it");
        }
        only.entry(agent).or_default().extend(extra);
    }
    only.retain(|_, extra| !extra.is_empty());
    if global {
        if let Some(agent) = scope_agents.iter().find(|agent| agent.global.is_none()) {
            bail!("{label}: {} has no skill folder at global scope", agent.name);
        }
    }
    Ok(Scope { label: label.to_string(), root, global, agents: scope_agents, plugins: entries("plugins")?, mcp: list("mcp")?, only })
}

fn strings(item: &Toml, name: &str) -> Result<Vec<String>> {
    item.as_array()
        .with_context(|| format!("`{name}` in {FILE} is not a list"))?
        .iter()
        .map(|value| value.as_str().map(str::to_string).with_context(|| format!("`{name}` in {FILE} has a value that is not text")))
        .collect()
}

/// Assigns `plugin` to the scope at `root` for every agent of the scope,
/// or takes it out for all of them. Writes `manifest.toml` and keeps its
/// comments. A project that the manifest does not have gets a table.
pub fn assign(library: &Path, home: &Path, root: &Path, plugin: &str, on: bool) -> Result<()> {
    let path = library.join(FILE);
    let text = fs::read_to_string(&path).unwrap_or_default();
    let manifest = parse(&text, home)?;
    let mut document: DocumentMut = text.parse().with_context(|| format!("{FILE} is not valid TOML"))?;

    let table = if root == home {
        document.entry(GLOBAL).or_insert(toml_edit::table())
    } else {
        let label = match manifest.scopes.iter().find(|scope| scope.root == root) {
            Some(scope) => scope.label.clone(),
            None => match root.strip_prefix(home) {
                Ok(rest) => format!("~/{}", rest.display()),
                Err(_) => root.display().to_string(),
            },
        };
        let projects = document.entry("project").or_insert(toml_edit::table());
        let projects = projects.as_table_mut().context("`project` is not a table")?;
        projects.set_implicit(true);
        projects.entry(&label).or_insert(toml_edit::table())
    };
    let table = table.as_table_mut().context("the scope is not a table")?;

    if on {
        let list = table.entry("plugins").or_insert(toml_edit::value(toml_edit::Array::new()));
        let list = list.as_array_mut().context("`plugins` is not a list")?;
        if !list.iter().any(|value| value.as_str() == Some(plugin)) {
            list.push(plugin);
        }
    } else {
        let keys: Vec<String> = table.iter().map(|(key, _)| key.to_string()).filter(|key| key == "plugins" || agents::find(key).is_ok()).collect();
        for key in keys {
            if let Some(list) = table.get_mut(&key).and_then(Toml::as_array_mut) {
                list.retain(|value| value.as_str() != Some(plugin));
                if list.is_empty() && key != "plugins" {
                    table.remove(&key);
                }
            }
        }
    }
    fs::write(&path, document.to_string()).with_context(|| format!("cannot write {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_switch_assigns_a_plugin_or_takes_it_out() {
        let root = std::env::temp_dir().join(format!("workflow-setup-manifest-{}", std::process::id()));
        fs::remove_dir_all(&root).ok();
        let (library, home) = (root.join("library"), root.join("home"));
        fs::create_dir_all(&library).unwrap();
        fs::write(library.join(FILE), "# kept\n[global]\nplugins = [\"h00man\"]\nclaude = [\"pstack\"]\n").unwrap();
        let plugins = |root: &Path, agent: &str| -> Vec<String> {
            let manifest = load(&library, &home).unwrap();
            let agent = agents::find(agent).unwrap();
            manifest.scopes.iter().find(|scope| scope.root == root).map(|scope| scope.plugins_for(agent)).unwrap_or_default()
        };
        assert_eq!(plugins(&home, "claude"), ["h00man", "pstack"]);
        assert_eq!(plugins(&home, "codex"), ["h00man"]);
        assert_eq!(plugins(&home, "opencode"), Vec::<String>::new(), "an agent the scope does not name got plugins");

        // Out takes the plugin from every agent of the scope.
        assign(&library, &home, &home, "pstack", false).unwrap();
        assert_eq!(plugins(&home, "claude"), ["h00man"]);
        assign(&library, &home, &home, "matt", true).unwrap();
        assign(&library, &home, &home, "matt", true).unwrap();
        assert_eq!(plugins(&home, "codex"), ["h00man", "matt"]);
        assert!(fs::read_to_string(library.join(FILE)).unwrap().starts_with("# kept\n"));

        // A project that the manifest does not have gets a table.
        let app = home.join("app");
        assign(&library, &home, &app, "matt", true).unwrap();
        assert_eq!(plugins(&app, "claude"), ["matt"]);
        assert!(fs::read_to_string(library.join(FILE)).unwrap().contains("[project.\"~/app\"]\nplugins = [\"matt\"]"));
        assert_eq!(plugins(&home, "codex"), ["h00man", "matt"]);
    }

    #[test]
    fn any_agent_of_the_table_works_in_harnesses_and_as_a_key() {
        let home = Path::new("/h");
        let text = "[project.\"~/app\"]\nharnesses = [\"claude\", \"cursor\", \"claude-code\"]\nclaude = [\"a\"]\nclaude-code = [\"b\"]\ncursor = [\"c\"]\n";
        let scope = &parse(text, home).unwrap().scopes[0];
        assert_eq!(scope.agents.iter().map(|agent| agent.name.as_str()).collect::<Vec<_>>(), ["claude-code", "cursor"]);
        assert_eq!(scope.plugins_for(agents::find("claude").unwrap()), ["a", "b"]);
        assert_eq!(scope.plugins_for(agents::find("cursor").unwrap()), ["c"]);

        assert!(parse("[global]\nharnesses = [\"nope\"]\n", home).is_err());
        assert!(parse("[global]\nplugin = [\"typo\"]\n", home).is_err(), "an unknown key passed");
        assert!(parse("[global]\ncursor = [\"a\"]\n", home).is_err(), "an agent outside `harnesses` got plugins");
        assert!(parse("[global]\nharnesses = [\"eve\"]\n", home).is_err(), "an agent without a global folder passed at global scope");
        assert!(parse("[global]\nharnesses = [\"opencode\"]\n", home).is_ok());
    }

    #[test]
    fn an_entry_is_a_plugin_or_one_skill_of_a_plugin() {
        let scope = &parse("[global]\nplugins = [\"h00man\", \"pstack/swarm\"]\n", Path::new("/h")).unwrap().scopes[0];
        let skill = Entry::Skill { plugin: "pstack".into(), skill: "swarm".into() };
        assert_eq!(scope.entries_for(agents::find("codex").unwrap()), [Entry::Plugin("h00man".into()), skill.clone()]);
        assert_eq!(scope.plugins_for(agents::find("codex").unwrap()), ["h00man"]);
        assert_eq!(skill.to_string(), "pstack/swarm");
        for bad in ["", "a/b/c", "/b", "a/", "../x"] {
            assert!(Entry::parse(bad).is_err(), "`{bad}` passed");
        }
    }
}
