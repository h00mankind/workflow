//! The library as an MCP server over stdio. Uses the same engine as the CLI.
//! Protocol revision 2025-11-25, with newline-delimited JSON-RPC messages.

use std::io::{self, BufRead, Write};

use anyhow::{Context as _, Result, bail};
use serde_json::{Value, json};

use crate::{apply::{self, Env, Plan}, cli, library, manifest, mcp, sources};

const PROTOCOL: &str = "2025-11-25";

pub fn run(env: &Env) -> Result<()> {
    serve(env, io::stdin().lock(), io::stdout().lock())
}

fn serve(env: &Env, input: impl BufRead, mut output: impl Write) -> Result<()> {
    for line in input.lines() {
        let line = line.context("cannot read stdin")?;
        let response = match serde_json::from_str(&line) {
            Ok(request) => handle(env, request),
            Err(_) => Some(error(Value::Null, -32700, "Parse error")),
        };
        if let Some(response) = response {
            serde_json::to_writer(&mut output, &response)?;
            output.write_all(b"\n")?;
            output.flush()?;
        }
    }
    Ok(())
}

/// Handles one request. Notifications return nothing and never run tools.
pub(crate) fn handle(env: &Env, request: Value) -> Option<Value> {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let method = request.get("method").and_then(Value::as_str);
    if !request.is_object() || request["jsonrpc"] != "2.0" || method.is_none() {
        return Some(error(Value::Null, -32600, "Invalid request"));
    }
    if request.get("id").is_none() {
        return None;
    }
    if !id.is_string() && !id.is_i64() && !id.is_u64() {
        return Some(error(Value::Null, -32600, "Invalid request id"));
    }
    let result = match method.unwrap() {
        "initialize" => {
            let params = &request["params"];
            if !params["protocolVersion"].is_string() || !params["capabilities"].is_object()
                || !params["clientInfo"]["name"].is_string() || !params["clientInfo"]["version"].is_string() {
                return Some(error(id, -32602, "Invalid initialize parameters"));
            }
            json!({
                "protocolVersion": PROTOCOL,
                "capabilities": { "tools": { "listChanged": false } },
                "serverInfo": { "name": "workflow-setup", "version": env!("CARGO_PKG_VERSION") },
            })
        }
        "ping" => json!({}),
        "tools/list" => json!({ "tools": tools() }),
        "tools/call" => match call(env, &request["params"]) {
            Ok(text) => json!({ "content": [{ "type": "text", "text": text }], "isError": false }),
            Err(error) => json!({ "content": [{ "type": "text", "text": format!("{error:#}") }], "isError": true }),
        },
        _ => return Some(error(id, -32601, "Method not found")),
    };
    Some(json!({ "jsonrpc": "2.0", "id": id, "result": result }))
}

