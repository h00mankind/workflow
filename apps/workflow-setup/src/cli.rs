//! The command line. It runs the same engine as the window.

use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};

use crate::{
    apply::{self, Env, Plan},
    config::Config,
    library, manifest, mcp_server, sources,
};

pub const COMMANDS: &[&str] = &["status", "apply", "sources", "library", "mcp-server", "help", "--help", "-h"];

const USAGE: &str = "\
workflow-setup [project-path]          open the window
workflow-setup status [scope]          show the drift between manifest.toml and the disk
workflow-setup apply [scope]           make the disk match manifest.toml (--dry-run: change nothing;
                                       --adopt: replace a link that something else made)
workflow-setup library                 list the plugins, their skills, and the names two plugins share
workflow-setup mcp-server              serve the library to agents over stdio
workflow-setup sources fetch [name]    get the newest commit of each source into the cache
workflow-setup sources diff [name]     show what the fetched commit would change (--stat: no patch)
workflow-setup sources accept <name>   write the copy and move the pin (--all: every source)

A scope is `global` or a project path from manifest.toml. Without one: every scope.
The library is $WORKFLOW_LIBRARY, or the folder above the current one that
has sources.toml, or the library the window uses.";

pub fn run(args: &[String]) -> Result<()> {
    let words: Vec<&str> = args.iter().map(String::as_str).filter(|arg| !arg.starts_with("--")).collect();
    let flag = |name: &str| args.iter().any(|arg| arg == name);
    match words.as_slice() {
        ["mcp-server"] => mcp_server::run(&Env::new(find_library()?)),
        ["library"] => list(&find_library()?),
        ["status", scopes @ ..] => status(scopes, false, false, true),
        ["apply", scopes @ ..] => status(scopes, !flag("--dry-run"), flag("--adopt"), flag("--dry-run")),
        ["sources", "fetch", names @ ..] => each(names, false, |library, source| {
            let commit = sources::fetch(library, source)?;
            let state = if source.commit.as_deref() == Some(&commit) { "at the pin" } else { "newer than the pin" };
            println!("{}: fetched {} ({state})", source.name, short(&commit));
            Ok(())
        }),
        ["sources", "diff", names @ ..] => each(names, false, |library, source| {
            print_diff(source, &sources::diff(library, source)?, !flag("--stat"));
            Ok(())
        }),
        ["sources", "accept", names @ ..] => each(names, !flag("--all"), |library, source| {
            let commit = sources::accept(library, source)?;
            println!("{}: plugins/{} is now the copy at {}", source.name, source.name, short(&commit));
            Ok(())
        }),
        _ => {
            println!("{USAGE}");
            Ok(())
        }
    }
}

fn find_library() -> Result<PathBuf> {
    if let Some(dir) = std::env::var_os("WORKFLOW_LIBRARY") {
        return Ok(PathBuf::from(dir));
    }
    let cwd = std::env::current_dir()?;
    if let Some(dir) = cwd.ancestors().find(|dir| dir.join(sources::FILE).is_file()) {
        return Ok(dir.to_path_buf());
    }
    Config::load().library.context("no library found. Set WORKFLOW_LIBRARY.")
}

fn each(names: &[&str], need_name: bool, run: impl Fn(&Path, &sources::Source) -> Result<()>) -> Result<()> {
    let library = find_library()?;
    let all = sources::load(&library)?;
    if names.is_empty() && need_name {
        bail!("name a source, or use --all");
    }
    for name in names {
        if !all.iter().any(|source| source.name == *name) {
            bail!("{} has no source named {name}", sources::FILE);
        }
    }
    for source in all.iter().filter(|source| names.is_empty() || names.contains(&source.name.as_str())) {
        run(&library, source)?;
    }
    Ok(())
}

