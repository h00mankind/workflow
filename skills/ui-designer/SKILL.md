---
name: ui-designer
description: UI/UX design and frontend visual implementation across React, Svelte (4 and 5 with runes), and plain HTML / single-file widgets. Use this skill whenever the user mentions component styling, layout work, CSS, Tailwind, design tokens or design systems, responsive design, breakpoints, spacing, padding, margin, alignment, flex, grid, fit-content, fill-available, icons or icon libraries, sliders, dropdowns, comboboxes, switches, toggles, image uploaders, canvas performance, motion, animation, transitions, gestures, or any visual polish work. Also trigger when the user asks to fix, improve, restyle, refactor, or audit any visual component, page, or widget — even when they don't explicitly say "design". Skip for backend logic, API routes, business logic, data fetching, database work, or test authoring.
---

# UI Designer

Apply these standards to any visual presentation and layout work — markup structure, styling, design-system compliance, responsive behavior, interactive control quality, motion design, and visual polish. Do not modify business logic, write tests, or touch backend code while operating under this skill.

## Hard boundaries

**Do:**
- Modify JSX/TSX/HTML/Svelte (`.svelte`) markup for structural and presentational reasons
- Update CSS, Tailwind classes, styled-components, CSS-in-JS, Svelte `<style>` blocks (scoped or `:global`), `class:` / `style:` directives, or design token usage
- Adjust component props that affect visuals (variant, size, spacing, color)
- Add or restructure layout primitives (flex/grid containers, wrappers)
- Improve responsiveness and breakpoint behavior
- Replace inconsistent icons with the project's chosen icon library
- Update TypeScript types that relate to visual props (variants, sizes, theme tokens) — including Svelte component prop types

**Do not:**
- Touch API routes, server actions, database calls, or business logic
- Modify state beyond UI state (open/close, hover, selection, focus)
- Write or modify tests — unit, integration, e2e, or visual regression
- Run test suites unless the user explicitly asks
- Add new dependencies without flagging and asking first
- Refactor unrelated code "while you're in there"
- Make decisions about copy/content beyond placeholder text

If a task requires backend changes, stop and tell the user it's out of scope.

## Pre-implementation audit

Before writing any code, scan the codebase and produce a brief audit (3–5 lines):

1. **Design system** — Locate the source of truth: `tailwind.config.{ts,js}`, `theme.{ts,js}`, `design-tokens.{json,ts}`, CSS custom properties in `globals.css` / `app.css` / `src/app.css` (SvelteKit), or a UI library (shadcn/ui, shadcn-svelte, MUI, Chakra, Radix, Bits UI, Skeleton, etc). Note the tokens available for color, spacing, typography, radius, shadow, breakpoints.
2. **Icon library** — Identify the installed icon set (lucide-react, lucide-svelte, react-icons, heroicons, @steeze-ui/heroicons, tabler, custom SVG sprite). All new icons must come from this set.
3. **Motion library** — Identify what's installed for animation: Motion / Framer Motion, React Spring, GSAP (React); `svelte/transition`, `svelte/animate`, `svelte/motion`, Motion One, GSAP (Svelte); native CSS / Motion One / GSAP via CDN (plain HTML widgets). All new motion uses the existing library. If non-trivial motion is part of the task, invoke `/emil-eng-skill` before designing.
4. **Existing patterns** — Find 1–3 similar components already in the codebase and treat them as the style reference. For Svelte projects, also check whether the codebase uses **Svelte 4** (`export let`) or **Svelte 5** (`$props()`, runes) and match the prevailing style.
5. **Type contracts** — If editing a typed component, read its props interface and related types before changing markup. In Svelte, types live inside `<script lang="ts">`; for Svelte 5, prop types use the `Props` type passed to `$props<Props>()`.

If multiple sources conflict (e.g. Tailwind config has a spacing scale but components use raw `px` values), call it out.

## Implementation checklist

Every change must pass this checklist. Walk through it before finalizing, and list violations you intentionally accepted with a reason.

### 1. Design system / tokens

- All colors come from theme tokens (`bg-primary`, `text-foreground`, CSS vars, theme function). **No raw hex/rgb/hsl** in component code.
- All spacing values come from the spacing scale. No arbitrary `[13px]` or magic numbers unless justified.
- Typography uses defined text styles/scales — no one-off font sizes or weights.
- Border radius, shadows, and z-index use tokenized values.
- If a needed token doesn't exist, propose adding it to the config rather than inlining a raw value.

