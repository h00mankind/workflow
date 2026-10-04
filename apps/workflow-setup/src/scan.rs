//! Reads what a project gives to its coding agents. Read-only.

use std::{
    collections::BTreeMap,
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
};

use serde_json::Value;

use crate::library;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    ClaudeCode,
    DotAgents,
    OpenCode,
    Codex,
    Cursor,
    Gemini,
    Copilot,
    VsCode,
}

impl Tool {
    pub fn label(self) -> &'static str {
        match self {
            Tool::ClaudeCode => "Claude Code",
            Tool::DotAgents => ".agents",
            Tool::OpenCode => "opencode",
            Tool::Codex => "Codex",
            Tool::Cursor => "Cursor",
            Tool::Gemini => "Gemini",
            Tool::Copilot => "Copilot",
            Tool::VsCode => "VS Code",
        }
    }
}

/// Project folders that hold installed skills. The first three are the ones
/// this app installs into.
pub const SKILL_DIRS: &[(&str, Tool)] = &[
    (".claude/skills", Tool::ClaudeCode),
    (".agents/skills", Tool::DotAgents),
    (".opencode/skill", Tool::OpenCode),
    (".opencode/skills", Tool::OpenCode),
    (".codex/skills", Tool::Codex),
    (".cursor/skills", Tool::Cursor),
    (".gemini/skills", Tool::Gemini),
    (".github/skills", Tool::Copilot),
];
pub const SKILL_TARGETS: usize = 3;

const AGENT_DIRS: &[(&str, Tool)] = &[
    (".claude/agents", Tool::ClaudeCode),
    (".opencode/agent", Tool::OpenCode),
    (".opencode/agents", Tool::OpenCode),
    (".codex/agents", Tool::Codex),
    (".cursor/agents", Tool::Cursor),
    (".github/agents", Tool::Copilot),
];

/// Library variant folder, the project folder it installs into, and its tool.
const AGENT_TARGETS: &[(&str, &str, Tool)] = &[
    ("claude", ".claude/agents", Tool::ClaudeCode),
    ("opencode", ".opencode/agent", Tool::OpenCode),
];

/// Config files that keep servers as `{ name: { command, args, env } }`
/// under one key. This app can add to them and remove from them.
pub const MCP_TARGETS: &[(&str, Tool, &str)] = &[
    (".mcp.json", Tool::ClaudeCode, "mcpServers"),
    (".cursor/mcp.json", Tool::Cursor, "mcpServers"),
    (".gemini/settings.json", Tool::Gemini, "mcpServers"),
    (".vscode/mcp.json", Tool::VsCode, "servers"),
];
const OPENCODE_CONFIGS: &[&str] = &["opencode.json", "opencode.jsonc", ".opencode/opencode.json"];
const CODEX_CONFIG: &str = ".codex/config.toml";
const CLAUDE_SETTINGS: &[&str] = &[".claude/settings.json", ".claude/settings.local.json"];

const INSTRUCTION_FILES: &[(&str, &str)] = &[
    ("CLAUDE.md", "Claude Code"),
    ("CLAUDE.local.md", "Claude Code"),
    ("AGENTS.md", "Codex + others"),
    ("AGENTS.override.md", "Codex + others"),
];
const SKIP_DIRS: &[&str] = &[
    "node_modules", "dist", "build", "out", "target", "vendor", "venv", "__pycache__",
];
const MAX_WALK_DEPTH: usize = 4;
const SECRET_NAMES: &[&str] = &[
    "token", "secret", "key", "password", "passwd", "auth", "bearer", "credential",
];
const REDACTED: &str = "•••";

pub struct Install {
    pub tool: Tool,
    /// Project-relative, for display.
    pub rel: String,
    pub abs: PathBuf,
    pub link_target: Option<PathBuf>,
    pub broken: bool,
    hash: Option<u64>,
}

pub struct Skill {
    pub name: String,
    pub description: String,
    pub content: String,
    /// The `SKILL.md` that `content` came from.
    pub file: Option<PathBuf>,
    pub files: Vec<String>,
    pub installs: Vec<Install>,
    /// This skill's folder in the library.
    pub library: Option<PathBuf>,
    /// The library origin that supplies it, such as `h00man`.
    pub origin: String,
    pub differs: bool,
    /// Where `skills-lock.json` says it came from.
    pub source: String,
}

