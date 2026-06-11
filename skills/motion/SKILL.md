---
name: motion
description: Animation and motion design for interfaces — what to animate, durations, easing, performance, and reduced motion. Use when adding or tuning animations, transitions, micro-interactions, page transitions, scroll effects, or anything that moves.
---

# motion

Motion explains — where things came from, that an action landed, where to look next. If an animation explains nothing, cut it.

## Defaults

- **Duration:** 150–250ms for micro-interactions, 250–400ms for larger moves. Nothing in the UI over 500ms.
- **Easing:** ease-out for entrances (fast start, gentle settle), ease-in for exits, ease-in-out for moves. Linear only for continuous motion (spinners, marquees).
- **Exits are quicker** than entrances — about 60–70% of the enter duration. Leaving shouldn't demand attention.
- **Distance is small.** 4–12px for micro-moves; larger travel only when the element semantically comes from somewhere.

## Performance

- Animate `transform` and `opacity` only. Never `top/left/width/height/margin` — that's layout thrash. Size changes via `scale` or the FLIP technique.
- `will-change` sparingly, and remove it when the animation ends.

## Reduced motion

Respect `prefers-reduced-motion` always: replace movement with a fast opacity change, keep the feedback. Wire it once at the system level — a CSS media query or `gsap.matchMedia` — not per animation.

## Tools

CSS transitions for state changes, CSS keyframes for simple loops. Reach for JS (GSAP, Motion) when you need sequencing, scroll-driving, FLIP, or interruptibility — and match whatever the project already uses.

## Verify

Watch it run — at full speed and slowed down (the devtools Animations panel). Then try to break it: spam-click the trigger, navigate away mid-animation.
