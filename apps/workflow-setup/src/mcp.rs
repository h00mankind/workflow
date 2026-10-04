//! MCP definitions in `mcp/<name>.toml`. Secrets are environment variable names.

use std::{collections::BTreeMap, fs, path::Path};

use anyhow::{Context as _, Result, bail};
use toml_edit::DocumentMut;

#[derive(Clone, Debug)]
pub enum Server {
    Stdio { command: String, args: Vec<String>, env: Vec<String> },
    Http { url: String, bearer_token_env: Option<String> },
}

pub fn load(library: &Path) -> Result<BTreeMap<String, Server>> {
    let dir = library.join("mcp");
    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(error) => return Err(error).with_context(|| format!("cannot read {}", dir.display())),
    };
    let mut servers = BTreeMap::new();
    for entry in entries {
        let path = entry?.path();
        if !path.is_file() || path.extension().is_none_or(|extension| extension != "toml") {
            continue;
        }
        let name = path.file_stem().context("an MCP definition has no name")?.to_string_lossy().into_owned();
        let text = fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?;
        let server = parse(&text).with_context(|| format!("invalid MCP definition {}", path.display()))?;
        servers.insert(name, server);
    }
    Ok(servers)
}

fn parse(text: &str) -> Result<Server> {
    let doc: DocumentMut = text.parse().context("not valid TOML")?;
    let string = |key: &str| -> Result<Option<String>> {
        doc.get(key).map(|item| item.as_str().map(str::to_string).with_context(|| format!("`{key}` must be text"))).transpose()
    };
    let list = |key: &str| -> Result<Vec<String>> {
        let Some(item) = doc.get(key) else { return Ok(Vec::new()) };
        item.as_array().with_context(|| format!("`{key}` must be a list"))?.iter()
            .map(|value| value.as_str().map(str::to_string).with_context(|| format!("`{key}` must contain text"))).collect()
    };
    if doc.contains_key("command") == doc.contains_key("url") {
        bail!("use exactly one of `command` and `url`");
    }
    let allowed: &[&str] = if doc.contains_key("command") { &["command", "args", "env"] } else { &["url", "bearer_token_env"] };
    for (key, _) in doc.iter() {
        if !allowed.contains(&key) {
            bail!("`{key}` is not a field for this server");
        }
    }
    if let Some(command) = string("command")? {
        if command.trim().is_empty() {
            bail!("`command` must not be empty");
        }
        let env = list("env")?;
        for name in &env {
            variable(name)?;
        }
        Ok(Server::Stdio { command, args: list("args")?, env })
    } else {
        let url = string("url")?.unwrap();
        if url.trim().is_empty() {
            bail!("`url` must not be empty");
        }
        let bearer_token_env = string("bearer_token_env")?;
        if let Some(name) = &bearer_token_env {
            variable(name)?;
        }
        Ok(Server::Http { url, bearer_token_env })
    }
}

fn variable(name: &str) -> Result<()> {
    let mut chars = name.chars();
    if !chars.next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_') || !chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
        bail!("secrets must be environment variable names, not values");
    }
    Ok(())
}
