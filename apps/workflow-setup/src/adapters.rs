//! One adapter per harness: where the harness reads each kind of item.
//!
//! The neutral layout is the `.agents/` convention. Codex reads it. Claude
//! Code and opencode do not, so each gets links in its own folder.

use anyhow::{Result, bail};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Harness {
    Claude,
    Codex,
    OpenCode,
}

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

    /// The folder that holds one link per skill, from the scope's root: the
    /// home folder for global scope, the project folder for project scope.
    /// `None` when this adapter does not install skills at that scope.
    pub fn skill_dir(self, global: bool) -> Option<&'static str> {
        match self {
            Harness::Claude => Some(".claude/skills"),
            Harness::Codex => Some(".agents/skills"),
            Harness::OpenCode if global => None,
            Harness::OpenCode => Some(".opencode/skill"),
        }
    }
}
