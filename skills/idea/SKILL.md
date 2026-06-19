---
name: idea
description: Capture, track, validate, and prototype ideas — one file per idea in docs/ideas/ with a status lifecycle (spark → validated → building → parked → dead). Capture is fast and question-free, and ends by offering a throwaway prototype to test the idea's riskiest assumption. Use when the user has a new idea to jot down, asks "what ideas do I have", wants to validate or kill an idea, or wants to prototype or sanity-check a data model, state machine, or logic before committing.
---

# idea

One file per idea in `docs/ideas/<slug>.md`. Four operations — infer which from the request. Default is **capture**.

## File format

```markdown
---
status: spark | validated | building | parked | dead
created: YYYY-MM-DD
updated: YYYY-MM-DD
---

One-liner: what it is and who it's for.

Free-form notes.

## Validation (added when validated)
## Prototype findings (added when prototyped)
```

## Capture (default)

Fast and question-free. Write down what the user said in their words, add the one-liner, set status `spark`. At most one clarifying question, and only if the idea is unintelligible without it.

Confirm with the file path, then offer the next step in one line, no pressure: "Want a prototype to test <the idea's riskiest assumption>?"

## List

Read `docs/ideas/`, show a table: idea, one-liner, status, age. Flag sparks older than a month — suggest validating, prototyping, or parking them.

## Validate

A short grill — one question at a time, five max, each with your recommended answer:

1. **Problem** — who has it, and how painful is it really?
2. **Alternatives** — what do they do today, and why isn't that enough?
3. **Smallest version** — what could exist in a day or a week?
4. **Why you, why now** — what's the unfair advantage or the forcing window?
5. **Kill criteria** — what finding would make this not worth doing?

Verdict with reasoning — `validated`, `parked`, or `dead` — recorded in a `## Validation` section. You recommend; the user decides. Validated → offer a prototype or `/plan`.

## Prototype

A prototype is **throwaway code that answers a question** — usually the idea's riskiest assumption. Write the question as a comment at the top; if you can't state it in one sentence, validate first. Prototypes answer logic questions; "what should it look like?" is a design-variants exercise (the `variants` skill), not a prototype.

1. **Throwaway from day one.** A `prototypes/` folder or `proto-` prefix — nobody mistakes it for production.
2. **One command to run**, using whatever runner the project already has.
3. **No persistence.** Memory only; a scratch file named "PROTOTYPE — wipe me" if the question is about storage.
4. **Skip the polish.** No tests, no error handling beyond runnable, no abstractions.
5. **Surface the state** after every action so the user sees what changed.

When the question is answered, write the answer into the idea file under `## Prototype findings` — the question, the verdict, what it changes — and update the status. Then delete the prototype or fold the validated decision into real code; don't leave it rotting.
