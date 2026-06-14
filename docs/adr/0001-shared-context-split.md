# 0001 — Use docs/ as the cross-tool knowledge base

**Status:** accepted
**Date:** 2026-06-14

## Context

Work in this repo jumps between Claude and Codex (and occasionally other agents).
Each tool only auto-loads its own instruction file — Claude reads `CLAUDE.md`,
Codex reads `AGENTS.md` — so context was being re-derived on every tool switch.

The need was a tool-agnostic place for shared knowledge that:
- both tools read at session start;
- both tools write back to as decisions and language crystallize;
- travels with the repo (survives clones, machine swaps).

## Decision

Use `docs/` as the single cross-tool knowledge base, with the `plan` skill's
existing conventions:

- `docs/CONTEXT.md` — domain glossary
- `docs/adr/NNNN-*.md` — decision records (settled, don't re-litigate)
- `docs/plans/<feature>.md` — PRDs

Both `CLAUDE.md` (global) and `AGENTS.md` (Codex global) carry a parallel
"Project knowledge base (docs/)" section that points here. Repo-local instruction
files are not required.

## Why

Plain markdown under `docs/` is genuinely tool-agnostic, committed, and
discoverable on GitHub. The `plan` skill already owned this convention — no new
machinery was needed, and the description of the protocol fit into a few lines
in each global instruction file.

A second folder for "live state" (running progress, architecture maps) was
considered and prototyped, then dropped — see Alternatives.

## Alternatives rejected

- **Add a `.context/` folder for live working state** (gitignored `STATE.md` +
  `MAP.md`). Prototyped end-to-end. Two problems: (1) `MAP.md` largely
  duplicated the project's `README.md` for repos of this size, and (2) the live
  files require manual upkeep that decays the moment a session skips a write.
  For a solo workflow with a good `README.md` and a populated `docs/`, the live
  half didn't earn its keep — it was noise.
- **Per-tool context** (parallel state in `CLAUDE.md` and `AGENTS.md`). Drifts
  apart fast; defeats the point of cross-tool continuity.
- **Auto-memory only** (Claude's per-project memory system). Claude-specific;
  Codex can't read it. Defeats tool-agnosticism.

## Consequences

- New repos that benefit from a shared knowledge base get one by running `/plan`
  for the first time — files are created lazily.
- HTML artifacts from the `html` and `name` skills go to `docs/html/`, committed
  by default (user can `.gitignore` case-by-case).
- No live-state convention exists — running progress lives in the conversation
  or in `.claude/notes/` (Claude) per the existing memory rule.
