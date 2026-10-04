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
workflow-setup sources fetch          # get the newest commit of each source into .cache/
workflow-setup sources diff pstack    # show what that commit would change, with the patch
workflow-setup sources accept pstack  # write plugins/pstack/ and move the pin in sources.toml
workflow-setup apply                  # give the new copy to Codex, which keeps its own copy
workflow-setup library                # list the plugins and their skills
```

Only `accept` changes the library. It takes the commit that the last `fetch` found, which is the commit that `diff` showed.

A source is one of two kinds. With `plugin = true`, its `path` is a complete plugin and is copied as it is (`pstack`). Without it, its `path` holds skills, `skills` can limit the copy to a list, and the engine writes the plugin manifests (`matt`).

`repository` can be a local clone. `pstack` reads my clone of the port, so a commit there is enough and no push is necessary.

## Manifest and apply

`manifest.toml` assigns plugins to `[global]` and to each `[project."<path>"]`, for every harness of the scope or for one harness only.

A scope can also set `mcp = ["<name>", ...]`. Each name is a definition in `mcp/<name>.toml`. Claude Code and Codex receive those servers when the scope names that harness. opencode skips them. An unknown plugin or server name is an error.

```bash
workflow-setup status                 # the drift between manifest.toml and the disk, for every scope
workflow-setup apply --dry-run        # the same, and change nothing
workflow-setup apply                  # make the disk match
workflow-setup apply global --adopt   # also replace links that something else made
```

Each harness receives a plugin in its own way:

- **Claude Code** loads a plugin from a link in `.claude/skills/`. It reads the library in place, so an edit or an accepted update is live in the next session.
- **Codex** installs from this library as the marketplace `workflow`, and keeps its own copy. `apply` copies a plugin again when its content changed, and sets `[plugins."<name>@workflow"]` in `~/.codex/config.toml` and in each project's `.codex/config.toml`. A plugin is off at global scope unless `[global]` assigns it.
- **opencode** has no plugins. It gets one link per skill in `.opencode/skill/`, with no prefix. Where two plugins share a name, `h00man` wins, then the order in `sources.toml`.

MCP servers go to each harness's config:

- **Claude Code** reads project servers from `.mcp.json`, under `mcpServers`. For global servers, `apply` runs `claude mcp add-json --scope user` and `claude mcp remove --scope user`. It reads `~/.claude.json` to check drift but never edits that file by hand.
- **Codex** reads `[mcp_servers.<name>]` in `.codex/config.toml`, or `~/.codex/config.toml` at global scope. The engine keeps the rest of the TOML file as it is.
- Each project also gets `.agents/mcp.json`, with the same format as `.mcp.json`, when it uses Claude Code or Codex. There is no global neutral file.

The record owns each MCP entry by its file, server name, and value hash. A foreign entry with the same name, or an owned entry edited outside the engine, is a conflict and stays untouched. `--adopt` does not replace it. Dropping an assignment removes only matching owned entries. JSON files with comments stay untouched and report a conflict.

`apply` owns a link when its record has it, or when the link points into this library. In a Codex config file it owns the entries of the `workflow` marketplace. It never changes or removes an item it does not own: a link that something else made is a conflict until `--adopt`, and a real file or folder is never replaced. Before it changes a link or a config file, it writes the old state to a backup. The record and the backups are in `~/Library/Application Support/workflow-setup/`. A second `apply` changes nothing.

## MCP server

An agent harness can run the engine as a stdio MCP server. It offers `library`, `manifest`, `status`, and `sources` to read state; `assign_plugin` and `assign_mcp` to edit only assignments in `manifest.toml`; and `apply` to install them. `apply` is a dry run unless `dry_run` is false, and it never adopts foreign items. Source updates stay with the human: fetch, diff, and accept are not tools.

After building, configure the harness to run this exact command. The binary path is machine-specific; change it and the library path on another machine.

```bash
env WORKFLOW_LIBRARY=/Users/h0_0man/Documents/h00man.code/workflow /Users/h0_0man/Documents/h00man.code/workflow/apps/workflow-setup/target/debug/workflow-setup mcp-server
```

The server uses MCP revision `2025-11-25`, with the `initialize` handshake and newline-delimited JSON-RPC on stdin and stdout.

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
