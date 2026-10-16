//! `manifest.toml`: which plugins and MCP servers go to each scope.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context as _, Result, bail};
use toml_edit::{DocumentMut, Item as Toml, Table};

use crate::adapters::Harness;

pub const FILE: &str = "manifest.toml";
pub const GLOBAL: &str = "global";

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
    pub harnesses: Vec<Harness>,
    /// Plugins for every harness of the scope.
    pub plugins: Vec<String>,
    /// Servers for every harness of the scope that supports MCP.
    pub mcp: Vec<String>,
    /// More plugins, for one harness only.
    pub only: BTreeMap<Harness, Vec<String>>,
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

    /// The plugins this scope assigns to `harness`, each one time.
    pub fn plugins_for(&self, harness: Harness) -> Vec<String> {
        let mut plugins = Vec::new();
        if self.harnesses.contains(&harness) {
            for plugin in self.plugins.iter().chain(self.only.get(&harness).into_iter().flatten()) {
                if !plugins.contains(plugin) {
                    plugins.push(plugin.clone());
                }
            }
        }
        plugins
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

fn parse(text: &str, home: &Path) -> Result<Manifest> {
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
    let harnesses = match table.get("harnesses") {
        Some(keys) => strings(keys, "harnesses")?
            .iter()
            .map(|key| Harness::from_key(key))
            .collect::<Result<Vec<_>>>()?,
        None => Harness::DEFAULT.to_vec(),
    };
    let mut only = BTreeMap::new();
    for harness in Harness::ALL {
        let extra = list(harness.key())?;
        if !extra.is_empty() {
            if !harnesses.contains(&harness) {
                bail!("{label}: `{}` has plugins, but `harnesses` does not name it", harness.key());
            }
            only.insert(harness, extra);
        }
    }
    let scope = Scope { label: label.to_string(), root, global, harnesses, plugins: list("plugins")?, mcp: list("mcp")?, only };
    for harness in &scope.harnesses {
        if global && !harness.installs_globally() {
            bail!("{label}: workflow-setup does not install for {} at global scope", harness.key());
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

/// Assigns `plugin` to the scope at `root` for every harness of the scope,
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
        for key in std::iter::once("plugins").chain(Harness::ALL.iter().map(|harness| harness.key())) {
            if let Some(list) = table.get_mut(key).and_then(Toml::as_array_mut) {
                list.retain(|value| value.as_str() != Some(plugin));
                if list.is_empty() && key != "plugins" {
                    table.remove(key);
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
        let plugins = |root: &Path, harness: Harness| -> Vec<String> {
            let manifest = load(&library, &home).unwrap();
            manifest.scopes.iter().find(|scope| scope.root == root).map(|scope| scope.plugins_for(harness)).unwrap_or_default()
        };
        assert_eq!(plugins(&home, Harness::Claude), ["h00man", "pstack"]);
        assert_eq!(plugins(&home, Harness::Codex), ["h00man"]);
        assert_eq!(plugins(&home, Harness::OpenCode), Vec::<String>::new(), "a harness the scope does not name got plugins");

        // Out takes the plugin from every harness of the scope.
        assign(&library, &home, &home, "pstack", false).unwrap();
        assert_eq!(plugins(&home, Harness::Claude), ["h00man"]);
        assign(&library, &home, &home, "matt", true).unwrap();
        assign(&library, &home, &home, "matt", true).unwrap();
        assert_eq!(plugins(&home, Harness::Codex), ["h00man", "matt"]);
        assert!(fs::read_to_string(library.join(FILE)).unwrap().starts_with("# kept\n"));

        // A project that the manifest does not have gets a table.
        let app = home.join("app");
        assign(&library, &home, &app, "matt", true).unwrap();
        assert_eq!(plugins(&app, Harness::Claude), ["matt"]);
        assert!(fs::read_to_string(library.join(FILE)).unwrap().contains("[project.\"~/app\"]\nplugins = [\"matt\"]"));
        assert_eq!(plugins(&home, Harness::Codex), ["h00man", "matt"]);
    }
}
