# h00man workflow

My personal Claude Code skills.

## What's here

**`skills/` — for CLI agents** (Claude Code, Codex, Cursor, etc.). Installable via `skills.sh`.

- `skills/to-html/` — Produce single-file HTML artifacts instead of walls of markdown, each themed with a personality that fits its content.
- `skills/name/` — Generate and screen brand name candidates the way naming agencies do (taxonomy spread, sound symbolism, Abercrombie distinctiveness). Two modes — `quick` (no questions, ~20 names) and `deep` (a 5–10 question grill, then 100+ candidates). Outputs an interactive HTML artifact you filter and star into your own shortlist. Bundles `reference.md`, a deep-dive on naming theory, law, and case studies.

### Workflow suite

From idea to shipped, each step a skill:

- `skills/idea/` — Capture, list, validate, and prototype ideas in `docs/ideas/` with a status lifecycle (spark → validated → building → parked → dead). Capture ends by offering a throwaway prototype of the idea's riskiest assumption.
- `skills/variations/` — 3–5 radically different takes on a component or UX flow, all on one route with a switcher.
- `skills/plan/` — PRD in `docs/plans/` plus tracer-bullet vertical-slice issues (optionally filed via `gh`). Two modes: `grill` (one-question-at-a-time interview first) or `trust` (no questions, synthesize and deliver). Maintains the knowledge base as it goes: a domain glossary in `docs/CONTEXT.md` and decision records in `docs/adr/`.
- `skills/frontend/` — Building and styling UI: components, state, design tokens, layout, responsive, interactive controls, accessibility, verify in a real browser.
- `skills/backend/` — Working discipline for server work: boundaries, authz, errors, migrations, verify against the real endpoint.
- `skills/motion/` — Animation defaults: durations, easing, transform/opacity only, reduced motion.
- `skills/audit/` — Audit-then-fix pass: `/audit ui`, `ux`, `code`, or `safety` (secrets, injection, authz, PII, destructive ops). Read-only severity-ranked findings by default, then offers to apply the fixes.
- `skills/copywriting/` — Product copywriting: website copy, UI microcopy, error messages, empty states.
- `skills/learn/` — Multi-session teaching: small HTML lessons grounded in a mission, with records, glossary, and trusted sources tracked in `docs/learn/<topic>/`. Bundles format files for each workspace doc.
- `skills/handoff/` — Compact the conversation into a handoff document a fresh agent can pick up.
- `skills/caveman/` — Ultra-compressed response mode: full technical accuracy, ~75% fewer tokens.
- `skills/write-a-skill/` — Create or revise skills in this repo's house style: one-word names, trigger-rich descriptions, concise bodies.

## Credits

A big chunk of this suite is adapted from **[Matt Pocock](https://github.com/mattpocock)**'s excellent [skills repo](https://github.com/mattpocock/skills) — simplified and reshaped to fit my workflow, but the ideas, structures, and much of the philosophy are his:

- `plan` ← `grill-me` + `grill-with-docs` + `to-prd` + `to-issues`
- `idea` ← `prototype` (the prototype operation)
- `variations` ← `design-an-interface` + `prototype`'s UI branch
- `audit` ← `review` + `qa`
- `learn` ← `teach` (including the workspace format files)
- `caveman` ← `caveman`
- `handoff` ← `handoff`
- `write-a-skill` ← `write-a-skill`

If you like these, go get the originals: `npx skills add mattpocock/skills`. Thanks Matt 🙏

## Install

Use [skills.sh](https://www.skills.sh/) — symlinks by default, so `npx skills update` pulls in any changes.

```bash
# All skills, globally (available to every project)
npx skills add h00mankind/workflow -g

# A single skill
npx skills add h00mankind/workflow --skill to-html -g

# Project-scoped (drops into ./<agent>/skills/ in your project)
npx skills add h00mankind/workflow
```

Restart Claude Code (or open a new session) and the skills will show up.

### Update / remove

```bash
npx skills update              # pull latest for installed skills
npx skills remove to-html      # uninstall a skill
npx skills list                # see what's installed
```

## CSS CDN (retired)

`h00man-variables.css` was removed — artifacts are now themed per-context instead of from shared tokens. Anything still linking the old CDN URLs keeps working: `@v2/skills/to-html/h00man-variables.css` and `@v1/h00man-variables.css` are pinned to immutable tags forever.

## What's a symlink?

A symbolic link is a tiny file that points to another path. `skills.sh` keeps one canonical copy of each skill in `~/.skills/` and symlinks `~/.claude/skills/<name>` to it. When Claude Code reads the symlink, the OS transparently uses the canonical copy. `npx skills update` refreshes the canonical copy, so every symlinked location sees the new content at once.