pub struct Agent {
    pub name: String,
    pub description: String,
    pub model: String,
    pub tools: String,
    pub tool: Tool,
    /// The file `content` came from: the installed file, or the library's.
    pub file: PathBuf,
    /// Where the project keeps it, or would after an install.
    pub target: PathBuf,
    pub rel: String,
    pub installed: bool,
    pub link_target: Option<PathBuf>,
    pub library: Option<PathBuf>,
    pub content: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Enabled,
    /// In `.mcp.json`, but not approved in Claude Code yet.
    Pending,
    Disabled,
}

impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Status::Enabled => "enabled",
            Status::Pending => "needs approval",
            Status::Disabled => "disabled",
        }
    }
}

pub struct McpDef {
    pub tool: Tool,
    pub scope: &'static str,
    /// For display.
    pub file: String,
    pub path: PathBuf,
    pub key: &'static str,
    /// The app can remove this definition, and copy it to other tools.
    pub editable: bool,
    pub transport: String,
    pub command: String,
    pub url: String,
    pub env_keys: Vec<String>,
    pub header_keys: Vec<String>,
    pub status: Status,
    /// The config as written, secrets included. Never shown.
    pub raw: Value,
}

pub struct McpServer {
    pub name: String,
    pub defs: Vec<McpDef>,
}

pub struct Instruction {
    pub rel: String,
    pub abs: PathBuf,
    pub tool: &'static str,
    pub scope: &'static str,
    pub link_target: Option<PathBuf>,
    pub imports: Vec<String>,
    pub content: String,
}

pub struct Project {
    pub root: PathBuf,
    pub name: String,
    /// The home folder: what every project gets.
    pub global: bool,
    pub skills: Vec<Skill>,
    pub mcp: Vec<McpServer>,
    pub agents: Vec<Agent>,
    pub instructions: Vec<Instruction>,
}

pub fn scan(root: &Path, library: Option<&Path>) -> Project {
    let global = root == home();
    Project {
        root: root.to_path_buf(),
        name: if global { "Global".to_string() } else { file_name(root) },
        global,
        skills: scan_skills(root, library),
        mcp: scan_mcp(root),
        agents: scan_agents(root, library),
        instructions: if global { global_instructions(root) } else { scan_instructions(root) },
    }
}

// ---------- file boundary ----------

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn sorted_dir(dir: &Path) -> Vec<PathBuf> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .map(|entries| entries.flatten().map(|entry| entry.path()).collect())
        .unwrap_or_default();
    entries.sort();
    entries
}

