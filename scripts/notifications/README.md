# notifications

Small shell scripts that Claude Code's hooks call to drive desktop notifications and the statusline's live state badge.

| Script | Fired by hook | What it does |
|---|---|---|
| `mark-working.sh` | `UserPromptSubmit` | Writes `working` to `~/.claude/agent-state` — statusline shows **WORKING**. |
| `notify-stop.sh` | `Stop` | Writes `ready` to `~/.claude/agent-state`, then sends a "Claude finished" notification routed to the session's host app. |
| `notify-input.sh` | `Notification` | Writes `waiting` to `~/.claude/agent-state`, then sends a "Claude needs you" notification routed to the session's host app. |
| `host-app.sh` | (sourced by the two notify scripts) | Detects which app hosts the session, whether you're looking at it, and builds the body label. |

The state file is read by the [statusline](../statusline/README.md) script to render the **WORKING / READY / WAITING** badge on line 1.

---

## What a notification looks like

| Line | Source |
|---|---|
| **title** | A random line from the message pool — `stop-messages.txt` (a "done" voice) or `input-messages.txt` (a "needs you" voice). The status is conveyed by the line itself. |
| **body** | `<branch> · <project>`, derived from the session's `cwd` (the project is the folder name). Falls back to just `<project>` outside a git repo. |

Example — Stop hook: title `Returning to standby mode.`, body `main · workflow`.

The hook receives a JSON payload on stdin; the scripts read `cwd` from it to build the body label. (Claude Code's hook payload has no human-readable session name, so the branch + project is the most useful glanceable label.)

---

## How notifications are routed

A session's **host app** is a property of its process tree, not of what's on screen. `host-app.sh` walks the hook's process ancestry up to the host `.app` and matches its command path — so a session in Ghostty stays "hosted by Ghostty" even after you alt-tab to a browser.

`host_app` echoes one of `conductor` / `ghostty` / `zed` / `""` (unknown). The notify scripts route sound, icon, and click target from it:

| Host | Sound (Stop / Input) | Icon | Click opens |
|---|---|---|---|
| `conductor` | `chu-chuu.aiff` (played via `afplay`) | `conductor-icon.png` | Conductor.app (`com.conductor.app`) |
| `ghostty` | `Pop` / `Glass` | `notification-icon.png` | Ghostty (`com.mitchellh.ghostty`) |
| `zed` / unknown | `Pop` / `Glass` | `notification-icon.png` | Zed (`dev.zed.Zed`) |

### When does it stay silent?

`frontmost_is` compares the detected host against the current frontmost app. If the session's own window is already focused (e.g. session in Ghostty **and** Ghostty is frontmost), there's no point pinging — the chat surface is the notification — so the script exits early. State writes still happen unconditionally so the statusline stays accurate.

**Conductor is the exception:** it runs a bundled `claude` binary with no clickable window, so there's no "looking at it" state to detect. Conductor sessions always notify (the Chu Chuu sound is the cue) and the click opens Conductor.app.

---

## Install

### 1. Requirements

- macOS (uses `osascript` for the frontmost check, `afplay` for custom sounds).
- [`terminal-notifier`](https://github.com/julienXX/terminal-notifier): `brew install terminal-notifier`. Scripts no-op gracefully if it's missing.
- Message pools at `~/.claude/assets/` — one phrase per line, picked at random for the title:
  - `stop-messages.txt` — when Claude finishes.
  - `input-messages.txt` — when Claude asks for input.
- Icons / sounds at `~/.claude/assets/` (each is optional — a missing file degrades gracefully):
  - `notification-icon.png` — shared Ghostty/Zed icon.
  - `conductor-icon.png` — Conductor icon.
  - `chu-chuu.aiff` — Conductor sound (any `afplay`-playable audio file).

### 2. Wire the hooks

Edit `~/.claude/settings.json`:

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
| The title wording | `~/.claude/assets/{stop,input}-messages.txt` — one line per phrase. |
| Add a new host app | Add a `case` arm in `host_app` (`host-app.sh`), then a matching arm in both notify scripts' routing `case`. |
| Different sounds per host | The routing `case` in `notify-stop.sh` / `notify-input.sh`. Built-in names from `/System/Library/Sounds/`, or a file path for a custom sound. |
| Different icon per host | Drop a PNG in `~/.claude/assets/` and point the `case` arm at it. |
| The body label | The `session_label` helper in `host-app.sh`. |
| Always notify, even when focused | Remove the `frontmost_is "$HOST" && exit 0` line. |

---

## Why these aren't inline in `settings.json`

`settings.json` originally had these as multi-line bash one-liners embedded as JSON strings — no highlighting, no comments, escapes everywhere, hard to diff. Real `.sh` files are readable, version-controlled, and shareable.
