---
name: to-html
description: Produce a single-file HTML artifact instead of a wall of markdown when the answer is spatial, side-by-side, interactive, or live-rendered — letting the reader see it at a glance rather than reconstruct it from prose. Use this skill whenever the user asks for an explanation, plan, design exploration, code review, diff or PR writeup, module or architecture map, design system or component sheet, prototype, animation tuning, clickable flow, slide deck, status report, incident timeline, post-mortem, research explainer, glossary, comparison table, flowchart, illustration set, custom editor, triage board, feature-flag toggler, prompt tuner, or any deliverable they'll skim, scan, point at, or hand off — even when they don't say "HTML" or "artifact". Skip for chat answers, quick replies, code snippets to paste into an editor, Word docs, .pptx, spreadsheets, or anything that must round-trip to a non-HTML format.
---

# to-html

A practical recognition guide for swapping a wall of markdown for a single self-contained `.html` file. Based on the catalog at `thariqs.github.io/html-effectiveness/`.

## Always start with the h00man design tokens

Every artifact this skill produces **must** link the h00man design tokens CSS in `<head>` and use those tokens (CSS custom properties) for color, type, spacing, radius, shadow, and motion — instead of hardcoding values. This keeps every artifact visually consistent and themable.

```html
<link rel="stylesheet" href="https://cdn.jsdelivr.net/gh/h00mankind/workflow@v2/skills/to-html/h00man-variables.css">
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Geist:wght@100..900&family=Geist+Mono:wght@100..900&family=Padauk:wght@400;700&display=swap" rel="stylesheet">
```

Use the tokens directly in your styles:

```css
body {
  background: var(--bg);
  color: var(--text);
  font-family: var(--font-body);
  font-size: var(--text-body);
  line-height: var(--leading-body);
}
.card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: var(--space-20);
  box-shadow: var(--shadow-sm);
  transition: background var(--duration-base) var(--ease-out);
}
.accent { color: var(--accent); }
.code { font-family: var(--font-mono); background: var(--bg-inset); }
```

Dark theme opt-in is a single attribute on `<html>`:

```html
<html data-theme="dark">
```

## Always include a theme switcher button

Every artifact this skill produces **must** include a theme switcher button so the reader can flip between light and dark. Place it in a fixed top-right corner so it's reachable from anywhere on the page without interfering with content. No browser storage — toggle the `data-theme` attribute on `<html>` directly. Default to whichever theme suits the artifact; the button lets the reader override.

```html
<button id="theme-toggle" type="button" aria-label="Toggle theme" title="Toggle theme">
  <span class="theme-icon-light" aria-hidden="true">☀</span>
  <span class="theme-icon-dark" aria-hidden="true">☾</span>
</button>
```

```css
#theme-toggle {
  position: fixed;
  top: var(--space-16);
  right: var(--space-16);
  z-index: var(--z-overlay);
  width: 36px;
  height: 36px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-card);
  color: var(--text);
  border: 1px solid var(--border);
  border-radius: var(--radius-full);
  box-shadow: var(--shadow-sm);
  cursor: pointer;
  transition: background var(--duration-base) var(--ease-out);
}
#theme-toggle:hover { background: var(--bg-card-hover); }
#theme-toggle:focus-visible { outline: none; box-shadow: var(--focus-ring); }
.theme-icon-dark { display: none; }
[data-theme="dark"] .theme-icon-light { display: none; }
[data-theme="dark"] .theme-icon-dark { display: inline; }
```

```js
const root = document.documentElement;
document.getElementById('theme-toggle').addEventListener('click', () => {
  const next = root.getAttribute('data-theme') === 'dark' ? 'light' : 'dark';
  root.setAttribute('data-theme', next);
});
```

The token vocabulary you have available (see the CDN file for full definitions):

