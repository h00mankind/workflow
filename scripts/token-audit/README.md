# token-audit

Two scripts that show where tokens go and stop the most expensive habit.

- `audit.py` reads the Claude Code and Codex transcripts and prints a cost report.
- `guard.py` is a `UserPromptSubmit` hook. It stops one prompt when the prompt would resend a large context after the cache expired.

Both need `python3` only. `audit.py` changes no files unless you pass `--json`.

## audit.py

```bash
python3 audit.py                 # last 14 days
python3 audit.py --days 7        # last 7 days
python3 audit.py --top 30 --json /tmp/audit.json
```

It reads `~/.claude/projects`, `~/.codex/sessions`, and `~/.codex/archived_sessions`. Prices come from `~/.t3/userdata/usage-model-rates.json`. The cost is an estimate at API rates, not your bill.

| Section | What it tells you |
|---|---|
| `COST SPLIT` | Cache reads, cache writes, output, and advisor calls. |
| `BY KIND` | Main threads, subagents, delegated threads, title and commit-message calls. |
| `COST BY MAX CONTEXT` | How much cost is in threads that grew past 200k tokens. |
| `CACHE_MISS` | Full-context rewrites, grouped by the idle time before them. |
| `ADVISOR` | Advisor calls. Each call sends the full transcript with no cache. |
| `TOOL_ONLY_REQUESTS` | Requests where the model wrote only a tool call. This is the work a cheaper model can do. |
| `TOP_TIER_SESSION_SHAPES` | Top-tier sessions with no edits (`read-only`) or few edits (`light-edit`). |
| `TOOLS_BY_RESULT_CHARS`, `BIGGEST_RESULTS` | Tools and single calls that put the most text in context. |
| `DUPLICATE_CALLS`, `SAME_TOOL_RUNS_5PLUS` | The same call sent again, and long runs of one tool. |
| `INLINE_SCRIPTS` | Scripts written inside a shell command. Look here for code to move into a library. |
| `SKILLS`, `MCP_SERVERS` | Skill loads and MCP result sizes. |

Token counts for tool results are an estimate: four characters for each token.

## guard.py

The prompt cache lasts one hour. After one hour, the next prompt writes the full context again at the cache-write rate. In a 300k-token Opus thread, that is about $2.40 for one prompt.

The hook blocks that prompt one time and shows the cost. Send the prompt again to continue in the same thread, or start a new thread with a handoff.

Add it to `~/.claude/settings.json`:

```json
{
  "hooks": {
    "UserPromptSubmit": [
      {"hooks": [{
        "type": "command",
        "command": "python3 /Users/YOU/Documents/h00man.code/workflow/scripts/token-audit/guard.py"
      }]}
    ]
  }
}
```

| Variable | Default | Meaning |
|---|---|---|
| `TOKEN_GUARD_MIN_TOKENS` | `150000` | Smallest context that the hook stops. |
| `TOKEN_GUARD_IDLE_MIN` | `60` | Idle minutes before the cache is cold. |

The hook is not tested inside T3 Code. A T3 thread may show the block differently from the Claude Code terminal.
