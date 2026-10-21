//! Takes things out of a project in a way the user can undo.

use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context as _, Result};

/// Takes `path` out of the project. A symlink is unlinked and its target
/// stays. A real file or folder moves to the Trash of `home`, so it can come back.
pub fn remove(path: &Path, home: &Path) -> Result<Removed> {
    let meta = path
        .symlink_metadata()
        .with_context(|| format!("{} is not there", path.display()))?;
    if meta.is_symlink() {
        fs::remove_file(path).with_context(|| format!("cannot unlink {}", path.display()))?;
        return Ok(Removed::Unlinked);
    }
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_secs())
        .unwrap_or_default();
    let trashed = home
        .join(".Trash")
        .join(format!("{} {stamp}", file_name(path)));
    make_parent(&trashed)?;
    fs::rename(path, &trashed).with_context(|| format!("cannot move {} to the Trash", path.display()))?;
    Ok(Removed::Trashed)
}

pub enum Removed {
    Unlinked,
    Trashed,
}

fn make_parent(path: &Path) -> Result<()> {
    match path.parent() {
        Some(parent) => fs::create_dir_all(parent).with_context(|| format!("cannot create {}", parent.display())),
        None => Ok(()),
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}
