//! One adapter per harness: how the harness receives a plugin.
//!
//! Claude Code loads a plugin from a link in its skills folder, and gives
//! its skills the prefix `<plugin>:`. Codex installs a plugin from this
//! library as a marketplace, and a config file enables it for a scope.
//! opencode has no plugins, so it gets one link per skill, with no prefix.
//! The MCP adapter holds the server config formats and Claude user commands.

use anyhow::{Result, bail};

pub mod mcp;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Harness {
    Claude,
    Codex,
    OpenCode,
}

/// Every folder, from a scope's root, in which the engine makes links or
/// made them before. `apply` removes its own links that are not assigned.
pub const LINK_DIRS: [&str; 3] = [".claude/skills", ".agents/skills", ".opencode/skill"];

impl Harness {
    pub const ALL: [Harness; 3] = [Harness::Claude, Harness::Codex, Harness::OpenCode];
    /// The harnesses a scope installs for when the manifest names none.
    pub const DEFAULT: [Harness; 2] = [Harness::Claude, Harness::Codex];

    pub fn key(self) -> &'static str {
        match self {
            Harness::Claude => "claude",
            Harness::Codex => "codex",
            Harness::OpenCode => "opencode",
        }
    }

    pub fn from_key(key: &str) -> Result<Harness> {
        match Harness::ALL.into_iter().find(|harness| harness.key() == key) {
            Some(harness) => Ok(harness),
            None => bail!("`{key}` is not a harness. Use claude, codex, or opencode."),
        }
    }

    /// False when this adapter installs for projects only.
    pub fn installs_globally(self) -> bool {
        self != Harness::OpenCode
    }

    /// The folder that holds one link per plugin, from the scope's root.
    pub fn plugin_link_dir(self) -> Option<&'static str> {
        (self == Harness::Claude).then_some(".claude/skills")
    }

    /// The folder that holds one link per skill, from the scope's root.
    pub fn skill_link_dir(self) -> Option<&'static str> {
        (self == Harness::OpenCode).then_some(".opencode/skill")
    }
}
