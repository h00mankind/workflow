#!/usr/bin/env bash
# h00man workflow installer — symlinks skills/ into ~/.claude/skills/
#
# Usage:
#   ./install.sh           Install (interactive on conflicts)
#   ./install.sh --force   Overwrite existing entries without asking
#   ./install.sh --dry-run Show what would happen, change nothing
#   ./install.sh --uninstall  Remove only the symlinks this installer created

set -euo pipefail

# Resolve repo root from this script's location, so it works from anywhere.
REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SKILLS_SRC="$REPO_DIR/skills"
SKILLS_DST="$HOME/.claude/skills"

FORCE=0
DRY_RUN=0
UNINSTALL=0
for arg in "$@"; do
  case "$arg" in
    --force) FORCE=1 ;;
    --dry-run) DRY_RUN=1 ;;
    --uninstall) UNINSTALL=1 ;;
    -h|--help)
      sed -n '2,8p' "$0" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *) echo "unknown flag: $arg" >&2; exit 2 ;;
  esac
done

# Colors (only if stdout is a tty)
if [ -t 1 ]; then
  C_DIM=$'\033[2m'; C_GREEN=$'\033[32m'; C_YELLOW=$'\033[33m'
  C_RED=$'\033[31m'; C_BOLD=$'\033[1m'; C_RESET=$'\033[0m'
else
  C_DIM=; C_GREEN=; C_YELLOW=; C_RED=; C_BOLD=; C_RESET=
fi

say()  { printf '%s\n' "$*"; }
ok()   { printf '%s✓%s %s\n' "$C_GREEN" "$C_RESET" "$*"; }
warn() { printf '%s!%s %s\n' "$C_YELLOW" "$C_RESET" "$*"; }
err()  { printf '%s✗%s %s\n' "$C_RED" "$C_RESET" "$*" >&2; }
run()  {
  if [ "$DRY_RUN" -eq 1 ]; then
    printf '%s[dry-run]%s %s\n' "$C_DIM" "$C_RESET" "$*"
  else
    eval "$@"
  fi
}

if [ ! -d "$SKILLS_SRC" ]; then
  err "no skills/ directory at $SKILLS_SRC"
  exit 1
fi

mkdir -p "$SKILLS_DST"

# Collect skill names from the repo.
skills=()
for entry in "$SKILLS_SRC"/*/; do
  [ -d "$entry" ] || continue
  skills+=("$(basename "$entry")")
done

if [ "${#skills[@]}" -eq 0 ]; then
  warn "no skills found under $SKILLS_SRC"
  exit 0
fi

say "${C_BOLD}h00man workflow installer${C_RESET}"
say "  repo:   $REPO_DIR"
say "  target: $SKILLS_DST"
say "  skills: ${skills[*]}"
say ""

# Uninstall path: remove only links that point back into this repo.
if [ "$UNINSTALL" -eq 1 ]; then
  removed=0
  for name in "${skills[@]}"; do
    link="$SKILLS_DST/$name"
    if [ -L "$link" ]; then
      target="$(readlink "$link")"
      case "$target" in
        "$SKILLS_SRC/$name"|"$SKILLS_SRC/$name/")
          run "rm \"$link\""
          ok "removed $link"
          removed=$((removed+1))
          ;;
        *)
          warn "skipped $link (points elsewhere: $target)"
          ;;
      esac
    elif [ -e "$link" ]; then
      warn "skipped $link (not a symlink, left alone)"
    fi
  done
  say ""
  ok "uninstall done — $removed link(s) removed"
  exit 0
fi

# Install path.
installed=0
skipped=0
replaced=0
for name in "${skills[@]}"; do
  src="$SKILLS_SRC/$name"
  link="$SKILLS_DST/$name"

  if [ -L "$link" ]; then
    current="$(readlink "$link")"
    if [ "$current" = "$src" ]; then
      ok "already linked: $name"
      installed=$((installed+1))
      continue
    fi
    warn "$name is a symlink to $current"
    action="replace"
  elif [ -e "$link" ]; then
    warn "$name already exists at $link (not a symlink)"
    action="backup"
  else
    action="create"
  fi

  if [ "$action" != "create" ]; then
    if [ "$FORCE" -eq 1 ]; then
      choice="y"
    elif [ "$DRY_RUN" -eq 1 ]; then
      say "  ${C_DIM}[dry-run] would prompt; pass --force to replace${C_RESET}"
      choice="n"
    else
      printf "  replace? [y/N/a(ll)] "
      read -r choice < /dev/tty || choice="n"
      if [ "$choice" = "a" ] || [ "$choice" = "A" ]; then
        FORCE=1; choice="y"
      fi
    fi
    if [ "$choice" != "y" ] && [ "$choice" != "Y" ]; then
      warn "skipped $name"
      skipped=$((skipped+1))
      continue
    fi
    if [ "$action" = "backup" ]; then
      # Backups go OUTSIDE ~/.claude/skills/ so Claude Code doesn't load them
      # as duplicate skills. Stored at ~/.claude/skills.backup/<ts>/<name>.
      backup_root="$HOME/.claude/skills.backup/$(date +%Y%m%d%H%M%S)"
      mkdir -p "$backup_root"
      run "mv \"$link\" \"$backup_root/$name\""
      ok "backed up existing $name → $backup_root/$name"
    else
      run "rm \"$link\""
    fi
    replaced=$((replaced+1))
  fi

  run "ln -s \"$src\" \"$link\""
  ok "linked $name → $src"
  installed=$((installed+1))
done

say ""
ok "done — $installed installed, $replaced replaced, $skipped skipped"
say ""
say "Restart Claude Code (or open a new session) to pick up new skills."
