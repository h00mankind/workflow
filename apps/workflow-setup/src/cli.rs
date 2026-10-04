//! The command line. It runs the same engine as the window.

use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};

use crate::{config::Config, library, sources};

pub const COMMANDS: &[&str] = &["sources", "library", "help", "--help", "-h"];

const USAGE: &str = "\
workflow-setup [project-path]          open the window
workflow-setup library                 list the library and its name collisions
workflow-setup sources fetch [name]    get the newest commit of each source into the cache
workflow-setup sources diff [name]     show what the fetched commit would change (--stat: no patch)
workflow-setup sources accept <name>   write the copy and move the pin (--all: every source)

The library is $WORKFLOW_LIBRARY, or the folder above the current one that
has sources.toml, or the library the window uses.";

pub fn run(args: &[String]) -> Result<()> {
    let words: Vec<&str> = args.iter().map(String::as_str).filter(|arg| !arg.starts_with("--")).collect();
    let flag = |name: &str| args.iter().any(|arg| arg == name);
    match words.as_slice() {
        ["library"] => list(&find_library()?),
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
            println!("{}: skills/{} is now the copy at {}", source.name, source.name, short(&commit));
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

fn list(library: &Path) -> Result<()> {
    let skills = library::skills(library);
    let (winners, collisions) = library::resolve(&skills);
    for origin in library::origins(library, "skills") {
        let names: Vec<&str> = skills.iter().filter(|skill| skill.origin == origin).map(|skill| skill.name.as_str()).collect();
        println!("skills/{origin} ({}): {}", names.len(), names.join(" "));
    }
    for agent in library::agents(library) {
        println!("agents/{}: {}", agent.origin, agent.name);
    }
    println!("{} skill names install.", winners.len());
    for collision in &collisions {
        println!("collision: {} is in {} and {}. {} wins.", collision.name, collision.winner, collision.losers.join(", "), collision.winner);
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