- **Surfaces** — `--bg`, `--bg-elev`, `--bg-card`, `--bg-card-hover`, `--bg-tinted`, `--bg-inset`
- **Borders** — `--border`, `--border-strong`, `--border-bright`
- **Text** — `--text`, `--text-dim`, `--text-muted`, `--text-faint`
- **Accent** — `--accent`, `--accent-soft`, `--accent-deep`, `--accent-vivid`, `--accent-wash`
- **Semantic** — `--cyan`, `--amber`, `--green`, `--violet`, `--rose` (each with `-soft` and `-wash` variants)
- **Gradients** — `--grad-accent`, `--grad-cyan`, `--grad-aurora`, `--grad-fade-text`, `--grad-atmosphere`
- **Type families** — `--font-display`, `--font-body`, `--font-mono`, `--font-my` (Burmese)
- **Type scale** — `--text-eyebrow`, `--text-caption`, `--text-body-sm`, `--text-body`, `--text-lede`, `--text-subheading`, `--text-h3`, `--text-h2`, `--text-h1`, `--text-display` (each paired with `--leading-*` and `--tracking-*`)
- **Weights** — `--weight-light` (300) through `--weight-bold` (700)
- **Spacing** — `--space-4`, `-8`, `-12`, `-16`, `-20`, `-24`, `-32`, `-48`, `-60`, `-80`, `-100`, `-128`
- **Layout** — `--doc-max`, `--content-max`, `--widget-max`, `--sidebar-w`, `--gutter`
- **Radius** — `--radius-xs`, `-sm`, `-md`, `-lg`, `-xl`, `-2xl`, `-full`
- **Shadow** — `--shadow-sm`, `-md`, `-lg`, `-xl`, `--shadow-glow-accent`, `--shadow-glow-cyan`, `--shadow-inset`
- **Motion** — `--ease-out`, `--ease-in-out`, `--ease-spring`, `--duration-fast`, `-base`, `-slow`, `-stagger`
- **State** — `--selection-bg`, `--focus-ring`
- **Z-index** — `--z-base`, `--z-sticky`, `--z-overlay`, `--z-modal`, `--z-toast`

Never hardcode hex colors, raw pixel values for spacing/radius, or font names when an equivalent token exists. If a token is missing for what you need, pick the closest one and note it inline rather than introducing a one-off value.

The thesis is simple: many things people ask for — plans, explainers, design explorations, post-mortems, module maps — are *spatial information* that markdown flattens. HTML restores the spatial layer. A small chart, a side-by-side, an annotated diff, or a clickable prototype turns something the reader would skim into something they would actually read.

## When to reach for this

Pick HTML over markdown when **at least one** of the following is true:

- The answer compares two or more things and the reader needs to point at one (designs, code approaches, vendor options).
- The answer has spatial structure (module maps, dataflow diagrams, timelines, flowcharts, org charts).
- The answer has interactive structure that prose cannot convey (animation easing, click-through, slider-tuned parameters, drag-and-drop, toggles with dependencies).
- The reader will return to it (status reports, design systems, glossaries, explainers) and benefits from navigation.
- The artifact will be handed off (PR writeups, implementation plans, incident reports) and structure helps the next person.
- The user will modify the output as part of their workflow (triage boards, feature-flag editors, prompt tuners) — end with an export button so their edits round-trip.

Stay in markdown / prose when the answer is short, conversational, or genuinely linear (a definition, a one-shot code snippet, a yes/no, an emotional reply).

## The non-negotiable constraints

1. **One file.** Inline CSS in `<style>`, inline JS in `<script>`. No external build, no bundler, no npm install. The user can save it, double-click it, and it works.
2. **CDN-only dependencies, sparingly.** Always link the h00man design tokens (`https://cdn.jsdelivr.net/gh/h00mankind/workflow@v2/skills/to-html/h00man-variables.css`) plus the Geist font family. Beyond that, if a chart or interaction genuinely needs a library, load it from a CDN (e.g. `https://cdnjs.cloudflare.com`). Default to vanilla. Never reference local modules or assume a dev server.
3. **No browser storage.** `localStorage` / `sessionStorage` fail inside Claude artifacts. Keep state in JS variables or React state for the session.
4. **Export back out.** For any editor-style artifact, include a "copy as markdown" or "download JSON" button so what the user did in the UI returns to a form they can paste into the next prompt or commit. The loop is: prompt → HTML → edit → export → next prompt.
5. **Open directly in a browser.** No server, no build step, no fetch to localhost. Click-to-open is the floor of the user experience.

## Where to save the file

The artifact is a file on disk — pick the location deliberately. Follow this order:

1. **If the user names a path, use it.** "Save to `docs/explainers/`" or "output to my desktop" beats every default.
2. **Otherwise detect context.** Run `git rev-parse --show-toplevel` (or check if the cwd is inside a git repo). If yes, save to `<repo-root>/docs/artifacts/`. Create the directory if it doesn't exist. This keeps artifacts versioned alongside the code they describe.
3. **If not in a repo, save to `~/Downloads/`.** This is the casual / one-off / "I just want to look at it" path.

**Filename**: derive a kebab-case slug from the user's request, no timestamp. `sharable-skills.html`, not `artifact-20260515.html`. Predictable names are easier to find again, and the user can rename if they want versioning.

**Always end the turn by reporting the absolute path and an open command**, like:

