//! The command line. It runs the same engine as the window.

use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};

use crate::{
    agents::{self, Agent, Kind},
    apply::{self, Env, Plan},
    config::Config,
    installed, library, manifest, sources,
};

pub const COMMANDS: &[&str] = &["status", "apply", "list", "install", "remove", "add", "update", "sources", "library", "help", "--help", "-h"];

const USAGE: &str = "\
workflow-setup [project-path]          open the window
workflow-setup status [scope]          show the drift between manifest.toml and the disk
workflow-setup apply [scope]           make the disk match manifest.toml (--dry-run: change nothing;
                                       --adopt: replace a link that something else made)
workflow-setup list [-g | -p path] [-a agent]...
                                       show the skills in each agent folder: made by
                                       workflow-setup or foreign, and the skills.sh source
workflow-setup install <entry>... [-g | -p path] [-a agent]...
                                       add the entries to the scope in manifest.toml, then apply
                                       that scope (--dry-run: change nothing)
workflow-setup remove <entry>... [-g | -p path] [-a agent]...
                                       take the entries out of the scope in manifest.toml, then
                                       apply that scope. Only links that workflow-setup made go.
workflow-setup remove --foreign <name>... [-g | -p path] [-a agent]...
                                       take out a skill that workflow-setup did not make: unlink
                                       a link, or move a real folder to ~/.Trash
workflow-setup add <owner/repo | URL | path> [-s skill]... [--name name]
                                       write a new source to sources.toml, fetch it, and show
                                       the diff and the skills. --yes: accept the copy.
                                       --dry-run: do not write sources.toml.
workflow-setup update [source]...      fetch each source and show the diff. --yes: accept
                                       the commit that the diff showed.
workflow-setup library                 list the plugins, their skills, and the names two plugins share
workflow-setup sources fetch [name]    get the newest commit of each source into the cache
workflow-setup sources diff [name]     show what the fetched commit would change (--stat: no patch)
workflow-setup sources accept <name>   write the copy of the commit that the last diff showed, and
                                       move the pin (--all: every source)
workflow-setup sources skills [name]   fetch each source and list every skill it has. * marks
                                       the skills the library copies.
workflow-setup sources edit <name> [--repo spec] [--branch branch | --default-branch]
                                       [-s skill]... [--all-skills]
                                       change the source in sources.toml, then fetch it and show
                                       the diff. --yes: accept the copy.
workflow-setup sources remove <name>   take the source out of every scope of manifest.toml and
                                       apply, then out of sources.toml. Its copy goes to ~/.Trash.

For status and apply, a scope is `global` or a project path from manifest.toml.
Without one: every scope.
An entry is a plugin of the library, or <plugin>/<skill> for one skill. One
skill installs as one link without a prefix, for every agent.
-g is the global scope. -p is a project. Without -g or -p, the scope is the
project of manifest.toml that holds the current folder. -a names an agent of
apps/workflow-setup/agents.toml, and you can give it more than one time.
The library is $WORKFLOW_LIBRARY, or the folder above the current one that
has sources.toml, or the library the window uses.";

/// The command line, parsed. Options with a value take the next argument.
#[derive(Default)]
struct Args<'a> {
    words: Vec<&'a str>,
    flags: Vec<&'a str>,
    global: bool,
    project: Option<&'a str>,
    agents: Vec<&'static Agent>,
    skills: Vec<&'a str>,
    name: Option<&'a str>,
    repo: Option<&'a str>,
    branch: Option<&'a str>,
}

