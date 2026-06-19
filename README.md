# h00man workflow

My personal Claude Code skills.

## What's here

**`docs/` — cross-tool knowledge base.** Glossary (`docs/CONTEXT.md`), decision records (`docs/adr/`), PRDs (`docs/plans/`). Read by Claude and Codex globals so context follows the work across tools. See [`docs/adr/0001-shared-context-split.md`](docs/adr/0001-shared-context-split.md).

**`skills/` — for CLI agents** (Claude Code, Codex, Cursor, etc.). Installable via `skills.sh`.

- `skills/html/` — Produce single-file HTML artifacts instead of walls of markdown, each themed with a personality that fits its content.
- `skills/name/` — Generate and screen brand name candidates the way naming agencies do (taxonomy spread, sound symbolism, Abercrombie distinctiveness). Two modes — `quick` (no questions, ~20 names) and `deep` (a 5–10 question grill, then 100+ candidates). Outputs an interactive HTML artifact you filter and star into your own shortlist, built from a bundled `template.html` (the triage UI and state logic, rethemed per run). Also bundles `reference.md`, a deep-dive on naming theory, law, and case studies.

### Workflow suite

From idea to shipped, each step a skill:

- `skills/idea/` — Capture, list, validate, and prototype ideas in `docs/ideas/` with a status lifecycle (spark → validated → building → parked → dead). Capture ends by offering a throwaway prototype of the idea's riskiest assumption.
- `skills/variants/` — 3–5 structurally different takes on a component or UX flow, all on one route with a switcher. Pushes for conceptual range (remove a constraint, blend domains, invert the problem), not restyles of one idea.
- `skills/plan/` — PRD in `docs/plans/` plus tracer-bullet vertical-slice issues (optionally filed via `gh`). Two modes: `grill` (one-question-at-a-time interview first) or `trust` (no questions, synthesize and deliver). Maintains the knowledge base as it goes: a domain glossary in `docs/CONTEXT.md` and decision records in `docs/adr/`.
- `skills/frontend/` — Building and styling UI as a full operating mode: forced design commitment, token system before pixels, components/state/layout/accessibility discipline, craft details, a banned list for the generic AI look, and a mandatory hostile self-critique loop run twice. Also works pasted into another model's system prompt.
- `skills/backend/` — Server work and non-UI coding as a full operating mode: evidence before edits, done-as-a-check, surgical diffs, boundaries/authz/errors/migrations discipline, a banned list for generic AI code tells, and the same mandatory self-critique loop. Also works pasted into another model's system prompt.
- `skills/karpathy/` — Working discipline for any coding task: think before coding, simplicity first, surgical changes, goal-driven execution. Pushes back on the common LLM pitfalls (overcomplication, silent assumptions, drive-by refactors, vague success criteria). Adapted from [multica-ai/andrej-karpathy-skills](https://github.com/multica-ai/andrej-karpathy-skills) (MIT).
- `skills/motion/` — Animation defaults: durations, easing, transform/opacity only, reduced motion.
- `skills/improve/` — Audit-then-improve pass over `ui`, `ux`, `code`, `security`, or `all` (deep, adds an architecture/deepening pass). Always asks area + mode first: audit only, or execute the findings in one go.
- `skills/ship/` — Commit, push, and open pull requests with reviewer-ready git discipline: scoped diffs, protected user work, verification, deliberate commits, and compact PR bodies.
- `skills/copywriting/` — Product copywriting: website copy, UI microcopy, error messages, empty states.
- `skills/learn/` — Multi-session teaching: small HTML lessons grounded in a mission, built from a shared `assets/` component library so a course reads as one consistent thing, with records, glossary, and trusted sources tracked in `docs/learn/<topic>/`. Bundles format files for each workspace doc.
- `skills/handoff/` — Compact the conversation into a handoff document a fresh agent can pick up.
- `skills/caveman/` — Ultra-compressed response mode: full technical accuracy, ~75% fewer tokens.
- `skills/write-a-skill/` — Create or revise skills in this repo's house style: one-word names, trigger-rich descriptions, concise bodies.
- `skills/benchmark/` — Benchmark models head-to-head or eval skills (with-skill vs bare baseline) via parallel sub-agents and a blind judge. HTML report in `benchmarks/` (gitignored) showing the work first, stats after. Quick mode by default; `deep` for more tasks and contenders.
- `skills/prompt/` — Generate prompts. Default `text` mode writes a clean, structured LLM/agent prompt from a task (or tidies a rough one); media modes `image`, `video`, `audio` cover MidJourney / Seedance / ElevenLabs for specific jobs. Output is a self-contained editorial HTML page in `docs/prompts/NNNN-<mode>-<slug>/` with one-click copy buttons.

### Experimental

**`experimental/`** — skills that are bets, not tools. Not installed by default.

- `experimental/frontend-fable-mode/` — an attempt to bottle frontier-model frontend taste as a loadable operating mode for *any* model (Opus, GPT, Gemini, …): forced design commitment, system-before-pixels, craft details, a banned list for the generic AI look, and a mandatory hostile self-critique loop run twice. Load it as a skill or paste it into a system prompt.
- `experimental/code-fable-mode/` — the same bet for agentic coding: bottle the *behaviors* behind frontier coding scores — evidence before edits, done-as-a-check (name the command that proves it before editing), surgical diffs, a banned list for generic AI code tells (swallowed exceptions, weakened tests, drive-by refactors), and the same mandatory hostile self-critique loop run twice. Load it as a skill or paste it into a system prompt.

Experimental skills aren't picked up by a plain `skills add` (it only scans `skills/`). Install them with `--full-depth`:

```bash
npx skills add h00mankind/workflow --skill frontend-fable-mode --full-depth -g
```

## Credits

A big chunk of this suite is adapted from **[Matt Pocock](https://github.com/mattpocock)**'s excellent [skills repo](https://github.com/mattpocock/skills) (MIT licensed) — simplified and reshaped to fit my workflow, but the ideas, structures, and much of the philosophy are his:

- `plan` ← `grill-me` + `grill-with-docs` + `to-prd` + `to-issues`
- `idea` ← `prototype` (the prototype operation)
- `variants` ← `design-an-interface` + `prototype`'s UI branch
- `audit` ← `review` + `qa`
- `learn` ← `teach` (including the workspace format files)
- `caveman` ← `caveman`
- `handoff` ← `handoff`
- `write-a-skill` ← `write-a-skill` + `writing-great-skills` (the `reference.md` theory — predictability, the two loads, the information-hierarchy ladder, leading words, failure modes — comes from his [1.0.0](https://github.com/mattpocock/skills/releases/tag/mattpocock-skills%401.0.0) rewrite)
- `backend`'s "Design deep modules" section ← `codebase-design` (module / interface / depth / seam, the deletion test)

If you like these, go get the originals: `npx skills add mattpocock/skills`. Thanks Matt 🙏

## Install

Use [skills.sh](https://www.skills.sh/) — symlinks by default, so `npx skills update` pulls in any changes.

```bash
# All skills, globally (available to every project)
npx skills add h00mankind/workflow -g

# A single skill
npx skills add h00mankind/workflow --skill html -g

# Project-scoped (drops into ./<agent>/skills/ in your project)
npx skills add h00mankind/workflow
```

Restart Claude Code (or open a new session) and the skills will show up.

### Update / remove

```bash
npx skills update              # pull latest for installed skills
npx skills remove html         # uninstall a skill
npx skills list                # see what's installed
```

> **Renamed (2026-06-12):** `to-html` → `html`, `audit` → `improve`. `npx skills update` won't rename an installed skill — reinstall under the new name:
>
> ```bash
> npx skills remove to-html && npx skills add h00mankind/workflow --skill html -g
> npx skills remove audit && npx skills add h00mankind/workflow --skill improve -g
> ```

## What's a symlink?

A symbolic link is a tiny file that points to another path. `skills.sh` keeps one canonical copy of each skill in `~/.skills/` and symlinks `~/.claude/skills/<name>` to it. When Claude Code reads the symlink, the OS transparently uses the canonical copy. `npx skills update` refreshes the canonical copy, so every symlinked location sees the new content at once.
