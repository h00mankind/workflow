# agents

Subagent definitions. Same layout as skills: folder per agent, one
variant per coding tool, since agent frontmatter differs per tool.

| Path | What |
|---|---|
| `advisor/claude/advisor.md` | Advisor subagent for Claude Code — advice only, no edits |
| `advisor/opencode/advisor.md` | Advisor subagent for OpenCode |

Install by copying the variant you need:

```bash
mkdir -p ~/.claude/agents
cp agents/advisor/claude/advisor.md ~/.claude/agents/advisor.md

# or for opencode
mkdir -p ~/.config/opencode/agent
cp agents/advisor/opencode/advisor.md ~/.config/opencode/agent/advisor.md
```

Skills are installed separately — see the repo README.