impl<'a> Args<'a> {
    fn parse(args: &'a [String]) -> Result<Args<'a>> {
        let mut parsed = Args::default();
        let mut args = args.iter().map(String::as_str);
        while let Some(arg) = args.next() {
            let mut value = || args.next().with_context(|| format!("{arg} needs a value"));
            match arg {
                "-g" | "--global" => parsed.global = true,
                "-p" | "--project" => parsed.project = Some(value()?),
                "-a" | "--agent" => parsed.agents.push(agents::find(value()?)?),
                "-s" | "--skill" => parsed.skills.push(value()?),
                "--name" => parsed.name = Some(value()?),
                "--repo" => parsed.repo = Some(value()?),
                "--branch" => parsed.branch = Some(value()?),
                "-h" | "--help" => parsed.words.push("help"),
                flag if flag.starts_with("--") => parsed.flags.push(flag),
                flag if flag.starts_with('-') => bail!("`{flag}` is not an option. Run workflow-setup help."),
                word => parsed.words.push(word),
            }
        }
        Ok(parsed)
    }

    fn flag(&self, name: &str) -> bool {
        self.flags.contains(&name)
    }

    /// The root of the scope that -g or -p names, and true for the global
    /// scope. Without either: the manifest project that holds the current folder.
    fn scope(&self, env: &Env, cwd: &Path) -> Result<(PathBuf, bool)> {
        let root = match (self.global, self.project) {
            (true, Some(_)) => bail!("give -g or -p, not both"),
            (true, None) => return Ok((env.home.clone(), true)),
            (false, Some(path)) => match path.strip_prefix("~/") {
                Some(rest) => env.home.join(rest),
                None => std::path::absolute(path)?,
            },
            (false, None) => {
                let manifest = manifest::load(&env.library, &env.home)?;
                let projects = manifest.scopes.into_iter().filter(|scope| !scope.global && cwd.starts_with(&scope.root));
                match projects.max_by_key(|scope| scope.root.components().count()) {
                    Some(scope) => scope.root,
                    None => bail!("{} has no project that holds the current folder. Give -g, or -p <project path>.", manifest::FILE),
                }
            }
        };
        let global = root == env.home;
        Ok((root, global))
    }
}

pub fn run(args: &[String]) -> Result<()> {
    let args = Args::parse(args)?;
    let flag = |name: &str| args.flag(name);
    match args.words.as_slice() {
        ["library"] => library_list(&find_library()?),
        ["list"] => list(&args),
        ["install", names @ ..] => change(&Env::new(find_library()?), &args, names, &std::env::current_dir()?, true),
        ["remove", names @ ..] if flag("--foreign") => remove_foreign(&Env::new(find_library()?), &args, names, &std::env::current_dir()?),
        ["remove", names @ ..] => change(&Env::new(find_library()?), &args, names, &std::env::current_dir()?, false),
        ["status", scopes @ ..] => status(scopes, false, false, true),
        ["apply", scopes @ ..] => status(scopes, !flag("--dry-run"), flag("--adopt"), flag("--dry-run")),
        ["add", spec] => add(&find_library()?, &args, spec),
        ["add", ..] => bail!("add takes one source: owner/repo, a git URL, or a local path"),
        ["update", names @ ..] => each(names, false, |library, source| review(library, source, flag("--yes"))),
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
        ["sources", "skills", names @ ..] => each(names, false, |library, source| {
            sources::fetch(library, source)?;
            let all = sources::available(library, source)?;
            let chosen = |skill: &String| source.plugin || source.skills.as_ref().is_none_or(|skills| skills.contains(skill));
            let marked: Vec<String> = all.iter().map(|skill| if chosen(skill) { format!("*{skill}") } else { skill.clone() }).collect();
            println!("{} ({} of {}): {}", source.name, all.iter().filter(|skill| chosen(skill)).count(), all.len(), marked.join(" "));
            Ok(())
        }),
        ["sources", "edit", name] => edit_source(&find_library()?, &args, name),
        ["sources", "remove", name] => {
            let env = Env::new(find_library()?);
            let applied = apply::remove_source(&env, name)?;
            let (written, removed) = applied.iter().fold((0, 0), |sum, one| (sum.0 + one.written, sum.1 + one.removed));
            println!("{name}: out of {} and {} ({written} written, {removed} removed). The copy is in the Trash.", manifest::FILE, sources::FILE);
            Ok(())
        }
        _ => {
            println!("{USAGE}");
            Ok(())
        }
    }
}

/// `add`: writes the source to `sources.toml`, then shows what it would bring.
fn add(library: &Path, args: &Args, spec: &str) -> Result<()> {
    let source = sources::new_source(spec, args.name, &args.skills, &crate::scan::home())?;
    if args.flag("--dry-run") {
        sources::check_new(library, &source)?;
        sources::fetch(library, &source)?;
        print_diff(&source, &sources::diff(library, &source)?, false);
        println!("dry run: {} did not change", sources::FILE);
        return Ok(());
    }
    sources::write_source(library, &source)?;
    println!("{}: [{}] from {}", sources::FILE, source.name, source.repository);
    let source = sources::load(library)?.into_iter().find(|known| known.name == source.name).context("the new source is missing")?;
    review(library, &source, args.flag("--yes"))
}

/// `sources edit`: writes the given fields, keeps the others, then reviews the source.
fn edit_source(library: &Path, args: &Args, name: &str) -> Result<()> {
    let source = sources::load(library)?.into_iter().find(|source| source.name == name).with_context(|| format!("{} has no source named {name}", sources::FILE))?;
    let repository = match args.repo {
        Some(spec) => sources::new_source(spec, Some(name), &[], &crate::scan::home())?.repository,
        None => source.repository.clone(),
    };
    let branch = if args.flag("--default-branch") { None } else { args.branch.map(str::to_string).or(source.branch.clone()) };
    let skills: Option<Vec<String>> = if args.flag("--all-skills") {
        None
    } else if args.skills.is_empty() {
        source.skills.clone()
    } else {
        Some(args.skills.iter().map(|skill| skill.to_string()).collect())
    };
    sources::set_fields(library, name, &repository, branch.as_deref(), skills.as_deref())?;
    println!("{}: [{name}] written", sources::FILE);
    let source = sources::load(library)?.into_iter().find(|source| source.name == name).context("the source is missing")?;
    review(library, &source, args.flag("--yes"))
}

/// Fetches a source and prints the diff without the patch. With `yes`,
/// accepts the commit that the diff showed.
fn review(library: &Path, source: &sources::Source, yes: bool) -> Result<()> {
    sources::fetch(library, source)?;
    let diff = sources::diff(library, source)?;
    print_diff(source, &diff, false);
    if diff.is_empty() {
        return Ok(());
    }
    if !yes {
        println!("{}: run `workflow-setup sources diff {}` to read the patch. Give --yes to accept.", source.name, source.name);
        return Ok(());
    }
    let commit = sources::accept(library, source)?;
    println!("{}: plugins/{} is now the copy at {}", source.name, source.name, short(&commit));
    Ok(())
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
        write_plans(&env, &plans, adopt)?;
    }
    Ok(())
}

