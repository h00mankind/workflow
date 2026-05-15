# h00man workflow

My personal Claude Code skills, plus a shared design-token CSS file served via jsDelivr.

## What's here

- `skills/to-html/` — Produce single-file HTML artifacts instead of walls of markdown. Bundles `h00man-variables.css` (also served via CDN).
- `skills/ui-designer/` — UI/UX design and frontend visual implementation across React, Svelte, and plain HTML.

## Install

Uses [skills.sh](https://www.skills.sh/) — symlinks by default, so `npx skills update` pulls in any changes.

```bash
# All skills, globally (available to every project)
npx skills add h00mankind/workflow -g

# A single skill
npx skills add h00mankind/workflow --skill to-html -g

# Project-scoped (drops into ./<agent>/skills/ in your project)
npx skills add h00mankind/workflow
```

Restart Claude Code (or open a new session) and the skills will show up.

### Update / remove

```bash
npx skills update              # pull latest for installed skills
npx skills remove to-html      # uninstall a skill
npx skills list                # see what's installed
```

## CSS CDN

The skill links the CSS automatically. If you want to use the tokens outside the skill, pin to a tag:

```html
<link rel="stylesheet" href="https://cdn.jsdelivr.net/gh/h00mankind/workflow@v2/skills/to-html/h00man-variables.css">
```

`@v1` (older path: `@v1/h00man-variables.css` at the repo root) stays pinned forever. `@main` works but isn't cache-stable.

## What's a symlink?

A symbolic link is a tiny file that points to another path. `skills.sh` keeps one canonical copy of each skill in `~/.skills/` and symlinks `~/.claude/skills/<name>` to it. When Claude Code reads the symlink, the OS transparently uses the canonical copy. `npx skills update` refreshes the canonical copy, so every symlinked location sees the new content at once.
