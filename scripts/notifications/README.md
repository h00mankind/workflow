# notifications

Three small shell scripts that Claude Code's hooks call to drive desktop notifications and the statusline's live state badge.

| Script | Fired by hook | What it does |
|---|---|---|
| `mark-working.sh` | `UserPromptSubmit` | Writes `working` to `~/.claude/agent-state` — statusline shows **WORKING**. |
| `notify-stop.sh` | `Stop` | Writes `ready` to `~/.claude/agent-state`. If Zed isn't frontmost, sends a "Claude finished" desktop notification with a random message. |
| `notify-input.sh` | `Notification` | Writes `waiting` to `~/.claude/agent-state`. If Zed isn't frontmost, sends a "Claude needs you" desktop notification. |

The state file is read by the [statusline](../statusline/README.md) script to render the **WORKING / READY / WAITING** badge on line 1.

---

## Why a "frontmost = Zed" check?

When Zed is already the foreground app, you can *see* Claude in the chat surface — a desktop notification would be noise. The scripts use `osascript` to check, and `exit 0` early when Zed is frontmost. State writes still happen unconditionally so the statusline stays accurate.

---

## Install

### 1. Requirements

- macOS (uses `osascript` for the frontmost-app check).
- [`terminal-notifier`](https://github.com/julienXX/terminal-notifier): `brew install terminal-notifier`. Scripts no-op gracefully if it's missing.
- Two random-message files at `~/.claude/assets/`:
  - `stop-messages.txt` — one phrase per line, picked at random when Claude finishes.
  - `input-messages.txt` — one phrase per line, picked at random when Claude asks for input.
- Optional: `~/.claude/assets/notification-icon.png` for the notification app icon.

### 2. Wire the hooks

Edit `~/.claude/settings.json` and replace any existing inline hook commands with calls to these files:

```json
{
  "hooks": {
    "UserPromptSubmit": [
      {"hooks": [{
        "type": "command",
        "command": "bash /Users/YOU/Documents/h00man.code/workflow/scripts/notifications/mark-working.sh"
      }]}
    ],
    "Stop": [
      {"hooks": [{
        "type": "command",
        "command": "bash /Users/YOU/Documents/h00man.code/workflow/scripts/notifications/notify-stop.sh"
      }]}
    ],
    "Notification": [
      {"hooks": [{
        "type": "command",
        "command": "bash /Users/YOU/Documents/h00man.code/workflow/scripts/notifications/notify-input.sh"
      }]}
    ]
  }
}
```

Replace `/Users/YOU/` with your real path.

---

## Customizing

| Want to change | Edit |
|---|---|
| Skip notifications when a *different* app is frontmost | The `front=` check in `notify-stop.sh` / `notify-input.sh` |
| Different sounds | `-sound Pop` (Stop) and `-sound Glass` (Notification) — use any name from `/System/Library/Sounds/` |
| Different icon | Replace `~/.claude/assets/notification-icon.png` (or change the `ICON=` path) |
| Different random message pool | Edit `~/.claude/assets/{stop,input}-messages.txt` |
| Always notify (even when Zed is frontmost) | Delete the `osascript` block in both scripts |

---

## Why these aren't inline in `settings.json`

`settings.json` originally had these as multi-line bash one-liners embedded as JSON strings. That works but is awful to maintain: no syntax highlighting, no comments, escapes everywhere, hard to diff. Extracting to real `.sh` files makes them readable, version-controlled, and shareable.
