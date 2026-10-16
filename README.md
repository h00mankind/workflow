# h00man workflow

One library for what my coding agents use: skills, agents, scripts, MCP servers, hooks, and instruction files. This repo is for my own use.

## Layout

```text
plugins/
  h00man/            my own skills, edited here
  matt/              verbatim copy from mattpocock/skills, never edited here
  pstack/            verbatim copy of my pstack port (skills, agents, hooks), never edited here
agents/              agents that are not in a plugin, by origin
mcp/                 one definition file per MCP server
scripts/             statusline, notifications, hooks
instructions/        shared AGENTS.md and CLAUDE.md parts
sources.toml         per source: repository, path, pinned commit
manifest.toml        which plugins go to global scope and to each project
apps/workflow-setup  the engine: one window and one command line
```

Each folder in `plugins/` is one plugin, and its name is the prefix of its skills: `/h00man:plan`, `/matt:teach`, `/pstack:teach`. With a prefix, two plugins can have a skill with the same name.

```bash
cd apps/workflow-setup && cargo build
alias workflow-setup="$PWD/target/debug/workflow-setup"
```

## Sources

An outside skill is a set of instructions that runs with my permissions. Thus each copy is committed to git at a pinned commit, and an update is a diff to read before it enters the library.

```bash
workflow-setup add mattpocock/skills -s teach   # write [skills] to sources.toml, fetch it, show the diff stat and the skills
workflow-setup add ~/code/kit --name kit --yes  # the same for a local clone, and accept the copy
workflow-setup update                           # fetch each source and show the diff stat
workflow-setup sources diff pstack              # read the patch
workflow-setup update pstack --yes              # accept: write plugins/pstack/ and move the pin
workflow-setup apply                            # give the new copy to Codex, which keeps its own copy
workflow-setup library                          # list the plugins and their skills
```

`add` takes a GitHub `owner/repo`, a git URL, or a local path. The name of the source is the last part of the repository, or `--name`. `add --dry-run` does not write `sources.toml`. `sources fetch`, `sources diff`, and `sources accept` do the three steps one at a time.

Only an accept changes the library. It takes the commit that the last diff showed, not a newer fetch. It swaps the plugin folder in one rename. When an error comes after the swap, the old copy and the old pin come back.

A source is one of two kinds. With `plugin = true`, its `path` is a complete plugin and is copied as it is (`pstack`). Without it, its `path` holds skills, `skills` can limit the copy to a list, and the engine writes the plugin manifests (`matt`).

`repository` can be a local clone. `pstack` reads my clone of the port, so a commit there is enough and no push is necessary.

## Manifest and apply

`manifest.toml` assigns plugins to `[global]` and to each `[project."<path>"]`. `plugins` is for every agent of the scope. A key with the name of an agent is for that agent only. `harnesses` names the agents of the scope. Without it, the agents are Claude Code and Codex.

An entry is a plugin (`h00man`) or one skill of a plugin (`pstack/swarm`). An unknown plugin or skill is an error.

A scope can also set `mcp = ["<name>", ...]`. Each name is a definition in `mcp/<name>.toml`. Claude Code and Codex receive those servers when the scope names that agent. The other agents skip them. An unknown server name is an error.

```bash
workflow-setup status                 # the drift between manifest.toml and the disk, for every scope
workflow-setup apply --dry-run        # the same, and change nothing
workflow-setup apply                  # make the disk match
workflow-setup apply global --adopt   # also replace links that something else made
```

`install` and `remove` edit `manifest.toml` and then apply the scope. They keep the comments of the file. Each takes `--dry-run`. `list` shows what is on the disk.

```bash
workflow-setup install h00man/plan -a cursor -p ~/code/app --dry-run   # show what changes, change nothing
workflow-setup install matt -g                    # give matt to every agent of the global scope
workflow-setup remove matt -g -a codex            # take matt from Codex only
workflow-setup list -g                            # each skill folder: made by workflow-setup or foreign
workflow-setup remove --foreign find-skills -g    # unlink a foreign link, or move a foreign folder to ~/.Trash
```

For a foreign item, `list` shows its skills.sh source when the lock file has it: `~/.agents/.skill-lock.json` at global scope, `skills-lock.json` in a project. `remove` without `--foreign` takes only links that the engine owns.