/// Prints the drift of the named scopes and, with `write`, corrects it.
fn status(names: &[&str], write: bool, adopt: bool, show: bool) -> Result<()> {
    let env = Env::new(find_library()?);
    let mut plans = apply::plan_all(&env)?;
    for name in names {
        if !plans.iter().any(|plan| is_scope(&env, plan, name)) {
            bail!("{} has no scope `{name}`", manifest::FILE);
        }
    }
    plans.retain(|plan| names.is_empty() || names.iter().any(|name| is_scope(&env, plan, name)));
    if !write || show {
        for plan in &plans {
            print_plan(&env, plan);
        }
    }
    if write {
        let (results, backup) = apply::apply(&env, &plans, adopt)?;
        for (plan, applied) in plans.iter().zip(results) {
            let mut line = format!("{}: {} written, {} removed", plan.scope.label, applied.written, applied.removed);
            if applied.left > 0 {
                line.push_str(&format!(", {} left because workflow-setup does not own them", applied.left));
            }
            println!("{line}");
        }
        if let Some(backup) = backup {
            println!("backup: {}", tilde(&env, &backup));
        }
    }
    Ok(())
}

pub(crate) fn is_scope(env: &Env, plan: &Plan, name: &str) -> bool {
    let path = std::path::absolute(name).unwrap_or_default();
    plan.scope.label == name || plan.scope.root == path || tilde(env, &plan.scope.root) == name
}

fn print_plan(env: &Env, plan: &Plan) {
    print!("{}", format_plan(env, plan));
}

/// The same drift text for the command line and the MCP server.
pub(crate) fn format_plan(env: &Env, plan: &Plan) -> String {
    let mut text = String::new();
    let root = tilde(env, &plan.scope.root);
    let title = if plan.scope.global { format!("global ({root})") } else { root };
    for collision in &plan.collisions {
        text.push_str(&format!("{title}: opencode has no prefix. {} is in {} and {}. {} wins.\n", collision.name, collision.winner, collision.losers.join(", "), collision.winner));
    }
    if plan.in_sync() {
        text.push_str(&format!("{title}: in sync\n"));
        return text;
    }
    text.push_str(&format!("{title}: {} to change\n", plan.changes.len()));
    for (label, names) in plan.groups(&env.home) {
        text.push_str(&format!("  {label}\n    {} ({})\n", names.join(" "), names.len()));
    }
    text
}

fn tilde(env: &Env, path: &Path) -> String {
    apply::tilde(&env.home, path)
}

fn list(library: &Path) -> Result<()> {
    let skills = library::skills(library);
    let (winners, collisions) = library::resolve(&skills);
    for plugin in library::plugins(library) {
        let names: Vec<&str> = skills.iter().filter(|skill| skill.origin == plugin).map(|skill| skill.name.as_str()).collect();
        println!("plugins/{plugin} ({}): {}", names.len(), names.join(" "));
    }
    for agent in library::agents(library) {
        println!("agents/{}: {}", agent.origin, agent.name);
    }
    println!("{} skills, {} names. With a prefix, every skill installs.", skills.len(), winners.len());
    for collision in &collisions {
        println!("same name: {} is in {} and {}. Without a prefix, {} wins.", collision.name, collision.winner, collision.losers.join(", "), collision.winner);
    }
    Ok(())
}

fn print_diff(source: &sources::Source, diff: &sources::Diff, patch: bool) {
    let from = diff.from.as_deref().map(short).unwrap_or("no pin");
    if diff.is_empty() {
        println!("{}: no change. The library copy is the fetched commit {}.", source.name, short(&diff.to));
    } else {
        println!("{}: {from} -> {}", source.name, short(&diff.to));
    }
    for (label, names) in [
        ("added", &diff.added),
        ("removed", &diff.removed),
        ("changed", &diff.changed),
        ("edited here, accept overwrites", &diff.edited_here),
        ("named in sources.toml, not in the source", &diff.missing),
    ] {
        if !names.is_empty() {
            println!("  {label} ({}): {}", names.len(), names.join(" "));
        }
    }
    if !diff.log.is_empty() {
        println!("  commits:");
        for line in diff.log.lines() {
            println!("    {line}");
        }
    }
    if patch && !diff.patch.is_empty() {
        println!("\n{}", diff.patch);
    }
}

fn short(commit: &str) -> &str {
    &commit[..commit.len().min(12)]
}