```
Saved to /Users/you/Downloads/sharable-skills.html
Open with: open ~/Downloads/sharable-skills.html
```

Don't bury the path in prose — the user needs to click/run it.

## Overflow guardrails — long strings can't break the layout

Any artifact you generate will eventually contain a long URL, file path, dependency ID, or shell one-liner. Without guardrails these punch out of cards, sidebars, and table cells. **Always include both rules in your `<style>`:**

```css
/* Inline code wraps anywhere — for URLs and long identifiers in prose */
code {
  overflow-wrap: anywhere;
  word-break: break-word;
}
/* Block code scrolls horizontally — preserve formatting in pre/code */
pre {
  overflow-x: auto;
  max-width: 100%;
}
pre code {
  overflow-wrap: normal;
  word-break: normal;
  white-space: pre;
}
```

Why this split: inline `<code>` lives inside paragraphs, sidebars, and tight columns — wrapping mid-string is uglier than overflowing, but only barely, and overflowing breaks the page. Block `<pre><code>` is supposed to preserve formatting, so it scrolls instead.

Also worth doing on any narrow container (cards, sidebars, grid children):

```css
.card, .sidebar, .grid > * {
  min-width: 0;   /* lets the child shrink below its content's intrinsic width */
}
```

Without `min-width: 0`, flex and grid children default to their content's minimum width — a long unbroken URL forces the column wider than the layout allows.

## Scrollspy that actually works — track sections, not headings

If the artifact has a sidebar table of contents, the TOC should highlight whichever section the reader is currently *looking at*, not the one whose `<h2>` happens to be in some narrow band. The bug we hit before: an `IntersectionObserver` watching `<h2>` headings with a tight `rootMargin` flipped the active link to the *next* section the moment its heading peeked into the band, even though the reader was still mid-way through the previous section's body. Wrong-feeling, especially on tall sections.

**Rule: wrap each `<h2>` + its content in a `<section>` element with the id, and observe the sections.** Then pick the section with the largest visible area. That's what the reader is actually reading.

```html
<section id="anatomy">
  <h2>Anatomy</h2>
  <!-- content -->
</section>
<section id="ship">
  <h2>Ship it</h2>
  <!-- content -->
</section>
```

```js
const sections = Array.from(document.querySelectorAll('main section[id]'));
const tocLinks = Array.from(document.querySelectorAll('.toc a'));
const linkFor = id => tocLinks.find(a => a.getAttribute('href') === '#' + id);

const visibleArea = new Map();

const spy = new IntersectionObserver((entries) => {
  entries.forEach(e => {
    visibleArea.set(e.target.id, e.isIntersecting ? e.intersectionRect.height : 0);
  });
  // Pick the section with the most visible area; fall back to the last one passed.
  let activeId = null, max = 0;
  for (const [id, area] of visibleArea) {
    if (area > max) { max = area; activeId = id; }
  }
  if (!activeId) return;
  tocLinks.forEach(a => a.classList.remove('is-active'));
  const link = linkFor(activeId);
  if (link) link.classList.add('is-active');
}, {
  // Threshold steps so the observer fires as visibility changes, not just on enter/exit.
  threshold: [0, 0.25, 0.5, 0.75, 1]
});
sections.forEach(s => spy.observe(s));
```

Don't use `rootMargin: '-30% 0px -60% 0px'` on headings — that's the broken pattern. The `threshold: [0, 0.25, 0.5, 0.75, 1]` is what makes "biggest visible area wins" possible: the callback fires whenever a section's visibility crosses any of those steps, so the map stays current.

If sections aren't an option (e.g. the markup is already shipped with bare `<h2>`s), the fallback is a manual scroll handler: on `scroll`, find the heading whose top is closest to but still above the viewport top. Less elegant but unambiguous.

## The 9 categories — recognize these triggers

When a user request maps to one of these, default to HTML. The phrasings on the right are what they tend to *actually say*, not what the pattern is called.

### 1. Exploration & Planning — when the user is not sure what they want yet

Lay options next to each other so they can point at one, instead of reading three sequential walls of text.

- **Three code approaches** — "compare these implementations", "which way should I solve this", "trade-offs of X vs Y vs Z". Side-by-side panels with trade-offs called out inline.
- **Visual design directions** — "give me a few layout ideas", "show me some palette options", "I don't know what vibe I want". A handful of rendered options, not described ones.
- **Implementation plan** — "draft a plan", "how would you build this", "write a spec". Milestones on a timeline, a dataflow diagram, inline mockups, the risky code, a risk table. The plan the implementer can actually read.

