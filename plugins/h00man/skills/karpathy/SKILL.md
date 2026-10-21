---
name: karpathy
description: Surgical coding discipline against overcomplication and scope drift. Use for non-trivial coding or reviews when the user says karpathy mode, be surgical, stay in scope, or do not overcomplicate; loop until verified.
license: MIT
---

# # karpathy

Behavioral guidelines that bias toward caution over speed. For trivial
tasks, use judgment — the pillars are overhead.

## 1. Think before coding

Don't assume. Don't hide confusion. Surface tradeoffs.

- State assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them — don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity first

Minimum code that solves the problem. Nothing speculative.

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Self-check: "Would a senior engineer call this overcomplicated?" If yes, simplify.

## 3. Surgical changes

Touch only what you must. Clean up only your own mess.

- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it — don't delete it.
- Remove imports/variables/functions that *your* changes made unused.
  Don't remove pre-existing dead code unless asked.

The test: every changed line should trace directly to the user's request.

## 4. Goal-driven execution

Define success criteria. Loop until verified.

Translate fuzzy tasks into verifiable goals:

- "Add validation" → write tests for invalid inputs, then make them pass.
- "Fix the bug" → write a test that reproduces it, then make it pass.
- "Refactor X" → ensure tests pass before *and* after.

For multi-step work, state a brief plan up front:

```
1. <step> → verify: <check>
2. <step> → verify: <check>
3. <step> → verify: <check>
```

Strong success criteria let you loop independently. Weak ones
("make it work") force constant clarification.

## Concrete example

> User: "Clean up `parse_input` — it's gotten messy."

Bad:

- Rewrites `parse_input` *and* refactors `validate_input` next to it.
- Renames a few helpers along the way ("while I'm here").
- Adds a `ParseOptions` config object "for flexibility".
- Reports: "Cleaned up the input layer."

Good:

- States the read: "Two issues — `parse_input` has nested conditionals, and an unused branch from an old format. Cleanup = flatten the conditionals + drop the dead branch. Anything else you want touched?"
- On approval: flattens conditionals, removes the dead branch, removes the now-unused helper that branch called. Touches nothing else.
- Reports: "Flattened the conditionals in `parse_input`; removed the v1-format branch and its helper `_parse_v1_row`. `validate_input` not touched."
