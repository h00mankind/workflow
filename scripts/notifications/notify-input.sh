#!/usr/bin/env bash
# Claude Code "Notification" hook — fires when Claude needs your input
# (asks a question, awaits permission, etc.).
# - Writes "waiting" to ~/.claude/agent-state (read by statusline).
# - If Zed is NOT frontmost, sends a desktop notification with a random message.
#
# Wired in ~/.claude/settings.json:
#   hooks.Notification[0].hooks[0].command = "bash <repo>/scripts/notify-input.sh"
set -uo pipefail

# 1. Mark agent state for the statusline.
printf waiting > "$HOME/.claude/agent-state"

# 2. Skip if Zed is frontmost.
front=$(osascript -e 'tell application "System Events" to name of first process whose frontmost is true' 2>/dev/null)
if [ "$front" = "Zed" ] || [ "$front" = "zed" ]; then
  exit 0
fi

# 3. Pick a random non-blank line.
MSG_FILE="$HOME/.claude/assets/input-messages.txt"
ICON="$HOME/.claude/assets/notification-icon.png"
[ -r "$MSG_FILE" ] || exit 0

msg=$(awk 'NF' "$MSG_FILE" | awk 'BEGIN{srand()} {a[NR]=$0} END{print a[int(rand()*NR)+1]}')

# 4. Fire it.
NOTIFIER=/opt/homebrew/bin/terminal-notifier
[ -x "$NOTIFIER" ] || exit 0

"$NOTIFIER" \
  -title 'Claude Code' \
  -message "$msg" \
  -sound Glass \
  -appIcon "$ICON" \
  -activate dev.zed.Zed
