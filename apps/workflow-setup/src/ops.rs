//! Every change the app makes to disk. Each function does one change and
//! refuses when the disk is not in the state the caller saw.

use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context as _, Result, bail};
use serde_json::{Map, Value};

use crate::scan::strip_json_comments;

/// Writes `text` to `path`, unless the file changed after the app read `seen`.
pub fn save(path: &Path, seen: &str, text: &str) -> Result<()> {
    let on_disk = fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))?;
    if on_disk != seen {
        bail!("{} changed on disk. Rescan, then edit again.", file_name(path));
    }
    fs::write(path, text).with_context(|| format!("cannot write {}", path.display()))
}

pub fn create(path: &Path, text: &str) -> Result<()> {
    if path.symlink_metadata().is_ok() {
        bail!("{} is there already", path.display());
    }
    make_parent(path)?;
    fs::write(path, text).with_context(|| format!("cannot write {}", path.display()))
}

/// Links `link` to `target`, as `npx skills add` does for a project install.
pub fn link(target: &Path, link: &Path) -> Result<()> {
    if link.symlink_metadata().is_ok() {
        bail!("{} is there already", link.display());
    }
    make_parent(link)?;
    std::os::unix::fs::symlink(target, link).with_context(|| format!("cannot link {}", link.display()))
}

pub fn copy(from: &Path, to: &Path) -> Result<()> {
    if to.symlink_metadata().is_ok() {
        bail!("{} is there already", to.display());
    }
    make_parent(to)?;
    fs::copy(from, to)
        .map(drop)
        .with_context(|| format!("cannot copy to {}", to.display()))
}

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

/// Adds server `name` to the map under `key` in a JSON config file, which it
/// creates when missing.
pub fn mcp_add(path: &Path, key: &str, name: &str, config: Value) -> Result<()> {
    let mut root = if path.exists() { read_plain_json(path)? } else { Value::Object(Map::new()) };
    let servers = root
        .as_object_mut()
        .with_context(|| format!("{} is not a JSON object", path.display()))?
        .entry(key)
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .with_context(|| format!("`{key}` in {} is not an object", path.display()))?;
    if servers.contains_key(name) {
        bail!("{} has a server named {name} already", file_name(path));
    }
    servers.insert(name.to_string(), config);
    make_parent(path)?;
    write_json(path, &root)
}

pub fn mcp_remove(path: &Path, key: &str, name: &str) -> Result<()> {
    let mut root = read_plain_json(path)?;
    let removed = root
        .get_mut(key)
        .and_then(Value::as_object_mut)
        .and_then(|servers| servers.shift_remove(name));
    if removed.is_none() {
        bail!("{} has no server named {name}", file_name(path));
    }
    write_json(path, &root)
}

/// A server config from what the user typed: a URL, or a command line.
pub fn mcp_config(command_line: &str) -> Result<Value> {
    let mut words = command_line.split_whitespace().map(str::to_string);
    let Some(first) = words.next() else {
        bail!("Type a command or a URL");
    };
    Ok(if first.starts_with("http://") || first.starts_with("https://") {
        serde_json::json!({ "type": "http", "url": first })
    } else {
        serde_json::json!({ "command": first, "args": words.collect::<Vec<_>>() })
    })
}

fn read_plain_json(path: &Path) -> Result<Value> {
    let text = fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))?;
    if strip_json_comments(&text) != text {
        bail!("{} has comments, which a rewrite would drop. Edit it by hand.", file_name(path));
    }
    serde_json::from_str(&text).with_context(|| format!("{} is not valid JSON", path.display()))
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    let mut text = serde_json::to_string_pretty(value)?;
    text.push('\n');
    fs::write(path, text).with_context(|| format!("cannot write {}", path.display()))
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
