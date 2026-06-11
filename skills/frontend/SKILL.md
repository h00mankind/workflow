---
name: frontend
description: Building and styling frontend UI — components, state, CSS, Tailwind, design tokens, layout (flex, grid, spacing, alignment), responsive design, interactive controls (dropdowns, sliders, switches, uploaders), icons, and accessibility, across React, Svelte, and plain HTML. Use when building, extending, restyling, or fixing any UI feature, page, component, or widget — even when the user doesn't say "design". Skip for backend logic, API routes, or data work. For animation use motion.
---

# frontend

Read before write. Open two or three neighboring components and note: the design-system source of truth (tokens, Tailwind config, CSS vars), the icon library, the framework idiom (e.g. Svelte 4 `export let` vs Svelte 5 runes), and how similar components are built. The codebase's convention beats your favorite. If sources conflict (tokens defined but raw `px` used everywhere), call it out.

## Components & state

- **Smallest sensible surface.** Props the caller actually needs, nothing speculative.
- **Colocate.** State, styles, and helpers live with the component until a second consumer exists.
- **Derive, don't sync.** Compute from source state; no mirrored state kept in sync with effects.
- **Server state ≠ UI state.** Server data goes through the project's fetch/cache layer; UI state stays local. Filters, tabs, and selection that should survive a refresh belong in the URL.

## Styling

- **Tokens, not magic values.** Colors, spacing, typography, radius, shadow, z-index all come from the project's scale — no raw hex, no `[13px]`. If a needed token is missing, propose adding it rather than inlining.
- **Gap and padding over margins** between siblings; no double spacing (parent `gap-4` plus child `mt-4`).
- **Every wrapper earns its place.** Delete divs with no layout, style, or semantic role.
- **Mobile-first** with the project's breakpoints; sanity-check 320 → 1536px; touch targets ≥ 44×44px.
- **Flex for one dimension, grid for two.** Intrinsic sizing over fixed widths; `min-w-0` on a flex child that needs text ellipsis.

## No layout shift

- Images and media get explicit `width`/`height` or `aspect-ratio` so the box is reserved before load.
- Skeletons and loading states match the loaded content's outer dimensions.
- Animate `transform` and `opacity` only — never `width`, `height`, `top`, `left`, or `margin`.

## Interactive controls

- **Headless primitives** (Radix/shadcn for React; Bits UI/Melt for Svelte) for dropdowns, comboboxes, sliders, switches — never hand-roll positioning or ARIA. Portal popovers out of `overflow: hidden` traps.
- **Full keyboard path:** arrows navigate, Enter selects, Escape closes, focus returns to the trigger, visible focus ring always — no `outline: none` without a replacement.
- **All states exist:** hover (gated `@media (hover: hover)`), focus-visible, active, disabled, loading, empty, error. A button whose action is in flight is disabled — that's where duplicate submissions come from.
- **Keep input fast.** Debounce search (~200ms), throttle pointer/scroll work with `requestAnimationFrame`, virtualize lists over ~50 items.

## Accessibility — non-negotiable

- Semantic elements: `button` for actions, `a` for navigation, `label` wired to every input.
- One icon library per project; decorative icons get `aria-hidden`, meaningful ones get a label.
- Announce async status changes with `aria-live="polite"`.

## Motion

Keep it simple: 150–400ms, ease-out in, ease-in out, transform and opacity only, respect `prefers-reduced-motion`. The `motion` skill goes deeper when something moves in earnest.

## Verify

Render it for real before reporting done — run the app, click the path, watch the console. Then the three quick passes: narrowest viewport, keyboard-only, content extremes (very long, missing, multi-line). "It compiles" is not verification.
