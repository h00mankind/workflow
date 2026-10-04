---
name: motion
description: Use to add or tune UI animation, transitions, gestures, scroll effects, or other interface motion.
---
# motion

Motion explains — where things came from, that an action landed, where to look next. If an animation explains nothing, cut it.

## Should it animate at all?

Frequency decides. Something seen 100+ times a day (keyboard shortcuts, command palette) gets **no animation** — animation makes repeated actions feel slow. Tens of times a day (hover, list navigation): minimal or none. Occasional (modals, drawers, toasts): standard. Rare or first-time (onboarding, celebrations): can add delight. Never animate keyboard-initiated actions.

Match motion to the component's personality: playful things can bounce a little; a professional dashboard is crisp and fast.

## Defaults

- **Duration:** 100–160ms button feedback, 125–200ms tooltips, 150–250ms dropdowns, 200–400ms modals/drawers. UI stays under 300ms wherever possible — a 180ms dropdown *feels* more responsive than a 400ms one, and easing amplifies it.
- **Easing:** ease-out for enters and exits (responsive — movement starts immediately), ease-in-out for on-screen moves/morphs, plain ease for hover/color, linear only for continuous motion (spinners, marquees). **Never ease-in for UI** — it delays the start, the exact moment the user is watching.
- **Custom curves beat built-ins** — the CSS defaults are weak. Good staples: `cubic-bezier(0.23, 1, 0.32, 1)` (strong ease-out), `cubic-bezier(0.77, 0, 0.175, 1)` (strong ease-in-out), `cubic-bezier(0.32, 0.72, 0, 1)` (iOS drawer).
- **Exits are quicker** than entrances — about 60–70% of the enter duration. Broadly: slow where the user is deciding, fast where the system is responding.
- **Distance is small.** 4–12px for micro-moves; larger travel only when the element semantically comes from somewhere. Never enter from `scale(0)` — nothing real appears from nothing; start at `scale(0.95)` + `opacity: 0`.

## Details that compound

- Pressables get `:active { transform: scale(0.97) }` with a ~150ms ease-out transition.
- Popovers/dropdowns/tooltips scale from their **trigger**, not center — set `transform-origin` to the anchor (Radix/Base UI expose it as a CSS var). Modals stay centered.
- Tooltips: delay the first, open adjacent ones instantly while one is already showing.
- Prefer **transitions over keyframes** for anything triggered rapidly — transitions retarget mid-flight, keyframes restart from zero. `@starting-style` animates entry without JS (fall back to a mounted-attribute flip).
- `translateY(100%)` (percent of own size) over hardcoded pixels for off-screen positions.
- A crossfade that won't gel: add a subtle `filter: blur(2px)` during the transition to blend the two states.

## Springs & gestures

Springs settle by physics, not duration, and keep velocity when interrupted — use them for drag interactions, interruptible gestures, and decorative mouse-tracking. Config: `{ type: "spring", duration: 0.5, bounce: 0.2 }` is easiest to reason about; keep bounce ≤ 0.3 and skip it for most UI.

For drag-to-dismiss: dismiss on **velocity** (a quick flick), not just distance threshold. Past a boundary, apply friction/damping rather than a hard stop. Capture the pointer once dragging starts; ignore extra touch points mid-drag.

## Performance

- Animate `transform` and `opacity` only. Never `top/left/width/height/margin` — layout thrash. Size changes via `scale`, `clip-path`, or FLIP.
- CSS animations run off the main thread; JS (`requestAnimationFrame`) drops frames when the page is busy. Use CSS for predetermined animations, JS for dynamic/interruptible ones; WAAPI (`element.animate`) when you need JS control with CSS performance.
- Motion/Framer Motion's shorthand `x`/`y`/`scale` props are **not** hardware-accelerated — pass a full `transform` string when smoothness under load matters.
- Don't drive per-frame values through a CSS variable on a parent (recalcs every child) — set `transform` on the element directly.
- `will-change` sparingly, removed when the animation ends. Blur ≤ 20px (expensive, especially Safari).

## Reduced motion

`prefers-reduced-motion` means fewer and gentler, not zero: keep opacity/color feedback, remove positional movement. Wire it once at the system level — a CSS media query or `gsap.matchMedia`/`useReducedMotion` — not per animation. Gate hover effects behind `@media (hover: hover) and (pointer: fine)` so touch taps don't stick.

## Verify

Watch it run — full speed, then slowed down (devtools Animations panel) or frame by frame; timing flaws are invisible at full speed. Try to break it: spam-click the trigger, interrupt mid-animation, navigate away. If it still feels off, look again tomorrow — fresh eyes catch what shipping eyes miss.

*Easing/gesture details draw on Emil Kowalski's design-engineering notes ([animations.dev](https://animations.dev/)).*