`-g` is the global scope and `-p` is a project. Without them, the scope is the project of `manifest.toml` that holds the current folder. `-a` names an agent, and you can give it more than one time. With `-a`, an agent that the scope does not have goes into `harnesses`, and then it also gets what `plugins` gives.

## Agents

`apps/workflow-setup/agents.toml` is the agent table: each agent that reads skills, and its skill folder in a project and in the home folder. `tools/agents-from-skills-sh.mjs` writes it from the table of skills.sh and from `agents.overrides.json`. Do not edit `agents.toml` by hand. Put a change in the overrides file with a reason, and run the script again. Also run it when skills.sh changes. Each agent of the table works in `harnesses` and as a key of a scope.

Each agent receives a plugin in one of three ways:

- **Claude Code** loads a plugin from a link in `.claude/skills/`. It reads the library in place, so an edit or an accepted update is live in the next session.
- **Codex** installs from this library as the marketplace `workflow`, and keeps its own copy. `apply` copies a plugin again when its content changed, and sets `[plugins."<name>@workflow"]` in `~/.codex/config.toml` and in each project's `.codex/config.toml`. A plugin is off at global scope unless `[global]` assigns it.
- **Every other agent**, opencode too, gets one link per skill in its skill folder, with no prefix. Many agents read the same folder, such as `.agents/skills/`. Each path gets one link. Where two plugins want the same path, `h00man` wins, then the order in `sources.toml`, and `status` reports the clash.

Codex also reads `.agents/skills/`. When Codex has a whole plugin and another agent, such as cursor, links a skill of that plugin into `.agents/skills/`, Codex gets the skill two times, as `h00man:plan` and as `plan`. `status` shows a warning for each such plugin. The links stay. For this reason opencode uses `.opencode/skills/`, which it also reads.

One skill (`<plugin>/<skill>`) is one link without a prefix, for every agent, Claude Code and Codex too.

Before the agent table, opencode got its links in `.opencode/skill/`. `apply` removes those owned links and makes them again in `.opencode/skills/`.

MCP servers go to the config of each agent:

- **Claude Code** reads project servers from `.mcp.json`, under `mcpServers`. For global servers, `apply` runs `claude mcp add-json --scope user` and `claude mcp remove --scope user`. It reads `~/.claude.json` to check drift but never edits that file by hand.
- **Codex** reads `[mcp_servers.<name>]` in `.codex/config.toml`, or `~/.codex/config.toml` at global scope. The engine keeps the rest of the TOML file as it is.
- Each project also gets `.agents/mcp.json`, with the same format as `.mcp.json`, when it uses Claude Code or Codex. There is no global neutral file.

The record owns each MCP entry by its file, server name, and value hash. A foreign entry with the same name, or an owned entry edited outside the engine, is a conflict and stays untouched. `--adopt` does not replace it. Dropping an assignment removes only matching owned entries. JSON files with comments stay untouched and report a conflict.

`apply`, `install`, `remove`, and `list` use one rule. The engine owns a link when its record has it, or when the link points into this library. In a Codex config file it owns the entries of the `workflow` marketplace. It never changes or removes an item it does not own: a link that something else made is a conflict until `--adopt`, and a real file or folder is never replaced. Before it changes a link or a config file, it writes the old state to a backup. The record and the backups are in `~/Library/Application Support/workflow-setup/`. A second `apply` changes nothing.

## My skills

**`plugins/h00man/skills/` — my own skills**, edited here. After an edit, run `workflow-setup apply` so that Codex gets the change.

- `plugins/h00man/skills/html/` — Produce single-file HTML artifacts instead of walls of markdown, each themed with a personality that fits its content.
- `plugins/h00man/skills/name/` — Generate and screen brand name candidates the way naming agencies do (taxonomy spread, sound symbolism, Abercrombie distinctiveness). Two modes — `quick` (no questions, ~20 names) and `deep` (a 5–10 question grill, then 100+ candidates). Outputs an interactive HTML artifact you filter and star into your own shortlist, built from a bundled `template.html` (the triage UI and state logic, rethemed per run). Also bundles `reference.md`, a deep-dive on naming theory, law, and case studies.

### Workflow suite

From idea to shipped, each step a skill:

- `plugins/h00man/skills/idea/` — Capture, list, validate, and prototype ideas in `docs/ideas/` with a status lifecycle (spark → validated → building → parked → dead). Capture ends by offering a throwaway prototype of the idea's riskiest assumption.
- `plugins/h00man/skills/variants/` — 3–5 structurally different takes on a component or UX flow, all on one route with a switcher. Pushes for conceptual range (remove a constraint, blend domains, invert the problem), not restyles of one idea.
- `plugins/h00man/skills/plan/` — PRD in `docs/plans/` plus tracer-bullet vertical-slice issues (optionally filed via `gh`). Two modes: `grill` (one-question-at-a-time interview first) or `trust` (no questions, synthesize and deliver). Maintains the knowledge base as it goes: a domain glossary in `docs/CONTEXT.md` and decision records in `docs/adr/`.
- `plugins/h00man/skills/frontend/` — Building and styling UI as a full operating mode: forced design commitment, token system before pixels, components/state/layout/accessibility discipline, craft details, a banned list for the generic AI look, and a self-critique loop before delivery. Also works pasted into another model's system prompt.
- `plugins/h00man/skills/backend/` — Server work and non-UI coding as a full operating mode: evidence before edits, done-as-a-check, surgical diffs, boundaries/authz/errors/migrations discipline, a banned list for generic AI code tells, and the same self-critique loop. Also works pasted into another model's system prompt.
- `plugins/h00man/skills/karpathy/` — Working discipline for any coding task: think before coding, simplicity first, surgical changes, goal-driven execution. Pushes back on the common LLM pitfalls (overcomplication, silent assumptions, drive-by refactors, vague success criteria). Adapted from [multica-ai/andrej-karpathy-skills](https://github.com/multica-ai/andrej-karpathy-skills) (MIT).
- `plugins/h00man/skills/motion/` — Animation defaults: durations, easing, transform/opacity only, reduced motion.
- `plugins/h00man/skills/improve/` — Audit-then-improve pass over `ui`, `ux`, `code`, `security`, or `all` (deep, adds an architecture/deepening pass). Always asks area + mode first: audit only, or execute the findings in one go.
- `plugins/h00man/skills/ship/` — Commit, push, and open pull requests with reviewer-ready git discipline: scoped diffs, protected user work, verification, deliberate commits, and compact PR bodies.
- `plugins/h00man/skills/copywriting/` — Product copywriting: website copy, UI microcopy, error messages, empty states.
- `plugins/h00man/skills/caveman/` — Ultra-compressed response mode: full technical accuracy, ~75% fewer tokens.
- `plugins/h00man/skills/benchmark/` — Benchmark models head-to-head or eval skills (with-skill vs bare baseline) via parallel sub-agents and a blind judge. HTML report in `benchmarks/` (gitignored) showing the work first, stats after. Quick mode by default; `deep` for more tasks and contenders.
- `plugins/h00man/skills/prompt/` — Generate prompts in `quick` mode for copyable one-offs or `structured` mode for Markdown records in `docs/prompts/`; default `text` mode writes a clean, structured LLM/agent prompt, while media modes `image`, `video`, `audio` cover MidJourney / Seedance / ElevenLabs. Use `plugins/h00man/skills/html/` for visual artifacts.

## Credits

Some of my skills started from **[Matt Pocock](https://github.com/mattpocock)**'s [skills repo](https://github.com/mattpocock/skills) (MIT), and still use his ideas:

- `plan` ← `grilling` + `domain-modeling` + `to-spec` + `to-tickets`
- `idea` ← `prototype` (the logic prototype)
- `variants` ← `prototype` (the UI prototype) and the "design it twice" method in `codebase-design`
- `improve` and `backend` ← `codebase-design` (module, interface, depth, seam, the deletion test)
- `caveman` ← `caveman`, which he removed later

My rewrites of `teach`, `handoff`, and `writing-for-agents` are retired. The originals are in `plugins/matt/`.

`karpathy` is adapted from [multica-ai/andrej-karpathy-skills](https://github.com/multica-ai/andrej-karpathy-skills) (MIT). `unslop` started from the pstack skill of the same name by Lauren Tan (MIT). `plugins/pstack/` is my port of [pstack](https://github.com/h00mankind/pstack).
