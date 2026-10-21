# t3-preview-update

Keeps `T3 Code (Alpha).app` on the newest **T3 Code Preview** build from [pingdotgg/t3code releases](https://github.com/pingdotgg/t3code/releases). Preview builds have no built-in updater (Nightly does), so this script fills that gap.

Preview builds are cut by maintainers at irregular times and are unsupported — they can be broken. Use Nightly if you want early builds with a real updater.

## Use

```bash
./update.sh               # check once; install if a newer preview exists
./update.sh --install     # copy to ~/.local/bin + LaunchAgent: login + hourly
./update.sh --uninstall   # remove the LaunchAgent
```

Log: `~/Library/Logs/t3-preview-update.log`.

`--install` copies the script to `~/.local/bin/t3-preview-update` because launchd can't read files under `~/Documents`. Re-run `--install` after editing the script.

## How it works

1. Reads the GitHub releases API (no `gh` or token needed) and picks the `-preview.` tag with the highest build number — the trailing `.2504` in `0.0.45-preview.20260930.2504`.
2. Installs only when that build is **strictly newer** than the installed one. It never downgrades.
3. If the app is running, it does not touch it. It sends one notification per new version and installs on the next run after you quit. Detection is by bundle path, because Alpha and Nightly share the bundle id `com.t3tools.t3code`.
4. Fails closed: the zip must match the SHA-256 `digest` GitHub publishes for the asset, pass `codesign --verify --deep --strict`, and be signed by team `ARK85ZXQ4Z`.
5. Swaps the bundle in `/Applications`, restoring the old one if the move fails.

It checks hourly rather than daily because the app is usually open. Login (before you open the app) and any hour it's closed are the install windows.

`T3_APP=/some/path/X.app ./update.sh` targets a different bundle path — handy for testing without touching the real app.
