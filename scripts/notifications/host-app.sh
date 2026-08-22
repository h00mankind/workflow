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
# Zed matches Zed.app, Zed Preview.app, and Zed Nightly.app.
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
      */Zed*.app/*)                            echo zed;       return ;;
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
# the project name when cwd isn't a git repo, or when the branch and project
# names are identical (common in worktrees). Project is the cwd's folder name.
session_label() {
  local cwd=$1 project branch
  [ -n "$cwd" ] || { echo ""; return; }
  project=$(basename "$cwd")
  branch=$(git -C "$cwd" rev-parse --abbrev-ref HEAD 2>/dev/null)
  # Worktrees are often named after their branch, which would make the label
  # "<branch> · <project>" repeat the same word. Show it once when they match.
  [ "$branch" = "$project" ] && branch=""
  echo "${branch:+$branch · }$project"
}

# session_summary TRANSCRIPT_PATH — echoes a one-line summary of Claude's last
# turn, extracted from the session transcript by last-summary.py. Echoes nothing
# when no transcript is given, the file is missing, or no text could be derived,
# so callers fall back to the session label. Needs python3 (no-op without it).
session_summary() {
  local transcript=$1
  [ -n "$transcript" ] && [ -r "$transcript" ] || return
  command -v python3 >/dev/null 2>&1 || return
  python3 "$(dirname "${BASH_SOURCE[0]}")/last-summary.py" "$transcript" 100 2>/dev/null
}
