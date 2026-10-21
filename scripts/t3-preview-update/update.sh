#!/usr/bin/env bash
# Keeps "T3 Code (Alpha).app" on the newest T3 Code Preview build from
# https://github.com/pingdotgg/t3code/releases. Preview builds have no
# built-in updater, so this script is the updater.
#
# Usage:
#   update.sh               check once; install if a newer preview exists
#   update.sh --install     copy to ~/.local/bin and add a LaunchAgent (login + hourly)
#   update.sh --uninstall   remove the LaunchAgent
#
# It never touches the app while it is running (Alpha and Nightly share a
# bundle id, so detection is by path). It sends one notification per new
# version instead, and installs on the next run after you quit the app.
# Set T3_APP to point at a different bundle path (useful for testing).
set -euo pipefail
export PATH=/usr/bin:/bin:/usr/sbin:/sbin

REPO=pingdotgg/t3code
TEAM_ID=ARK85ZXQ4Z
APP=${T3_APP:-"/Applications/T3 Code (Alpha).app"}
LABEL=com.h00man.t3-preview-update
PLIST="$HOME/Library/LaunchAgents/$LABEL.plist"
LOG="$HOME/Library/Logs/t3-preview-update.log"
STATE="$HOME/Library/Application Support/t3-preview-update"

log() { printf '%s %s\n' "$(date '+%F %T')" "$*"; }
notify() { osascript -e "display notification \"$1\" with title \"T3 Code Preview\"" >/dev/null 2>&1 || true; }

case "${1:-}" in
  --install)
    # launchd can't read ~/Documents (privacy protection), so run a copy.
    SELF="$HOME/.local/bin/t3-preview-update"
    mkdir -p "$(dirname "$SELF")" "$(dirname "$PLIST")"
    cp "${BASH_SOURCE[0]}" "$SELF"; chmod +x "$SELF"
    cat > "$PLIST" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>$LABEL</string>
  <key>ProgramArguments</key><array><string>/bin/bash</string><string>$SELF</string></array>
  <key>RunAtLoad</key><true/>
  <key>StartInterval</key><integer>3600</integer>
  <key>StandardOutPath</key><string>$LOG</string>
  <key>StandardErrorPath</key><string>$LOG</string>
</dict>
</plist>
EOF
    launchctl bootout "gui/$(id -u)" "$PLIST" 2>/dev/null || true
    launchctl bootstrap "gui/$(id -u)" "$PLIST"
    echo "Installed $PLIST (log: $LOG)"
    exit 0 ;;
  --uninstall)
    launchctl bootout "gui/$(id -u)" "$PLIST" 2>/dev/null || true
    rm -f "$PLIST" "$HOME/.local/bin/t3-preview-update"
    echo "Removed $PLIST"
    exit 0 ;;
esac

# 1. Find the newest preview release (highest trailing build number) and
#    its macOS zip for this CPU.
ARCH=$(uname -m); [ "$ARCH" = x86_64 ] && ARCH=x64
RELEASE=$(curl -fsSL "https://api.github.com/repos/$REPO/releases?per_page=100" | /usr/bin/jq -c --arg arch "$ARCH" '
  [ .[] | select(.tag_name | test("-preview\\.")) ]
  | max_by(.tag_name | split(".") | last | tonumber)
  | if . == null then empty else
      { tag: .tag_name,
        asset: (.assets[] | select(.name | test("^T3-Code-.*-" + $arch + "\\.zip$"))) }
    end')
if [ -z "$RELEASE" ]; then log "no preview release found on the first page"; exit 0; fi

TAG=$(/usr/bin/jq -r .tag <<<"$RELEASE")
URL=$(/usr/bin/jq -r .asset.browser_download_url <<<"$RELEASE")
DIGEST=$(/usr/bin/jq -r '.asset.digest // empty' <<<"$RELEASE")
LATEST=${TAG#v}
LATEST_BUILD=${LATEST##*.}

# 2. Compare with the installed build. Install only if strictly newer.
INSTALLED=""
[ -d "$APP" ] && INSTALLED=$(defaults read "$APP/Contents/Info" CFBundleShortVersionString 2>/dev/null || true)
INSTALLED_BUILD=0
[[ "$INSTALLED" == *-preview.* ]] && INSTALLED_BUILD=${INSTALLED##*.}
if [ "$LATEST_BUILD" -le "$INSTALLED_BUILD" ]; then exit 0; fi

# 3. Never replace the bundle under a running app. Notify once per version.
PROCS=$(ps -axo command=)
if grep -qF "$APP/Contents/MacOS/" <<<"$PROCS"; then
  mkdir -p "$STATE"
  if [ "$(cat "$STATE/notified" 2>/dev/null)" != "$LATEST" ]; then
    notify "$LATEST is available. Quit the app to install it."
    echo "$LATEST" > "$STATE/notified"
    log "$LATEST available; app is running, waiting"
  fi
  exit 0
fi

# 4. Download, then fail closed on any checksum or signature problem.
case "$DIGEST" in sha256:*) ;; *) log "no sha256 digest for $URL; skipping"; exit 1 ;; esac
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT
log "downloading $LATEST"
curl -fsSL -o "$TMP/app.zip" "$URL"
GOT=$(shasum -a 256 "$TMP/app.zip" | cut -d' ' -f1)
if [ "$GOT" != "${DIGEST#sha256:}" ]; then log "checksum mismatch; skipping"; exit 1; fi

ditto -x -k "$TMP/app.zip" "$TMP/x"
NEW=$(find "$TMP/x" -maxdepth 1 -name '*.app' | head -1)
[ -n "$NEW" ] || { log "no .app in zip; skipping"; exit 1; }
codesign --verify --deep --strict "$NEW" || { log "signature check failed; skipping"; exit 1; }
SIGNER=$(codesign -dv "$NEW" 2>&1)
grep -qx "TeamIdentifier=$TEAM_ID" <<<"$SIGNER" || { log "unexpected signing team; skipping"; exit 1; }

# 5. Swap the bundle. Keep the old one until the new one is in place.
mkdir -p "$(dirname "$APP")"
[ -d "$APP" ] && mv "$APP" "$TMP/old.app"
if ! mv "$NEW" "$APP"; then
  [ -d "$TMP/old.app" ] && mv "$TMP/old.app" "$APP"
  log "install failed; restored previous version"; exit 1
fi
log "installed $LATEST (was ${INSTALLED:-none})"
notify "Updated to $LATEST."
