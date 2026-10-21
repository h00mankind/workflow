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
    /// The agents that get `plugins` and `mcp`: from `harnesses`, else the default.
    pub harnesses: Vec<&'static Agent>,
    /// Plugins and skills for each agent of `harnesses`.
    pub plugins: Vec<Entry>,
    /// Servers for each agent of `harnesses` that supports MCP.
    pub mcp: Vec<String>,
    /// Plugins and skills for one agent only. The agent need not be in `harnesses`.
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
            harnesses: Vec::new(),
            plugins: Vec::new(),
            mcp: Vec::new(),
            only: BTreeMap::new(),
        }
    }

    /// Every agent that gets something in this scope: `harnesses`, then
    /// each agent with a list of its own.
    pub fn agents(&self) -> Vec<&'static Agent> {
        let mut all = self.harnesses.clone();
        all.extend(self.only.keys().filter(|agent| !self.harnesses.contains(agent)));
        all
    }

    /// The plugins and skills this scope assigns to `agent`, each one time.
    pub fn entries_for(&self, agent: &Agent) -> Vec<Entry> {
        let shared = if self.harnesses.contains(&agent) { self.plugins.as_slice() } else { &[] };
        let mut entries = Vec::new();
        for entry in shared.iter().chain(self.only.get(agent).into_iter().flatten()) {
            if !entries.contains(entry) {
                entries.push(entry.clone());
            }
        }
        entries
    }

    /// The whole plugins this scope assigns to `agent`.
    pub fn plugins_for(&self, agent: &Agent) -> Vec<String> {
        let entries = self.entries_for(agent).into_iter();
        entries.filter_map(|entry| if let Entry::Plugin(plugin) = entry { Some(plugin) } else { None }).collect()
    }

    /// True when an agent of this kind gets something in the scope.
    pub fn has(&self, kind: Kind) -> bool {
        self.agents().iter().any(|agent| agent.kind == kind)
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
    let mut harnesses: Vec<&'static Agent> = Vec::new();
    match table.get("harnesses") {
        Some(keys) => {
            for key in strings(keys, "harnesses")? {
                let agent = agents::find(&key).with_context(|| format!("{label}: `harnesses` names {key}"))?;
                if !harnesses.contains(&agent) {
                    harnesses.push(agent);
                }
            }
        }
        None => harnesses = agents::default(),
    }
    let mut only: BTreeMap<&'static Agent, Vec<Entry>> = BTreeMap::new();
    for (key, _) in table.iter().filter(|(key, _)| !SCOPE_KEYS.contains(key)) {
        let agent = agents::find(key).with_context(|| format!("{label}: `{key}` is not a key of a scope"))?;
        only.entry(agent).or_default().extend(entries(key)?);
    }
    only.retain(|_, extra| !extra.is_empty());
    let scope = Scope { label: label.to_string(), root, global, harnesses, plugins: entries("plugins")?, mcp: list("mcp")?, only };
    if global {
        if let Some(agent) = scope.agents().iter().find(|agent| agent.global.is_none()) {
            bail!("{label}: {} has no skill folder at global scope", agent.name);
        }
    }
    Ok(scope)
}

fn strings(item: &Toml, name: &str) -> Result<Vec<String>> {
    item.as_array()
        .with_context(|| format!("`{name}` in {FILE} is not a list"))?
        .iter()
        .map(|value| value.as_str().map(str::to_string).with_context(|| format!("`{name}` in {FILE} has a value that is not text")))
        .collect()
}

/// The text of `manifest.toml`. Empty when the library has none.
pub fn read(library: &Path) -> String {
    fs::read_to_string(library.join(FILE)).unwrap_or_default()
}

pub fn write(library: &Path, text: &str) -> Result<()> {
    let path = library.join(FILE);
    fs::write(&path, text).with_context(|| format!("cannot write {}", path.display()))
}

/// Takes every entry of `plugin` out of every scope, and returns the new text.
pub fn drop_plugin(text: &str, home: &Path, plugin: &str) -> Result<String> {
    let mut text = text.to_string();
    for scope in parse(&text, home)?.scopes {
        let mut entries: Vec<Entry> = scope.plugins.iter().chain(scope.only.values().flatten()).filter(|entry| entry.plugin() == plugin).cloned().collect();
        entries.sort();
        entries.dedup();
        for entry in entries {
            text = edit(&text, home, &scope.root, &[entry], &[], false)?;
        }
    }
    Ok(text)
}

