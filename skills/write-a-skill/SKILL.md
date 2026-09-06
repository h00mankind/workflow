---
name: write-a-skill
description: Create, revise, or combine repo skills in house style with one-word names and modes over siblings. Use when the user says make a skill for X, improve a skill, or simplify skills.
---

# # write-a-skill

A skill exists to wrangle determinism out of a stochastic system. **Predictability** — the agent taking the same *process* every run, not producing the same output — is the goal, and every rule below serves it. A skill is loaded by its description and judged by its brevity: the description does the triggering, the body does the teaching.

For the theory behind these rules — the full vocabulary, the information-hierarchy ladder, and the failure-mode diagnostics — load `reference.md` in this skill folder. The essentials are below; reach for `reference.md` when a skill is misbehaving or you're deciding how to split one.

## House style

- **One-word name** when possible. Prefer a target argument (`/improve` ui) or a mode (`plan` grill/trust) over a family of narrow sibling skills.
- **Check the suite first.** If an existing skill half-covers the request, extend or combine it instead of adding a sibling.
- **Default postures:** report first, fix on approval; capture outputs to `docs/` (ideas, plans, learn) so sessions compound; end by offering the natural next step (idea → plan, improve → fixes).
- **Every skill stands alone.** Sibling skills are optional pointers, never requirements — if another skill's rules are genuinely needed, inline the one or two essential lines instead of "follow /x".

## Invocation — two kinds, paying two different costs

Every skill spends one of two loads, and the choice of how it's invoked decides which:

- **Model-invoked** (the default here) keeps a `description`, so the agent can fire it on its own *and* other skills can reach it. It costs **context load** — the description sits in the window every turn. Write a rich, trigger-laden description.
- **User-invoked** sets `disable-model-invocation: true`. Only the user, typing its name, can invoke it; the description becomes a human-facing one-liner. Zero context load, but it costs **cognitive load** — the user is now the index that must remember it exists.

Pick model-invocation when the agent (or another skill) must reach it autonomously — which is nearly all of this suite, since these skills trigger on what the user *says*. Reserve `disable-model-invocation` for a skill only ever run by hand.

## Description — the only part the agent sees when choosing

- First sentence: what it does, leading word first. Then "Use when …" with the actual phrases users say. Third person, under 1024 characters.
- **One trigger per branch.** Synonyms renaming a single trigger are duplication — collapse them; keep only genuinely distinct branches.
- Include skip conditions if the skill could over-trigger ("Skip for backend logic…").

## Body

- Under ~100 lines, every line earning its place: a one-line philosophy, then process or rules.
- **Lead with a leading word** — a compact concept already in the model's pretraining (*tracer bullet*, *red loop*, *surgical*) that anchors a whole region of behaviour in one token. A triad spelled out at three sites is a passage begging to collapse into one. Hunt for these.
- **End steps on a checkable completion criterion** — a condition the agent can tell done from not-done ("every modified model accounted for", not "produce a change list"). A vague criterion invites premature completion.
- Concrete beats abstract — one good/bad pair beats three paragraphs of theory.
- Depth that's rarely needed goes in a bundled file (`reference.md`) linked one level deep — see `name/` and this skill for the pattern. The pointer's *wording* decides whether the agent reaches it; say what it holds and when to load it.
- No time-sensitive info. One term per concept, used consistently.

## When you're revising, not writing

Diagnose against the failure modes (full list in `reference.md`):

- **Sediment / sprawl** — stale or simply too-long. Cure by pruning and by disclosing reference behind a pointer.
- **Duplication** — the same meaning in two places. Keep one source of truth.
- **No-op** — a line the model already obeys by default (*be thorough* when it already is). Delete it or swap in a stronger leading word.
- **Premature completion** — a step ending before it's done. Sharpen its completion criterion first; only split the steps apart if that fails.

Hunt no-ops sentence by sentence. When one fails the test, delete the whole sentence — don't trim words from it. Be aggressive; most prose that fails should go, not be rewritten.

## Checklist before done

- [ ] Description has "Use when …" triggers, one per distinct branch
- [ ] Invocation kind is deliberate (model-invoked unless it's hand-only)
- [ ] Body under ~100 lines (disclose to a bundled file if over)
- [ ] At least one concrete example, and a leading word doing real work
- [ ] Steps end on checkable completion criteria
- [ ] Stands alone — sibling mentions are optional pointers, not dependencies
- [ ] No TBD, "coming soon", or unresolved placeholders in the body
- [ ] Top-level `README.md` "What's here" entry added or updated
