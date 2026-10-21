//! The agent table: every coding agent that reads skills, and where it reads
//! them. The data is `agents.toml`, which `tools/agents-from-skills-sh.mjs`
//! writes from the table of skills.sh and `agents.overrides.json`.
//!
//! The kind says how an agent receives a whole plugin. Claude Code loads a
//! plugin from one link and gives its skills the prefix `<plugin>:`. Codex
//! installs a plugin from this library as a marketplace. Every other agent
//! gets one link per skill, with no prefix. A single skill is always one
//! link, for every kind. For a plugin kind, `project` and `global` are also
//! the folders where the agent reads plain skills, so `plan` can warn when
//! another agent links a skill there that the plugin agent already has.

use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use anyhow::{Context as _, Result, bail};
use toml_edit::DocumentMut;

const DATA: &str = include_str!("../agents.toml");

/// Folders, from a project root, where the engine made links before the
/// agent table. `apply` still removes its own links there.
pub const LEGACY_DIRS: [&str; 1] = [".opencode/skill"];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    ClaudePlugin,
    CodexPlugin,
    Skills,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Agent {
    pub name: String,
    pub display: String,
    /// The skill folder, from a project root.
    pub project: String,
    /// The skill folder at global scope, from `~/`. None: projects only.
    pub global: Option<String>,
    pub kind: Kind,
    /// Other names that the manifest can use for this agent.
    pub aliases: Vec<String>,
}

impl Agent {
    /// The folder that holds this agent's skills in a scope. `root` is the
    /// home folder at global scope.
    pub fn skill_dir(&self, root: &Path, global: bool) -> Option<PathBuf> {
        match global {
            false => Some(root.join(&self.project)),
            true => self.global.as_ref().map(|dir| root.join(dir.trim_start_matches("~/"))),
        }
    }

    /// The name the manifest knows this agent by: its first alias, else its name.
    pub fn key(&self) -> &str {
        self.aliases.first().unwrap_or(&self.name)
    }
}

pub fn all() -> &'static [Agent] {
    static TABLE: OnceLock<Vec<Agent>> = OnceLock::new();
    TABLE.get_or_init(|| parse(DATA).expect("agents.toml is not valid. Run tools/agents-from-skills-sh.mjs again."))
}

/// The agent with this name or alias.
pub fn find(key: &str) -> Result<&'static Agent> {
    match all().iter().find(|agent| agent.name == key || agent.aliases.iter().any(|alias| alias == key)) {
        Some(agent) => Ok(agent),
        None => bail!("`{key}` is not an agent. apps/workflow-setup/agents.toml lists them."),
    }
}

/// The one agent of a plugin kind.
pub fn of_kind(kind: Kind) -> &'static Agent {
    all().iter().find(|agent| agent.kind == kind).expect("agents.toml has an agent of each plugin kind")
}

/// The agents a scope installs for when the manifest names none.
pub fn default() -> Vec<&'static Agent> {
    vec![of_kind(Kind::ClaudePlugin), of_kind(Kind::CodexPlugin)]
}

fn parse(text: &str) -> Result<Vec<Agent>> {
    let document: DocumentMut = text.parse().context("agents.toml is not valid TOML")?;
    let mut agents = Vec::new();
    for (name, item) in document.iter() {
        let text = |key: &str| item.get(key).and_then(|value| value.as_str()).map(str::to_string);
        let kind = match text("kind").as_deref() {
            None => Kind::Skills,
            Some("claude-plugin") => Kind::ClaudePlugin,
            Some("codex-plugin") => Kind::CodexPlugin,
            Some(other) => bail!("{name}: `{other}` is not a kind"),
        };
        let aliases = item
            .get("aliases")
            .and_then(|list| list.as_array())
            .map(|list| list.iter().filter_map(|alias| alias.as_str().map(str::to_string)).collect())
            .unwrap_or_default();
        agents.push(Agent {
            name: name.to_string(),
            display: text("display").unwrap_or_else(|| name.to_string()),
            project: text("project").with_context(|| format!("{name} has no `project`"))?,
            global: text("global"),
            kind,
            aliases,
        });
    }
    Ok(agents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_has_both_plugin_kinds_and_the_skills_sh_folders() {
        let claude = find("claude").unwrap();
        assert_eq!((claude.name.as_str(), claude.kind), ("claude-code", Kind::ClaudePlugin));
        assert_eq!(find("claude-code").unwrap(), claude);
        assert_eq!(find("codex").unwrap().kind, Kind::CodexPlugin);
        let opencode = find("opencode").unwrap();
        assert_eq!(opencode.kind, Kind::Skills);
        let home = Path::new("/h");
        assert_eq!(opencode.skill_dir(Path::new("/p"), false).unwrap(), Path::new("/p/.opencode/skills"), "the override for opencode is gone");
        assert_eq!(opencode.skill_dir(home, true).unwrap(), Path::new("/h/.config/opencode/skills"));
        assert_eq!(claude.skill_dir(home, true).unwrap(), Path::new("/h/.claude/skills"));
        for name in ["amp", "cline", "cursor", "droid", "gemini-cli", "github-copilot", "kilo", "kimi-code-cli", "pi", "warp", "zed"] {
            assert!(find(name).is_ok(), "{name} is not in the table");
        }
        assert!(find("nope").is_err());
        assert_eq!(all().iter().filter(|agent| agent.kind != Kind::Skills).count(), 2);
    }
}
