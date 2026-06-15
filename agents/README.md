# agents

Snapshots of the user-level instruction files each CLI agent reads by
default. Copies, not symlinks — the live files are the source of truth;
these are the version you'd seed onto a new machine.

| Agent | Snapshot | Lives at |
|---|---|---|
| Claude Code | `claude/CLAUDE.md` | `~/.claude/CLAUDE.md` |
| Codex CLI   | `codex/AGENTS.md`  | `~/.codex/AGENTS.md` |

Add a new agent as a sibling folder when its config matters
(e.g. `agents/cursor/`, `agents/opencode/`).

## Re-sync from the live files

```bash
cp ~/.claude/CLAUDE.md  agents/claude/CLAUDE.md
cp ~/.codex/AGENTS.md   agents/codex/AGENTS.md
```

## Bootstrap a new machine

```bash
mkdir -p ~/.claude ~/.codex
cp agents/claude/CLAUDE.md  ~/.claude/CLAUDE.md
cp agents/codex/AGENTS.md   ~/.codex/AGENTS.md
```
