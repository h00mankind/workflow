#!/usr/bin/env bash
# Claude Code "Notification" hook — fires when Claude needs your input
# (asks a question, awaits permission, etc.).
# - Writes "waiting" to ~/.claude/agent-state (read by statusline).
# - Sends a desktop notification routed to the session's HOST app (see
#   host-app.sh): Conductor gets its own sound/icon and opens Conductor.app;
#   Ghostty and Zed share a sound/icon and open their own window. We stay
#   silent when you're already looking at the session's window (frontmost),
#   except for Conductor which has no clickable window.
#
# Wired in ~/.claude/settings.json:
#   hooks.Notification[0].hooks[0].command = "bash <repo>/scripts/notify-input.sh"
set -uo pipefail

# 0. Read the hook JSON from stdin and pull out cwd (for the body label).
HOOK_JSON=$(cat)
CWD=$(printf '%s' "$HOOK_JSON" | sed -n 's/.*"cwd"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')

# 1. Mark agent state for the statusline.
printf waiting > "$HOME/.claude/agent-state"

# 2. Figure out which app hosts this session.
source "$(dirname "${BASH_SOURCE[0]}")/host-app.sh"
HOST=$(host_app)

# 3. Stay silent if you're already looking at the session's own window.
frontmost_is "$HOST" && exit 0

# 4. Route sound / icon / click-target by host.
ASSETS="$HOME/.claude/assets"
case "$HOST" in
  conductor)
    SOUND="$ASSETS/chu-chuu.aiff"; ICON="$ASSETS/conductor-icon.png"; ACTIVATE=com.conductor.app ;;
  ghostty)
    SOUND=Glass; ICON="$ASSETS/notification-icon.png"; ACTIVATE=com.mitchellh.ghostty ;;
  *)
    SOUND=Glass; ICON="$ASSETS/notification-icon.png"; ACTIVATE=dev.zed.Zed ;;
esac

# 5. Title = a random "needs you" line from the pool; body = "<branch> · <project>".
MSG_FILE="$ASSETS/input-messages.txt"
TITLE='Needs you'
[ -r "$MSG_FILE" ] && TITLE=$(awk 'NF' "$MSG_FILE" | awk -v seed="$$" 'BEGIN{srand(seed)} {a[NR]=$0} END{print a[int(rand()*NR)+1]}')
BODY=$(session_label "$CWD")

# 6. Fire the notification (silent if terminal-notifier is missing).
NOTIFIER=/opt/homebrew/bin/terminal-notifier
[ -x "$NOTIFIER" ] || exit 0

# A custom sound file isn't a -sound name; play it ourselves and pass no -sound.
SOUND_ARGS=(-sound "$SOUND")
case "$SOUND" in
  */*) [ -r "$SOUND" ] && afplay "$SOUND" &>/dev/null & SOUND_ARGS=() ;;
esac

"$NOTIFIER" \
  -title "$TITLE" \
  -message "$BODY" \
  "${SOUND_ARGS[@]}" \
  -appIcon "$ICON" \
  -activate "$ACTIVATE"