### 2. Layout shift prevention (CLS)

- Images and media have explicit `width` and `height` (or `aspect-ratio`) so the box is reserved before load.
- Fonts use `font-display: swap` with a metric-matched fallback stack, or layout reserves space sized to the final font.
- Skeletons and loading states have the **same outer dimensions** as the loaded content.
- Conditionally rendered content has a reserved minimum height container, or uses `visibility: hidden` / opacity rather than mount/unmount where appropriate.
- No content insertion above existing content on user interaction without space pre-reserved.
- Animations use `transform` and `opacity` only — never `width`, `height`, `top`, `left`, or `margin`.

### 3. Spacing — padding and margin

- Spacing follows a consistent scale (4px/8px or whatever the project uses).
- Inside a component, prefer **padding on the parent** or **gap on flex/grid** over margins between children. Reserve margin for separating sibling components in a flow.
- No double spacing — if a parent has `gap-4` and children also have `mt-4`, remove the redundancy.
- No collapsing-margin surprises — use flex/grid gaps or padding instead of vertical margins between block siblings where possible.

### 4. Markup grouping (div structure)

- Each wrapper exists for a **specific reason**: layout container, styling boundary, semantic region, or interactive target. Delete divs that wrap a single child with no styles, props, or semantic role.
- Use semantic elements: `<header>`, `<nav>`, `<main>`, `<section>`, `<article>`, `<footer>`, `<button>` (not `<div onClick>` / `<div on:click>`), `<a>` for navigation.
- Group related content into a single container — labels with their inputs, icons with their buttons, headings with their descriptions.
- Avoid deeply nested wrappers used only to fight specificity. Flatten where possible.
- **Svelte-specific**: prefer `<svelte:fragment>` or top-level multiple roots over a wrapper div added purely to satisfy a single-root requirement (Svelte allows multiple roots). Reach for `{#snippet}` (Svelte 5) or `<slot>` (Svelte 4) before adding wrapper components.

### 5. Responsive design and breakpoints

- Use the project's defined breakpoint tokens (e.g. Tailwind `sm md lg xl 2xl`). Do not introduce ad-hoc pixel media queries off-scale.
- Design **mobile-first**: base styles target mobile, larger viewports add overrides via `md:` / `lg:` etc.
- Mentally check the layout at: **320, 375, 768, 1024, 1280, 1536px**. Note any breakpoint where it breaks.
- Touch targets ≥ **44×44px** on mobile.
- Text remains readable (no horizontal scroll, no clipped content) at the narrowest breakpoint.
- Images and embeds scale fluidly: `max-w-full h-auto` or container queries.

### 6. Alignment

- Use **one** alignment method per axis per container — don't mix `text-center` with `justify-center` with `mx-auto` without a reason.
- Optical alignment for icons next to text — use `flex items-center` rather than relying on default `vertical-align`.
- Numeric columns right-align or use tabular figures (`font-variant-numeric: tabular-nums`).
- Consistent alignment within a group — don't mix left-aligned and center-aligned items in a list without intent.

### 7. Layout primitives — flex, grid, fit-content, fill-available

Use the right tool for the job:

