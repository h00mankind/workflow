# statusline

A four-line Claude Code statusline ported from the Antigravity CLI's [`agy-statusline`](https://codeberg.org/jochenkirstaetter/agy-statusline), adapted for Claude Code's JSON payload.

Shows agent state, model + effort, context window, working dir + git branch, 5h and 7d rate limits with reset countdowns, and session cost.

![preview](docs/preview.png)

---

## What it shows

```
 WORKING │  Opus 4.7 [xhi] │ 󱍏 13% 130K/1M
 ~/Documents/h00man.code/workflow │  main
 5h  █████▓░░░░░░░░  41% 2h 29m
 7d  ▓░░░░░░░░░░░░░   6% 5d 21h │  $3.02
```

| Line | Content |
|---|---|
| 1 | **State badge** · model + effort · context (icon + %, color follows usage) |
| 2 | Current dir · git branch (own line so worktree paths don't push branch off-screen) |
| 3 | 5-hour rate limit: bar · % · time until reset |
| 4 | 7-day rate limit: bar · % · time until reset · session cost |

**State badge** lights up by hook signal:
- 🟢 `READY` — last `Stop` hook (Claude finished a turn)
- 🟡 `WORKING` — last `UserPromptSubmit` hook (you submitted a prompt)
- 🔴 `WAITING` — last `Notification` hook (Claude needs your input)

A 10-minute staleness guard drops back to `READY` if no hook has fired recently (catches abandoned sessions).

---

## Install

### 1. Requires a Nerd Font

The icons (` 󱍏     `) need a Nerd Font installed and selected in your terminal. Recommended:

```bash
brew install --cask font-jetbrains-mono-nerd-font
```

Then point your terminal/editor at `JetBrainsMono Nerd Font`. For Zed users:

```json
// ~/.config/zed/settings.json
{
  "terminal": {
    "font_family": "JetBrainsMono Nerd Font"
  }
}
```

If you don't want a Nerd Font, set `USE_NERD_FONTS=false` in your shell env — the script falls back to ASCII labels.

### 2. Copy the script

```bash
cp statusline.sh ~/.claude/statusline-command.sh
chmod +x ~/.claude/statusline-command.sh
```

### 3. Wire it into Claude Code

Edit `~/.claude/settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "bash /Users/YOU/.claude/statusline-command.sh"
  }
}
```

### 4. Add the state hooks (optional but recommended)

Without these, the state badge always reads `READY`. With them, you get live `WORKING` / `READY` / `WAITING` transitions.

```json
{
  "hooks": {
    "UserPromptSubmit": [
      {"hooks": [{"type": "command",
        "command": "printf working > /Users/YOU/.claude/agent-state"}]}
    ],
    "Stop": [
      {"hooks": [{"type": "command",
        "command": "printf ready > /Users/YOU/.claude/agent-state"}]}
    ],
    "Notification": [
      {"hooks": [{"type": "command",
        "command": "printf waiting > /Users/YOU/.claude/agent-state"}]}
    ]
  }
}
```

Replace `/Users/YOU/` with your real `$HOME`.

---

## Customizing

All ANSI colors, icon glyphs, and percentage thresholds are at the top of the file. Common tweaks:

| Want to change | Edit |
|---|---|
| Color thresholds (green/yellow/red) | `pct_color()` — currently 40 / 80 |
| Bar length per terminal width | `BAR_LEN` block — scales 8/10/12/14 |
| Model color per model name | `MODEL_COLOR` block — Opus purple, Sonnet sky-blue, Haiku mint, Fable pink |
| Effort short labels | `effort_label()` — `low/med/high/xhi/max` |
| Path shortening | `shorten_path()` — currently keeps last 2 segments if path > 60 chars |
| Cost estimate (when payload has none) | `COST_TXT` awk line — currently `$3/M in, $15/M out` (Opus rates) |
| Stale-state grace period | `if [ "$age" -lt 600 ]` — 600 seconds |

---

## Why a port and not just use agy-statusline

The agy script reads Antigravity CLI's JSON payload. Claude Code emits a **different** payload shape — no `agent_state`, no `quota`, no `artifact_count`; different field names (`workspace.current_dir` not `cwd`, `session_id` not `conversation_id`); different rate-limit keys.

This port keeps the agy visual style (Nerd Font icons, multi-line layout, block bars, color tiers) but reads the actual fields Claude Code provides and drops the ones it doesn't:

- **Kept from agy:** Nerd Font icon set, block bars (`█▓▒░`), color-by-percentage, model color tinting.
- **Adapted for Claude Code:** payload field paths, 5h + 7d rate-limit bars (replacing agy's gemini quota), `.workspace.current_dir`, `.cost.total_cost_usd`.
- **Added:** state badge via hook file, lowercase effort label, branch on its own row (worktree-safe), circular/icon context indicator, ClaudeMascot snapshot writer.

---

## Credits

- Original layout & visual style: [`jochenkirstaetter/agy-statusline`](https://codeberg.org/jochenkirstaetter/agy-statusline) (MIT)
- Nerd Font icons: [nerdfonts.com](https://www.nerdfonts.com)