/// Adds entries to the scope at `root`, or takes them out, and returns the
/// new text. Comments stay. With no `only`, the change is for every agent
/// of the scope: `plugins` gets the entry, or the entry goes out of every
/// list. With `only`, the change is for those agents and their own lists
/// only, and `harnesses` does not change. To take out an entry that
/// `plugins` has, it moves to the lists of the other harnesses. A project
/// that the manifest does not have gets a table when an entry goes in.
pub fn edit(text: &str, home: &Path, root: &Path, entries: &[Entry], only: &[&'static Agent], add: bool) -> Result<String> {
    let manifest = parse(text, home)?;
    let scope = manifest.scopes.iter().find(|scope| scope.root == root);
    if scope.is_none() && !add {
        bail!("{FILE} has no scope at {}", root.display());
    }
    let label = match (scope, root.strip_prefix(home)) {
        (Some(scope), _) => scope.label.clone(),
        (None, Ok(rest)) => format!("~/{}", rest.display()),
        (None, Err(_)) => root.display().to_string(),
    };
    let harnesses = scope.map(|scope| scope.harnesses.clone()).unwrap_or_else(agents::default);
    let everyone = scope.map(Scope::agents).unwrap_or_else(agents::default);

    let mut document: DocumentMut = text.parse().with_context(|| format!("{FILE} is not valid TOML"))?;
    let table = if root == home {
        document.entry(GLOBAL).or_insert(toml_edit::table())
    } else {
        let projects = document.entry("project").or_insert(toml_edit::table());
        let projects = projects.as_table_mut().context("`project` is not a table")?;
        projects.set_implicit(true);
        projects.entry(&label).or_insert(toml_edit::table())
    };
    let table = table.as_table_mut().context("the scope is not a table")?;
    let mut lists = Lists { table, harnesses: &harnesses };
    for entry in entries {
        let text = entry.to_string();
        if add {
            if only.is_empty() {
                lists.insert("plugins", &text)?;
                continue;
            }
            for &agent in only {
                if !lists.gives(agent, &text) {
                    let key = lists.key_of(agent);
                    lists.insert(&key, &text)?;
                }
            }
            continue;
        }

        let targets: &[&'static Agent] = if only.is_empty() { &everyone } else { only };
        let missing: Vec<&'static Agent> = targets.iter().copied().filter(|agent| !lists.gives(agent, &text)).collect();
        let blocked = if only.is_empty() { missing.len() == targets.len() } else { !missing.is_empty() };
        if blocked {
            let Some(agent) = missing.first() else { bail!("{label} does not have {entry}") };
            if let Entry::Skill { plugin, .. } = entry {
                if lists.gives(agent, plugin) {
                    bail!("{entry} comes with the whole plugin {plugin} in {label}. Remove {plugin}, then install the skills you want.");
                }
            }
            bail!("{label} does not give {entry} to {}", agent.name);
        }
        if only.is_empty() {
            let keys: Vec<String> = lists.table.iter().map(|(key, _)| key.to_string()).filter(|key| key == "plugins" || agents::find(key).is_ok()).collect();
            for key in keys {
                lists.delete(&key, &text);
            }
            continue;
        }
        if lists.has("plugins", &text) {
            lists.delete("plugins", &text);
            for &other in harnesses.iter().filter(|agent| !only.contains(agent)) {
                let key = lists.key_of(other);
                lists.insert(&key, &text)?;
            }
        }
        for &agent in only {
            let key = lists.key_of(agent);
            lists.delete(&key, &text);
        }
    }
    Ok(document.to_string())
}

/// The lists of one scope table.
struct Lists<'a> {
    table: &'a mut Table,
    /// The agents that `plugins` gives to.
    harnesses: &'a [&'static Agent],
}

