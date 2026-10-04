//! `manifest.toml`: which bundles go to global scope and to each project.
//!
//! A bundle is a named set of items. An item is `<origin>/<name>`, or
//! `<origin>/*` for every item of that origin.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context as _, Result, bail};
use toml_edit::{DocumentMut, Item as Toml, Table};

use crate::{adapters::Harness, library::Item};

pub const FILE: &str = "manifest.toml";
pub const GLOBAL: &str = "global";

#[derive(Debug, Default)]
pub struct Manifest {
    pub bundles: BTreeMap<String, Vec<String>>,
    pub scopes: Vec<Scope>,
}

/// One place that receives bundles: the home folder, or one project.
#[derive(Clone, Debug)]
pub struct Scope {
    /// `global`, or the project path as the manifest gives it.
    pub label: String,
    pub root: PathBuf,
    pub global: bool,
    pub harnesses: Vec<Harness>,
    /// Bundles for every harness of the scope.
    pub bundles: Vec<String>,
    /// More bundles, for one harness only.
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
            bundles: Vec::new(),
            only: BTreeMap::new(),
        }
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
    if let Some(bundles) = document.get("bundles").and_then(Toml::as_table) {
        for (name, items) in bundles.iter() {
            manifest.bundles.insert(name.to_string(), strings(items, name)?);
        }
    }
    if let Some(table) = document.get(GLOBAL).and_then(Toml::as_table) {
        manifest.scopes.push(scope(GLOBAL, home.to_path_buf(), true, table, &manifest.bundles)?);
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
            manifest.scopes.push(scope(path, root, false, table, &manifest.bundles)?);
        }
    }
    Ok(manifest)
}

fn scope(label: &str, root: PathBuf, global: bool, table: &Table, bundles: &BTreeMap<String, Vec<String>>) -> Result<Scope> {
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
                bail!("{label}: `{}` has bundles, but `harnesses` does not name it", harness.key());
            }
            only.insert(harness, extra);
        }
    }
    let scope = Scope { label: label.to_string(), root, global, harnesses, bundles: list("bundles")?, only };
    for harness in &scope.harnesses {
        if harness.skill_dir(global).is_none() {
            bail!("{label}: workflow-setup does not install skills for {} at this scope", harness.key());
        }
    }
    for name in scope.bundles.iter().chain(scope.only.values().flatten()) {
        if !bundles.contains_key(name) {
            bail!("{label}: {FILE} has no bundle named {name}");
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

impl Manifest {
    /// The library items that `scope` assigns to `harness`, in the library's
    /// order of precedence. Fails when a bundle names an item that the
    /// library does not have.
    pub fn items(&self, scope: &Scope, harness: Harness, library: &[Item]) -> Result<Vec<Item>> {
        let extra = scope.only.get(&harness).into_iter().flatten();
        let patterns: Vec<&String> = scope
            .bundles
            .iter()
            .chain(extra)
            .flat_map(|bundle| self.bundles.get(bundle).into_iter().flatten())
            .collect();
        for pattern in &patterns {
            let Some((origin, name)) = pattern.split_once('/') else {
                bail!("`{pattern}` in {FILE} is not `<origin>/<name>`");
            };
            if !library.iter().any(|item| item.origin == origin && (name == "*" || item.name == name)) {
                bail!("{FILE} names {pattern}, which the library does not have");
            }
        }
        Ok(library
            .iter()
            .filter(|item| patterns.iter().any(|pattern| **pattern == format!("{}/*", item.origin) || **pattern == format!("{}/{}", item.origin, item.name)))
            .cloned()
            .collect())
    }
}
