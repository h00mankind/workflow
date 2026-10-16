//! MCP config formats for Claude Code, Codex, and the neutral project file.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context as _, Result, bail};
use serde_json::{Value, json};
use toml_edit::{DocumentMut, Item, Table};

use crate::{agents::Kind, manifest::Scope, mcp::Server, scan::strip_json_comments};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Json,
    ClaudeUser,
    Codex,
}

pub struct Config {
    pub file: PathBuf,
    pub format: Format,
    pub assigned: bool,
}

pub fn configs(scope: &Scope) -> Vec<Config> {
    let claude = scope.has(Kind::ClaudePlugin);
    let codex = scope.has(Kind::CodexPlugin);
    let mut configs = vec![
        Config {
            file: scope.root.join(if scope.global { ".claude.json" } else { ".mcp.json" }),
            format: if scope.global { Format::ClaudeUser } else { Format::Json },
            assigned: claude,
        },
        Config { file: scope.root.join(".codex/config.toml"), format: Format::Codex, assigned: codex },
    ];
    if !scope.global {
        configs.push(Config { file: scope.root.join(".agents/mcp.json"), format: Format::Json, assigned: claude || codex });
    }
    configs
}

impl Format {
    pub fn set(self, file: &Path, name: &str, entry: Option<&Value>, exists: bool, claude: &dyn Fn(&[&str]) -> Result<()>) -> Result<()> {
        if self == Format::ClaudeUser {
            if exists {
                claude(&["mcp", "remove", "--scope", "user", name])?;
            }
            if let Some(entry) = entry {
                claude(&["mcp", "add-json", "--scope", "user", name, &entry.to_string()])?;
            }
            Ok(())
        } else if entry.is_some() || exists {
            self.write(file, name, entry)
        } else {
            Ok(())
        }
    }

    pub fn entry(self, server: &Server) -> Value {
        match (self, server) {
            (Format::Codex, Server::Stdio { command, args, env }) => json!({ "command": command, "args": args, "env_vars": env }),
            (Format::Codex, Server::Http { url, bearer_token_env }) => {
                let mut entry = json!({ "url": url });
                if let Some(name) = bearer_token_env {
                    entry["bearer_token_env_var"] = json!(name);
                }
                entry
            }
            (_, Server::Stdio { command, args, env }) => {
                let env: serde_json::Map<String, Value> = env.iter().map(|name| (name.clone(), json!(format!("${{{name}}}")))).collect();
                json!({ "command": command, "args": args, "env": env })
            }
            (_, Server::Http { url, bearer_token_env }) => {
                let mut entry = json!({ "type": "http", "url": url });
                if let Some(name) = bearer_token_env {
                    entry["headers"] = json!({ "Authorization": format!("Bearer ${{{name}}}") });
                }
                entry
            }
        }
    }

    /// The entries on disk and a reason this file must not be rewritten.
    pub fn read(self, file: &Path) -> Result<(BTreeMap<String, Value>, Option<String>)> {
        let text = read(file)?;
        if self == Format::Codex {
            let doc: DocumentMut = text.parse().with_context(|| format!("{} is not valid TOML", file.display()))?;
            let Some(all) = doc.get("mcp_servers") else { return Ok((BTreeMap::new(), None)) };
            let Some(all) = all.as_table() else {
                return Ok((BTreeMap::new(), Some("`mcp_servers` is not a table".to_string())));
            };
            let entries = all.iter().map(|(name, item)| (name.to_string(), item_json(item))).collect();
            return Ok((entries, None));
        }
        let stripped = strip_json_comments(&text);
        if self == Format::Json && stripped != text {
            return Ok((BTreeMap::new(), Some("the file has comments".to_string())));
        }
        let doc = json_document(&stripped, file)?;
        let entries = doc.get("mcpServers").map(|all| {
            all.as_object().with_context(|| format!("`mcpServers` in {} is not an object", file.display()))
        }).transpose()?;
        let mut entries: BTreeMap<String, Value> = entries.into_iter().flatten().map(|(name, entry)| (name.clone(), entry.clone())).collect();
        if self == Format::ClaudeUser {
            // Claude's command line adds this default to stdio entries.
            for entry in entries.values_mut() {
                if entry.get("type").and_then(Value::as_str) == Some("stdio") {
                    entry.as_object_mut().unwrap().remove("type");
                }
            }
        }
        Ok((entries, None))
    }

