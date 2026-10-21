//! What the app remembers between runs.

use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::scan::home;

const MAX_RECENT: usize = 8;

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    /// The folder whose `skills/` and `agents/` the app installs from.
    pub library: Option<PathBuf>,
    /// Projects opened, newest first.
    #[serde(default)]
    pub recent: Vec<PathBuf>,
}

impl Config {
    fn path() -> PathBuf {
        home().join("Library/Application Support/workflow-setup/config.json")
    }

    pub fn load() -> Self {
        fs::read_to_string(Self::path())
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn opened(&mut self, project: PathBuf) {
        self.recent.retain(|recent| *recent != project);
        self.recent.insert(0, project);
        self.recent.truncate(MAX_RECENT);
    }
}
