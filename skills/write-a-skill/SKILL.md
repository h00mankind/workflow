---
name: write-a-skill
description: Create or revise a skill in this repo, following its house style — one-word names where possible, trigger-rich descriptions, concise bodies, combined skills with modes or targets over many narrow ones. Use when the user wants to create, write, improve, simplify, or combine a skill, or says "make a skill for X".
---

# write-a-skill

A skill is loaded by its description and judged by its brevity. The description does the triggering; the body does the teaching.

## House style

- **One-word name** when possible. Prefer a target argument (`/audit ui`) or a mode (`plan` grill/trust) over a family of narrow sibling skills.
- **Check the suite first.** If an existing skill half-covers the request, extend or combine it instead of adding a sibling.
- **Default postures:** report first, fix on approval; capture outputs to `docs/` (ideas, plans, learn) so sessions compound; end by offering the natural next step (idea → plan, audit → fixes).
- **Every skill stands alone.** Sibling skills are optional pointers, never requirements — if another skill's rules are genuinely needed, inline the one or two essential lines instead of "follow /x".

## Description — the only part the agent sees when choosing

- First sentence: what it does. Then "Use when …" with the actual phrases users say. Third person, under 1024 characters.
- Include skip conditions if the skill could over-trigger ("Skip for backend logic…").

## Body

- Under ~100 lines, every line earning its place: a one-line philosophy, then process or rules.
- Concrete beats abstract — one good/bad pair beats three paragraphs of theory.
- Depth that's rarely needed goes in a bundled file (`reference.md`) linked one level deep — see `name/` for the pattern.
- No time-sensitive info. One term per concept, used consistently.

## Checklist before done

- [ ] Description has "Use when …" triggers
- [ ] Body under ~100 lines (split to a bundled file if over)
- [ ] At least one concrete example
- [ ] Stands alone — sibling mentions are optional pointers, not dependencies
- [ ] README "What's here" entry added or updated