### 2. Code Review & Understanding — when the *shape* of the change matters

Diffs and call graphs are spatial; markdown flattens them.

- **Annotated pull request** — "review this PR", "what's wrong with this diff". Diff rendered with margin notes, severity tags, jump links.
- **PR writeup for reviewers** — "write the PR description", "help me explain this change". Motivation, before/after, file-by-file tour with the *why*, where to focus the review.
- **Module map** — "explain this package", "I'm new to this codebase". Boxes and arrows, hot path highlighted, entry points listed.

### 3. Design — when the medium *is* HTML

HTML is the medium your design system ships in, so it's the natural format for talking about it.

- **Living design system** — "show me the tokens", "extract the design system from this repo". Colors, type scale, spacing tokens as swatches you can copy from.
- **Component variants** — "show every state of the button", "lay out all the sizes". Every size/state/intent of one component on one sheet.

### 4. Prototyping — when motion and interaction must be felt

Motion can't be described, only felt.

- **Animation sandbox** — "what should this transition feel like", "tune the easing". The animation in isolation with sliders for duration and easing.
- **Clickable flow** — "show the user flow", "prototype this interaction". Four screens linked — enough fidelity to feel whether the interaction is right.

### 5. Illustrations & Diagrams — when the agent should pick up the pen

Inline SVG gives the agent a real pen.

- **SVG figure sheet** — "draw the diagrams for this post", "I need some figures". Vector art that can be tweaked by hand or pasted into the final doc.
- **Annotated flowchart** — "diagram this pipeline", "draw the deploy flow". A real flowchart — click any step to see what runs, timings, failure paths.

### 6. Decks — when twenty lines of JS is a slide deck

A handful of `<section>` tags plus arrow-key navigation. No Keynote, no export step.

- **Arrow-key slide deck** — "make slides from this thread", "turn this doc into a presentation". Left/right to navigate, one HTML file.

### 7. Research & Learning — when a topic needs scaffolding to be navigable

The same words read very differently with collapsible sections, tabs, and a glossary in the margin.

- **How a feature works** — "explain rate limiting in this repo", "how does X work here". TL;DR box, collapsible request-path steps, tabbed config snippets, FAQ.
- **Concept explainer** — "teach me consistent hashing", "how does X actually work". Live ring you can add/remove nodes from, comparison table, hover-linked glossary.

### 8. Reports — recurring documents that benefit from a bit of structure

Status updates and post-mortems read once and then are revisited. A small chart and a colored timeline turn something people skim into something they read.

- **Weekly status** — "write my status update", "what shipped this week". What shipped, what slipped, a small chart, formatted for a quick Monday-morning skim.
- **Incident timeline** — "write the post-mortem", "summarize the outage". Minute-by-minute timeline, log excerpts, follow-up checklist.

### 9. Custom Editing Interfaces — when it's hard to describe what you want in a text box

Sometimes the right move is a throwaway editor for the exact thing the user is working on. **Always end with an export button** that turns whatever they did in the UI back into something pastable.

- **Ticket triage board** — "help me prioritize these tickets". Drag tickets across Now / Next / Later / Cut, copy the ordering out as markdown.
- **Feature flag editor** — "let me toggle these settings". Toggles grouped by area, dependency warnings when a prerequisite is off, "copy diff" button for just the changed keys.
- **Prompt tuner** — "help me iterate on this prompt". Editable template with variable slots highlighted, sample inputs on the right that re-render live.

## How to use this catalog

1. **Recognize the pattern in the user's request.** The user almost never says "give me an HTML artifact with collapsible sections and a sidebar glossary". They say "explain how rate limiting works in this repo". Map the request to one of the 18 patterns above — or to a *combination* of them (an implementation plan often pulls from #1, #5, and #8 at once).
2. **Pick one pattern; build it well.** Resist the urge to combine four patterns into one cluttered page. A status report is a status report.
3. **Apply the constraints.** Single file. No build step. No browser storage. Export button for editors.
4. **For deeper pattern detail**, see `references/pattern-deep-dives.md` — descriptions of what each of the 20 patterns includes, when to choose between similar patterns, and small examples.

## What this skill is not

- It is not a CSS or visual-polish guide. For typography, design tokens, spacing, motion, and component quality, defer to the `ui-designer` skill.
- It is not a framework recommendation. The default is vanilla HTML + CSS + JS. React via the artifacts environment is fine when the interaction genuinely benefits.
- It is not a license to produce an HTML file for every question. Many requests are still best answered in chat. The trigger is *spatial structure or interaction*, not length.
