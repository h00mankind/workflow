#!/bin/sh
# Builds a release binary, installs it as /Applications/Workflow Setup.app,
# and links the command line to ~/.local/bin/workflow-setup.
# Run it again after a change to update the installed app.
set -eu

crate=$(cd "$(dirname "$0")/.." && pwd)
app="/Applications/Workflow Setup.app"
bin="$HOME/.local/bin/workflow-setup"
version=$(sed -n 's/^version = "\(.*\)"/\1/p' "$crate/Cargo.toml" | head -1)

cargo build --release --manifest-path "$crate/Cargo.toml"

rm -rf "$app"
mkdir -p "$app/Contents/MacOS"
cp "$crate/target/release/workflow-setup" "$app/Contents/MacOS/workflow-setup"
cat > "$app/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Workflow Setup</string>
  <key>CFBundleDisplayName</key><string>Workflow Setup</string>
  <key>CFBundleIdentifier</key><string>design.h00man.workflow-setup</string>
  <key>CFBundleExecutable</key><string>workflow-setup</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$version</string>
  <key>CFBundleVersion</key><string>$version</string>
  <key>LSMinimumSystemVersion</key><string>13.0</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
EOF
codesign --force --sign - "$app"

mkdir -p "$(dirname "$bin")"
ln -sf "$app/Contents/MacOS/workflow-setup" "$bin"
echo "installed: $app"
echo "command:   $bin"
