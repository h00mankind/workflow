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

- Define a small token set inline — 6–10 CSS custom properties in `:root` (background, text, muted, accent, border, radius, fonts, a spacing unit) — and use them throughout. Consistent within the artifact, varied across artifacts.
- One or two font families max — a system stack or a single Google Fonts link chosen for the personality.
- Light or dark by content, not habit. A theme toggle is optional; if included, flip a `data-theme` attribute — no browser storage.
- Refuse the default AI look: Inter + purple gradient + identical rounded cards is a non-choice.

## Constraints

1. **One file.** Inline `<style>` and `<script>`. No build, no npm. Save it, double-click it, it works.
2. **CDN-only dependencies, sparingly.** A fonts link is fine; a chart library from a CDN if genuinely needed. Default to vanilla.
3. **No browser storage.** `localStorage` fails in sandboxed artifacts — keep state in JS variables.
4. **Export back out.** Editor-style artifacts get a "copy as markdown" / "download JSON" button so edits round-trip to the next prompt.
5. **Opens directly in a browser.** No server, no fetch to localhost.

## Where to save

1. A path the user named, if any.
2. In a git repo → `<repo-root>/docs/artifacts/` (create it if missing).
3. Otherwise → `~/Downloads/`.

Filename: kebab-case slug from the request, no timestamp. Always end the turn with the absolute path and an open command — don't bury it in prose.

## Layout guardrails

Long URLs and shell one-liners will eventually punch out of cards and table cells. Always include:

```css
code { overflow-wrap: anywhere; word-break: break-word; }   /* inline code wraps */
pre { overflow-x: auto; max-width: 100%; }                   /* block code scrolls */
pre code { overflow-wrap: normal; word-break: normal; white-space: pre; }
.card, .sidebar, .grid > * { min-width: 0; }                 /* flex/grid children can shrink */
```

If there's a sidebar TOC with scrollspy: wrap each `<h2>` + its content in a `<section id>`, observe the **sections** with an `IntersectionObserver` (`threshold: [0, .25, .5, .75, 1]`), and highlight the section with the **largest visible area**. Don't observe bare headings with a tight `rootMargin` — it flips the active link to the next section while the reader is still mid-way through the previous one.

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
