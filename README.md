# h00man workflow

My personal Claude Code skills, plus a shared design-token CSS file served via jsDelivr.

## What's here

- `skills/to-html/` — Produce single-file HTML artifacts instead of walls of markdown. Always links the h00man design tokens.
- `skills/ui-designer/` — UI/UX design and frontend visual implementation across React, Svelte, and plain HTML.
- `h00man-variables.css` — CSS custom properties (color, type, spacing, motion). Light-first with `data-theme="dark"` opt-in.
- `install.sh` — Symlinks `skills/*` into `~/.claude/skills/` so Claude Code picks them up.

## Install

```bash
git clone https://github.com/h00mankind/workflow.git ~/.h00man-workflow
cd ~/.h00man-workflow
./install.sh
```

Restart Claude Code (or open a new session) and the skills will show up.

### Flags

```bash
./install.sh              # interactive: prompts on conflicts
./install.sh --force      # overwrite without prompting (existing dirs get backed up)
./install.sh --dry-run    # show what would happen, change nothing
./install.sh --uninstall  # remove only the symlinks this installer created
```

Existing entries that aren't symlinks get backed up to `~/.claude/skills.backup/<timestamp>/` before being replaced — never into `~/.claude/skills/` itself, since Claude Code would load the backup as a duplicate skill.

## Update

Symlinks point back into the cloned repo, so updating is just:

```bash
cd ~/.h00man-workflow
git pull
```

No re-install needed. Add a new skill to `skills/` and run `./install.sh` again to link it.

## CSS CDN

```html
<link rel="stylesheet" href="https://cdn.jsdelivr.net/gh/h00mankind/workflow@v1/h00man-variables.css">
```

Pin to `@v1` (or whichever tag you trust). `@main` works too but isn't cache-stable.

## What's a symlink?

A symbolic link is a tiny file that points to another path. `~/.claude/skills/to-html` is a symlink to `~/.h00man-workflow/skills/to-html`. When Claude Code reads the first path, the OS transparently uses the second. Edit the file once in the repo; both paths see the update.