fn write_plans(env: &Env, plans: &[Plan], adopt: bool) -> Result<()> {
    let (results, backup) = apply::apply(env, plans, adopt)?;
    for (plan, applied) in plans.iter().zip(results) {
        let mut line = format!("{}: {} written, {} removed", plan.scope.label, applied.written, applied.removed);
        if applied.left > 0 {
            line.push_str(&format!(", {} left because workflow-setup does not own them", applied.left));
        }
        println!("{line}");
    }
    if let Some(backup) = backup {
        println!("backup: {}", tilde(env, &backup));
    }
    Ok(())
}

/// `install` and `remove`: edits the scope in the manifest, then makes the
/// scope match. The global scope comes too when the scope has Codex,
/// because Codex keeps one copy of each plugin for all scopes.
fn change(env: &Env, args: &Args, names: &[&str], cwd: &Path, add: bool) -> Result<()> {
    if names.is_empty() {
        bail!("name a plugin, or <plugin>/<skill>");
    }
    let entries = names.iter().map(|name| manifest::Entry::parse(name)).collect::<Result<Vec<_>>>()?;
    let (root, _) = args.scope(env, cwd)?;
    let text = manifest::edit(&manifest::read(&env.library), &env.home, &root, &entries, &args.agents, add)?;
    let mut plans = apply::plan_manifest(env, manifest::parse(&text, &env.home)?)?;
    let codex = plans.iter().any(|plan| plan.scope.root == root && plan.scope.has(Kind::CodexPlugin));
    plans.retain(|plan| plan.scope.root == root || plan.scope.global && codex);
    for plan in &plans {
        print_plan(env, plan);
    }
    if args.flag("--dry-run") {
        println!("dry run: {} and the disk did not change", manifest::FILE);
        return Ok(());
    }
    manifest::write(&env.library, &text)?;
    println!("{}: written", manifest::FILE);
    write_plans(env, &plans, args.flag("--adopt"))
}

