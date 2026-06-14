# Cross-tool knowledge base via docs/

## Problem

When working in this repo, the user jumps between Claude and Codex. Each tool
auto-loads only its own instruction file, so context — what the active goal is,
what's already been decided, what the project's terms mean — is re-derived
every time the tool changes.

## Solution

Use `docs/` as the single cross-tool knowledge base. The `plan` skill already
owns the conventions: `docs/CONTEXT.md` (glossary), `docs/adr/` (decision
records), `docs/plans/` (PRDs). Both Claude and Codex globals carry a parallel
"Project knowledge base (docs/)" section that points here, so both tools read
and write the same files.

A `.context/` folder for live working state was prototyped end-to-end and
dropped — it was noise for solo work with a good `README.md`. See
`docs/adr/0001-shared-context-split.md` for the full reasoning.

## User stories

1. As a solo developer switching from Claude to Codex mid-task, I want both
   tools to see the same glossary, decisions, and active PRD so I don't
   re-explain.
2. As a returning user opening this repo on a new machine, I want the knowledge
   base present (via `docs/`) so the project's history survives.
3. As an agent on the first turn in a repo with a populated `docs/`, I want to
   read what bears on the work before acting and use the project's vocabulary
   throughout.

## Decisions

(All assumed — synthesized from conversation history, no grilling.)

- **`docs/` is the cross-tool knowledge base.** Owned by the `plan` skill;
  read by Claude (via global `CLAUDE.md`) and Codex (via global `AGENTS.md`).
- **No `.context/` folder.** Prototyped, rejected as noise for this scale.
  Live state lives in the conversation or `.claude/notes/`.
- **HTML artifacts go to `docs/html/`.** Committed by default; user can
  `.gitignore` case-by-case for private outputs.
- **Names kept conventional.** `CONTEXT.md`, `adr/`, `plans/` keep their
  industry-standard names.
- **No repo-local instruction files required.** The globals carry the pointer;
  per-repo `CLAUDE.md` / `AGENTS.md` only when a repo needs something custom.

## Open questions

- Should `docs/html/` outputs be selectively gitignored per-file when private?
  Current default: commit all; user can opt out case-by-case.

## Out of scope

- Cross-repo shared context. This is per-repo.
- Automation (hooks that auto-write the knowledge base). Manual via `/plan`.
- Tool-specific extensions inside `docs/`. Kept tool-agnostic.

---

## Slices

### Slice 1: Skill paths — DONE

- [x] `plan` writes to `docs/CONTEXT.md`, `docs/adr/`, `docs/plans/`
- [x] `html` and `name` skills write to `docs/html/`
- [x] `.gitignore` drops `docs/artifacts/`

### Slice 2: Global pointers — DONE

- [x] Add "Project knowledge base (docs/)" section to `~/.claude/CLAUDE.md`
- [x] Add the same section to Codex's `~/.codex/AGENTS.md`

### Slice 3: Seed durable content — DONE

- [x] `docs/CONTEXT.md` — initial glossary
- [x] `docs/adr/0001-shared-context-split.md` — this decision
- [x] `docs/plans/shared-context-split.md` — this PRD

### Slice 4: Commit and push — NEXT

- [ ] Commit `docs/` additions
- [ ] Push to `origin/main`
