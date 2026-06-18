# html — paste-in plumbing

The mechanical parts of an artifact that are identical every run — theme switcher, scrollspy, layout guardrails, the TOC tick, the icon webfont. Copy these verbatim; they don't carry personality, so don't redesign them. `SKILL.md` holds the judgment; this holds the boilerplate.

## Spacing & layout scale

Set once in `:root`, use everywhere — section gaps, card padding, stack rhythm. No hard-coded pixel/em spacing: if you reach for `padding: 3px`, use a `--space-*` token or extend the scale once. A single stray `3px` breaks the rhythm and is the fastest tell of a sloppy artifact.

```css
:root {
  --space-0: 2px;  --space-1: 4px;  --space-2: 8px;  --space-3: 12px;
  --space-4: 16px; --space-5: 24px; --space-6: 32px; --space-7: 48px;
  --space-8: 64px; --space-9: 96px;
  --measure: 68ch;   /* prose width cap */
  --radius: 6px;
  --tracking-tight: -0.022em;
  --tracking-body: -0.011em;
}
```

Section gaps `--space-7`–`--space-8`. Card padding `--space-4`–`--space-5`. Cap prose at `--measure`. Don't let cards or tables stretch edge-to-edge in a wide viewport.

## Icon webfont (Remix Icon)

A *webfont*, so one CSS link, `<i class="ri-…">` markup, and **no JS init call** — which sidesteps the whole class of "icon didn't render after a DOM update" bugs.

```html
<link href="https://cdn.jsdelivr.net/npm/remixicon@4.5.0/fonts/remixicon.css" rel="stylesheet">
<button><i class="ri-star-line"></i></button>
```

Icons ship in `-line`/`-fill` pairs — perfect for toggles. Common ones: `ri-star-line`/`ri-star-fill`, `ri-error-warning-line`, `ri-check-line`, `ri-close-line`, `ri-clipboard-line`, `ri-arrow-go-back-line` (reset/undo), `ri-arrow-down-s-line`, `ri-sun-line`/`ri-moon-line`, `ri-download-line`, `ri-external-link-line`, `ri-arrow-right-line`. Size with `font-size`, colour with `color` — they inherit like text (`i { font-size: 14px; line-height: 1 }`).

Runtime swap is trivial — just change the class, no re-init:

```js
function setIcon(el, name) { el.className = name }   // setIcon(themeIcon, dark ? 'ri-sun-line' : 'ri-moon-line')
```

(This is why a webfont beats an SVG-injection library like Lucide: Lucide *replaces* each `<i data-lucide>` with an inline `<svg>`, so re-querying the `<i>` after the first `createIcons()` fails and naive toggle code stacks duplicate icons. The webfont has no such trap.) Phosphor or Heroicons are fine if the mood calls for them — one library per artifact, don't mix. Inline SVG is fine for 2–3 icons.

## Theme switcher

Top-right button, **text label only** ("Dark mode" / "Light mode") — no sun/moon glyphs, they render inconsistently and look like placeholder. Define both palettes in `:root` and `[data-theme="dark"]`; flip `data-theme` on `<html>`. No browser storage — keep the current theme in a JS variable. Respect `prefers-color-scheme` on first paint and sync the label then.

```html
<button id="theme" aria-label="Switch theme">Dark mode</button>
<script>
  const html = document.documentElement
  const btn = document.getElementById('theme')
  const syncLabel = () => btn.textContent = html.dataset.theme === 'dark' ? 'Light mode' : 'Dark mode'
  if (matchMedia('(prefers-color-scheme: dark)').matches) html.dataset.theme = 'dark'
  syncLabel()
  btn.onclick = () => { html.dataset.theme = html.dataset.theme === 'dark' ? 'light' : 'dark'; syncLabel() }
</script>
```

## Right-side TOC + scrollspy

For long artifacts (3+ `<h2>` sections). On the **right**, not the left — the reader's eye lives on the content; the TOC is peripheral.

**Header and footer sit outside the grid**, so their border spans the full page width. The 2-column grid wraps only `<main>` + `<aside>`; otherwise the header rule stops at the content column and strands a divider next to the TOC. Grid: `minmax(0, 1fr)` content + `220px` TOC, gap `--space-7` — the `minmax(0, ...)` stops code blocks blowing out the column. Below 960px, hide the TOC. No "On this page" label; the position and shape make it obvious.

**The tick gap is the easy thing to get wrong.** Put the vertical spacing on the `<li>` (outside the tick) and draw the tick with a `::before` sized to the link only — otherwise the ticks meet edge-to-edge and read as one continuous rail. Active state swaps the pseudo-element to `--text` and bumps the link to `600`. Tick is 2px; don't grow it on active — the colour does the work.

```html
<div class="page">
  <header>…<button id="theme">Dark mode</button></header>
  <div class="layout">
    <main>…<section id="…">…</section>…</main>
    <aside class="toc">
      <ul>
        <li><a href="#a">Section A</a></li>
        <li><a href="#b">Section B</a></li>
      </ul>
    </aside>
  </div>
  <footer>…</footer>
</div>
```

```css
.toc li { padding: var(--space-2) 0; }      /* gap lives on the li, outside the tick */
.toc a {
  display: block; position: relative;
  padding-left: var(--space-4); color: var(--muted);
}
.toc a::before {                             /* the tick — only as tall as the link */
  content: ''; position: absolute;
  left: 0; top: 0; bottom: 0; width: 2px;
  background: var(--border);
}
.toc a.active { color: var(--text); font-weight: 600; }
.toc a.active::before { background: var(--text); }
```

Scrollspy: wrap each `<h2>` + its content in a `<section id>`, observe the **sections** (not bare headings — a tight `rootMargin` on headings flips the active link while the reader is still mid-section), and highlight the one with the largest visible area.

```js
const links = new Map([...document.querySelectorAll('.toc a')].map(a => [a.hash.slice(1), a]))
const ratios = new Map()
const io = new IntersectionObserver((entries) => {
  entries.forEach(e => ratios.set(e.target.id, e.intersectionRatio))
  const top = [...ratios.entries()].sort((a, b) => b[1] - a[1])[0]
  links.forEach(a => a.classList.remove('active'))
  if (top && top[1] > 0) links.get(top[0])?.classList.add('active')
}, { threshold: [0, .25, .5, .75, 1] })
document.querySelectorAll('main section[id]').forEach(s => io.observe(s))
```

## Layout guardrails

Long URLs and shell one-liners eventually punch out of cards and table cells. Always include:

```css
:not(pre) > code { overflow-wrap: anywhere; word-break: break-word; }  /* inline code wraps */
pre { overflow-x: auto; max-width: 100%; }                              /* block code scrolls */
pre code { overflow-wrap: normal; word-break: normal; white-space: pre; }
.card, .sidebar, .grid > * { min-width: 0; }                            /* flex/grid children can shrink */
```
