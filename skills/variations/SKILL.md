---
name: variations
description: Generate 3–5 radically different variations of a component, screen, or UX flow so the user can compare and pick — all rendered in one place, with a switcher. Use when the user wants design options, alternative takes on a component or flow, says "show me a few versions", "design it twice", or can't decide between directions.
---

# variations

Your first design is rarely the best. Generate radically different alternatives, then compare — the value is in the contrast.

## 1. Pin what varies

One sentence: what is being varied (component, screen, flow) and what stays fixed (brand, data, core functionality). Confirm only if genuinely ambiguous.

## 2. Assign directions

3–5 variants, each from a **genuinely different direction** — not the same layout with different colors. Pick the axes that matter for this case:

- **Density** — compact and data-rich vs airy and focused
- **Structure** — single view vs stepped, list vs board vs timeline
- **Emphasis** — data-first vs action-first vs explanation-first
- **Interaction** — inline editing vs modal vs dedicated page
- **Tone** — playful vs sober, editorial vs utilitarian

Name each variant after its direction, not "Option A/B/C".

## 3. Build them all in one place

- **App project:** one throwaway route, variants switched via a URL param and a floating switcher bar. Obey the project's routing conventions; mark the route as a prototype.
- **No app:** a single self-contained HTML file with the same switcher.

Use the same realistic data in every variant so the comparison is fair. Mock data, never wired to real mutations.

## 4. Compare and synthesize

For each variant, 2–3 lines: what it optimizes for, what it sacrifices. Then recommend one — and steal the best detail from the losers. The winner is often variant 2's structure with variant 4's detail.

Once a direction is chosen, the variant graduates into real code and the throwaway route gets deleted.

---

*Adapted from [Matt Pocock's skills](https://github.com/mattpocock/skills): `design-an-interface` and `prototype`'s UI branch.*
