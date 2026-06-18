#!/usr/bin/env bash
# Shared helper for the notification hooks.
#
# host_app — echoes which app hosts the CURRENT Claude Code session, by walking
# the hook's process ancestry up to the host .app and matching its command path.
# We do NOT use the frontmost app for this: a session running in Ghostty stays
# in Ghostty even after you alt-tab away, so the host is a property of the
# process tree, not of what's on screen right now.
#
# Echoes one of: conductor | ghostty | zed | "" (unknown).
# Conductor runs its own bundled claude from
# ~/Library/Application Support/com.conductor.app/..., so we match that path too
# in case the .app bundle isn't a direct ancestor.
host_app() {
  local pid ppid comm
  pid=$$
  while [ "${pid:-0}" -gt 1 ]; do
    read -r ppid comm < <(ps -o ppid=,command= -p "$pid" 2>/dev/null)
    [ -z "${ppid:-}" ] && break
    case "$comm" in
      */Conductor.app/*|*/com.conductor.app/*) echo conductor; return ;;
      */Ghostty.app/*)                         echo ghostty;   return ;;
      */Zed.app/*)                             echo zed;       return ;;
    esac
    pid=$ppid
  done
  echo ""
}

# frontmost_is — returns 0 (true) when the macOS frontmost app matches the given
# host name. Used to stay silent when you're already looking at the session's
# own window. Conductor has no clickable window, so callers skip this for it.
frontmost_is() {
  local host=$1 front
  front=$(osascript -e 'tell application "System Events" to name of first process whose frontmost is true' 2>/dev/null)
  case "$host:$front" in
    ghostty:Ghostty|ghostty:ghostty) return 0 ;;
    zed:Zed|zed:zed)                 return 0 ;;
  esac
  return 1
}

# session_label CWD — echoes a short "<branch> · <project>" label for the
# notification body, derived from the hook payload's cwd. Falls back to just
# the project name when cwd isn't a git repo. Project is the cwd's folder name.
session_label() {
  local cwd=$1 project branch
  [ -n "$cwd" ] || { echo ""; return; }
  project=$(basename "$cwd")
  branch=$(git -C "$cwd" rev-parse --abbrev-ref HEAD 2>/dev/null)
  echo "${branch:+$branch · }$project"
}