impl Lists<'_> {
    /// The key of `agent` in the table: the name or alias it has there, else its first alias or name.
    fn key_of(&self, agent: &Agent) -> String {
        let present = self.table.iter().map(|(key, _)| key).find(|key| !SCOPE_KEYS.contains(key) && agents::find(key).is_ok_and(|found| found == agent));
        present.unwrap_or(agent.key()).to_string()
    }

    /// True when the scope gives `text` to `agent`.
    fn gives(&self, agent: &Agent, text: &str) -> bool {
        (self.harnesses.contains(&agent) && self.has("plugins", text)) || self.has(&self.key_of(agent), text)
    }

    fn has(&self, key: &str, text: &str) -> bool {
        self.table.get(key).and_then(Toml::as_array).is_some_and(|list| list.iter().any(|value| value.as_str() == Some(text)))
    }

    fn insert(&mut self, key: &str, text: &str) -> Result<()> {
        if self.has(key, text) {
            return Ok(());
        }
        let list = self.table.entry(key).or_insert(toml_edit::value(toml_edit::Array::new()));
        list.as_array_mut().with_context(|| format!("`{key}` is not a list"))?.push(text);
        Ok(())
    }

    /// Takes `text` out of the list. An agent list that becomes empty goes away.
    fn delete(&mut self, key: &str, text: &str) {
        if let Some(list) = self.table.get_mut(key).and_then(Toml::as_array_mut) {
            list.retain(|value| value.as_str() != Some(text));
            if list.is_empty() && !SCOPE_KEYS.contains(&key) {
                self.table.remove(key);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entries(text: &str, root: &Path, agent: &str) -> Vec<String> {
        let manifest = parse(text, Path::new("/h")).unwrap();
        let agent = agents::find(agent).unwrap();
        let scope = manifest.scopes.iter().find(|scope| scope.root == root);
        scope.map(|scope| scope.entries_for(agent).iter().map(Entry::to_string).collect()).unwrap_or_default()
    }

    fn change(text: &str, root: &Path, entry: &str, only: &[&str], add: bool) -> Result<String> {
        let only: Vec<&'static Agent> = only.iter().map(|key| agents::find(key).unwrap()).collect();
        edit(text, Path::new("/h"), root, &[Entry::parse(entry).unwrap()], &only, add)
    }

    #[test]
    fn an_edit_adds_or_takes_out_an_entry_for_the_scope() {
        let home = Path::new("/h");
        let text = "# kept\n[global]\nplugins = [\"h00man\"]\nclaude = [\"pstack\"]\n";
        assert_eq!(entries(text, home, "claude"), ["h00man", "pstack"]);
        assert_eq!(entries(text, home, "opencode"), Vec::<String>::new(), "an agent the scope does not name got plugins");

        // Out takes the entry from every agent of the scope.
        let text = change(text, home, "pstack", &[], false).unwrap();
        assert_eq!(entries(&text, home, "claude"), ["h00man"]);
        assert!(!text.contains("claude ="), "an empty agent list stayed");
        let text = change(&text, home, "matt", &[], true).unwrap();
        assert_eq!(change(&text, home, "matt", &[], true).unwrap(), text, "a second install changed the text");
        assert_eq!(entries(&text, home, "codex"), ["h00man", "matt"]);
        assert!(text.starts_with("# kept\n"));
        assert!(change(&text, home, "pstack", &[], false).is_err(), "a remove of an entry that the scope does not have passed");

        // A project that the manifest does not have gets a table.
        let app = Path::new("/h/app");
        let text = change(&text, app, "matt", &[], true).unwrap();
        assert!(text.contains("[project.\"~/app\"]\nplugins = [\"matt\"]"));
        assert!(change(&text, Path::new("/h/other"), "matt", &[], false).is_err());
    }

    #[test]
    fn an_edit_for_one_agent_writes_only_its_own_list_and_splits_a_shared_entry() {
        let app = Path::new("/h/app");
        let text = "[project.\"~/app\"]\nplugins = [\"h00man\", \"pstack\"]\n";

        // An agent outside `harnesses` gets its own list, not `plugins`.
        let text = change(text, app, "matt/teach", &["cursor"], true).unwrap();
        assert!(!text.contains("harnesses"), "{text}");
        assert_eq!(entries(&text, app, "cursor"), ["matt/teach"]);
        assert_eq!(entries(&text, app, "claude"), ["h00man", "pstack"]);
        assert_eq!(change(&text, app, "matt/teach", &["cursor"], true).unwrap(), text, "a second install changed the text");
        assert_eq!(change(&text, app, "pstack", &["codex"], true).unwrap(), text, "an entry that `plugins` gives went into a list");

        // Out for one agent moves a shared entry to the lists of the other harnesses.
        let text = change(&text, app, "pstack", &["claude"], false).unwrap();
        assert_eq!(entries(&text, app, "claude"), ["h00man"]);
        assert_eq!(entries(&text, app, "codex"), ["h00man", "pstack"]);
        assert_eq!(entries(&text, app, "cursor"), ["matt/teach"]);

        // A skill of a whole plugin cannot go out alone.
        let error = change(&text, app, "h00man/plan", &[], false).unwrap_err().to_string();
        assert!(error.contains("whole plugin h00man"), "{error}");
        assert!(change(&text, app, "matt/teach", &["claude"], false).is_err());
        let text = change(&text, app, "matt/teach", &["cursor"], false).unwrap();
        assert!(!text.contains("cursor"), "{text}");
    }

    #[test]
    fn any_agent_of_the_table_works_in_harnesses_and_as_a_key() {
        let home = Path::new("/h");
        let text = "[project.\"~/app\"]\nharnesses = [\"claude\", \"cursor\", \"claude-code\"]\nclaude = [\"a\"]\nclaude-code = [\"b\"]\ncursor = [\"c\"]\n";
        let scope = &parse(text, home).unwrap().scopes[0];
        assert_eq!(scope.harnesses.iter().map(|agent| agent.name.as_str()).collect::<Vec<_>>(), ["claude-code", "cursor"]);
        assert_eq!(scope.plugins_for(agents::find("claude").unwrap()), ["a", "b"]);
        assert_eq!(scope.plugins_for(agents::find("cursor").unwrap()), ["c"]);

        assert!(parse("[global]\nharnesses = [\"nope\"]\n", home).is_err());
        assert!(parse("[global]\nplugin = [\"typo\"]\n", home).is_err(), "an unknown key passed");
        let scope = &parse("[global]\nplugins = [\"p\"]\ncursor = [\"a\"]\n", home).unwrap().scopes[0];
        assert_eq!(scope.plugins_for(agents::find("cursor").unwrap()), ["a"], "an agent outside `harnesses` got `plugins`");
        assert_eq!(scope.agents().iter().map(|agent| agent.key()).collect::<Vec<_>>(), ["claude", "codex", "cursor"]);
        assert!(parse("[global]\neve = [\"a\"]\n", home).is_err(), "an agent key without a global folder passed at global scope");
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