/// `remove --foreign`: takes items that workflow-setup did not make out of
/// the skill folders of the scope. A link goes. A real folder moves to the Trash.
fn remove_foreign(env: &Env, args: &Args, names: &[&str], cwd: &Path) -> Result<()> {
    if names.is_empty() {
        bail!("name a skill folder. `list` shows them.");
    }
    let (root, global) = args.scope(env, cwd)?;
    let folders = installed::folders(env, &root, global, &args.agents);
    for name in names {
        let items: Vec<&installed::Item> = folders.iter().flat_map(|folder| &folder.items).filter(|item| item.name == *name).collect();
        if items.is_empty() {
            bail!("no skill folder of {} has {name}", tilde(env, &root));
        }
        for item in items {
            let path = tilde(env, &item.path);
            if item.owned {
                println!("{path}: workflow-setup made it. Use remove without --foreign.");
                continue;
            }
            let verb = if item.link.is_some() { "unlink" } else { "move to the Trash" };
            if args.flag("--dry-run") {
                println!("{path}: would {verb}");
                continue;
            }
            crate::ops::remove(&item.path, &env.home)?;
            println!("{path}: {}", if item.link.is_some() { "unlinked" } else { "moved to the Trash" });
        }
    }
    Ok(())
}

fn is_scope(env: &Env, plan: &Plan, name: &str) -> bool {
    let path = std::path::absolute(name).unwrap_or_default();
    plan.scope.label == name || plan.scope.root == path || tilde(env, &plan.scope.root) == name
}

fn print_plan(env: &Env, plan: &Plan) {
    let root = tilde(env, &plan.scope.root);
    let title = if plan.scope.global { format!("global ({root})") } else { root };
    for clash in &plan.clashes {
        println!("{title}: {} is in {} and {}. {} wins.", tilde(env, &clash.path), clash.winner, clash.losers.join(", "), clash.winner);
    }
    for double in &plan.doubles {
        println!("{title}: warning: {}", double.message(&place(env, plan, &double.dir)));
    }
    if plan.in_sync() {
        println!("{title}: in sync");
        return;
    }
    println!("{title}: {} to change", plan.changes.len());
    for (label, names) in plan.groups(&env.home) {
        println!("  {label}\n    {} ({})", names.join(" "), names.len());
    }
}

/// A folder of a scope as the user reads it: from the project root, or from `~`.
fn place(env: &Env, plan: &Plan, dir: &Path) -> String {
    match plan.scope.global {
        true => tilde(env, dir),
        false => dir.strip_prefix(&plan.scope.root).unwrap_or(dir).display().to_string(),
    }
}

fn tilde(env: &Env, path: &Path) -> String {
    apply::tilde(&env.home, path)
}

