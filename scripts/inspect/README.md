# inspect

A local page that shows what a project gives to its coding agents: skills, MCP servers, agents, and `CLAUDE.md` / `AGENTS.md` files.

It reads the project scope only. It does not read your global setup (`~/.claude/skills`, `~/.claude/CLAUDE.md`, user-scope MCP servers). It changes no files.

## Use

```bash
node server.mjs                    # inspect the current folder
node server.mjs ~/code/my-project  # inspect a different project
node server.mjs --port 5000        # default port is 4317
node server.mjs --no-open          # do not open the browser
```

Needs Node 18 or later. There are no dependencies and no build step.

To inspect a different project from the page, type its path in the field at the top and press Enter. The field suggests folders while you type and keeps your last eight projects.

Keys: `/` moves to the filter, the up and down arrows move through the list, `Esc` goes back to the list on a narrow window.

## What it reads

| Tab | Project paths |
|---|---|
| Skills | `.claude/skills/`, `.agents/skills/`, `.opencode/skill/`, `.opencode/skills/`, `.codex/skills/`, `.cursor/skills/`, `.gemini/skills/`, `.github/skills/`, `skills/` |
| MCP servers | `.mcp.json`, `.cursor/mcp.json`, `.vscode/mcp.json`, `.gemini/settings.json`, `opencode.json`, `opencode.jsonc`, `.opencode/opencode.json`, `.codex/config.toml` |
| Agents | `.claude/agents/`, `.opencode/agent/`, `.opencode/agents/`, `.codex/agents/`, `.cursor/agents/`, `.github/agents/`, `agents/` |
| Instructions | `CLAUDE.md`, `CLAUDE.local.md`, `AGENTS.md`, `AGENTS.override.md` at the root and up to four folders deep, `.claude/CLAUDE.md`, `.claude/rules/` |

A skill that is in more than one folder shows as one skill with one row for each location. The page tells you when a location is a symlink, when the link is broken, and when the `SKILL.md` copies are not the same.

`skills/` and `agents/` at the project root are authoring folders. No tool loads them from there, so items found only in these folders show as **not installed**.

There is one file outside the project that the page reads: `~/.claude.json`. Claude Code keeps the local-scope MCP servers of each project there, together with your approvals for `.mcp.json` servers. The page reads only the entry for the project you inspect.

## Limits

- The MCP status for Claude Code comes from the approval lists in `~/.claude.json` and `.claude/settings*.json`. A server shows **needs approval** when it is in `.mcp.json` but not in those lists. Other tools have no approval step, so they show **enabled** unless the config disables the server.
- The page does not start MCP servers or connect to them. It shows what the config says, not if the server works.
- Only the `[mcp_servers.*]` tables of `.codex/config.toml` are read, with a small parser. Multi-line TOML values are not supported.
- Files larger than 200 KB are cut off.

## Safety

The server listens on `127.0.0.1` only and refuses requests for other hostnames. MCP `env` values, header values, and URL query values stay on disk: the page gets the key names only. Arguments after flags such as `--token` or `--api-key` are replaced with `•••`.
