# agents

Shared agent config, single source of truth. Each tool's global memory
file is a pointer into this folder — edit here, never in the global file.

Layout: `agents/<agent>/<tool>/<tool>.md` — one subfolder per coding
agent, since agent frontmatter differs per tool.

| Path | What | Installed to |
|---|---|---|
| `discipline/discipline.md` | Working rules shared by every CLI agent (tool-neutral, no variants needed) | `~/.claude/CLAUDE.md`, `~/.codex/AGENTS.md` point at it |
| `advisor/claude/advisor.md` | Advisor subagent for Claude Code | `~/.claude/agents/advisor.md` |
| `advisor/opencode/advisor.md` | Advisor subagent for OpenCode | `~/.config/opencode/agent/advisor.md` |

Codex has no stable user-defined subagent format yet — it gets
`discipline.md` only. Add `agents/advisor/codex/` when that changes.

## Install / re-sync

```bash
cp agents/advisor/claude/advisor.md   ~/.claude/agents/advisor.md
cp agents/advisor/opencode/advisor.md ~/.config/opencode/agent/advisor.md
```

`discipline.md` needs no copy — the global files are two-line stubs:

`~/.claude/CLAUDE.md`:

```
# CLAUDE.md

@/<path-to-this-repo>/agents/discipline/discipline.md
```

`~/.codex/AGENTS.md`:

```
# AGENTS.md

Read /<path-to-this-repo>/agents/discipline/discipline.md first and follow it for every task in every repo.
```

## Editing the advisor prompt

The body is duplicated per tool because frontmatter schemas differ.
Keep the body identical across variants; only the frontmatter may differ.