fn link_target(path: &Path) -> Option<PathBuf> {
    fs::read_link(path).ok()
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

fn hash_of(text: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

pub fn read_json(path: &Path) -> Option<Value> {
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(&strip_json_comments(&text)).ok()
}

pub fn strip_json_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut in_string = false;
    while let Some(c) = chars.next() {
        if in_string {
            out.push(c);
            if c == '\\' {
                out.extend(chars.next());
            } else if c == '"' {
                in_string = false;
            }
        } else if c == '"' {
            in_string = true;
            out.push(c);
        } else if c == '/' && chars.peek() == Some(&'/') {
            for next in chars.by_ref() {
                if next == '\n' {
                    out.push('\n');
                    break;
                }
            }
        } else if c == '/' && chars.peek() == Some(&'*') {
            chars.next();
            let mut prev = ' ';
            for next in chars.by_ref() {
                if prev == '*' && next == '/' {
                    break;
                }
                prev = next;
            }
        } else {
            out.push(c);
        }
    }
    out
}

// ---------- parsing ----------

fn frontmatter(text: &str) -> BTreeMap<String, String> {
    let mut meta = BTreeMap::new();
    let mut lines = text.lines();
    if lines.next().map(str::trim_end) != Some("---") {
        return meta;
    }
    let mut key: Option<String> = None;
    for line in lines {
        if line.trim_end() == "---" {
            break;
        }
        let indented = line.starts_with([' ', '\t']);
        match line.split_once(':') {
            Some((name, value)) if !indented && is_key(name) => {
                let value = value.trim();
                let value = if matches!(value, ">" | "|" | ">-" | "|-" | ">+" | "|+") {
                    ""
                } else {
                    unquote(value)
                };
                meta.insert(name.to_string(), value.to_string());
                key = Some(name.to_string());
            }
            _ if indented && !line.trim().is_empty() => {
                if let Some(value) = key.as_ref().and_then(|key| meta.get_mut(key)) {
                    if !value.is_empty() {
                        value.push(' ');
                    }
                    value.push_str(line.trim());
                }
            }
            _ => {}
        }
    }
    meta
}

fn is_key(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn unquote(value: &str) -> &str {
    for quote in ['"', '\''] {
        if value.len() >= 2 && value.starts_with(quote) && value.ends_with(quote) {
            return &value[1..value.len() - 1];
        }
    }
    value
}

/// Reads only the `[mcp_servers.*]` tables of a Codex `config.toml`.
fn parse_codex_mcp(text: &str) -> serde_json::Map<String, Value> {
    let mut servers = serde_json::Map::new();
    let mut current: Option<(String, Option<String>)> = None;
    for line in text.lines().map(str::trim) {
        if let Some(section) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            current = section.strip_prefix("mcp_servers.").map(|rest| {
                let (name, sub) = match rest.split_once('.') {
                    Some((name, sub)) => (name, Some(sub.to_string())),
                    None => (rest, None),
                };
                (name.trim_matches('"').to_string(), sub)
            });
            continue;
        }
        let Some((name, sub)) = &current else { continue };
        let Some((key, value)) = line.split_once('=') else { continue };
        let (key, value) = (key.trim(), value.trim());
        if !is_key(key) {
            continue;
        }
        let value = if value.starts_with('{') {
            // Inline table: keep the keys only. The values can be secrets.
            Value::Object(
                value
                    .trim_matches(['{', '}'])
                    .split(',')
                    .filter_map(|pair| pair.split_once('='))
                    .map(|(key, _)| (key.trim().to_string(), Value::Null))
                    .collect(),
            )
        } else {
            serde_json::from_str(value).unwrap_or_else(|_| Value::String(value.to_string()))
        };
        let server = servers
            .entry(name.clone())
            .or_insert_with(|| Value::Object(Default::default()));
        let Some(mut table) = server.as_object_mut() else { continue };
        if let Some(sub) = sub {
            let nested = table
                .entry(sub.clone())
                .or_insert_with(|| Value::Object(Default::default()));
            match nested.as_object_mut() {
                Some(nested) => table = nested,
                None => continue,
            }
        }
        table.insert(key.to_string(), value);
    }
    servers
}

fn is_secret_name(name: &str) -> bool {
    let name = name.to_lowercase();
    SECRET_NAMES.iter().any(|word| name.contains(word))
}

fn redact_url(url: &str) -> String {
    let Some((base, query)) = url.split_once('?') else {
        return url.to_string();
    };
    let keys: Vec<String> = query
        .split('&')
        .map(|pair| format!("{}={REDACTED}", pair.split('=').next().unwrap_or_default()))
        .collect();
    format!("{base}?{}", keys.join("&"))
}

fn redact_args(args: &[String]) -> String {
    let mut out = Vec::with_capacity(args.len());
    for (ix, arg) in args.iter().enumerate() {
        let after_secret_flag = ix > 0
            && args[ix - 1].starts_with('-')
            && is_secret_name(&args[ix - 1])
            && !arg.starts_with('-');
        out.push(if after_secret_flag {
            REDACTED.to_string()
        } else if let Some((name, _)) = arg.split_once('=').filter(|(name, _)| is_secret_name(name)) {
            format!("{name}={REDACTED}")
        } else if arg.starts_with("http://") || arg.starts_with("https://") {
            redact_url(arg)
        } else {
            arg.clone()
        });
    }
    out.join(" ")
}

fn strings(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|item| item.as_str().map(str::to_string))
            .collect(),
        Some(Value::String(text)) => vec![text.clone()],
        _ => Vec::new(),
    }
}

fn keys(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_object)
        .map(|map| map.keys().cloned().collect())
        .unwrap_or_default()
}

// ---------- skills ----------

fn list_files(dir: &Path, prefix: &str, depth: usize, out: &mut Vec<String>) {
    for path in sorted_dir(dir) {
        let name = file_name(&path);
        if name == ".DS_Store" {
            continue;
        }
        if !path.is_dir() {
            out.push(format!("{prefix}{name}"));
        } else if depth < 2 {
            list_files(&path, &format!("{prefix}{name}/"), depth + 1, out);
        } else {
            out.push(format!("{prefix}{name}/"));
        }
    }
}

