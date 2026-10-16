//! What is on the disk in the skill folders of one scope, whoever put it
//! there. `list` shows it. `remove --foreign` acts on it.

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    agents::{self, Agent},
    apply::{Env, Record},
};

/// One skill folder of a scope and the agents that read it.
pub struct Folder {
    pub dir: PathBuf,
    /// Empty for a folder that only an older version of the engine used.
    pub agents: Vec<&'static Agent>,
    pub items: Vec<Item>,
}

pub struct Item {
    pub name: String,
    pub path: PathBuf,
    /// The link target. None: a real file or folder.
    pub link: Option<PathBuf>,
    /// True when workflow-setup made it.
    pub owned: bool,
    /// True for a link whose target does not exist.
    pub broken: bool,
    /// Where the skills.sh lock file says a foreign item came from.
    pub source: Option<String>,
}

/// The skill folders of the scope at `root`, each with what is in it. With
/// `only`, the folders that those agents read.
pub fn folders(env: &Env, root: &Path, global: bool, only: &[&'static Agent]) -> Vec<Folder> {
    let record = Record::load(env);
    let lock = match global {
        true => env.home.join(".agents/.skill-lock.json"),
        false => root.join("skills-lock.json"),
    };
    let lock: serde_json::Value = fs::read_to_string(lock).ok().and_then(|text| serde_json::from_str(&text).ok()).unwrap_or_default();

    let mut folders: Vec<Folder> = Vec::new();
    for agent in agents::all() {
        let Some(dir) = agent.skill_dir(root, global) else { continue };
        match folders.iter_mut().find(|folder| folder.dir == dir) {
            Some(folder) => folder.agents.push(agent),
            None => folders.push(Folder { dir, agents: vec![agent], items: Vec::new() }),
        }
    }
    if !global {
        folders.extend(agents::LEGACY_DIRS.iter().map(|dir| Folder { dir: root.join(dir), agents: Vec::new(), items: Vec::new() }));
    }
    folders.retain(|folder| only.is_empty() || folder.agents.iter().any(|agent| only.contains(agent)));

    for folder in &mut folders {
        let mut paths: Vec<PathBuf> = fs::read_dir(&folder.dir).into_iter().flatten().flatten().map(|entry| entry.path()).collect();
        paths.sort();
        for path in paths {
            let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
            if name.starts_with('.') {
                continue;
            }
            let link = fs::read_link(&path).ok();
            let owned = link.as_ref().is_some_and(|link| record.owns(env, root, &path, link));
            let source = match owned {
                true => None,
                false => lock["skills"][&name]["source"].as_str().map(str::to_string),
            };
            let broken = link.is_some() && !path.exists();
            folder.items.push(Item { name, path, link, owned, broken, source });
        }
    }
    folders.retain(|folder| !folder.items.is_empty());
    folders
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::symlink;

    use super::*;
    use crate::apply::tests::{fixture, run};

    #[test]
    fn list_tells_owned_links_from_foreign_items_and_gives_the_lock_source() {
        let env = fixture("list");
        run(&env, false);
        let app = env.home.join("app");
        let skills = app.join(".agents/skills");
        fs::create_dir_all(skills.join("find-skills")).unwrap();
        symlink("/elsewhere/grill", skills.join("grill")).unwrap();
        fs::write(app.join("skills-lock.json"), r#"{"skills":{"find-skills":{"source":"vercel-labs/skills"}}}"#).unwrap();

        let all = folders(&env, &app, false, &[]);
        let folder = all.iter().find(|folder| folder.dir == skills).expect("the shared folder is missing");
        assert!(folder.agents.contains(&agents::find("codex").unwrap()) && folder.agents.contains(&agents::find("cursor").unwrap()));
        let item = |name: &str| folder.items.iter().find(|item| item.name == name).unwrap();
        assert!(!item("find-skills").owned && item("find-skills").link.is_none());
        assert_eq!(item("find-skills").source.as_deref(), Some("vercel-labs/skills"));
        assert!(!item("grill").owned && item("grill").source.is_none() && item("grill").broken);
        assert!(!item("plan").broken);
        assert!(item("swarm").owned && item("plan").owned);

        let claude = all.iter().find(|folder| folder.dir == app.join(".claude/skills")).unwrap();
        let names: Vec<(&str, bool)> = claude.items.iter().map(|item| (item.name.as_str(), item.owned)).collect();
        assert_eq!(names, [("h00man", true), ("pstack", true)]);

        // With an agent, only the folders it reads.
        let cursor = folders(&env, &app, false, &[agents::find("cursor").unwrap()]);
        assert_eq!(cursor.iter().map(|folder| folder.dir.clone()).collect::<Vec<_>>(), [skills]);

        // At global scope the lock file is in ~/.agents.
        let global = env.home.join(".agents/skills/find-skills");
        fs::create_dir_all(&global).unwrap();
        fs::write(env.home.join(".agents/.skill-lock.json"), r#"{"skills":{"find-skills":{"source":"vercel-labs/skills"}}}"#).unwrap();
        let all = folders(&env, &env.home, true, &[agents::find("codex").unwrap()]);
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].items[0].source.as_deref(), Some("vercel-labs/skills"));
    }
}
