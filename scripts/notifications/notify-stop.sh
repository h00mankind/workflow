#!/usr/bin/env bash
# Claude Code "Stop" hook — fires when Claude finishes a turn.
# - Writes "ready" to ~/.claude/agent-state (read by statusline).
# - Sends a desktop notification routed to the session's HOST app (see
#   host-app.sh): Conductor gets its own sound/icon and opens Conductor.app;
#   Ghostty gets a sound/icon and opens its own window. Zed is silent —
#   it now sends its own completion notification, so ours would double.
#   We also stay silent when you're already looking at the session's
#   window (frontmost), except for Conductor which has no clickable window.
#
# Wired in ~/.claude/settings.json:
#   hooks.Stop[0].hooks[0].command = "bash <repo>/scripts/notify-stop.sh"
set -uo pipefail

# 0. Read the hook JSON from stdin and pull out cwd (for the label) and the
#    transcript path (for the turn summary).
HOOK_JSON=$(cat)
CWD=$(printf '%s' "$HOOK_JSON" | sed -n 's/.*"cwd"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
TRANSCRIPT=$(printf '%s' "$HOOK_JSON" | sed -n 's/.*"transcript_path"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')

# 1. Mark agent state for the statusline.
printf ready > "$HOME/.claude/agent-state"

# 2. Figure out which app hosts this session.
source "$(dirname "${BASH_SOURCE[0]}")/host-app.sh"
HOST=$(host_app)

# 3. Zed sends its own desktop notification when a turn finishes. Skip
#    ours so the user does not get two.
[ "$HOST" = zed ] && exit 0

# 4. Stay silent if you're already looking at the session's own window.
frontmost_is "$HOST" && exit 0

# 5. Route sound / icon / click-target by host.
ASSETS="$HOME/.claude/assets"
case "$HOST" in
  conductor)
    SOUND="$ASSETS/chu-chuu.aiff"; ICON="$ASSETS/conductor-icon.png"; ACTIVATE=com.conductor.app ;;
  ghostty)
    SOUND=Pop; ICON="$ASSETS/notification-icon.png"; ACTIVATE=com.mitchellh.ghostty ;;
  *)
    SOUND=Pop; ICON="$ASSETS/notification-icon.png"; ACTIVATE=dev.zed.Zed ;;
esac

# 6. Title = a random "done" line from the pool. Body = a one-line summary of
#    the turn, with the "<branch> · <project>" label as the subtitle. When no
#    summary can be derived, fall back to the label as the body and no subtitle.
MSG_FILE="$ASSETS/stop-messages.txt"
TITLE='Done'
[ -r "$MSG_FILE" ] && TITLE=$(awk 'NF' "$MSG_FILE" | awk -v seed="$$" 'BEGIN{srand(seed)} {a[NR]=$0} END{print a[int(rand()*NR)+1]}')
LABEL=$(session_label "$CWD")
SUMMARY=$(session_summary "$TRANSCRIPT")
if [ -n "$SUMMARY" ]; then
  BODY=$SUMMARY
  SUBTITLE=$LABEL
else
  BODY=$LABEL
  SUBTITLE=""
fi

# 7. Fire the notification (silent if terminal-notifier is missing).
NOTIFIER=/opt/homebrew/bin/terminal-notifier
[ -x "$NOTIFIER" ] || exit 0

# A custom sound file isn't a -sound name; play it ourselves and pass no -sound.
SOUND_ARGS=(-sound "$SOUND")
case "$SOUND" in
  */*) [ -r "$SOUND" ] && afplay "$SOUND" &>/dev/null & SOUND_ARGS=() ;;
esac

# Notifications are best-effort. A notification failure must not make the
# Claude Code Stop hook fail after the turn has completed.
"$NOTIFIER" \
  -title "$TITLE" \
  ${SUBTITLE:+-subtitle "$SUBTITLE"} \
  -message "$BODY" \
  ${SOUND_ARGS[@]+"${SOUND_ARGS[@]}"} \
  -appIcon "$ICON" \
  -activate "$ACTIVATE" || true
