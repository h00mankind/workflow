#!/usr/bin/env bash
# Claude Code "Stop" hook — fires when Claude finishes a turn.
# - Writes "ready" to ~/.claude/agent-state (read by statusline).
# - If Zed is NOT frontmost, sends a desktop notification with a random message.
#   (When Zed is already up front, the chat surface itself is the notification.)
#
# Wired in ~/.claude/settings.json:
#   hooks.Stop[0].hooks[0].command = "bash <repo>/scripts/notify-stop.sh"
set -uo pipefail

# 1. Mark agent state for the statusline.
printf ready > "$HOME/.claude/agent-state"

# 2. Skip the desktop notification if Zed is the frontmost app.
front=$(osascript -e 'tell application "System Events" to name of first process whose frontmost is true' 2>/dev/null)
if [ "$front" = "Zed" ] || [ "$front" = "zed" ]; then
  exit 0
fi

# 3. Pick a random non-blank line from stop-messages.txt.
MSG_FILE="$HOME/.claude/assets/stop-messages.txt"
ICON="$HOME/.claude/assets/notification-icon.png"
[ -r "$MSG_FILE" ] || exit 0

msg=$(awk 'NF' "$MSG_FILE" | awk 'BEGIN{srand()} {a[NR]=$0} END{print a[int(rand()*NR)+1]}')

# 4. Fire the notification (silent if terminal-notifier is missing).
NOTIFIER=/opt/homebrew/bin/terminal-notifier
[ -x "$NOTIFIER" ] || exit 0

"$NOTIFIER" \
  -title 'Claude Code' \
  -message "$msg" \
  -sound Pop \
  -appIcon "$ICON" \
  -activate dev.zed.Zed
