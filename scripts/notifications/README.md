# notifications

Small shell scripts that Claude Code's hooks call to drive desktop notifications and the statusline's live state badge.

| Script | Fired by hook | What it does |
|---|---|---|
| `mark-working.sh` | `UserPromptSubmit` | Writes `working` to `~/.claude/agent-state` — statusline shows **WORKING**. |
| `notify-stop.sh` | `Stop` | Writes `ready` to `~/.claude/agent-state`, then sends a "Claude finished" notification routed to the session's host app. The body is a one-line summary of the turn. |
| `notify-input.sh` | `Notification` | Writes `waiting` to `~/.claude/agent-state`, then sends a "Claude needs you" notification routed to the session's host app. |
| `host-app.sh` | (sourced by the two notify scripts) | Detects which app hosts the session, whether you're looking at it, builds the label, and extracts the turn summary. |
| `last-summary.py` | (called by `host-app.sh`) | Reads the session transcript and condenses Claude's last turn into a one-line summary. |

The state file is read by the [statusline](../statusline/README.md) script to render the **WORKING / READY / WAITING** badge on line 1.

---

## What a notification looks like

| Line | Source |
|---|---|
| **title** | A random line from the message pool — `stop-messages.txt` (a "done" voice) or `input-messages.txt` (a "needs you" voice). The status is conveyed by the line itself. |
| **subtitle** | *(Stop only, when a summary is available)* The `<branch> · <project>` label. |
| **body** | **Stop:** a one-line summary of what Claude just did, pulled from the transcript. Falls back to the `<branch> · <project>` label (as body, no subtitle) when no summary can be derived. **Input:** always the `<branch> · <project>` label. |

The label is `<branch> · <project>`, derived from the session's `cwd` (the project is the folder name). It collapses to just `<project>` outside a git repo, **and when the branch and folder names are identical** — common in worktrees, where `naruto · naruto` would otherwise read as a stutter.

Example — Stop hook: title `Returning to standby mode.`, subtitle `main · workflow`, body `Fixed the duplicate label in the worktree case.`

The hook receives a JSON payload on stdin. The scripts read `cwd` from it to build the label, and `notify-stop.sh` also reads `transcript_path` to build the summary. (Claude Code's hook payload has no human-readable session name, so the branch + project is the most useful glanceable label.)

### The turn summary

`session_summary` (in `host-app.sh`) shells out to `last-summary.py`, which reads the JSONL transcript, finds the last assistant message containing a text block, strips its markdown, and trims the first sentence to ~100 chars. It prints nothing when there's no transcript, no text, or `python3` is missing — so the body cleanly falls back to the label.

---

## How notifications are routed

A session's **host app** is a property of its process tree, not of what's on screen. `host-app.sh` walks the hook's process ancestry up to the host `.app` and matches its command path — so a session in Ghostty stays "hosted by Ghostty" even after you alt-tab to a browser.

`host_app` echoes one of `conductor` / `ghostty` / `zed` / `""` (unknown). The notify scripts route sound, icon, and click target from it:

| Host | Sound (Stop / Input) | Icon | Click opens |
|---|---|---|---|
| `conductor` | `chu-chuu.aiff` (played via `afplay`) | `conductor-icon.png` | Conductor.app (`com.conductor.app`) |
| `ghostty` | `Pop` / `Glass` | `notification-icon.png` | Ghostty (`com.mitchellh.ghostty`) |
| `zed` | *(none — Zed sends its own)* | — | — |
| unknown | `Pop` / `Glass` | `notification-icon.png` | Zed (`dev.zed.Zed`) |

### When does it stay silent?

`frontmost_is` compares the detected host against the current frontmost app. If the session's own window is already focused (e.g. session in Ghostty **and** Ghostty is frontmost), there's no point pinging — the chat surface is the notification — so the script exits early. State writes still happen unconditionally so the statusline stays accurate.

**Zed is also silent.** Zed now sends its own OS notification (and optional sound) when an agent turn finishes or waits for input — `agent.notify_when_agent_waiting` / `agent.play_sound_when_agent_done`, and the same for Terminal Threads on a bell. Our hook would fire a second notification, so Zed-hosted sessions skip the desktop ping after writing state.

**Conductor is the exception:** it runs a bundled `claude` binary with no clickable window, so there's no "looking at it" state to detect. Conductor sessions always notify (the Chu Chuu sound is the cue) and the click opens Conductor.app.

---

## Install

### 1. Requirements

- macOS (uses `osascript` for the frontmost check, `afplay` for custom sounds).
- `python3` for the Stop-hook turn summary. Without it, the Stop body falls back to the `<branch> · <project>` label.
- [`terminal-notifier`](https://github.com/julienXX/terminal-notifier): `brew install terminal-notifier`. Scripts no-op gracefully if it's missing.
- Message pools at `~/.claude/assets/` — one phrase per line, picked at random for the title:
  - `stop-messages.txt` — when Claude finishes.
  - `input-messages.txt` — when Claude asks for input.
- Icons / sounds at `~/.claude/assets/` (each is optional — a missing file degrades gracefully):
  - `notification-icon.png` — Ghostty (and unknown-host) icon.
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
| The turn summary (length, what counts) | `last-summary.py` — `max_len` arg in `session_summary`, or the extraction logic itself. |
| Always notify, even when focused | Remove the `frontmost_is "$HOST" && exit 0` line. |
| Notify Zed-hosted sessions again | Remove the `[ "$HOST" = zed ] && exit 0` line in both notify scripts. |

---

## Why these aren't inline in `settings.json`

`settings.json` originally had these as multi-line bash one-liners embedded as JSON strings — no highlighting, no comments, escapes everywhere, hard to diff. Real `.sh` files are readable, version-controlled, and shareable.