fn skill_entry(by_name: &mut BTreeMap<String, Skill>, name: String) -> &mut Skill {
    by_name.entry(name.clone()).or_insert_with(|| Skill {
        name,
        description: String::new(),
        content: String::new(),
        file: None,
        files: Vec::new(),
        installs: Vec::new(),
        library: None,
        origin: String::new(),
        differs: false,
        source: String::new(),
    })
}

/// The first `SKILL.md` found for a skill gives its text.
fn fill_skill(skill: &mut Skill, dir: &Path, text: String) {
    if skill.file.is_some() {
        return;
    }
    skill.description = frontmatter(&text).remove("description").unwrap_or_default();
    skill.file = Some(dir.join("SKILL.md"));
    list_files(dir, "", 0, &mut skill.files);
    skill.content = text;
}

fn scan_skills(root: &Path, library: Option<&Path>) -> Vec<Skill> {
    let lock = read_json(&root.join("skills-lock.json"));
    let mut by_name: BTreeMap<String, Skill> = BTreeMap::new();

    for &(rel_dir, tool) in SKILL_DIRS {
        for abs in sorted_dir(&root.join(rel_dir)) {
            let link = link_target(&abs);
            let text = fs::read_to_string(abs.join("SKILL.md")).ok();
            let broken = link.is_some() && !abs.exists();
            if text.is_none() && !broken {
                continue;
            }
            let skill = skill_entry(&mut by_name, file_name(&abs));
            skill.installs.push(Install {
                tool,
                rel: relative(root, &abs),
                abs: abs.clone(),
                link_target: link,
                broken,
                hash: text.as_deref().map(hash_of),
            });
            if let Some(text) = text {
                fill_skill(skill, &abs, text);
            }
        }
    }

    if let Some(library) = library {
        // Of two origins with one name, the library installs the winner.
        let (winners, _) = library::resolve(&library::skills(library));
        for item in winners.into_values() {
            let Ok(text) = fs::read_to_string(item.dir.join("SKILL.md")) else { continue };
            let skill = skill_entry(&mut by_name, item.name);
            skill.library = Some(item.dir.clone());
            skill.origin = item.origin;
            fill_skill(skill, &item.dir, text);
        }
    }

    let mut skills: Vec<Skill> = by_name.into_values().collect();
    for skill in &mut skills {
        let mut hashes: Vec<u64> = skill.installs.iter().filter_map(|i| i.hash).collect();
        hashes.dedup();
        skill.differs = hashes.len() > 1;
        skill.source = lock
            .as_ref()
            .and_then(|lock| lock["skills"][&skill.name]["source"].as_str())
            .unwrap_or_default()
            .to_string();
    }
    skills
}

// ---------- agents ----------

fn walk_files(dir: &Path, extensions: &[&str], depth: usize, out: &mut Vec<PathBuf>) {
    for path in sorted_dir(dir) {
        if path.is_dir() {
            if depth < 3 {
                walk_files(&path, extensions, depth + 1, out);
            }
        } else if path
            .extension()
            .is_some_and(|ext| extensions.iter().any(|wanted| ext == *wanted))
        {
            out.push(path);
        }
    }
}

fn read_agent(root: &Path, file: &Path, target: &Path, tool: Tool, installed: bool) -> Option<Agent> {
    let content = fs::read_to_string(file).ok()?;
    let mut meta = frontmatter(&content);
    let mut take = |key: &str| meta.remove(key).unwrap_or_default();
    let name = take("name");
    Some(Agent {
        name: if name.is_empty() {
            file.file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
                .unwrap_or_default()
        } else {
            name
        },
        description: take("description"),
        model: take("model"),
        tools: take("tools"),
        tool,
        file: file.to_path_buf(),
        target: target.to_path_buf(),
        rel: relative(root, target),
        installed,
        link_target: link_target(target),
        library: None,
        content,
    })
}

