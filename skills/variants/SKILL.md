---
name: variants
description: Generate 3–5 structurally different variants of a component, screen, or UX flow so the user can compare and pick — all rendered in one place, with a switcher. Use when the user wants design options, alternative takes on a component or flow, says "show me a few versions", "design it twice", or can't decide between directions.
---

# variants

Your first idea is rarely the best — it's just the one you've seen before. The value is **conceptual range**: generate variants that differ in *concept*, not in coats of paint, then converge on one.

## 1. Pin what varies

One sentence: what is being varied (component, screen, flow) and what stays fixed (brand, data, core functionality). Confirm only if genuinely ambiguous.

## 2. Push for range, not depth

3–5 variants, each a **structurally different answer** — not the same layout restyled. The test: a numpad, a numpad with +/- steppers, and a numpad with preset chips are *one* idea with depth. A numpad vs. a slider vs. a gamified picker are three ideas with **range**. Aim for range.

Pick the axes that actually differ for this case — not all at once:

- **Density** — compact and data-rich vs airy and focused
- **Structure** — single view vs stepped, list vs board vs timeline
- **Emphasis** — data-first vs action-first vs explanation-first
- **Interaction** — inline editing vs modal vs dedicated page
- **Tone** — playful vs sober, editorial vs utilitarian

The first one or two variants come easy because you've built them before. To reach the non-obvious rest, deliberately apply at least two:

- **Remove or add a constraint** — what if this needed no screen at all? What if it happened automatically? (Manual photo-backup → auto-backup everything: the interface disappears.)
- **Blend from another domain** — what if this were a game? A physical product? What would Muji ship?
- **Invert the problem** — instead of helping users pick what they want, help them rule out what they don't.
- **Force a count** — commit to 5 (or 12) directions before judging any; the back half is where the surprises live.

Name each variant after its direction (`auto`, `slider`, `gamified`), never "Option A/B/C". If you can't tell two apart in one sentence, they're the same idea — drop one and push harder.

## 3. Build them all in one place

- **App project:** one throwaway route, variants switched via a URL param and a floating switcher bar. Obey the project's routing conventions; mark the route as a prototype.
- **No app:** a single self-contained HTML file with the same switcher. Save to `docs/html/<thing>-variants.html`; delete after picking a winner — it's a comparison artifact, not a permanent record.

Use the same realistic data in every variant so the comparison is fair. Mock data, never wired to real mutations.

## 4. Compare and synthesize

For each variant, 2–3 lines: what it optimizes for, what it sacrifices. Then recommend one — and steal the best detail from the losers. The winner is often variant 2's structure with variant 4's detail.

Once a direction is chosen, the variant graduates into real code and the throwaway route gets deleted.
