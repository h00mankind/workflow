---
name: audit
description: Audit-then-fix pass for UI, UX, code, or safety — invoked as `/audit ui`, `/audit ux`, `/audit code`, or `/audit safety`, plus an optional scope (a page, flow, module, branch, or diff). Default is a read-only audit producing severity-ranked findings with evidence; it then offers to apply the fixes, and nothing changes until the user says yes. Use when the user wants a review, audit, critique, security review, asks "is this safe to ship", or wants something polished, cleaned up, hardened, or improved.
---

# audit

Inspect and report first. Fix only after the user says yes.

## Process

1. **Pin the target and scope.** `ui` / `ux` / `code` / `safety`, plus what to audit. If not given, default to the current diff or most recent work and state the assumption.
2. **Inspect the real thing.** For `ui` and `ux`, run the app and look at it — screenshots, clicked-through flows — not just source. For `code` and `safety`, read the actual call paths, not just the files that changed.
3. **Sweep with the matching checklist** below, then go past it: cross-cutting patterns, inconsistencies between areas, things the checklist wouldn't catch.
4. **Report** (format below) and **offer the fixes**: "Want me to apply these?" The user can take all of them or pick by number.

## Applying fixes (on approval)

- Smallest diff per finding; behavior stays the same — this raises quality, not functionality.
- Verify after: render the UI, walk the flow, run the code and tests.
- Safety findings too risky to patch inline (auth-model changes, migration rewrites) stay report-only — say so explicitly.
- Report what changed; anything approved but not fixed gets listed with the reason.

## ui

- Spacing and alignment consistent with neighboring views; project tokens and scale, not magic numbers.
- Hierarchy: one primary action per view; size, weight, and color match importance.
- Contrast and legibility at real sizes, not just zoomed-in.
- All states exist: hover, focus-visible, active, disabled, loading, empty, error.
- Responsive: nothing overflows or squishes at narrow widths.

## ux

- Shortest path: cut steps, fields, and confirmations that don't earn their place.
- Feedback: every action gets an acknowledgment the user can't miss.
- Errors are recoverable: say what happened and what to do; never throw away the user's input.
- Defaults do the right thing for the common case.
- No dead ends: every empty state and error points somewhere.

## code

- Delete first: dead code, unused exports, commented-out blocks, speculative abstraction.
- Dedupe against existing helpers before keeping anything new.
- Names say what things are, in the project's domain terms.
- Error handling lives at the boundary, not scattered through internals.
- Shallow wrappers get inlined — deletion test: would removing it concentrate complexity, or just move it?

## safety

- Secrets out of code, config defaults, logs, error messages, and client bundles.
- Anything user input reaches is parameterized or encoded: SQL, shell commands, HTML (XSS), file paths, redirects.
- Every endpoint and query has an explicit answer to "who can call this?" — including object-level access (can user A fetch user B's record by changing an id?).
- External input validated at the boundary; size limits on bodies and uploads; file types checked.
- No PII in logs, analytics, or URLs.
- Destructive operations guarded: confirmation, dry-run, scoped WHERE clause.

State each safety finding as an attack or failure scenario in one sentence.

## Report format

Each finding:

- **Severity** — blocker / major / minor / nit (safety: critical / high / medium / low)
- **Where** — `file:line` for code; screen + element for UI/UX
- **What's wrong** and **why it matters** — one or two sentences
- **Suggested fix** — one line

Order by severity. End with the **top 3 to fix first** and an overall verdict in 2–3 sentences — for `safety`, an explicit **go / no-go** for shipping. Then the offer to apply.