fn scan_agents(root: &Path, library: Option<&Path>) -> Vec<Agent> {
    let mut agents = Vec::new();
    for &(rel_dir, tool) in AGENT_DIRS {
        let mut files = Vec::new();
        walk_files(&root.join(rel_dir), &["md", "toml"], 0, &mut files);
        agents.extend(
            files
                .iter()
                .filter_map(|file| read_agent(root, file, file, tool, true)),
        );
    }

    // Library layout: agents/<origin>/<name>/<variant>/<file>.md, one variant per tool.
    for agent_dir in library.map(library::agents).unwrap_or_default().into_iter().map(|agent| agent.dir) {
        for &(variant, rel_dir, tool) in AGENT_TARGETS {
            let mut files = Vec::new();
            walk_files(&agent_dir.join(variant), &["md"], 3, &mut files);
            for file in files {
                let target = root.join(rel_dir).join(file_name(&file));
                match agents.iter_mut().find(|agent| agent.target == target) {
                    Some(agent) => agent.library = Some(file),
                    None => {
                        if let Some(mut agent) = read_agent(root, &file, &target, tool, false) {
                            agent.library = Some(file);
                            agents.push(agent);
                        }
                    }
                }
            }
        }
    }
    agents.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.rel.cmp(&b.rel)));
    agents
}

// ---------- MCP ----------

struct Origin {
    tool: Tool,
    scope: &'static str,
    file: String,
    path: PathBuf,
    key: &'static str,
    editable: bool,
}

fn to_def(config: &Value, origin: Origin) -> McpDef {
    let command: Vec<String> = match &config["command"] {
        Value::Array(_) => strings(config.get("command")),
        command => strings(Some(command))
            .into_iter()
            .chain(strings(config.get("args")))
            .collect(),
    };
    let url = ["url", "serverUrl", "httpUrl"]
        .iter()
        .find_map(|key| config[*key].as_str())
        .unwrap_or_default();
    let off = config["enabled"] == Value::Bool(false) || config["disabled"] == Value::Bool(true);
    McpDef {
        tool: origin.tool,
        scope: origin.scope,
        file: origin.file,
        path: origin.path,
        key: origin.key,
        editable: origin.editable,
        transport: match config["type"].as_str() {
            Some(kind) => kind.to_string(),
            None if url.is_empty() => "stdio".to_string(),
            None => "http".to_string(),
        },
        command: redact_args(&command),
        url: redact_url(url),
        env_keys: keys(config.get("env").or_else(|| config.get("environment"))),
        header_keys: keys(config.get("headers").or_else(|| config.get("http_headers"))),
        status: if off { Status::Disabled } else { Status::Enabled },
        raw: config.clone(),
    }
}

fn string_set(sources: &[&Value], key: &str) -> Vec<String> {
    sources
        .iter()
        .flat_map(|source| strings(source.get(key)))
        .collect()
}

