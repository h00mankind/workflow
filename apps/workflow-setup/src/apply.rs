//! Makes the disk match `manifest.toml`.
//!
//! `plan` compares the manifest to the disk and returns the drift. `apply`
//! makes a backup, then writes. It keeps a record of what it installed and
//! never changes or removes an item it does not own. An item is owned when
//! the record has it, or when it is a link into the library.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};

use crate::{
    adapters::Harness,
    library,
    manifest::{self, Manifest, Scope},
};

/// The folders the engine works with. Tests give it folders of their own.
pub struct Env {
    pub library: PathBuf,
    pub home: PathBuf,
    /// Where the ownership record and the backups are kept.
    pub data: PathBuf,
}

impl Env {
    pub fn new(library: PathBuf) -> Env {
        let home = crate::scan::home();
        Env { library, data: home.join("Library/Application Support/workflow-setup"), home }
    }

    fn record_file(&self) -> PathBuf {
        self.data.join("installed.json")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    /// Nothing is at `path`.
    Link { path: PathBuf, target: PathBuf },
    /// An owned link is at `path` and points at `from`.
    Relink { path: PathBuf, from: PathBuf, target: PathBuf },
    /// An owned link is at `path` and the manifest does not assign it.
    Remove { path: PathBuf, from: PathBuf },
    /// Something not owned is at `path`. `apply` leaves it. `foreign_link`
    /// is its target when it is a link, which `apply` with `adopt` replaces.
    Conflict { path: PathBuf, target: PathBuf, foreign_link: Option<PathBuf> },
}

impl Change {
    pub fn path(&self) -> &Path {
        match self {
            Change::Link { path, .. } | Change::Relink { path, .. } | Change::Remove { path, .. } | Change::Conflict { path, .. } => path,
        }
    }
}

/// The drift of one scope.
#[derive(Debug)]
pub struct Plan {
    pub scope: Scope,
    pub changes: Vec<Change>,
    /// Names that two assigned origins supply. The first origin wins.
    pub collisions: Vec<library::Collision>,
    /// Every path the manifest assigns in this scope, with its link target.
    desired: BTreeMap<PathBuf, PathBuf>,
}

impl Plan {
    pub fn in_sync(&self) -> bool {
        self.changes.is_empty()
    }
}

/// Scope root to the links installed there, each with its target.
#[derive(Default, Serialize, Deserialize)]
struct Record {
    scopes: BTreeMap<PathBuf, BTreeMap<PathBuf, PathBuf>>,
}

impl Record {
    fn load(env: &Env) -> Record {
        fs::read_to_string(env.record_file())
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    fn save(&self, env: &Env) -> Result<()> {
        fs::create_dir_all(&env.data)?;
        let path = env.record_file();
        fs::write(&path, serde_json::to_string_pretty(self)?).with_context(|| format!("cannot write {}", path.display()))
    }
}

/// The drift of every scope: each scope in the manifest, and each scope that
/// left the manifest and still has installed items.
pub fn plan_all(env: &Env) -> Result<Vec<Plan>> {
    let manifest = manifest::load(&env.library, &env.home)?;
    let record = Record::load(env);
    let mut scopes = manifest.scopes.clone();
    for root in record.scopes.keys() {
        if !scopes.iter().any(|scope| scope.root == *root) {
            scopes.push(Scope::empty(root.clone(), &env.home));
        }
    }
    scopes.iter().map(|scope| plan(env, &manifest, &record, scope)).collect()
}

fn plan(env: &Env, manifest: &Manifest, record: &Record, scope: &Scope) -> Result<Plan> {
    let skills = library::skills(&env.library);
    let mut desired: BTreeMap<PathBuf, PathBuf> = BTreeMap::new();
    let mut collisions = Vec::new();
    for &harness in &scope.harnesses {
        let Some(dir) = harness.skill_dir(scope.global) else { continue };
        let (winners, collided) = library::resolve(&manifest.items(scope, harness, &skills)?);
        for (name, item) in winners {
            desired.insert(scope.root.join(dir).join(name), item.dir);
        }
        for collision in collided {
            if !collisions.contains(&collision) {
                collisions.push(collision);
            }
        }
    }

    let owned = record.scopes.get(&scope.root);
    let is_owned = |path: &Path, link: &Path| link.starts_with(&env.library) || owned.is_some_and(|owned| owned.get(path).is_some_and(|target| target == link));

    let mut changes = Vec::new();
    for (path, target) in &desired {
        let Ok(meta) = path.symlink_metadata() else {
            changes.push(Change::Link { path: path.clone(), target: target.clone() });
            continue;
        };
        let link = meta.is_symlink().then(|| fs::read_link(path).unwrap_or_default());
        match link {
            Some(link) if link == *target => {}
            Some(link) if is_owned(path, &link) => changes.push(Change::Relink { path: path.clone(), from: link, target: target.clone() }),
            foreign_link => changes.push(Change::Conflict { path: path.clone(), target: target.clone(), foreign_link }),
        }
    }
    // Owned links that the manifest does not assign, in every folder an adapter installs into.
    for dir in Harness::ALL.iter().filter_map(|harness| harness.skill_dir(scope.global)) {
        for entry in fs::read_dir(scope.root.join(dir)).into_iter().flatten().flatten() {
            let path = entry.path();
            if desired.contains_key(&path) {
                continue;
            }
            if let Ok(link) = fs::read_link(&path) {
                if is_owned(&path, &link) && !changes.iter().any(|change| change.path() == path) {
                    changes.push(Change::Remove { path, from: link });
                }
            }
        }
    }
    changes.sort_by(|a, b| a.path().cmp(b.path()));
    Ok(Plan { scope: scope.clone(), changes, collisions, desired })
}

/// What `apply` did in one scope.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Applied {
    pub written: usize,
    pub removed: usize,
    pub left: usize,
}

/// Makes the disk match the plan. With `adopt`, a link that something else
/// made is replaced too. A real file or folder is never replaced. Returns
/// what it did and the backup folder, if it changed the disk.
pub fn apply(env: &Env, plans: &[Plan], adopt: bool) -> Result<(Vec<Applied>, Option<PathBuf>)> {
    // (path, the link target it had) for each link that goes away.
    let mut backup: Vec<(&Path, &Path)> = Vec::new();
    for change in plans.iter().flat_map(|plan| &plan.changes) {
        match change {
            Change::Relink { path, from, .. } | Change::Remove { path, from } => backup.push((path, from)),
            Change::Conflict { path, foreign_link: Some(from), .. } if adopt => backup.push((path, from)),
            _ => {}
        }
    }
    let backup_dir = if backup.is_empty() {
        None
    } else {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map(|since| since.as_millis()).unwrap_or_default();
        let dir = env.data.join("backups").join(stamp.to_string());
        fs::create_dir_all(&dir)?;
        fs::write(dir.join("links.json"), serde_json::to_string_pretty(&backup)?).with_context(|| format!("cannot write the backup in {}", dir.display()))?;
        Some(dir)
    };

    let mut record = Record::load(env);
    let mut results = Vec::new();
    for plan in plans {
        let mut applied = Applied::default();
        // Start from what is right on disk now; each write below adds to it.
        let mut owned = plan.desired.clone();
        for change in &plan.changes {
            match change {
                Change::Link { path, target } => {
                    link(path, target)?;
                    applied.written += 1;
                }
                Change::Relink { path, target, .. } => {
                    relink(path, target)?;
                    applied.written += 1;
                }
                Change::Conflict { path, target, foreign_link: Some(_) } if adopt => {
                    relink(path, target)?;
                    applied.written += 1;
                }
                Change::Remove { path, .. } => {
                    fs::remove_file(path).with_context(|| format!("cannot unlink {}", path.display()))?;
                    applied.removed += 1;
                }
                Change::Conflict { path, .. } => {
                    owned.remove(path);
                    applied.left += 1;
                }
            }
        }
        if owned.is_empty() {
            record.scopes.remove(&plan.scope.root);
        } else {
            record.scopes.insert(plan.scope.root.clone(), owned);
        }
        results.push(applied);
    }
    record.save(env)?;
    Ok((results, backup_dir))
}

fn relink(path: &Path, target: &Path) -> Result<()> {
    fs::remove_file(path).with_context(|| format!("cannot unlink {}", path.display()))?;
    link(path, target)
}

fn link(path: &Path, target: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("cannot create {}", parent.display()))?;
    }
    std::os::unix::fs::symlink(target, path).with_context(|| format!("cannot link {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFEST: &str = r#"
[bundles]
core = ["h00man/*"]
pstack = ["poteto/*"]

[global]
bundles = ["core"]
claude = ["pstack"]

[project."~/app"]
bundles = ["core"]
harnesses = ["claude", "codex", "opencode"]
"#;

    fn fixture(test: &str) -> Env {
        let root = std::env::temp_dir().join(format!("workflow-setup-apply-{test}-{}", std::process::id()));
        fs::remove_dir_all(&root).ok();
        let env = Env { library: root.join("library"), home: root.join("home"), data: root.join("data") };
        for skill in ["h00man/plan", "h00man/unslop", "poteto/unslop", "poteto/swarm"] {
            let dir = env.library.join("skills").join(skill);
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join("SKILL.md"), skill).unwrap();
        }
        fs::create_dir_all(env.home.join("app")).unwrap();
        fs::write(env.library.join(manifest::FILE), MANIFEST).unwrap();
        env
    }

    fn run(env: &Env, adopt: bool) -> Vec<Applied> {
        apply(env, &plan_all(env).unwrap(), adopt).unwrap().0
    }

    fn target(env: &Env, path: &str) -> String {
        let link = fs::read_link(env.home.join(path)).unwrap_or_default();
        link.strip_prefix(&env.library).unwrap_or(&link).to_string_lossy().into_owned()
    }

    fn backups(env: &Env) -> usize {
        fs::read_dir(env.data.join("backups")).map(Iterator::count).unwrap_or(0)
    }

    #[test]
    fn apply_installs_the_manifest_and_a_second_run_changes_nothing() {
        let env = fixture("twice");
        assert_eq!(run(&env, false), [Applied { written: 5, removed: 0, left: 0 }, Applied { written: 6, removed: 0, left: 0 }]);
        // Codex gets the bundles for every harness. Claude Code gets pstack too.
        assert_eq!(target(&env, ".agents/skills/plan"), "skills/h00man/plan");
        assert!(!env.home.join(".agents/skills/swarm").exists());
        assert_eq!(target(&env, ".claude/skills/swarm"), "skills/poteto/swarm");
        // h00man wins the name that both origins have.
        assert_eq!(target(&env, ".claude/skills/unslop"), "skills/h00man/unslop");
        assert_eq!(target(&env, "app/.opencode/skill/plan"), "skills/h00man/plan");
        assert_eq!(plan_all(&env).unwrap()[0].collisions.len(), 1);

        assert!(plan_all(&env).unwrap().iter().all(Plan::in_sync));
        assert_eq!(run(&env, false), [Applied::default(), Applied::default()]);
        assert_eq!(backups(&env), 0, "a run that changed nothing made a backup");
    }

    #[test]
    fn apply_removes_only_what_it_owns() {
        let env = fixture("owned");
        let skills = env.home.join(".claude/skills");
        fs::create_dir_all(skills.join("by-hand")).unwrap();
        std::os::unix::fs::symlink("/somewhere/else", skills.join("from-a-plugin")).unwrap();
        run(&env, false);

        // The manifest stops assigning pstack and the project.
        fs::write(env.library.join(manifest::FILE), "[bundles]\ncore = [\"h00man/plan\"]\n[global]\nbundles = [\"core\"]\n").unwrap();
        assert_eq!(run(&env, false), [Applied { written: 0, removed: 3, left: 0 }, Applied { written: 0, removed: 6, left: 0 }]);
        assert!(skills.join("plan").exists());
        assert!(skills.join("unslop").symlink_metadata().is_err());
        assert!(skills.join("swarm").symlink_metadata().is_err());
        assert!(skills.join("by-hand").is_dir(), "apply removed a folder it does not own");
        assert!(skills.join("from-a-plugin").symlink_metadata().is_ok(), "apply removed a link it does not own");
        assert!(env.home.join("app/.claude/skills/plan").symlink_metadata().is_err());
        assert_eq!(backups(&env), 1);
        assert!(plan_all(&env).unwrap().iter().all(Plan::in_sync));
    }

    #[test]
    fn a_link_into_the_library_is_owned_and_a_foreign_item_is_a_conflict() {
        let env = fixture("conflict");
        let skills = env.home.join(".claude/skills");
        fs::create_dir_all(skills.join("plan")).unwrap();
        // A link from the layout before the move: into the library, and broken.
        std::os::unix::fs::symlink(env.library.join("skills/unslop"), skills.join("unslop")).unwrap();
        std::os::unix::fs::symlink(env.library.join("skills/retired"), skills.join("retired")).unwrap();
        std::os::unix::fs::symlink("/a/plugin/swarm", skills.join("swarm")).unwrap();

        let global = &plan_all(&env).unwrap()[0];
        let conflicts: Vec<_> = global.changes.iter().filter(|change| matches!(change, Change::Conflict { .. })).map(|change| change.path().to_path_buf()).collect();
        assert_eq!(conflicts, [skills.join("plan"), skills.join("swarm")]);

        assert_eq!(run(&env, false)[0], Applied { written: 3, removed: 1, left: 2 });
        assert_eq!(target(&env, ".claude/skills/unslop"), "skills/h00man/unslop");
        assert!(skills.join("retired").symlink_metadata().is_err());
        assert_eq!(target(&env, ".claude/skills/swarm"), "/a/plugin/swarm");

        // With adopt, the foreign link is replaced. The real folder stays.
        assert_eq!(run(&env, true)[0], Applied { written: 1, removed: 0, left: 1 });
        assert_eq!(target(&env, ".claude/skills/swarm"), "skills/poteto/swarm");
        assert!(skills.join("plan").is_dir() && !skills.join("plan").is_symlink());
        let mut all: Vec<_> = fs::read_dir(env.data.join("backups")).unwrap().flatten().map(|entry| entry.path()).collect();
        all.sort();
        let backup = all.pop().unwrap();
        assert!(fs::read_to_string(backup.join("links.json")).unwrap().contains("/a/plugin/swarm"));
    }

    #[test]
    fn a_bundle_that_names_a_missing_item_is_an_error() {
        let env = fixture("missing");
        fs::write(env.library.join(manifest::FILE), "[bundles]\ncore = [\"h00man/nope\"]\n[global]\nbundles = [\"core\"]\n").unwrap();
        assert!(plan_all(&env).is_err());
        assert!(!env.home.join(".claude").exists());
    }
}
