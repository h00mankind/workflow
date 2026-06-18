---
name: html
description: Produce a single-file HTML artifact instead of a wall of markdown when the answer is spatial, side-by-side, interactive, or live-rendered — letting the reader see it at a glance rather than reconstruct it from prose. Use whenever the user asks for an explanation, plan, design exploration, code review or PR writeup, architecture map, design-system sheet, clickable flow, slide deck, status report, incident timeline, research explainer, glossary, comparison, flowchart, illustration set, triage board, or any deliverable they'll skim, scan, point at, or hand off — even when they don't say "HTML" or "artifact". Skip for chat answers, quick replies, code snippets to paste into an editor, or anything that must round-trip to a non-HTML format.
---

# html

Swap the wall of markdown for one self-contained `.html` file. Plans, explainers, comparisons, maps, and post-mortems are *spatial information* that markdown flattens — a side-by-side, an annotated diff, or a clickable prototype turns something the reader would skim into something they actually read.

## When

Reach for HTML when at least one is true; otherwise stay in chat:

- The answer **compares things** the reader needs to point at.
- It has **spatial structure** — maps, timelines, flowcharts, diagrams.
- It has **interaction** prose can't convey — easing, click-throughs, sliders, toggles.
- It will be **returned to or handed off** — status reports, design systems, plans, post-mortems.
- The user will **edit it** as part of their workflow — boards, editors, tuners. These always end with an export button.

## Personality — theme to the context

No shared stylesheet; the subject dictates the look. A post-mortem reads like a serious report (restrained, near-monochrome, one alarm color); an architecture map like a blueprint (monospace labels, thin rules); an explainer editorial (serif headings, generous measure); a triage board like a tool (dense, utilitarian).

- Define a token set inline in `:root` and a `[data-theme="dark"]` override — colors (`--bg`, `--text`, `--muted`, `--border`, `--card`, `--accent`), radius, fonts, and the spacing scale. Use them throughout. Consistent within the artifact, varied across artifacts. (Scale, theme switcher, icons, TOC, guardrails: `snippets.md` — paste-in plumbing, no need to re-derive it.)
- One sans + one mono, sometimes one serif. Pick by content, not habit:
  - **Tools, maps, dashboards, code review** — sans + mono, tight tracking. Reads technical.
  - **Reports, post-mortems, status** — humanist sans + mono, neutral tracking. Reads institutional.
  - **Explainers, research, learning** — serif headings + sans body, generous measure. Reads editorial.
  - **Decks, hero pages** — display serif headings + sans body. Reads designed.
- Refuse the default AI look: Inter + purple gradient + identical rounded cards is a non-choice. A clean technical pairing is fine — just don't reach for it by reflex.
- **Icons: a library, never unicode glyphs** (`★`, `⚠`, `→` render inconsistently and look like placeholder). Default Remix Icon — a webfont, one CSS link, no JS init. See `snippets.md`.

## Plumbing — paste from `snippets.md`

These parts are identical every run and carry no personality, so don't re-derive them — copy from `snippets.md` and theme the tokens to fit:

- **Spacing scale** — pick a scale, not arithmetic; set once in `:root`, use everywhere. No stray pixel/em spacing — a single `3px` in the wrong place is the fastest tell of a sloppy artifact.
- **Theme switcher** — if the artifact is returned to or read in different lighting, ship both themes (default by content), text-label button, no storage, respect `prefers-color-scheme`.
- **Right-side TOC + scrollspy** — for 3+ `<h2>` sections. The two traps the bundle handles: header/footer outside the grid (or the rule strands next to the TOC), and the per-item tick spacing on the `<li>` not the link (or the ticks merge into one rail).

## Constraints

1. **One file.** Inline `<style>` and `<script>`. No build, no npm. Save it, double-click it, it works.
2. **CDN-only dependencies, sparingly.** A fonts link is fine; a chart library from a CDN if genuinely needed. Default to vanilla.
3. **No browser storage.** `localStorage` fails in sandboxed artifacts — keep state in JS variables.
4. **Export back out.** Editor-style artifacts get a "copy as markdown" / "download JSON" button so edits round-trip to the next prompt.
5. **Opens directly in a browser.** No server, no fetch to localhost.

## Where to save

1. A path the user named, if any.
2. In a git repo → `<repo-root>/docs/html/` (create it if missing).
3. Otherwise → `~/Downloads/`.

Filename: kebab-case slug from the request, no timestamp. Always end the turn with the absolute path and an open command — don't bury it in prose.

## Layout guardrails

Long URLs and shell one-liners eventually punch out of cards and table cells — always include the wrap/scroll/`min-width:0` guardrail CSS from `snippets.md`. The scrollspy JS (observe sections, highlight the largest-visible) lives there too.

## The catalog

Map the request to a pattern — users say the phrase, not the pattern name. Pick one (or a deliberate combo) and build it well; don't cram four patterns into one page.

1. **Exploration & planning** — "compare these approaches", "draft a plan" → options side-by-side with trade-offs inline; plans as timeline + dataflow diagram + risk table.
2. **Code review & understanding** — "review this PR", "explain this package" → diff with margin notes and severity tags; module map with boxes, arrows, hot path highlighted.
3. **Design** — "show me the tokens", "every state of the button" → swatch sheets you can copy from; variant grids on one sheet.
4. **Prototyping** — "what should this feel like" → animation sandbox with duration/easing sliders; clickable four-screen flow.
5. **Illustrations & diagrams** — "diagram this pipeline" → inline SVG figures; flowcharts where clicking a step shows what runs and how it fails.
6. **Decks** — "make slides from this" → `<section>`s + arrow-key navigation, one file.
7. **Research & learning** — "explain how X works here" → TL;DR box, collapsible steps, tabbed snippets, hover-linked glossary, live mini-demos.
8. **Reports** — "status update", "post-mortem" → what shipped/slipped with a small chart; minute-by-minute timeline with log excerpts and a follow-up checklist.
9. **Editing interfaces** — "help me prioritize / toggle / tune" → drag-to-rank board, flag editor with dependency warnings, prompt tuner with live re-render — each with an export button.

## What this skill is not

- Not a CSS guide — component quality is `frontend`, animation is `motion`.
- Not a framework recommendation — vanilla by default; React only when the interaction genuinely benefits.
- Not a license to answer every question with an HTML file. The trigger is spatial structure or interaction, not length.