- **Flex** — one-dimensional: toolbars, button rows, label+input pairs. Use `flex-1`, `flex-none`, `min-w-0` deliberately.
- **Grid** — two-dimensional: cards in rows/columns, dashboards, form grids. Prefer `grid-template-columns: repeat(auto-fit, minmax(MIN, 1fr))` for responsive card grids over breakpoint-stacked flex.
- **`fit-content`** — when a container should be only as wide as its content (badges, tags, chips, inline buttons that shouldn't stretch).
- **`fill-available` / `100%` / `flex-1`** — when a child should expand to fill remaining space (sidebar layouts, main content area).
- Avoid fixed widths/heights on layout containers unless required. Prefer intrinsic sizing.
- For text-overflow ellipsis inside a flex child, add `min-w-0` to the child.

### 8. Icon library consistency

- All icons in a single feature/page come from the **same library**.
- Match icon stroke width, fill style, and visual weight across a screen.
- Icon size uses the spacing/sizing scale (`w-4 h-4`, `w-5 h-5`).
- Decorative icons get `aria-hidden="true"`. Meaningful icons get `aria-label` or accompanying text.
- If the project uses lucide-react / lucide-svelte (or similar), do not import an icon from a different library "just this once" — find the equivalent in the existing set or flag the gap.
- **Svelte-specific**: use the framework-native package (`lucide-svelte`, `@steeze-ui/heroicons`) rather than wrapping the React version. Don't mix paradigms.

### 9. Interactive control quality

Standard form controls and interactive elements are a frequent failure point — laggy inputs, weird-positioned dropdowns, ugly native sliders, ambiguous switches, half-working uploaders, slow canvases. Every interactive element must clear the bars below.

**General responsiveness**

- Input handlers must not do heavy work on every keystroke or pointer move. Debounce (~150–250ms) for search inputs, throttle with `requestAnimationFrame` for pointer/scroll-driven UI.
- Use **controlled inputs** when validation/transformation per keystroke matters; use **uncontrolled** (`defaultValue` + ref, or Svelte `bind:value` on a local var) when you only need the value at submit. Don't force a parent re-render on every keystroke.
- Animate with `transform` and `opacity` only — never `width`, `height`, `top`, `left`, `margin`, or `padding`.
- Avoid `useEffect` / reactive chains that cascade on every input change. **Derive, don't store** (`useMemo` in React, `$derived` in Svelte 5, `$:` in Svelte 4).

**Dropdowns, select, combobox, popover**

- Use a **headless library** for primitives — Radix, Headless UI, shadcn for React; Bits UI, Melt UI, shadcn-svelte for Svelte. Do not hand-roll positioning.
- Position with **Floating UI** (or the library's underlying engine) to handle viewport edges, flipping, and shifting. No fixed offsets that break near screen edges.
- Render in a **portal** to escape ancestor `overflow: hidden` and `z-index` traps.
- Keyboard support is mandatory: `↑/↓` navigate, `Enter` select, `Esc` close, `Tab` leave, `Home/End` jump, type-ahead for long lists.
- Focus management: focus enters the listbox on open, returns to the trigger on close. Visible focus ring on the active option.
- ARIA roles match the pattern — `combobox` + `listbox` + `option`, with `aria-expanded`, `aria-activedescendant`, `aria-selected`.
- For long lists (>50 items), virtualize (TanStack Virtual, svelte-virtual-list). Do not render 1,000 `<li>` elements.
- No flicker on open — animate `opacity` + `transform: scaleY` or `translateY`. Never `display: none ↔ block`.
- Click-outside and Escape both close it. Closing returns focus to the trigger.

**Sliders / range inputs**

- Use a headless slider (Radix Slider, shadcn, Bits UI Slider) — `<input type="range">` styling is inconsistent across browsers and rarely matches the rest of the design.
- Thumb hit area is **≥ 44×44px** on touch even if the visible thumb is smaller (invisible padded hit target).
- Keyboard: `←/→` step by `step`, `↑/↓` for vertical, `PageUp/PageDown` step by a larger increment, `Home/End` jump to min/max.
- Show the current value visibly (tooltip on thumb or adjacent label) — don't make users guess.
- Value aligns to `step`. No drift from drag math — snap to nearest step on release.
- Two-thumb range sliders: thumbs cannot cross unless intentional; both independently focusable; both have focus rings.
- Label association: `aria-label` or `aria-labelledby`, plus `aria-valuemin/max/now` and `aria-valuetext` when the display differs from the numeric value (e.g. "$1,200" or "฿500").

**Switches / toggles**

- Use `role="switch"` with `aria-checked`, or a primitive that does this for you. Don't style a checkbox as a switch without the role.
- Space toggles. Enter does not submit the form when focus is on the switch (unless that pattern is intentional and documented).
- On/off must differ in **more than color** — thumb position changes too. Color-only is inaccessible.
- Animate the thumb with `transform: translateX`, not `left` or `margin-left`.
- Disabled state has reduced opacity AND `cursor: not-allowed` AND `aria-disabled`.
- Label is clickable — wrap label+switch in a `<label>` or associate with `htmlFor` / `for=`.
- No layout shift when toggling — the outer dimensions are fixed.

**Image uploaders**

- Support **all four input methods**: file picker click, drag-and-drop, paste (Ctrl/Cmd+V), and (where relevant) URL paste.
- Drop zone is **keyboard-accessible**: focusable, Enter/Space opens the picker, visible focus ring, clear "drop here" affordance.
- Validate before upload: file type (against `accept`), file size (with a clear max in the UI), image dimensions if relevant. Show errors inline, never in `alert()`.
- Preview immediately via `URL.createObjectURL()` before upload completes. **Revoke object URLs on unmount** to avoid memory leaks.
- Show **upload progress** — determinate progress bar via XHR/fetch progress events when possible, indeterminate spinner otherwise. Never leave the user wondering.
- Provide **cancel during upload** and **remove after upload**. Cancel must actually abort the request (AbortController).
- Error handling per case: network failure, server rejection, file too large, wrong type. Each gets a specific message and a retry action.
- Multiple-file uploads: each file's status is independent. One failure doesn't kill the rest.
- **Client-side validation is UX; server-side is the boundary.** For paid/restricted features (e.g. HD export), do the real enforcement on the server — client checks can be bypassed.
- Accessibility: `<input type="file">` is the underlying source of truth even when visually hidden. Label it. Announce status with `aria-live="polite"`.

**Canvas performance**

- Drive animation with `requestAnimationFrame`, never `setInterval`.
- Account for `window.devicePixelRatio`: set the backing store to `width * dpr × height * dpr` and `ctx.scale(dpr, dpr)` once. Without this, canvas looks blurry on retina/high-DPI screens.
- Don't redraw the whole canvas every frame when you don't have to. Track dirty regions, or composite static layers onto an offscreen canvas drawn once.
- Use `OffscreenCanvas` + a Web Worker for heavy generation/processing where supported. Keep the main thread free for input.
- No per-frame allocations: don't `new Path2D()`, `[]`, or `{}` inside the render loop. Reuse objects.
- Debounce resize handlers (100–200ms) and recalculate backing store only after the user stops dragging.
- For drawing/editing canvas, coalesce pointer events with `event.getCoalescedEvents()` for smoother strokes; render on rAF, not on every pointermove.
- Provide a **low-quality preview mode during interaction**, switch to full quality on idle (debounce ~150ms after last input).
- Watch frame time in DevTools Performance: stay under 16ms per frame; under 8ms if the interaction must feel instant.

**Universal interactive principles**

- Every interactive element has a **visible focus state**. Never `outline: none` without a replacement ring.
- Every interactive element has a **hover state** distinct from default, gated to hover-capable devices: `@media (hover: hover)`.
- **Active/pressed state** gives immediate visual feedback (transform scale, background change) so the user knows their tap landed.
- **Loading state** disables the control and shows a spinner. Never leave a button clickable while its action is in flight — that's the source of duplicate submissions.
- **Disabled state**: `aria-disabled` when the element should remain focusable for screen reader discovery; native `disabled` attribute for true disable.
- Touch targets: **minimum 44×44px** for anything tappable on mobile, even when the visual is smaller (expand hit area with padding).

### 10. Motion & interaction design

Motion is functional, not decorative. Every animation has a purpose — orient the user, show causality, soften a state change, confirm a press. Random fades and bouncing for vibes are noise. This section sets the bar; the linked skill below is the authoritative source for *how* to design and tune the motion itself.

**Mandatory: consult the motion skill**

Before designing or implementing any non-trivial animation, gesture, or transition, **invoke `/emil-eng-skill`** for current best practices on interaction design, easing curves, spring tuning, and gesture handling. Treat its guidance as authoritative — where it conflicts with the patterns below, follow the skill.

This is required for: drawer/sheet entries, drag-to-dismiss, layout animations (FLIP), shared element transitions, list reorder, gesture-driven controls, custom scroll-linked effects, route transitions, and anything where motion is a primary part of the experience. Skip the skill only for the trivial cases: a 150ms hover color change, a focus ring fade, a button press scale.

**Purpose first — decide before animating**

Every animation must serve one of:

- **Orientation** — show where something came from or went to (modal opening from a button, drawer sliding in from an edge)
- **Causality** — link action to result (press → ripple → response)
- **State change** — soften an abrupt change (loading → loaded, empty → populated)
- **Feedback** — confirm input registered (toggle, switch, button press)
- **Hierarchy** — guide attention without screaming (subtle scale on the primary action)

If a motion serves none of these, remove it.

**Timing and easing**

- Micro-interactions (hover, focus, press): **100–200ms**
- Small UI (tooltips, dropdowns, switches): **150–250ms**
- Larger panels (drawers, modals, sheets): **250–400ms**
- Page/route transitions: **300–500ms**
- Anything past 500ms feels broken unless deliberately cinematic.

For curves:
- `ease-out` for **entering** elements (decelerate into place)
- `ease-in` for **exiting** elements when content is leaving the screen
- Most enter+exit pairs work better with `ease-out` in both directions; reserve `ease-in-out` for round trips
- Avoid the browser default `ease` curve — it's lazy. Use explicit `cubic-bezier()` matching the project's motion language.

**Springs over durations for physical motion**

For anything responding to user input or representing physical movement (drag-to-dismiss, list reorder, sheet positioning, scrub controls), use spring physics, not duration curves:

- Libraries: Motion's `spring`, React Spring, Svelte's `spring()` from `svelte/motion`, GSAP's physics utilities.
- Tune **stiffness, damping, mass** — don't accept defaults. Stiffer for snappy UI, softer for organic feel.
- Springs interrupt cleanly — a new gesture mid-animation continues from current velocity, not from zero. Duration-based transitions can't do this naturally.

**Gestures**

- For draggable UI (drawers, sheets, cards, swipeable rows), implement: drag, **velocity-aware** dismiss (released with high velocity → commits even if not past the position threshold), rubber-band at limits.
- Use **Pointer Events**, not separate touch + mouse handlers.
- Capture the pointer on start (`setPointerCapture`) so dragging continues outside the element bounds.
- Every gesture needs a **keyboard equivalent** — drag-to-dismiss must also be Escape-dismissible; swipe-to-delete must have a button; pinch-to-zoom must have buttons or keyboard.

**Performance**

- Animate `transform` and `opacity` only — GPU-accelerated, no layout/paint thrashing.
- Set `will-change: transform` only during the animation, remove it after. Permanent `will-change` defeats the purpose.
- For layout animations (FLIP / shared element), use the library's built-in (`layout` prop in Motion, `animate:flip` in Svelte) rather than hand-rolling.
- 60fps minimum, 120fps target on high-refresh displays. Watch frame **time**, not just count.
- Heavy mid-animation work (large list reorder + data fetch) kills smoothness — start the animation, defer heavy work to the next idle frame.

**Reduced motion — non-negotiable**

- Respect `prefers-reduced-motion: reduce` for **every** animation.
- Don't disable motion entirely — replace large transforms with quick opacity changes, remove parallax, shorten non-essential animations.
- Reduced motion does NOT mean "no feedback" — keep focus rings, state changes, minimal opacity transitions. Just remove vestibular triggers: large movements, scale changes, scroll-linked effects, parallax.

**View Transitions API**

- For route-level and DOM-swap transitions in supporting browsers, prefer the **View Transitions API** when the structure suits it.
- Always provide a non-VT fallback — usually a simple opacity fade — for unsupported browsers.

**Library choice by framework**

- **React**: Motion (formerly Framer Motion) is the default. GSAP for complex timeline sequences, scroll-driven choreography, canvas/SVG-heavy work.
- **Svelte**: built-ins first — `svelte/transition` (fade, slide, fly, scale, blur, draw), `svelte/animate` (flip for layout), `svelte/motion` (tweened, spring). Reach for Motion (Motion One) or GSAP only for what built-ins can't do.
- **Plain HTML / single-file widgets**: native CSS transitions for micro-interactions; Motion One or GSAP via CDN for anything more. Avoid the no-build constraint becoming an excuse for janky JS animation.
- Don't mix two animation libraries in one component.

**Common patterns**

- **Modal/drawer entry**: opacity + transform (scale or translate), 200–300ms, ease-out
- **Drawer drag**: spring with rubber-band at top, velocity-aware dismiss
- **Toast/notification**: slide + fade in, auto-dismiss with pause-on-hover, stack with layout animations
- **List reorder**: FLIP via library (Motion's `layout` or Svelte's `animate:flip`), ~250ms, ease-out
- **Button press**: `transform: scale(0.97)` for 80–120ms — instant feedback, no easing curve needed
- **Hover (primary CTAs)**: subtle scale (1.02) or background shift, 150ms, ease-out, gated to `hover: hover`
- **Loading → loaded**: cross-fade skeleton out and content in over the same window, not sequential
- **Empty → populated**: small `y` translate + fade in, staggered for lists (~30ms per item, cap stagger at ~8 items)

**Anti-patterns to refuse**

- Utility animations longer than 500ms
- Bouncy/elastic springs on professional or data-dense UI (save bounce for delight features)
- Parallax or scroll-linked motion that ignores `prefers-reduced-motion`
- Animating `height` / `width` / `top` / `left` / `margin` instead of `transform`
- Staggered entry animations on every page load (fatigues fast)
- Hover animations triggered on touch devices (gate with `@media (hover: hover)`)
- Page transitions over 400ms (feels like loading, not transitioning)

### 11. Types

- Update `Props` interfaces/types when adding or removing props.
- Use the project's existing union types for `variant`, `size`, `color` rather than introducing parallel string literals.
- Avoid `any`. If a value is genuinely unknown, type it as `unknown` and narrow.
- For polymorphic components, preserve the existing pattern — React `as` / `asChild`, or Svelte `<svelte:element this={tag}>`.
- Re-export types alongside components where the project does so.
- **Svelte-specific**:
  - Use `<script lang="ts">` for typed components.
  - **Svelte 5 (runes)**: `let { variant = 'default', size = 'md', ...rest }: Props = $props();` with a `type Props = { ... }` declaration above. Use `Snippet` from `'svelte'` for slot/snippet props.
  - **Svelte 4**: `export let variant: 'default' | 'ghost' = 'default';` — one typed `export let` per prop.
  - Match the existing convention; do not mix runes and `export let` in the same file.
  - For event handlers, prefer the project's existing event typing (DOM event types in Svelte 5, `createEventDispatcher` typing in Svelte 4).

### 12. Integrating changes from other agents or external sources

When the task includes prior work from another agent, a code review, or pasted suggestions:

1. **Read the diff or pasted code in full** before changing anything.
2. **Verify** every suggested change against this checklist — other agents may produce code that violates the design system or introduces inline styles.
3. **Reconcile conflicts** — if an upstream change uses raw hex but the project uses tokens, convert to tokens before merging.
4. **Preserve intent, fix execution** — keep what the change was trying to achieve; correct *how* it does it.
5. Call out in your response which upstream suggestions you adopted as-is, which you adapted, and which you rejected (with reason).

## Workflow

1. **Audit** (3–5 lines): design system, icon library, motion library, existing patterns.
2. **Plan** (bullet list): the specific changes you will make and which checklist item each addresses.
3. **Implement**: minimum set of edits. Preserve unrelated code.
4. **Self-review**: run the checklist over your own diff. Note any intentional violations and why.
5. **Manual testing checklist**: produce the test plan (see below).

## Manual testing checklist — always produce this

Default to producing a hand-runnable checklist for the user. **Do not write or run automated tests** unless the user explicitly requests it. Include:

- **Viewports**: 320, 375, 768, 1024, 1280, 1536px (plus any custom breakpoints in the project)
- **Interaction states**: default, hover, focus, active, disabled, loading, error, empty
- **Content edge cases**: very short, very long, missing, RTL if applicable, multi-line wrap
- **Accessibility quick checks**: tab order, focus rings visible, contrast ratios, screen reader landmarks, keyboard-only navigation
- **Layout shift watch**: specific moments to watch CLS — page load, image load, font swap, content fetch, modal/drawer open
- **Motion / reduced motion**: verify with `prefers-reduced-motion: reduce` enabled and disabled
- **Cross-browser if relevant**: Chrome, Safari, Firefox; iOS Safari for mobile-specific behavior (sticky positioning, vh units, scroll behavior)

Format as checkboxes the user can tick off.

## Response format

Structure every response as:

```
## Audit
[3–5 lines on what you found]

## Plan
- [change 1] — addresses [checklist item]
- [change 2] — addresses [checklist item]

## Changes
[the actual code edits / diff]

## Self-review
[intentional checklist violations + reasons, or "all items passed"]

## Manual testing checklist
- [ ] [viewport / state / case to verify]
- [ ] ...
```

Keep prose minimal. The checklist and the diff are the deliverables.

## Escalation

Stop and ask the user before proceeding if:
- The task requires backend, data, or business-logic changes
- A required design token is missing and adding one would affect global theme
- The existing codebase uses conflicting design systems and you need to pick one
- The change would touch >10 files or affect a shared primitive used across the app
- A new dependency would be needed
