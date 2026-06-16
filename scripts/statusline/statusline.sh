#!/usr/bin/env bash
# Claude Code status line — labeled bars, Nerd Font icons.
# Backup: ~/.claude/statusline-command.sh.bak.20260616
set -uo pipefail

INPUT_JSON=$(cat)

# >>> ClaudeMascot snapshot (do not remove the markers; the Mac app reads this)
{
  echo "$INPUT_JSON" | jq -c --arg ts "$(date +%s)" '{
    captured_at: ($ts | tonumber),
    subscription_type: (.subscription_type // .subscription.type // null),
    rate_limits: (.rate_limits // {}),
    context_window: (.context_window // {}),
    cost: (.cost // {}),
    model: (.model // {}),
    session: (.session // {})
  }' >| "$HOME/.claude/usage-snapshot.json"
} 2>/dev/null || true
# <<< ClaudeMascot snapshot

# ─── ANSI ────────────────────────────────────────────────────────────────────
R="\033[0m"; B="\033[1m"; D="\033[2m"
FG_GREEN="\033[32m"; FG_YELLOW="\033[33m"; FG_BLUE="\033[34m"
FG_MAGENTA="\033[35m"; FG_CYAN="\033[36m"; FG_GRAY="\033[90m"
FG_BRIGHT_RED="\033[91m"; FG_BRIGHT_YELLOW="\033[93m"
FG_BRIGHT_BLUE="\033[94m"; FG_BRIGHT_WHITE="\033[97m"
NUM="${FG_BRIGHT_WHITE}${B}"

# ─── Parse stdin ─────────────────────────────────────────────────────────────
{
  read -r MODEL_ID
  read -r MODEL_NAME
  read -r EFFORT
  read -r CWD
  read -r CTX_SIZE
  read -r USED_PCT
  read -r INPUT_TOKENS
  read -r OUTPUT_TOKENS
  read -r SESS_PCT
  read -r SESS_RESETS
  read -r WK_PCT
  read -r WK_RESETS
  read -r SESSION_COST
  read -r COLS
} <<< "$(
  echo "$INPUT_JSON" | jq -r '
    (.model.id // ""),
    (.model.display_name // "Claude"),
    (.effort.level // ""),
    (.workspace.current_dir // .cwd // ""),
    (.context_window.context_window_size // 0),
    (.context_window.used_percentage // 0),
    (.context_window.total_input_tokens // 0),
    (.context_window.total_output_tokens // 0),
    (.rate_limits.five_hour.used_percentage // -1),
    (.rate_limits.five_hour.resets_at // 0),
    (.rate_limits.seven_day.used_percentage // -1),
    (.rate_limits.seven_day.resets_at // 0),
    (.cost.total_cost_usd // -1),
    (.terminal_width // 80)
  ' 2>/dev/null || printf "\nClaude\n\n\n0\n0\n0\n0\n-1\n0\n-1\n0\n-1\n80\n"
)"

# ─── Nerd Font icons (built from explicit UTF-8 escapes for round-trip safety) ─
USE_NERD_FONTS=${USE_NERD_FONTS:-true}
if [ "$USE_NERD_FONTS" = "true" ]; then
  ICON_MODEL=$(printf   '\xef\x83\xab')   # U+F0EB nf-fa-lightbulb
  ICON_FOLDER=$(printf  '\xee\x97\xbb')   # U+E5FB nf-cod-folder
  ICON_BRANCH=$(printf  '\xee\x9c\xa5')   # U+E725 nf-dev-git_branch
  ICON_COIN=$(printf    '\xef\x85\x95')   # U+F155 nf-fa-dollar
  ICON_READY=$(printf   '\xef\x81\x98')   # U+F058 nf-fa-check_circle
  ICON_WORKING=$(printf '\xef\x80\x93')   # U+F013 nf-fa-cog
  ICON_WAITING=$(printf '\xef\x81\xb1')   # U+F071 nf-fa-warning
  ICON_5H=$(printf      '\xef\x80\x97')   # U+F017 nf-fa-clock_o (stopwatch)
  ICON_7D=$(printf      '\xef\x81\xb3')   # U+F073 nf-fa-calendar
else
  ICON_MODEL="*"; ICON_FOLDER="DIR"; ICON_BRANCH="git"
  ICON_COIN="\$"
  ICON_READY="OK"; ICON_WORKING="..."; ICON_WAITING="?"
  ICON_5H="5h"; ICON_7D="7d"
fi

# Circular progress glyphs: pick one based on percentage (0/25/50/75/100)
ctx_circle() {
  local pct="$1"
  local n
  n=$(printf "%.0f" "$pct" 2>/dev/null || echo 0)
  if   [ "$n" -le 12 ];  then printf '\xe2\x97\x8b'        # ○ empty
  elif [ "$n" -le 37 ];  then printf '\xe2\x97\x94'        # ◔ quarter
  elif [ "$n" -le 62 ];  then printf '\xe2\x97\x91'        # ◑ half
  elif [ "$n" -le 87 ];  then printf '\xe2\x97\x95'        # ◕ three-quarter
  else                        printf '\xe2\x97\x8f'        # ● full
  fi
}

SEP="${FG_GRAY} │ ${R}"

# ─── Git branch ──────────────────────────────────────────────────────────────
VCS_BRANCH=""; VCS_DIRTY="false"
GIT_DIR="${CWD:-.}"
if [ -n "$GIT_DIR" ] && command -v git &>/dev/null; then
  VCS_BRANCH=$(git -C "$GIT_DIR" symbolic-ref --short HEAD 2>/dev/null \
               || git -C "$GIT_DIR" rev-parse --short HEAD 2>/dev/null \
               || echo "")
  if [ -n "$VCS_BRANCH" ] && git -C "$GIT_DIR" status --porcelain 2>/dev/null | grep -q .; then
    VCS_DIRTY="true"
  fi
fi

# ─── Formatters ──────────────────────────────────────────────────────────────
fmt_size() {
  awk -v n="$1" 'BEGIN {
    if (n+0 >= 1000000) printf "%.0fM", n/1000000
    else if (n+0 >= 1000) printf "%.0fK", n/1000
    else printf "%d", n
  }'
}

time_until() {
  local resets_at="$1" now diff
  now=$(date +%s)
  diff=$(( resets_at - now ))
  [ "$diff" -lt 0 ] && diff=0
  awk -v d="$diff" 'BEGIN {
    days=int(d/86400); hours=int((d%86400)/3600); mins=int((d%3600)/60)
    if (days > 0)     printf "%dd %dh", days, hours
    else if (hours>0) printf "%dh %dm", hours, mins
    else              printf "%dm", mins
  }'
}

effort_label() {
  case "$1" in
    low) echo "low" ;; medium) echo "med" ;; high) echo "high" ;;
    xhigh) echo "xhi" ;; max) echo "max" ;;
    "") echo "" ;; *) echo "$1" ;;
  esac
}

pct_color() {
  local n
  n=$(printf "%.0f" "$1" 2>/dev/null || echo 0)
  if [ "$n" -le 40 ]; then echo "$FG_GREEN"
  elif [ "$n" -le 80 ]; then echo "$FG_YELLOW"
  else echo "$FG_BRIGHT_RED"; fi
}

# ─── Model color ─────────────────────────────────────────────────────────────
model_lower=$(echo "$MODEL_NAME" | tr '[:upper:]' '[:lower:]')
if   echo "$model_lower" | grep -q "opus";   then MODEL_COLOR='\033[38;5;141m'
elif echo "$model_lower" | grep -q "sonnet"; then MODEL_COLOR='\033[38;5;75m'
elif echo "$model_lower" | grep -q "haiku";  then MODEL_COLOR='\033[38;5;120m'
elif echo "$model_lower" | grep -q "fable";  then MODEL_COLOR='\033[38;5;213m'
else                                              MODEL_COLOR="$FG_CYAN"
fi

# ─── Path shortener ──────────────────────────────────────────────────────────
shorten_path() {
  local path="$1" max_len="$2"
  [ -z "$path" ] && { echo ""; return; }
  path="${path/#$HOME/~}"
  if [ "$max_len" -eq 0 ] || [ "${#path}" -le "$max_len" ]; then
    echo "$path"
  else
    local short
    short=$(echo "$path" | awk -F/ '{ if (NF>=2) printf "…/%s/%s", $(NF-1), $NF; else print $NF }')
    if [ "${#short}" -le "$max_len" ]; then
      echo "$short"
    else
      basename "$path"
    fi
  fi
}

# ─── Bar ─────────────────────────────────────────────────────────────────────
make_bar() {
  local pct="$1" len="$2" color="$3"
  local n
  n=$(printf "%.0f" "$pct" 2>/dev/null || echo 0)
  [ "$n" -lt 0 ] && n=0
  [ "$n" -gt 100 ] && n=100
  local filled=$((n * len / 100))
  local rem=$(( (n * len) % 100 ))
  local bar="" i
  for ((i = 0; i < len; i++)); do
    if [ "$i" -lt "$filled" ]; then
      bar="${bar}${color}█${R}"
    elif [ "$i" -eq "$filled" ]; then
      if   [ "$rem" -ge 66 ]; then bar="${bar}${color}▓${R}"
      elif [ "$rem" -ge 33 ]; then bar="${bar}${color}▒${R}"
      else                         bar="${bar}${color}░${R}"
      fi
    else
      bar="${bar}${FG_GRAY}░${R}"
    fi
  done
  echo -e "$bar"
}

# ─── Labeled bar: "label  bar  XX%" ──────────────────────────────────────────
labeled_bar() {
  local label="$1" pct="$2" len="$3"
  local n c bar pct_fmt
  n=$(printf "%.0f" "$pct" 2>/dev/null || echo 0)
  c=$(pct_color "$pct")
  bar=$(make_bar "$pct" "$len" "$c")
  pct_fmt=$(printf "%3d" "$n")
  printf "%b%-3s%b %b %b%s%%%b" "$D" "$label" "$R" "$bar" "$NUM" "$pct_fmt" "$R"
}

# ─── State badge ─────────────────────────────────────────────────────────────
STATE_FILE="$HOME/.claude/agent-state"
STATE="ready"
if [ -r "$STATE_FILE" ]; then
  raw=$(cat "$STATE_FILE" 2>/dev/null)
  if [ -n "$raw" ]; then
    mtime=$(stat -f %m "$STATE_FILE" 2>/dev/null || stat -c %Y "$STATE_FILE" 2>/dev/null || echo 0)
    age=$(( $(date +%s) - mtime ))
    if [ "$age" -lt 600 ]; then STATE="$raw"; fi
  fi
fi

case "$STATE" in
  working)  STATE_BADGE="${FG_BRIGHT_YELLOW}${B}${ICON_WORKING} WORKING${R}" ;;
  waiting)  STATE_BADGE="${FG_BRIGHT_RED}${B}${ICON_WAITING} WAITING${R}" ;;
  *)        STATE_BADGE="${FG_GREEN}${B}${ICON_READY} READY${R}" ;;
esac

# ─── Components ──────────────────────────────────────────────────────────────

# Model + effort
MODEL_TXT="${MODEL_COLOR}${B}${ICON_MODEL} ${MODEL_NAME}${R}"
EFFORT_TXT=""
if [ -n "$EFFORT" ]; then
  el=$(effort_label "$EFFORT")
  EFFORT_TXT=" ${D}[${el}]${R}"
fi

# Context — nf-md-progress_close icon (color follows %) + percentage + used/total
ICON_CTX=$(printf '\xf3\xb1\x8d\x8f')   # U+1F134F nf-md-progress_close
CTX_FRAG=""
if [ "$CTX_SIZE" -gt 0 ] 2>/dev/null; then
  pct_int=$(printf "%.0f" "$USED_PCT" 2>/dev/null || echo 0)
  ctx_color=$(pct_color "$USED_PCT")
  used_t=$(fmt_size "$INPUT_TOKENS")
  size_t=$(fmt_size "$CTX_SIZE")
  CTX_FRAG="${ctx_color}${B}${ICON_CTX}${R} ${ctx_color}${B}${pct_int}%${R} ${D}${used_t}/${size_t}${R}"
fi

# Dir
DIR_PATH=$(shorten_path "$CWD" 60)
DIR_TXT=""
[ -n "$DIR_PATH" ] && DIR_TXT="${FG_CYAN}${ICON_FOLDER} ${R}${DIR_PATH}"

# Branch
BRANCH_TXT=""
if [ -n "$VCS_BRANCH" ]; then
  if [ "$VCS_DIRTY" = "true" ]; then
    BRANCH_TXT="${FG_BRIGHT_RED}${ICON_BRANCH} ${VCS_BRANCH}${FG_BRIGHT_YELLOW}*${R}"
  else
    BRANCH_TXT="${FG_BRIGHT_BLUE}${ICON_BRANCH} ${VCS_BRANCH}${R}"
  fi
fi

# Bar widths scale with terminal
if [ "$COLS" -ge 140 ]; then
  BAR_LEN=14
elif [ "$COLS" -ge 100 ]; then
  BAR_LEN=12
elif [ "$COLS" -ge 70 ]; then
  BAR_LEN=10
else
  BAR_LEN=8
fi

# 5h
SESS_LINE=""
if [ "$SESS_PCT" != "-1" ] && [ "$SESS_RESETS" != "0" ]; then
  sc=$(pct_color "$SESS_PCT")
  s_bar=$(make_bar "$SESS_PCT" "$BAR_LEN" "$sc")
  s_pct=$(printf "%.0f" "$SESS_PCT" 2>/dev/null || echo 0)
  s_t=$(time_until "$SESS_RESETS")
  SESS_LINE=$(printf "%b%s %b 5h  %b %b%3d%%%b %b%s%b" \
    "${FG_BRIGHT_YELLOW}" "$ICON_5H" "$R" \
    "$s_bar" "$NUM" "$s_pct" "$R" \
    "$D" "$s_t" "$R")
fi

# 7d
WK_LINE=""
if [ "$WK_PCT" != "-1" ] && [ "$WK_RESETS" != "0" ]; then
  wc=$(pct_color "$WK_PCT")
  w_bar=$(make_bar "$WK_PCT" "$BAR_LEN" "$wc")
  w_pct=$(printf "%.0f" "$WK_PCT" 2>/dev/null || echo 0)
  w_t=$(time_until "$WK_RESETS")
  WK_LINE=$(printf "%b%s %b 7d  %b %b%3d%%%b %b%s%b" \
    "${FG_BRIGHT_MAGENTA:-\033[95m}" "$ICON_7D" "$R" \
    "$w_bar" "$NUM" "$w_pct" "$R" \
    "$D" "$w_t" "$R")
fi

# Cost
COST_TXT=""
if [ "$SESSION_COST" != "-1" ]; then
  COST_TXT=$(awk -v c="$SESSION_COST" 'BEGIN { printf "$%.2f", c }')
elif [ "$INPUT_TOKENS" -gt 0 ] || [ "$OUTPUT_TOKENS" -gt 0 ] 2>/dev/null; then
  COST_TXT=$(awk -v i="$INPUT_TOKENS" -v o="$OUTPUT_TOKENS" 'BEGIN {
    c = (i/1000000*3) + (o/1000000*15)
    printf "~$%.2f", c
  }')
fi
COST_FRAG=""
[ -n "$COST_TXT" ] && COST_FRAG="${FG_BRIGHT_YELLOW}${ICON_COIN} ${NUM}${COST_TXT}${R}"

# ─── Output ──────────────────────────────────────────────────────────────────
if ! [[ "$COLS" =~ ^[0-9]+$ ]]; then COLS=80; fi

# Line 1: state · model [effort] · ctx
LINE1_PARTS=("$STATE_BADGE" "${MODEL_TXT}${EFFORT_TXT}")
[ -n "$CTX_FRAG" ] && LINE1_PARTS+=("$CTX_FRAG")
LINE1=""
for ((i=0; i<${#LINE1_PARTS[@]}; i++)); do
  if [ "$i" -eq 0 ]; then LINE1="${LINE1_PARTS[$i]}"
  else LINE1="${LINE1}${SEP}${LINE1_PARTS[$i]}"; fi
done
echo -e "$LINE1"

# Line 2: dir · branch
LINE2_PARTS=()
[ -n "$DIR_TXT" ]    && LINE2_PARTS+=("$DIR_TXT")
[ -n "$BRANCH_TXT" ] && LINE2_PARTS+=("$BRANCH_TXT")
LINE2=""
for ((i=0; i<${#LINE2_PARTS[@]}; i++)); do
  if [ "$i" -eq 0 ]; then LINE2="${LINE2_PARTS[$i]}"
  else LINE2="${LINE2}${SEP}${LINE2_PARTS[$i]}"; fi
done
[ -n "$LINE2" ] && echo -e "$LINE2"

# Line 3: 5h
[ -n "$SESS_LINE" ] && echo -e "$SESS_LINE"

# Line 4: 7d (+ cost)
LINE4_PARTS=()
[ -n "$WK_LINE" ]   && LINE4_PARTS+=("$WK_LINE")
[ -n "$COST_FRAG" ] && LINE4_PARTS+=("$COST_FRAG")
LINE4=""
for ((i=0; i<${#LINE4_PARTS[@]}; i++)); do
  if [ "$i" -eq 0 ]; then LINE4="${LINE4_PARTS[$i]}"
  else LINE4="${LINE4}${SEP}${LINE4_PARTS[$i]}"; fi
done
[ -n "$LINE4" ] && echo -e "$LINE4"
