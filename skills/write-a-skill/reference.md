# Writing skills — the theory

The deep version of `SKILL.md`. Load this when a skill is misbehaving, when you're deciding whether and how to split one, or when you want the *why* behind a house-style rule. Routine skill edits don't need it.

Adapted from Matt Pocock's [`writing-great-skills`](https://github.com/mattpocock/skills) (MIT). The vocabulary is his; condensed to fit this repo.

## The root virtue: predictability

A skill exists to wrangle determinism out of a stochastic system. **Predictability** means the agent takes the same *process* every run — not that it produces the same output. Every lever below serves it. When you're unsure whether a rule earns its place, ask: does it make the agent's process more repeatable? If not, cut it.

## The two loads

Every design choice spends one of two budgets. Naming them makes the trade-off visible.

- **Context load** — tokens sitting in the window every turn. A model-invoked description pays this. More skills the agent can see autonomously = more context load.
- **Cognitive load** — what the *user* must hold in their own head. A user-invoked skill (`disable-model-invocation: true`) pays this: the user is now the index that has to remember it exists.

A skill trades one for the other; it can't escape both. When user-invoked skills pile up past what the user can remember, that cognitive load is cured by a **router skill** — one user-invoked skill that names the others and when to reach each.

## The information hierarchy

A skill is built from two content types — **steps** (ordered actions) and **reference** (definitions, rules, facts) — that mix freely. The core decision is *where each sits* on a ladder ranked by how immediately the agent needs it:

1. **In-skill step** — an ordered action in `SKILL.md`. The primary tier: what the agent does, in order. Each ends on a **completion criterion**.
2. **In-skill reference** — a definition or rule in `SKILL.md`, consulted on demand. Often a legitimately flat peer-set (every rule of a review on one rung) — fine, not a smell.
3. **External reference** — reference pushed out of `SKILL.md` into a linked file, reached by a **context pointer**, loaded only when the pointer fires. (This `reference.md` is that tier.)

Push too little down and the top bloats; push too much and you hide what the agent needs. That tension is the whole decision. **Progressive disclosure** is the move down the ladder — out of `SKILL.md` into a linked file — so the top stays legible.

**Branching is the cleanest disclosure test.** Each distinct way a skill is used is a **branch**. Inline what *every* branch needs; push behind a pointer what only *some* branches reach. A context pointer's *wording*, not its target, decides when and how reliably the agent reaches the material.

**Co-location** decides what sits *beside* a piece once it's placed: keep a concept's definition, rules, and caveats under one heading rather than scattered, so reading one part brings its neighbours along.

## Completion criteria

Each step ends on the condition that tells the agent the work is done. Make it:

- **Checkable** — can the agent tell done from not-done? "Produce a change list" can't; "every modified model accounted for" can.
- **Exhaustive** where it matters — "every rule applied", not "apply rules". A vague criterion invites **premature completion**.

A demanding completion criterion drives thorough **legwork** — the digging the agent does within the work — whether the skill has steps or not, since "every rule applied" binds flat reference just as "every step done" binds a sequence.

## Leading words

A **leading word** is a compact concept already living in the model's pretraining that the agent thinks *with* while running the skill — *lesson*, *fog of war*, *tracer bullet*, *tight loop*, *red*, *surgical*. Repeated across the text, it accumulates a distributed definition and anchors a whole region of behaviour in the fewest tokens, by recruiting priors the model already holds.

It serves predictability twice:

- **In the body** it anchors *execution* — the agent reaches for the same behaviour every time the word appears.
- **In the description** it anchors *invocation* — when the same word lives in the user's prompts, docs, and code, the agent links that shared language to the skill and fires it more reliably.

Hunt for refactors that introduce one. Each of these begs to collapse into a single token:

- "fast, deterministic, low-overhead" → *tight* (a *tight* loop).
- "a loop you believe in" → *red* (the loop goes *red* on the bug, or it doesn't) — converts a fuzzy gate into a binary observable.

You win twice: fewer tokens, *and* a sharper hook for the agent to hang its thinking on. Assume every skill is carrying restatements that leading words retire — go find them.

## When to split

**Granularity** is how finely you divide skills. Each cut spends one of the two loads, so split only when the cut earns it:

- **By invocation** — split off a model-invoked skill when you have a distinct leading word that should trigger it on its own, or another skill must reach it. You pay context load for the new always-loaded description, so that independent reach has to be worth it.
- **By sequence** — split a run of steps when the steps still ahead tempt the agent to rush the one in front of it. Keeping them out of view encourages more legwork on the current task.

In this repo the bias is *against* splitting: prefer a mode or a target argument over a new sibling (`plan` grill/trust, `improve` ui/code/security). A new skill has to clear a high bar.

## Pruning

- **Single source of truth.** Keep each meaning in one authoritative place, so changing the behaviour is a one-place edit.
- **Relevance.** Check every line: does it still bear on what the skill does?
- **No-ops, sentence by sentence.** Run the no-op test on each sentence in isolation; when one fails, delete the whole sentence rather than trim words. Be aggressive — most prose that fails should go, not be rewritten.

## Failure modes — diagnose a misbehaving skill against these

- **Premature completion** — a step ending before it's genuinely done, attention slipping to *being done*. Defence in order: sharpen the completion criterion first (cheap, local); only if it's irreducibly fuzzy *and* you observe the rush, hide the later steps by splitting.
- **Duplication** — the same meaning in more than one place. Costs maintenance and tokens, and inflates a meaning's prominence on the ladder past its real rank.
- **Sediment** — stale layers that settle because adding feels safe and removing feels risky. The default fate of any skill without a pruning discipline.
- **Sprawl** — a skill simply too long, even when every line is live and unique. Cure with the ladder: disclose reference behind pointers, split by branch or sequence so each path carries only what it needs.
- **No-op** — a line the model already obeys by default, so you pay load to say nothing. The test: does it change behaviour versus the default? A weak leading word (*be thorough* when the agent is already thorough-ish) is a no-op; the fix is a stronger word (*relentless*), not a different technique.

## Glossary

**Module-invoked / user-invoked** — see [The two loads](#the-two-loads).
**Context load / cognitive load** — the two budgets a skill spends.
**Information hierarchy** — the ladder (step → in-skill reference → external reference) ranked by immediacy of need.
**Progressive disclosure** — moving a piece down the ladder into a linked file.
**Context pointer** — the line that reaches a disclosed file; its wording sets reliability.
**Co-location** — keeping a concept's parts under one heading.
**Branch** — a distinct way the skill is used; different runs taking different paths.
**Completion criterion** — the checkable condition that ends a step.
**Legwork** — the digging the agent does within the work, driven by a demanding criterion.
**Leading word** — a pretrained concept the agent thinks with; anchors behaviour in one token.
**Granularity** — how finely skills are divided; each cut spends a load.
**Single source of truth** — one authoritative place per meaning.
