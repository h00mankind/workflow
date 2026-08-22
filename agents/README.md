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

On any machine: clone this repo and run `./install.sh` from its root.

- **Global** (every tool, every project): only `caveman` + `html`,
  plus the discipline pointers and advisor subagents.
- **Per-project**: from this repo, run
  `./install.sh --project <dir> <skill...>` (or `all`) to link chosen
  skills into that project's `.claude/skills`, `.agents/skills`, and
  `.opencode/skill`.

Everything is symlinked — edits in this repo apply everywhere instantly.
Re-run anytime to repair links or change the global set
(`GLOBAL_SKILLS` at the top of `install.sh`).

## Editing the advisor prompt

The body is duplicated per tool because frontmatter schemas differ.
Keep the body identical across variants; only the frontmatter may differ.
