---
name: handoff
description: Hand off a session so another agent continues without re-deriving context. Use when the user says handoff, wraps a session for later, or transfers context.
---

# # handoff

Write a handoff document so a fresh agent continues the work without re-deriving it. Save to the OS temp directory — not the workspace — and report the absolute path.

If the user passed an argument, that's what the next session is for; tailor the whole document to it.

## Contents

- **Goal** — what the work is ultimately for, one paragraph.
- **State** — done and verified vs in progress vs not started. Honest: failing tests and skipped steps included.
- **Decisions** — choices made and why, plus what was tried and rejected so it isn't retried.
- **Next steps** — ordered and concrete; the first one immediately actionable.
- **Gotchas** — non-obvious context the next agent would burn time rediscovering.
- **Pointers** — don't duplicate what's already captured in plans, ideas, issues, commits, or diffs; reference them by path or URL.
- **Suggested skills** — which skills the next agent should invoke, and for what.

Redact secrets: API keys, passwords, tokens, PII.
