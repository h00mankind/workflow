# h00man workflow

One library for what my coding agents use: skills, agents, scripts, MCP servers, hooks, and instruction files. This repo is for my own use.

## Layout

```text
skills/
  h00man/            my own skills, edited here
  matt/              verbatim copy from mattpocock/skills, never edited here
  poteto/            verbatim copy from h00mankind/pstack, never edited here
agents/              same split by origin
mcp/                 one definition file per MCP server
scripts/             statusline, notifications, hooks
instructions/        shared AGENTS.md and CLAUDE.md parts
sources.toml         per source: repository, path, pinned commit
apps/workflow-setup  the engine: one window and one command line
```

A skill is one of two kinds: mine, or a verbatim copy from a source. When two origins have a skill with the same name, `h00man` wins, then the order in `sources.toml`. `workflow-setup library` reports each collision.

## Sources

An outside skill is a set of instructions that runs with my permissions. Thus each copy is committed to git at a pinned commit, and an update is a diff to read before it enters the library.

```bash
cd apps/workflow-setup && cargo build
alias workflow-setup="$PWD/target/debug/workflow-setup"

workflow-setup sources fetch          # get the newest commit of each source into .cache/
workflow-setup sources diff matt      # show what that commit would change, with the patch
workflow-setup sources accept matt    # write skills/matt/ and move the pin in sources.toml
workflow-setup library                # list the library and its name collisions
```

Only `accept` changes the library. It takes the commit that the last `fetch` found, which is the commit that `diff` showed. To add a source, add a table to `sources.toml`, then fetch, diff, and accept.

## My skills

**`skills/h00man/` — my own skills**, edited here.

- `skills/h00man/html/` — Produce single-file HTML artifacts instead of walls of markdown, each themed with a personality that fits its content.
- `skills/h00man/name/` — Generate and screen brand name candidates the way naming agencies do (taxonomy spread, sound symbolism, Abercrombie distinctiveness). Two modes — `quick` (no questions, ~20 names) and `deep` (a 5–10 question grill, then 100+ candidates). Outputs an interactive HTML artifact you filter and star into your own shortlist, built from a bundled `template.html` (the triage UI and state logic, rethemed per run). Also bundles `reference.md`, a deep-dive on naming theory, law, and case studies.

### Workflow suite

From idea to shipped, each step a skill:

- `skills/h00man/idea/` — Capture, list, validate, and prototype ideas in `docs/ideas/` with a status lifecycle (spark → validated → building → parked → dead). Capture ends by offering a throwaway prototype of the idea's riskiest assumption.
- `skills/h00man/variants/` — 3–5 structurally different takes on a component or UX flow, all on one route with a switcher. Pushes for conceptual range (remove a constraint, blend domains, invert the problem), not restyles of one idea.
- `skills/h00man/plan/` — PRD in `docs/plans/` plus tracer-bullet vertical-slice issues (optionally filed via `gh`). Two modes: `grill` (one-question-at-a-time interview first) or `trust` (no questions, synthesize and deliver). Maintains the knowledge base as it goes: a domain glossary in `docs/CONTEXT.md` and decision records in `docs/adr/`.
- `skills/h00man/frontend/` — Building and styling UI as a full operating mode: forced design commitment, token system before pixels, components/state/layout/accessibility discipline, craft details, a banned list for the generic AI look, and a self-critique loop before delivery. Also works pasted into another model's system prompt.
- `skills/h00man/backend/` — Server work and non-UI coding as a full operating mode: evidence before edits, done-as-a-check, surgical diffs, boundaries/authz/errors/migrations discipline, a banned list for generic AI code tells, and the same self-critique loop. Also works pasted into another model's system prompt.
- `skills/h00man/karpathy/` — Working discipline for any coding task: think before coding, simplicity first, surgical changes, goal-driven execution. Pushes back on the common LLM pitfalls (overcomplication, silent assumptions, drive-by refactors, vague success criteria). Adapted from [multica-ai/andrej-karpathy-skills](https://github.com/multica-ai/andrej-karpathy-skills) (MIT).
- `skills/h00man/motion/` — Animation defaults: durations, easing, transform/opacity only, reduced motion.
- `skills/h00man/improve/` — Audit-then-improve pass over `ui`, `ux`, `code`, `security`, or `all` (deep, adds an architecture/deepening pass). Always asks area + mode first: audit only, or execute the findings in one go.
- `skills/h00man/ship/` — Commit, push, and open pull requests with reviewer-ready git discipline: scoped diffs, protected user work, verification, deliberate commits, and compact PR bodies.
- `skills/h00man/copywriting/` — Product copywriting: website copy, UI microcopy, error messages, empty states.
- `skills/h00man/caveman/` — Ultra-compressed response mode: full technical accuracy, ~75% fewer tokens.
- `skills/h00man/benchmark/` — Benchmark models head-to-head or eval skills (with-skill vs bare baseline) via parallel sub-agents and a blind judge. HTML report in `benchmarks/` (gitignored) showing the work first, stats after. Quick mode by default; `deep` for more tasks and contenders.
- `skills/h00man/prompt/` — Generate prompts in `quick` mode for copyable one-offs or `structured` mode for Markdown records in `docs/prompts/`; default `text` mode writes a clean, structured LLM/agent prompt, while media modes `image`, `video`, `audio` cover MidJourney / Seedance / ElevenLabs. Use `skills/h00man/html/` for visual artifacts.

## Credits

Some of my skills started from **[Matt Pocock](https://github.com/mattpocock)**'s [skills repo](https://github.com/mattpocock/skills) (MIT), and still use his ideas:

- `plan` ← `grilling` + `domain-modeling` + `to-spec` + `to-tickets`
- `idea` ← `prototype` (the logic prototype)
- `variants` ← `prototype` (the UI prototype) and the "design it twice" method in `codebase-design`
- `improve` and `backend` ← `codebase-design` (module, interface, depth, seam, the deletion test)
- `caveman` ← `caveman`, which he removed later

My rewrites of `teach`, `handoff`, and `writing-for-agents` are retired. The originals are in `skills/matt/`.

`karpathy` is adapted from [multica-ai/andrej-karpathy-skills](https://github.com/multica-ai/andrej-karpathy-skills) (MIT). `unslop` started from the pstack skill of the same name by Lauren Tan (MIT). `skills/poteto/` is my port of [pstack](https://github.com/h00mankind/pstack).