    pub fn write(self, file: &Path, name: &str, entry: Option<&Value>) -> Result<()> {
        if self == Format::ClaudeUser {
            bail!("Claude user MCP entries must be written by claude");
        }
        let text = read(file)?;
        let text = if self == Format::Codex {
            let mut doc: DocumentMut = text.parse().with_context(|| format!("{} is not valid TOML", file.display()))?;
            match entry {
                Some(entry) => {
                    let all = doc.entry("mcp_servers").or_insert(toml_edit::table());
                    let all = all.as_table_mut().with_context(|| format!("`mcp_servers` in {} is not a table", file.display()))?;
                    all.set_implicit(true);
                    let mut table = Table::new();
                    for (key, value) in entry.as_object().unwrap() {
                        table.insert(key, json_toml(value));
                    }
                    all.insert(name, Item::Table(table));
                }
                None => {
                    if let Some(all) = doc.get_mut("mcp_servers").and_then(Item::as_table_mut) {
                        all.remove(name);
                        if all.is_empty() {
                            doc.remove("mcp_servers");
                        }
                    }
                }
            }
            doc.to_string()
        } else {
            if strip_json_comments(&text) != text {
                bail!("{} has comments; leave it unchanged", file.display());
            }
            let mut doc = json_document(&text, file)?;
            let all = doc.as_object_mut().unwrap().entry("mcpServers").or_insert(json!({}));
            let all = all.as_object_mut().with_context(|| format!("`mcpServers` in {} is not an object", file.display()))?;
            match entry {
                Some(entry) => { all.insert(name.to_string(), entry.clone()); }
                None => { all.remove(name); }
            }
            format!("{}\n", serde_json::to_string_pretty(&doc)?)
        };
        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(file, text).with_context(|| format!("cannot write {}", file.display()))
    }
}

fn read(file: &Path) -> Result<String> {
    match fs::read_to_string(file) {
        Ok(text) => Ok(text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(error).with_context(|| format!("cannot read {}", file.display())),
    }
}

fn json_document(text: &str, file: &Path) -> Result<Value> {
    let doc: Value = if text.is_empty() { json!({}) } else { serde_json::from_str(text).with_context(|| format!("{} is not valid JSON", file.display()))? };
    if !doc.is_object() {
        bail!("{} is not an object", file.display());
    }
    Ok(doc)
}

fn item_json(item: &Item) -> Value {
    if let Some(table) = item.as_table_like() {
        return Value::Object(table.iter().map(|(key, item)| (key.to_string(), item_json(item))).collect());
    }
    if let Some(tables) = item.as_array_of_tables() {
        return Value::Array(tables.iter().map(|table| item_json(&Item::Table(table.clone()))).collect());
    }
    match item.as_value() {
        Some(value) => value_json(value),
        None => Value::Null,
    }
}

fn value_json(value: &toml_edit::Value) -> Value {
    match value {
        toml_edit::Value::String(value) => json!(value.value()),
        toml_edit::Value::Integer(value) => json!(value.value()),
        toml_edit::Value::Float(value) => json!(value.value()),
        toml_edit::Value::Boolean(value) => json!(value.value()),
        toml_edit::Value::Datetime(value) => json!(value.value().to_string()),
        toml_edit::Value::Array(values) => Value::Array(values.iter().map(value_json).collect()),
        toml_edit::Value::InlineTable(table) => Value::Object(table.iter().map(|(key, value)| (key.to_string(), value_json(value))).collect()),
    }
}

fn json_toml(value: &Value) -> Item {
    match value {
        Value::String(value) => toml_edit::value(value),
        Value::Array(values) => {
            let mut array = toml_edit::Array::new();
            for value in values {
                array.push(value.as_str().unwrap());
            }
            toml_edit::value(array)
        }
        _ => unreachable!("MCP definitions contain only text and lists"),
    }
}