fn error(id: Value, code: i32, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

fn tools() -> Vec<Value> {
    let scope = json!({ "type": "string", "description": "global or a project path from manifest.toml. Accepts ~/ or an absolute path." });
    let on = json!({ "type": "boolean" });
    [
        ("library", "List plugins, their skills, shared names, and MCP definitions.", json!({}), vec![]),
        ("manifest", "List every scope, its harnesses, plugins per harness, and MCP servers.", json!({}), vec![]),
        ("status", "Show drift in the same groups as workflow-setup status. Without scope: every scope.", json!({ "scope": scope }), vec![]),
        ("sources", "List source repositories, kinds, and pinned commits. Updates stay with the human.", json!({}), vec![]),
        ("assign_plugin", "Assign or unassign a known plugin for every harness of an existing scope. Edits only manifest.toml.",
            json!({ "scope": scope, "plugin": { "type": "string" }, "on": on }), vec!["scope", "plugin", "on"]),
        ("assign_mcp", "Assign or unassign a known MCP server in an existing scope. Edits only manifest.toml.",
            json!({ "scope": scope, "server": { "type": "string" }, "on": on }), vec!["scope", "server", "on"]),
        ("apply", "Return the plan by default. Set dry_run to false to apply it. Never adopts foreign items. Without scope: every scope.",
            json!({ "scope": scope, "dry_run": { "type": "boolean", "default": true } }), vec![]),
    ].into_iter().map(|(name, description, properties, required)| json!({
        "name": name,
        "description": description,
        "inputSchema": { "type": "object", "properties": properties, "required": required, "additionalProperties": false },
    })).collect()
}

fn call(env: &Env, params: &Value) -> Result<String> {
    let name = params["name"].as_str().context("name must be text")?;
    let tool = tools().into_iter().find(|tool| tool["name"] == name).with_context(|| format!("unknown tool `{name}`"))?;
    let empty = json!({});
    let args = params.get("arguments").unwrap_or(&empty);
    let args = args.as_object().context("arguments must be an object")?;
    let schema = &tool["inputSchema"];
    let properties = schema["properties"].as_object().unwrap();
    for key in schema["required"].as_array().unwrap().iter().map(|key| key.as_str().unwrap()) {
        if !args.contains_key(key) {
            bail!("missing argument `{key}`");
        }
    }
    for (key, value) in args {
        let property = properties.get(key).with_context(|| format!("unknown argument `{key}`"))?;
        match property["type"].as_str().unwrap() {
            "string" if !value.is_string() => bail!("`{key}` must be text"),
            "boolean" if !value.is_boolean() => bail!("`{key}` must be true or false"),
            _ => {}
        }
    }
    let scope = args.get("scope").and_then(Value::as_str);
    match name {
        "library" => list_library(env),
        "manifest" => list_manifest(env),
        "sources" => {
            let mut text = String::new();
            for source in sources::load(&env.library)? {
                let kind = if source.plugin { "plugin" } else { "skills" };
                text.push_str(&format!("{}: {kind}\n  repository: {}\n  pin: {}\n", source.name, source.repository, source.commit.as_deref().unwrap_or("none")));
            }
            Ok(if text.is_empty() { "No sources.\n".into() } else { text })
        }
        "assign_plugin" | "assign_mcp" => {
            let scope = scope.unwrap();
            let root = scope_root(env, scope)?;
            let on = args["on"].as_bool().unwrap();
            let item = if name == "assign_plugin" { args["plugin"].as_str().unwrap() } else { args["server"].as_str().unwrap() };
            if name == "assign_plugin" {
                if !library::plugins(&env.library).iter().any(|plugin| plugin == item) {
                    bail!("the library has no plugin `{item}`");
                }
                manifest::assign(&env.library, &env.home, &root, item, on)?;
            } else {
                if !mcp::load(&env.library)?.contains_key(item) {
                    bail!("the library has no MCP server `{item}`");
                }
                manifest::assign_mcp(&env.library, &env.home, &root, item, on)?;
            }
            let verb = if on { "assigned" } else { "unassigned" };
            Ok(format!("{scope}: {item} {verb}. Only manifest.toml was written.\n"))
        }
        "status" | "apply" => {
            let plans = plans(env, scope)?;
            if name == "status" || args.get("dry_run").and_then(Value::as_bool).unwrap_or(true) {
                let mut text = if name == "apply" { "Dry run. Nothing was written.\n".into() } else { String::new() };
                for plan in &plans {
                    text.push_str(&cli::format_plan(env, plan));
                }
                return Ok(text);
            }
            let (results, backup) = apply::apply(env, &plans, false)?;
            let mut text = String::new();
            for (plan, result) in plans.iter().zip(results) {
                text.push_str(&format!("{}: {} written, {} removed, {} left\n", plan.scope.label, result.written, result.removed, result.left));
            }
            text.push_str(&format!("backup: {}\n", backup.as_ref().map(|path| apply::tilde(&env.home, path)).unwrap_or_else(|| "none".into())));
            Ok(text)
        }
        _ => unreachable!("the tool list and handlers must agree"),
    }
}

fn scope_root(env: &Env, name: &str) -> Result<std::path::PathBuf> {
    let manifest = manifest::load(&env.library, &env.home)?;
    if name == manifest::GLOBAL {
        return Ok(env.home.clone());
    }
    let path = name.strip_prefix("~/").map(|rest| env.home.join(rest)).unwrap_or_else(|| name.into());
    manifest.scopes.into_iter().find(|scope| !scope.global && (scope.label == name || scope.root == path))
        .map(|scope| scope.root).with_context(|| format!("{} has no scope `{name}`", manifest::FILE))
}

fn plans(env: &Env, scope: Option<&str>) -> Result<Vec<Plan>> {
    let mut plans = apply::plan_all(env)?;
    if let Some(name) = scope {
        if !plans.iter().any(|plan| cli::is_scope(env, plan, name)) {
            bail!("{} has no scope `{name}`", manifest::FILE);
        }
        plans.retain(|plan| cli::is_scope(env, plan, name));
    }
    Ok(plans)
}

fn list_library(env: &Env) -> Result<String> {
    let skills = library::skills(&env.library);
    let (_, collisions) = library::resolve(&skills);
    let mut text = String::new();
    for plugin in library::plugins(&env.library) {
        let names: Vec<_> = skills.iter().filter(|skill| skill.origin == plugin).map(|skill| skill.name.as_str()).collect();
        text.push_str(&format!("plugins/{plugin} ({}): {}\n", names.len(), names.join(" ")));
    }
    for collision in collisions {
        text.push_str(&format!("same name: {} is in {} and {}. Without a prefix, {} wins.\n", collision.name, collision.winner, collision.losers.join(", "), collision.winner));
    }
    let servers = mcp::load(&env.library)?;
    if servers.is_empty() {
        text.push_str("No MCP definitions.\n");
    }
    for (name, server) in servers {
        text.push_str(&format!("mcp/{name}.toml:\n"));
        match server {
            mcp::Server::Stdio { command, args, env } => {
                text.push_str(&format!("  command: {command}\n  args: {}\n  env names: {}\n", json!(args), json!(env)));
            }
            mcp::Server::Http { url, bearer_token_env } => {
                text.push_str(&format!("  url: {url}\n  bearer token env: {}\n", bearer_token_env.as_deref().unwrap_or("none")));
            }
        }
    }
    Ok(text)
}

fn list_manifest(env: &Env) -> Result<String> {
    let mut text = String::new();
    for scope in manifest::load(&env.library, &env.home)?.scopes {
        let harnesses: Vec<_> = scope.harnesses.iter().map(|harness| harness.key()).collect();
        text.push_str(&format!("{}:\n  harnesses: {}\n", scope.label, harnesses.join(" ")));
        for harness in scope.harnesses.iter().copied() {
            text.push_str(&format!("  {} plugins: {}\n", harness.key(), scope.plugins_for(harness).join(" ")));
        }
        text.push_str(&format!("  mcp: {}\n", scope.mcp.join(" ")));
    }
    Ok(if text.is_empty() { "No scopes in manifest.toml.\n".into() } else { text })
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, fs, path::Path};

    use super::*;
    use crate::{adapters::mcp::Format, apply::tests::{fixture, mcp_fixture}};

    fn request(env: &Env, method: &str, params: Value) -> Value {
        handle(env, json!({ "jsonrpc": "2.0", "id": "test", "method": method, "params": params })).unwrap()
    }

    fn call_tool(env: &Env, name: &str, args: Value) -> Value {
        let response = request(env, "tools/call", json!({ "name": name, "arguments": args }));
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], "test");
        assert!(response.get("error").is_none(), "{response}");
        let result = response["result"].clone();
        assert_eq!(result["content"].as_array().unwrap().len(), 1);
        assert_eq!(result["content"][0]["type"], "text");
        result
    }

    fn text(result: &Value) -> &str {
        result["content"][0]["text"].as_str().unwrap()
    }

    fn ok(env: &Env, name: &str, args: Value) -> String {
        let result = call_tool(env, name, args);
        assert_eq!(result["isError"], false, "{result}");
        text(&result).to_string()
    }

    // Includes empty folders and link targets; never follows a foreign link.
    fn snapshot(root: &Path) -> BTreeMap<std::path::PathBuf, Vec<u8>> {
        fn walk(root: &Path, path: &Path, out: &mut BTreeMap<std::path::PathBuf, Vec<u8>>) {
            let meta = path.symlink_metadata().unwrap();
            let value = if meta.is_symlink() {
                fs::read_link(path).unwrap().as_os_str().as_encoded_bytes().to_vec()
            } else if meta.is_dir() {
                Vec::new()
            } else {
                fs::read(path).unwrap()
            };
            out.insert(path.strip_prefix(root).unwrap().to_path_buf(), value);
            if meta.is_dir() {
                for entry in fs::read_dir(path).unwrap() {
                    walk(root, &entry.unwrap().path(), out);
                }
            }
        }
        let mut out = BTreeMap::new();
        walk(root, root, &mut out);
        out
    }

    #[test]
    fn protocol_returns_initialization_and_all_tool_schemas() {
        let env = fixture("server-protocol");
        let response = request(&env, "initialize", json!({ "protocolVersion": PROTOCOL, "capabilities": {}, "clientInfo": { "name": "test", "version": "1" } }));
        assert_eq!(response["result"], json!({ "protocolVersion": PROTOCOL, "capabilities": { "tools": { "listChanged": false } }, "serverInfo": { "name": "workflow-setup", "version": env!("CARGO_PKG_VERSION") } }));
        let fallback = request(&env, "initialize", json!({ "protocolVersion": "unknown", "capabilities": {}, "clientInfo": { "name": "test", "version": "1" } }));
        assert_eq!(fallback["result"]["protocolVersion"], PROTOCOL);
        assert_eq!(request(&env, "initialize", json!({}))["error"]["code"], -32602);
        assert_eq!(request(&env, "ping", json!({}))["result"], json!({}));
        let response = request(&env, "tools/list", json!({}));
        let tools = response["result"]["tools"].as_array().unwrap();
        let names: Vec<_> = tools.iter().map(|tool| tool["name"].as_str().unwrap()).collect();
        assert_eq!(names, ["library", "manifest", "status", "sources", "assign_plugin", "assign_mcp", "apply"]);
        for tool in tools {
            let schema = &tool["inputSchema"];
            assert_eq!(schema["type"], "object");
            assert!(schema["properties"].is_object());
            assert_eq!(schema["additionalProperties"], false);
            for key in schema["required"].as_array().unwrap() {
                assert!(schema["properties"].get(key.as_str().unwrap()).is_some());
            }
        }
        assert_eq!(tools[4]["inputSchema"]["required"], json!(["scope", "plugin", "on"]));
        assert_eq!(tools[5]["inputSchema"]["required"], json!(["scope", "server", "on"]));
        assert_eq!(tools[6]["inputSchema"]["properties"]["dry_run"]["default"], true);
        assert_eq!(request(&env, "no-such-method", json!({}))["error"]["code"], -32601);
        assert!(handle(&env, json!({ "jsonrpc": "2.0", "method": "notifications/initialized" })).is_none());
        let before = snapshot(env.library.parent().unwrap());
        assert!(handle(&env, json!({ "jsonrpc": "2.0", "method": "tools/call", "params": { "name": "apply", "arguments": { "dry_run": false } } })).is_none());
        assert_eq!(snapshot(env.library.parent().unwrap()), before, "a notification ran a tool");
    }

    #[test]
    fn stdio_reports_parse_errors_and_keeps_serving() {
        let env = fixture("server-stdio");
        let input = b"{broken\n{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n{\"jsonrpc\":\"2.0\",\"id\":42,\"method\":\"ping\"}\n";
        let mut output = Vec::new();
        serve(&env, &input[..], &mut output).unwrap();
        let messages: Vec<Value> = std::str::from_utf8(&output).unwrap().lines().map(|line| serde_json::from_str(line).unwrap()).collect();
        assert_eq!(messages, [error(Value::Null, -32700, "Parse error"), json!({ "jsonrpc": "2.0", "id": 42, "result": {} })]);
        assert_eq!(request(&env, "ping", json!({}))["id"], "test");
        assert_eq!(handle(&env, json!([])).unwrap()["error"]["code"], -32600);
    }

    #[test]
    fn assignments_change_manifest_and_drift_and_keep_comments() {
        let env = mcp_fixture("server-assign");
        let file = env.library.join(manifest::FILE);
        fs::write(&file, "# kept\n[global]\nplugins = [] # plugins kept\n[project.\"~/app\"]\nmcp = [] # servers kept\n").unwrap();
        let root = env.library.parent().unwrap();
        let mut before = snapshot(root);
        assert!(ok(&env, "status", json!({})).contains("~/app: in sync"));
        ok(&env, "assign_plugin", json!({ "scope": "global", "plugin": "h00man", "on": true }));
        let assigned = fs::read_to_string(&file).unwrap();
        ok(&env, "assign_plugin", json!({ "scope": "global", "plugin": "h00man", "on": true }));
        assert_eq!(fs::read_to_string(&file).unwrap(), assigned, "a duplicate was added");
        let manifest = ok(&env, "manifest", json!({}));
        assert!(manifest.contains("claude plugins: h00man") && manifest.contains("codex plugins: h00man"), "{manifest}");
        let status = ok(&env, "status", json!({ "scope": "global" }));
        assert!(status.contains("link     .claude/skills\n    h00man (1)") && status.contains("copy to the Codex plugin cache"), "{status}");

        let absolute = env.home.join("app").to_string_lossy().into_owned();
        ok(&env, "assign_mcp", json!({ "scope": absolute, "server": "local", "on": true }));
        let assigned = fs::read_to_string(&file).unwrap();
        ok(&env, "assign_mcp", json!({ "scope": "~/app", "server": "local", "on": true }));
        assert_eq!(fs::read_to_string(&file).unwrap(), assigned, "a duplicate server was added");
        assert!(ok(&env, "manifest", json!({})).contains("~/app:\n  harnesses: claude codex\n  claude plugins: \n  codex plugins: \n  mcp: local"));
        let status = ok(&env, "status", json!({ "scope": "~/app" }));
        assert!(status.contains("~/app: 3 to change") && status.contains("mcp      write in ~/app/.mcp.json\n    local (1)"), "{status}");
        assert!(!status.contains("global"), "scope filtering failed: {status}");
        assert_eq!(status, ok(&env, "status", json!({ "scope": absolute })));
        before.insert(file.strip_prefix(root).unwrap().into(), fs::read(&file).unwrap());
        assert_eq!(snapshot(root), before, "assignment wrote beyond the manifest");
        ok(&env, "assign_plugin", json!({ "scope": "global", "plugin": "h00man", "on": false }));
        ok(&env, "assign_mcp", json!({ "scope": "~/app", "server": "local", "on": false }));
        assert!(ok(&env, "status", json!({})).lines().all(|line| line.ends_with(": in sync")));
        let text = fs::read_to_string(file).unwrap();
        for comment in ["# kept", "# plugins kept", "# servers kept"] {
            assert!(text.contains(comment), "{text}");
        }
    }

    #[test]
    fn bad_tools_and_arguments_are_tool_errors_and_never_edit_the_manifest() {
        let env = mcp_fixture("server-errors");
        let before = snapshot(env.library.parent().unwrap());
        for (name, args, message) in [
            ("assign_plugin", json!({ "scope": "global", "plugin": "missing", "on": true }), "no plugin"),
            ("assign_mcp", json!({ "scope": "global", "server": "missing", "on": false }), "no MCP server"),
            ("assign_plugin", json!({ "scope": "~/missing", "plugin": "h00man", "on": true }), "no scope"),
            ("assign_mcp", json!({ "scope": "~/missing", "server": "local", "on": true }), "no scope"),
            ("status", json!({ "scope": "~/missing" }), "no scope"),
            ("apply", json!({ "scope": "~/missing", "dry_run": false }), "no scope"),
            ("assign_plugin", json!({ "scope": "global", "plugin": "h00man" }), "missing argument `on`"),
            ("assign_mcp", json!({ "scope": "global", "server": "local", "on": "yes" }), "must be true or false"),
            ("apply", json!({ "dry_run": null }), "must be true or false"),
            ("apply", json!({ "adopt": true, "dry_run": false }), "unknown argument `adopt`"),
            ("status", json!({ "scope": 1 }), "must be text"),
            ("manifest", json!([]), "must be an object"),
            ("fetch", json!({}), "unknown tool"),
        ] {
            let result = call_tool(&env, name, args);
            assert_eq!(result["isError"], true, "{result}");
            assert!(text(&result).contains(message), "{result}");
            assert_eq!(snapshot(env.library.parent().unwrap()), before, "{name} changed disk on an error");
        }
        fs::write(env.library.join("mcp/local.toml"), "command = 1\n").unwrap();
        let before = snapshot(env.library.parent().unwrap());
        let result = call_tool(&env, "apply", json!({ "dry_run": false }));
        assert_eq!(result["isError"], true);
        assert!(text(&result).contains("invalid MCP definition"));
        assert_eq!(snapshot(env.library.parent().unwrap()), before);
    }

    #[test]
    fn apply_defaults_to_dry_run_then_installs_and_reports_no_changes() {
        let env = mcp_fixture("server-apply");
        ok(&env, "assign_plugin", json!({ "scope": "global", "plugin": "h00man", "on": true }));
        let root = env.library.parent().unwrap();
        let before = snapshot(root);
        let result = request(&env, "tools/call", json!({ "name": "apply" }))["result"].clone();
        assert_eq!(result["isError"], false);
        assert!(text(&result).starts_with("Dry run. Nothing was written."));
        assert!(text(&result).contains("local remote (2)"));
        assert_eq!(snapshot(root), before, "the default apply wrote to disk");
        assert!(ok(&env, "apply", json!({ "dry_run": true })).starts_with("Dry run."));
        assert_eq!(snapshot(root), before);
        let result = ok(&env, "apply", json!({ "dry_run": false }));
        assert!(result.contains("global: 8 written, 0 removed, 0 left") && result.contains("~/app: 6 written, 0 removed, 0 left"), "{result}");
        assert_eq!(fs::read_link(env.home.join(".claude/skills/h00man")).unwrap(), env.library.join("plugins/h00man"));
        assert!(env.home.join(".codex/plugins/cache/workflow/h00man").is_dir());
        for (file, format) in [(".claude.json", Format::ClaudeUser), (".codex/config.toml", Format::Codex), ("app/.mcp.json", Format::Json), ("app/.codex/config.toml", Format::Codex), ("app/.agents/mcp.json", Format::Json)] {
            assert_eq!(format.read(&env.home.join(file)).unwrap().0.len(), 2, "{file}");
        }
        let before = snapshot(root);
        assert_eq!(ok(&env, "apply", json!({ "dry_run": false })), "global: 0 written, 0 removed, 0 left\n~/app: 0 written, 0 removed, 0 left\nbackup: none\n");
        assert_eq!(snapshot(root), before);
        ok(&env, "assign_mcp", json!({ "scope": "~/app", "server": "remote", "on": false }));
        let result = ok(&env, "apply", json!({ "scope": "~/app", "dry_run": false }));
        assert!(result.starts_with("~/app: 0 written, 3 removed, 0 left\nbackup: "), "{result}");
        let backup = result.lines().last().unwrap().strip_prefix("backup: ").unwrap();
        assert!(Path::new(backup).is_dir(), "backup was not returned: {result}");
        assert_eq!(fs::read_dir(backup).unwrap().count(), 4);
        assert_eq!(Format::ClaudeUser.read(&env.home.join(".claude.json")).unwrap().0.len(), 2, "a scoped apply changed another scope");
    }

    #[test]
    fn apply_leaves_a_foreign_link_and_never_adopts_it() {
        let env = fixture("server-foreign");
        let path = env.home.join(".claude/skills/h00man");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let foreign = env.home.join("foreign-plugin");
        std::os::unix::fs::symlink(&foreign, &path).unwrap();
        let result = ok(&env, "apply", json!({ "dry_run": false }));
        assert!(result.contains("1 left"), "{result}");
        assert_eq!(fs::read_link(&path).unwrap(), foreign);
        let status = ok(&env, "status", json!({ "scope": "global" }));
        assert!(status.contains("conflict .claude/skills") && status.contains("h00man (1)"), "{status}");
        let result = ok(&env, "apply", json!({ "scope": "global", "dry_run": false }));
        assert!(result.contains("global: 0 written, 0 removed, 1 left"), "{result}");
        assert_eq!(fs::read_link(&path).unwrap(), foreign);
    }

    #[test]
    fn library_and_sources_report_definitions_and_pins_without_writing() {
        let env = mcp_fixture("server-read");
        fs::write(env.library.join(sources::FILE), "[pstack]\nrepository = 'https://example.com/plugin'\nplugin = true\ncommit = 'abc123'\n[matt]\nrepository = 'https://example.com/skills'\ncommit = 'def456'\n").unwrap();
        let before = snapshot(env.library.parent().unwrap());
        let library = ok(&env, "library", json!({}));
        for part in ["plugins/h00man (2): plan unslop", "plugins/pstack (2): swarm unslop", "same name: unslop is in h00man and pstack", "mcp/local.toml:", "command: npx", "env names: [\"GITHUB_TOKEN\"]", "mcp/remote.toml:", "url: https://example.com/mcp", "bearer token env: EXAMPLE_TOKEN"] {
            assert!(library.contains(part), "missing {part}: {library}");
        }
        assert_eq!(ok(&env, "sources", json!({})), "pstack: plugin\n  repository: https://example.com/plugin\n  pin: abc123\nmatt: skills\n  repository: https://example.com/skills\n  pin: def456\n");
        assert_eq!(snapshot(env.library.parent().unwrap()), before);
    }
}
