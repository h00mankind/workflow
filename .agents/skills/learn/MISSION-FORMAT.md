# MISSION.md format

Captures *why* the user is learning this. Every teaching decision — what to teach next, which sources to use, which exercises to design — traces back here.

```md
# Mission: {Topic}

## Why
{1–3 sentences. The concrete real-world goal. "Ship a Rust CLI to my team" beats "learn Rust".}

## Success looks like
- {A specific, observable thing the user will be able to do}
- {Another}

## Constraints
- {Time, budget, learning preferences — anything that bounds the approach}

## Out of scope
- {Adjacent topics the user explicitly isn't chasing right now}
```

Rules:

- **One mission per topic folder.** Two unrelated goals = two folders.
- **Push back on vagueness.** If the user can't articulate why, interview before writing — a bad mission is worse than none.
- **Revise when reality shifts** — update in place, and note the shift in `RECORDS.md`. Confirm with the user first.
- **Keep it under a screen.** Past that it's a plan, not a compass.
