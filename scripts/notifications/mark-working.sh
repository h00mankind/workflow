#!/usr/bin/env bash
# Claude Code "UserPromptSubmit" hook — fires when you submit a prompt.
# Writes "working" to ~/.claude/agent-state so the statusline shows WORKING.
# (Counterparts: notify-stop.sh writes "ready"; notify-input.sh writes "waiting".)
#
# Wired in ~/.claude/settings.json:
#   hooks.UserPromptSubmit[0].hooks[0].command = "bash <repo>/scripts/mark-working.sh"
printf working > "$HOME/.claude/agent-state"
