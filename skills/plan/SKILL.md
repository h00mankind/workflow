---
name: plan
description: Turn an idea or feature into a PRD plus thin vertical-slice issues, in one of two modes — grill (a relentless one-question-at-a-time interview to stress-test the idea first) or trust (no questions, synthesize from existing context and deliver). Maintains the project's knowledge base as it goes — a domain glossary in docs/CONTEXT.md and decision records in docs/adr/. Writes the plan to docs/plans/ and optionally files the slices as GitHub issues. Use when the user wants to plan or brainstorm a feature, stress-test a design, write a PRD, break down work into tickets, or says "grill me" or "plan this".
---

# plan

Two modes — offer the choice up front unless the user already picked:

- **grill** — interview first, then write the plan. Right when the idea is fuzzy, decisions are unresolved, or the user wants their thinking stress-tested.
- **trust** — no questions. Synthesize from the conversation and codebase and deliver. Right when the context is already rich or the user wants speed.

If unstated, recommend one based on how much context exists, in one line: "Context is thin — I'd grill; say *trust* to skip the questions."

## The knowledge base

Planning compounds when decisions and language outlive the session. Two files carry that:

- **`docs/CONTEXT.md`** — the domain glossary: one term per concept, a one-line definition each, no implementation details. Read it before planning; it's the project's language.
- **`docs/adr/NNNN-<slug>.md`** — decision records: context, the decision, why, what was rejected. Read the ones in the area you're touching; they are settled — don't re-litigate them, plan within them.

Create both lazily — only when there's something to write. Update them **inline as decisions crystallize**, not as a batch at the end.

Offer an ADR only when all three hold: the decision is **hard to reverse**, would be **surprising without context**, and resolved a **real trade-off**. Anything less belongs in the PRD's Decisions section, not an ADR.

## Grill mode

Interview the user relentlessly until shared understanding, walking down each branch of the decision tree, resolving dependencies between decisions one at a time:

- **One question at a time.** Wait for the answer before the next.
- **Always recommend.** Every question comes with your recommended answer and a one-line why, so the user can reply "yes" to move fast.
- **Explore instead of asking.** If the codebase can answer it, read the codebase and state what you found.
- **Challenge against the knowledge base.** If the user's words conflict with `CONTEXT.md` ("your glossary says a *cancellation* voids the whole Order — you seem to mean partial") or with an ADR, call it out immediately.
- **Sharpen fuzzy terms.** "You said 'account' — the Customer or the User?" When a term resolves, add it to `CONTEXT.md` right there.
- **Stress-test with scenarios.** Concrete edge cases that force precision: "Two users edit the same draft at once — what happens?"
- **Surface contradictions.** If something the user says conflicts with what the code does, call it out.

When the tree is resolved (or the user says "done"), the decisions flow into the PRD — and the ones that pass the ADR bar get offered as ADRs.

## Trust mode

Work from what's already in the conversation, the codebase, and the knowledge base. Don't ask — state your assumptions in the Decisions section instead. Use `CONTEXT.md` vocabulary throughout; respect existing ADRs. A genuinely blocking unknown goes in Open questions rather than being guessed at.

## The PRD (both modes)

`docs/plans/NNNN-<slug>.md` — 4-digit zero-padded, incrementing from the highest existing number in `docs/plans/` (its own sequence, independent of `docs/adr/`). First plan in a repo starts at `0001`.

- **Problem** — from the user's perspective, not the system's.
- **Solution** — what changes for the user.
- **User stories** — a numbered, extensive list: "As a <actor>, I want <feature>, so that <benefit>."
- **Decisions** — modules touched, schema and API contracts, trade-offs settled (grilled or assumed — mark which), in the glossary's vocabulary. No file paths or code snippets; the one exception is a prototype-validated snippet that encodes a decision more precisely than prose.
- **Open questions** — anything unresolved, so it isn't silently dropped.
- **Out of scope** — explicitly.

## Slice

Break the work into **tracer-bullet vertical slices**: each cuts through every layer end-to-end (schema → API → UI → tests) and is demoable on its own. Many thin slices beat few thick ones. For each: title, what to build (behavior, not layers), acceptance criteria as checkboxes, blocked-by.

In grill mode, present the breakdown and iterate until approved. In trust mode, deliver it — the user edits after, not before.

## Publish

Default: slices live in the plan doc as sections with checkboxes. If the repo has a GitHub remote and the user wants issues, file them with `gh issue create` in dependency order — blockers first, so "Blocked by" references real issue numbers.
