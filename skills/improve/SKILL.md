---
name: improve
description: Audit-then-improve pass over UI, UX, code, security, or all of them at once (deep) — always starts by asking what area to improve and which mode: audit (report only) or execute (audit, then apply every safe fix in one go). Produces severity-ranked findings with evidence; in execute mode it applies the fixes immediately after. Use when the user wants a review, audit, critique, security review, asks "is this safe to ship", or wants something polished, cleaned up, hardened, refactored, or improved.
---

# improve

Find what's weak, prove it, then make it better. Two questions first, always.

## Always ask first

Open with **one AskUserQuestion call** containing both questions — even if the user named an area, confirm the mode (and vice versa):

1. **Area** — `ui` / `ux` / `code` / `security` / `all (deep)`. Deep runs every area and adds an architecture pass.
2. **Mode** — **audit** (report findings, change nothing) or **execute** (audit, then apply every safe fix in one go without a second confirmation).

If the user also gave a scope (a page, flow, module, branch, diff), keep it; otherwise default to the current diff or most recent work and state the assumption.

## Process

1. **Inspect the real thing.** For `ui` and `ux`, run the app and look at it — screenshots, clicked-through flows — not just source. For `code` and `security`, read the actual call paths, not just the files that changed.
2. **Sweep with the matching checklist** below, then go past it: cross-cutting patterns, inconsistencies between areas, things the checklist wouldn't catch.
3. **Report** in the format below.
4. **Execute mode:** apply the fixes immediately, smallest diff per finding, behavior unchanged — this raises quality, not functionality. Verify after: render the UI, walk the flow, run the code and tests. Findings too risky to patch inline (auth-model changes, migration rewrites, large architecture refactors) stay report-only — say so explicitly. **Audit mode:** stop after the report; end with "run `/improve` again in execute mode to apply these."

## ui

- Spacing and alignment consistent with neighboring views; project tokens and scale, not magic numbers.
- Hierarchy: one primary action per view; size, weight, and color match importance.
- Contrast and legibility at real sizes; all states exist: hover, focus-visible, active, disabled, loading, empty, error.
- Responsive: nothing overflows or squishes at narrow widths.

## ux

- Shortest path: cut steps, fields, and confirmations that don't earn their place.
- Feedback: every action gets an acknowledgment the user can't miss.
- Errors are recoverable: say what happened and what to do; never throw away the user's input.
- Defaults do the right thing; no dead ends — every empty state and error points somewhere.

## code

- Delete first: dead code, unused exports, commented-out blocks, speculative abstraction.
- Dedupe against existing helpers before keeping anything new; names say what things are, in the project's domain terms.
- Error handling lives at the boundary, not scattered through internals.
- **Depth over fragmentation.** A module (function, class, package) is *deep* when a small interface hides a lot of behavior, *shallow* when the interface is nearly as complex as the implementation. Flag places where understanding one concept means bouncing between many small modules, and where pure functions were extracted "for testability" but the real bugs hide in how they're called.
- **Deletion test** on anything suspect: would deleting it concentrate complexity in one place (it was earning its keep), or just move it (pass-through — inline it)?
- Propose *deepening* refactors — fold shallow fragments behind one small interface; the interface is the test surface. Explain the benefit as locality (change, bugs, and knowledge concentrated in one place) and leverage (callers know less).

## security

- Secrets out of code, config defaults, logs, error messages, and client bundles.
- Anything user input reaches is parameterized or encoded: SQL, shell commands, HTML (XSS), file paths, redirects.
- Every endpoint and query has an explicit answer to "who can call this?" — including object-level access (can user A fetch user B's record by changing an id?).
- External input validated at the boundary; size limits on bodies and uploads; file types checked.
- No PII in logs, analytics, or URLs; destructive operations guarded: confirmation, dry-run, scoped WHERE clause.

State each security finding as an attack or failure scenario in one sentence.

## all (deep)

Run every checklist above, then an architecture pass over the scope: where are the shallow modules, leaky seams, and untestable interfaces? Apply the deletion test and propose the top deepening opportunities, each rated **Strong / Worth exploring / Speculative**. Use Explore subagents to cover areas in parallel.

## Report format

Each finding:

- **Severity** — blocker / major / minor / nit (security: critical / high / medium / low)
- **Where** — `file:line` for code; screen + element for UI/UX
- **What's wrong** and **why it matters** — one or two sentences
- **Suggested fix** — one line

Order by severity. End with the **top 3 to fix first** and an overall verdict in 2–3 sentences — for `security`, an explicit **go / no-go** for shipping. In execute mode, follow with what was changed and anything left report-only with the reason.
