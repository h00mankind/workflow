//! The library as plugins. Each folder in `plugins/` is one plugin, and its
//! name is the prefix of its skills: `<plugin>:<skill>`.
//!
//! A source with `plugin = true` brings its own manifests. For every other
//! plugin the engine writes them, and it writes the Codex marketplace file
//! that lists all plugins.

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context as _, Result};
use serde_json::json;

use crate::{library, sources};

/// The name of this library as a Codex marketplace.
pub const MARKETPLACE: &str = "workflow";
const MARKETPLACE_FILE: &str = ".agents/plugins/marketplace.json";
pub const CLAUDE_MANIFEST: &str = ".claude-plugin/plugin.json";
pub const CODEX_MANIFEST: &str = ".codex-plugin/plugin.json";

pub fn dir(library: &Path, plugin: &str) -> PathBuf {
    library.join("plugins").join(plugin)
}

/// Writes the manifests of a plugin whose content is only skills.
pub fn write_manifests(plugin_dir: &Path, name: &str, description: &str) -> Result<()> {
    let claude = json!({ "name": name, "version": "0.0.0", "description": description });
    let codex = json!({ "name": name, "version": "0.0.0", "description": description, "skills": "./skills/" });
    write_json(&plugin_dir.join(CLAUDE_MANIFEST), &claude)?;
    write_json(&plugin_dir.join(CODEX_MANIFEST), &codex)
}

/// Makes the generated files match the library: the manifests of each
/// plugin that does not bring its own, and the marketplace file. Writes
/// nothing when they match already.
pub fn sync(library: &Path) -> Result<()> {
    let sources = sources::load(library)?;
    let mut entries = Vec::new();
    for plugin in library::plugins(library) {
        let dir = dir(library, &plugin);
        let source = sources.iter().find(|source| source.name == plugin);
        if !source.is_some_and(|source| source.plugin) {
            let description = match source {
                Some(source) => format!("Skills from {}, pinned in sources.toml.", source.repository),
                None => "My own skills.".to_string(),
            };
            write_manifests(&dir, &plugin, &description)?;
        }
        if dir.join(CODEX_MANIFEST).is_file() {
            entries.push(json!({
                "name": plugin,
                "source": { "source": "local", "path": format!("./plugins/{plugin}") },
                "policy": { "installation": "AVAILABLE", "authentication": "ON_INSTALL", "products": ["CODEX"] },
                "category": "Developer Tools",
            }));
        }
    }
    let marketplace = json!({ "name": MARKETPLACE, "interface": { "displayName": MARKETPLACE }, "plugins": entries });
    write_json(&library.join(MARKETPLACE_FILE), &marketplace)
}

fn write_json(path: &Path, value: &serde_json::Value) -> Result<()> {
    let mut text = serde_json::to_string_pretty(value)?;
    text.push('\n');
    if fs::read_to_string(path).is_ok_and(|on_disk| on_disk == text) {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, text).with_context(|| format!("cannot write {}", path.display()))
}