/// Prints each skill folder of the scope and what is in it.
fn list(args: &Args) -> Result<()> {
    let env = Env::new(find_library()?);
    let (root, global) = args.scope(&env, &std::env::current_dir()?)?;
    let folders = installed::folders(&env, &root, global, &args.agents);
    for folder in &folders {
        let mut names: Vec<&str> = folder.agents.iter().map(|agent| agent.name.as_str()).collect();
        let readers = match names.len() {
            0 => "an old folder of workflow-setup. No agent reads it.".to_string(),
            1..=4 => names.join(", "),
            count => {
                names.truncate(3);
                format!("{}, and {} more", names.join(", "), count - 3)
            }
        };
        println!("{}  ({readers})", tilde(&env, &folder.dir));
        let width = folder.items.iter().map(|item| item.name.len()).max().unwrap_or(0);
        for item in &folder.items {
            let owner = if item.owned { "workflow" } else { "foreign " };
            let what = match &item.link {
                Some(link) => match link.strip_prefix(&env.library) {
                    Ok(inside) => format!("link to {}", inside.display()),
                    Err(_) => format!("link to {}", tilde(&env, link)),
                },
                None => "a real folder".to_string(),
            };
            let broken = if item.broken { "  (broken: the target is missing)" } else { "" };
            let source = item.source.as_deref().map(|source| format!("  (skills.sh: {source})")).unwrap_or_default();
            println!("  {:width$}  {owner}  {what}{broken}{source}", item.name);
        }
    }
    let codex = agents::of_kind(Kind::CodexPlugin);
    if args.agents.is_empty() || args.agents.contains(&codex) {
        let plugins = apply::codex_plugins(&root)?;
        if !plugins.is_empty() {
            let names: Vec<String> = plugins.iter().map(|(plugin, on)| if *on { plugin.clone() } else { format!("{plugin} (off)") }).collect();
            println!("codex plugins of the workflow marketplace in {}: {}", tilde(&env, &root.join(".codex/config.toml")), names.join(" "));
        }
    }
    if folders.is_empty() {
        println!("{}: no skills", tilde(&env, &root));
    }
    Ok(())
}

