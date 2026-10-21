//! The library: every item this repo can install, by category and origin.
//! Read-only.
//!
//! Layout: `plugins/<origin>/skills/<name>/SKILL.md` and
//! `agents/<origin>/<name>/`. Each folder in `plugins/` is one plugin. The
//! origin `h00man` holds the items edited here. Each other origin is a
//! verbatim copy of a source in `sources.toml`.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use crate::sources;

/// The origin whose items are edited in this repo.
pub const OWN: &str = "h00man";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub name: String,
    pub origin: String,
    pub dir: PathBuf,
}

/// Two or more origins supply the same name. Install folders are flat, so
/// only the winner can be installed under that name.
#[derive(Debug, PartialEq, Eq)]
pub struct Collision {
    pub name: String,
    pub winner: String,
    pub losers: Vec<String>,
}

/// Origins in order of precedence: `h00man`, then the sources in the order
/// `sources.toml` gives them, then any other folder by name.
pub fn origins(library: &Path, category: &str) -> Vec<String> {
    let mut found = vec![OWN.to_string()];
    found.extend(sources::load(library).unwrap_or_default().into_iter().map(|source| source.name));
    for dir in sorted_dirs(&library.join(category)) {
        found.push(file_name(&dir));
    }
    let mut seen = Vec::new();
    found.retain(|origin| {
        let first = !seen.contains(origin) && library.join(category).join(origin).is_dir();
        seen.push(origin.clone());
        first
    });
    found
}

/// The plugins in the library, in order of precedence.
pub fn plugins(library: &Path) -> Vec<String> {
    origins(library, "plugins")
}

/// Every skill in the library, in order of precedence.
pub fn skills(library: &Path) -> Vec<Item> {
    let mut items = Vec::new();
    for origin in plugins(library) {
        for dir in sorted_dirs(&library.join("plugins").join(&origin).join("skills")) {
            if dir.join("SKILL.md").is_file() {
                items.push(Item { name: file_name(&dir), origin: origin.clone(), dir });
            }
        }
    }
    items
}

/// Every agent folder in the library, in order of precedence.
pub fn agents(library: &Path) -> Vec<Item> {
    let mut items = Vec::new();
    for origin in origins(library, "agents") {
        for dir in sorted_dirs(&library.join("agents").join(&origin)) {
            items.push(Item { name: file_name(&dir), origin: origin.clone(), dir });
        }
    }
    items
}

/// The item that installs under each name, and the names that collide.
/// `items` must be in order of precedence: the first origin wins.
pub fn resolve(items: &[Item]) -> (BTreeMap<String, Item>, Vec<Collision>) {
    let mut winners: BTreeMap<String, Item> = BTreeMap::new();
    let mut losers: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for item in items {
        match winners.get(&item.name) {
            Some(_) => losers.entry(item.name.clone()).or_default().push(item.origin.clone()),
            None => {
                winners.insert(item.name.clone(), item.clone());
            }
        }
    }
    let collisions = losers
        .into_iter()
        .map(|(name, losers)| Collision { winner: winners[&name].origin.clone(), name, losers })
        .collect();
    (winners, collisions)
}

fn sorted_dirs(dir: &Path) -> Vec<PathBuf> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .map(|entries| entries.flatten().map(|entry| entry.path()).filter(|path| path.is_dir()).collect())
        .unwrap_or_default();
    entries.sort();
    entries
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(name: &str, origin: &str) -> Item {
        Item { name: name.into(), origin: origin.into(), dir: PathBuf::from(origin).join(name) }
    }

    #[test]
    fn first_origin_wins_and_the_collision_is_reported() {
        let items = [item("unslop", "h00man"), item("plan", "h00man"), item("unslop", "poteto"), item("teach", "matt"), item("teach", "poteto")];
        let (winners, collisions) = resolve(&items);
        assert_eq!(winners["unslop"].origin, "h00man");
        assert_eq!(winners["teach"].origin, "matt");
        assert_eq!(
            collisions,
            [
                Collision { name: "teach".into(), winner: "matt".into(), losers: vec!["poteto".into()] },
                Collision { name: "unslop".into(), winner: "h00man".into(), losers: vec!["poteto".into()] },
            ]
        );
    }
}