fn scan_mcp(root: &Path) -> Vec<McpServer> {
    let mut found: Vec<(String, McpDef)> = Vec::new();
    let mut add = |map: Option<&serde_json::Map<String, Value>>, origin: &dyn Fn() -> Origin| {
        for (name, config) in map.into_iter().flatten() {
            if config.is_object() {
                found.push((name.clone(), to_def(config, origin())));
            }
        }
    };

    for &(file, tool, key) in MCP_TARGETS {
        let path = root.join(file);
        let Ok(text) = fs::read_to_string(&path) else { continue };
        let stripped = strip_json_comments(&text);
        let Ok(config) = serde_json::from_str::<Value>(&stripped) else { continue };
        // Writing the file back drops comments, so a file with comments is read-only here.
        let editable = stripped == text;
        add(config[key].as_object(), &|| Origin {
            tool,
            scope: "project",
            file: file.to_string(),
            path: path.clone(),
            key,
            editable,
        });
    }
    for &file in OPENCODE_CONFIGS {
        let path = root.join(file);
        let Some(config) = read_json(&path) else { continue };
        add(config["mcp"].as_object(), &|| Origin {
            tool: Tool::OpenCode,
            scope: "project",
            file: file.to_string(),
            path: path.clone(),
            key: "mcp",
            editable: false,
        });
    }
    let codex = root.join(CODEX_CONFIG);
    if let Ok(text) = fs::read_to_string(&codex) {
        add(Some(&parse_codex_mcp(&text)), &|| Origin {
            tool: Tool::Codex,
            scope: "project",
            file: CODEX_CONFIG.to_string(),
            path: codex.clone(),
            key: "mcp_servers",
            editable: false,
        });
    }

    // Claude Code keeps local-scope servers and approvals in ~/.claude.json,
    // keyed by project path.
    let user_file = home().join(".claude.json");
    let user = read_json(&user_file).unwrap_or(Value::Null);
    let local = &user["projects"][root.to_string_lossy().as_ref()];
    add(local["mcpServers"].as_object(), &|| Origin {
        tool: Tool::ClaudeCode,
        scope: "local",
        file: "~/.claude.json".to_string(),
        path: user_file.clone(),
        key: "mcpServers",
        editable: false,
    });

    let settings: Vec<Value> = CLAUDE_SETTINGS
        .iter()
        .filter_map(|file| read_json(&root.join(file)))
        .collect();
    let sources: Vec<&Value> = std::iter::once(local).chain(&settings).collect();
    let approved = string_set(&sources, "enabledMcpjsonServers");
    let rejected = string_set(&sources, "disabledMcpjsonServers");
    let switched_off = string_set(&sources, "disabledMcpServers");
    let approve_all = settings
        .iter()
        .any(|s| s["enableAllProjectMcpServers"] == Value::Bool(true));

    for (name, def) in &mut found {
        if def.tool != Tool::ClaudeCode {
            continue;
        }
        if switched_off.contains(name) {
            def.status = Status::Disabled;
        } else if def.scope != "project" {
            continue;
        } else if rejected.contains(name) {
            def.status = Status::Disabled;
        } else if !approve_all && !approved.contains(name) {
            def.status = Status::Pending;
        }
    }

    let mut by_name: BTreeMap<String, Vec<McpDef>> = BTreeMap::new();
    for (name, def) in found {
        by_name.entry(name).or_default().push(def);
    }
    by_name
        .into_iter()
        .map(|(name, defs)| McpServer { name, defs })
        .collect()
}

pub fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default()
}

// ---------- instructions ----------

fn imports(content: &str) -> Vec<String> {
    content
        .split_whitespace()
        .filter_map(|word| word.strip_prefix('@'))
        .filter(|path| path.ends_with(".md"))
        .map(str::to_string)
        .collect()
}

fn read_instruction(root: &Path, abs: &Path, tool: &'static str, scope: &'static str) -> Option<Instruction> {
    let content = fs::read_to_string(abs).ok()?;
    Some(Instruction {
        rel: relative(root, abs),
        abs: abs.to_path_buf(),
        tool,
        scope,
        link_target: link_target(abs),
        imports: imports(&content),
        content,
    })
}

fn walk_instructions(root: &Path, dir: &Path, depth: usize, out: &mut Vec<Instruction>) {
    let scope = if depth == 0 { "root" } else { "nested" };
    for path in sorted_dir(dir) {
        let name = file_name(&path);
        if let Some(&(_, tool)) = INSTRUCTION_FILES.iter().find(|(file, _)| *file == name) {
            out.extend(read_instruction(root, &path, tool, scope));
        } else if name == ".claude" {
            out.extend(read_instruction(root, &path.join("CLAUDE.md"), "Claude Code", scope));
            let mut rules = Vec::new();
            walk_files(&path.join("rules"), &["md"], 0, &mut rules);
            out.extend(
                rules
                    .iter()
                    .filter_map(|rule| read_instruction(root, rule, "Claude Code", "rule")),
            );
        } else if depth < MAX_WALK_DEPTH
            && !name.starts_with('.')
            && !SKIP_DIRS.contains(&name.as_str())
            && path.is_dir()
            && link_target(&path).is_none()
        {
            walk_instructions(root, &path, depth + 1, out);
        }
    }
}

/// The instruction files each harness reads for every project.
fn global_instructions(home: &Path) -> Vec<Instruction> {
    [(".claude/CLAUDE.md", "Claude Code"), (".codex/AGENTS.md", "Codex"), ("AGENTS.md", "Codex + others")]
        .into_iter()
        .filter_map(|(file, tool)| read_instruction(home, &home.join(file), tool, "global"))
        .collect()
}

fn scan_instructions(root: &Path) -> Vec<Instruction> {
    let mut found = Vec::new();
    walk_instructions(root, root, 0, &mut found);
    found.sort_by(|a, b| a.rel.cmp(&b.rel));
    found
}