fn library_list(library: &Path) -> Result<()> {
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
        ("skills", &diff.skills),
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

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::apply::tests::fixture;

    fn args(text: &str) -> Vec<String> {
        text.split_whitespace().map(str::to_string).collect()
    }

    #[test]
    fn the_scope_is_the_option_or_the_manifest_project_that_holds_the_current_folder() {
        let env = fixture("cli-scope");
        let app = env.home.join("app");
        let scope = |text: &str, cwd: &Path| Args::parse(&args(text)).and_then(|parsed| parsed.scope(&env, cwd));
        assert_eq!(scope("list", &app.join("src")).unwrap(), (app.clone(), false));
        assert_eq!(scope("list -g", Path::new("/")).unwrap(), (env.home.clone(), true));
        assert_eq!(scope("list -p ~/other", Path::new("/")).unwrap(), (env.home.join("other"), false));
        assert!(scope("list", &env.home).is_err(), "a folder outside every manifest project got a scope");
        assert!(scope("list -g -p ~/app", &app).is_err());

        let line = args("list -a claude -a cursor --dry-run");
        let parsed = Args::parse(&line).unwrap();
        assert_eq!(parsed.agents.iter().map(|agent| agent.name.as_str()).collect::<Vec<_>>(), ["claude-code", "cursor"]);
        assert!(parsed.flag("--dry-run") && parsed.words == ["list"]);
        assert!(Args::parse(&args("list -a nope")).is_err());
        assert!(Args::parse(&args("list -z")).is_err());
        assert!(Args::parse(&args("list -p")).is_err());
    }

    fn command(env: &Env, text: &str) -> Result<()> {
        let line = args(text);
        let parsed = Args::parse(&line)?;
        let names = &parsed.words[1..];
        match parsed.words[0] {
            "install" => change(env, &parsed, names, Path::new("/"), true),
            "remove" if parsed.flag("--foreign") => remove_foreign(env, &parsed, names, Path::new("/")),
            "remove" => change(env, &parsed, names, Path::new("/"), false),
            other => bail!("the test does not know {other}"),
        }
    }

    #[test]
    fn install_and_remove_edit_the_manifest_then_apply_and_a_dry_run_changes_nothing() {
        let env = fixture("cli-install");
        apply::tests::run(&env, false);
        let manifest_text = || manifest::read(&env.library);
        let before = manifest_text();
        let link = env.home.join(".cursor/skills/swarm");

        command(&env, "install pstack/swarm -g -a cursor --dry-run").unwrap();
        assert_eq!(manifest_text(), before);
        assert!(link.symlink_metadata().is_err(), "a dry run made a link");

        command(&env, "install pstack/swarm -g -a cursor").unwrap();
        let installed = manifest_text();
        assert_eq!(installed, before.replace("claude = [\"pstack\"]\n", "claude = [\"pstack\"]\ncursor = [\"pstack/swarm\"]\n"), "install did more than add the cursor list");
        assert_eq!(fs::read_link(&link).unwrap(), env.library.join("plugins/pstack/skills/swarm"));
        let names: Vec<_> = fs::read_dir(env.home.join(".cursor/skills")).unwrap().flatten().map(|entry| entry.file_name()).collect();
        assert_eq!(names, ["swarm"], "cursor got more than the one skill");
        assert!(apply::plan_all(&env).unwrap().iter().all(Plan::in_sync));
        command(&env, "install pstack/swarm -g -a cursor").unwrap();
        assert_eq!(manifest_text(), installed, "a second install changed the manifest");
        assert!(apply::plan_all(&env).unwrap().iter().all(Plan::in_sync));

        command(&env, "remove pstack/swarm -g -a cursor").unwrap();
        assert!(link.symlink_metadata().is_err(), "remove left the link");
        assert!(apply::plan_all(&env).unwrap().iter().all(Plan::in_sync));

        assert!(command(&env, "install nope -g").is_err(), "an unknown plugin passed");
        assert!(command(&env, "install h00man/nope -g").is_err(), "an unknown skill passed");
        assert!(command(&env, "remove h00man/plan -g").is_err(), "a skill of a whole plugin went out alone");
    }

    #[test]
    fn add_and_update_show_the_diff_and_accept_only_with_yes() {
        let (upstream, library) = sources::tests::fixture("cli-add", "");
        let spec = upstream.to_string_lossy().into_owned();
        let parse = |text: String| args(&text);

        let line = parse(format!("add {spec} --name other -s one --dry-run"));
        add(&library, &Args::parse(&line).unwrap(), &spec).unwrap();
        assert!(sources::load(&library).unwrap().iter().all(|source| source.name != "other"), "a dry run wrote sources.toml");

        let line = parse(format!("add {spec} --name other -s one"));
        add(&library, &Args::parse(&line).unwrap(), &spec).unwrap();
        let source = sources::load(&library).unwrap().into_iter().find(|source| source.name == "other").expect("add wrote no source");
        assert_eq!(source.skills, Some(vec!["one".to_string()]));
        assert!(!library.join("plugins/other").exists(), "add without --yes changed the library");

        review(&library, &source, true).unwrap();
        assert_eq!(fs::read_to_string(library.join("plugins/other/skills/one/SKILL.md")).unwrap(), "one v1");

        sources::tests::commit(&upstream, &[("skills/a/one/SKILL.md", "one v2")], &[]);
        let source = sources::load(&library).unwrap().into_iter().find(|source| source.name == "other").unwrap();
        review(&library, &source, false).unwrap();
        assert_eq!(fs::read_to_string(library.join("plugins/other/skills/one/SKILL.md")).unwrap(), "one v1", "update without --yes accepted");
        review(&library, &source, true).unwrap();
        assert_eq!(fs::read_to_string(library.join("plugins/other/skills/one/SKILL.md")).unwrap(), "one v2");
    }

    #[test]
    fn remove_foreign_moves_a_real_folder_to_the_trash_and_leaves_owned_links() {
        let env = fixture("cli-foreign");
        apply::tests::run(&env, false);
        let foreign = env.home.join(".agents/skills/find-skills");
        fs::create_dir_all(&foreign).unwrap();

        command(&env, "remove --foreign find-skills -g --dry-run").unwrap();
        assert!(foreign.is_dir(), "a dry run moved the folder");
        command(&env, "remove --foreign find-skills -g").unwrap();
        assert!(!foreign.exists());
        let trash: Vec<String> = fs::read_dir(env.home.join(".Trash")).unwrap().flatten().map(|entry| entry.file_name().to_string_lossy().into_owned()).collect();
        assert!(trash.len() == 1 && trash[0].starts_with("find-skills "), "{trash:?}");

        command(&env, "remove --foreign pstack -g").unwrap();
        assert!(env.home.join(".claude/skills/pstack").symlink_metadata().is_ok(), "--foreign took an owned link");
        assert!(command(&env, "remove --foreign nothing -g").is_err());
    }
}
