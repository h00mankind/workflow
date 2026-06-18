---
name: improve
description: Audit-then-improve pass over UI, UX, code, security, or all of them at once (deep) — always starts by asking what area to improve and which mode: audit (report only) or execute (audit, then apply every safe fix in one go). Sweeps a checklist, then critiques like a hostile reviewer; for UI it inspects the rendered result. Produces severity-ranked findings with evidence; in execute mode it applies the fixes immediately after. Use when the user wants a review, audit, critique, security review, asks "is this safe to ship", or wants something polished, cleaned up, hardened, refactored, or improved.
---

# improve

Find what's weak, prove it, then make it better. Two questions first, always.

## Always ask first

Open with **one AskUserQuestion call** containing both questions — even if the user named an area, confirm the mode (and vice versa):

1. **Area** — `ui` / `ux` / `code` / `security` / `all (deep)`. Deep runs every area and adds an architecture pass.
2. **Mode** — **audit** (report findings, change nothing) or **execute** (audit, then apply every safe fix in one go without a second confirmation).

If the user also gave a scope (a page, flow, module, branch, diff), keep it; otherwise default to the current diff or most recent work and state the assumption.

## Process

1. **Inspect the real thing.** For `ui` and `ux`, look at the rendered result, not the source — see *Seeing the UI* below. For `code` and `security`, read the actual call paths, not just the files that changed.
2. **Sweep with the matching checklist** below, then go past it: cross-cutting patterns, inconsistencies between areas, things the checklist wouldn't catch.
3. **Critique like a hostile reviewer.** A checklist sweep finds the obvious; the findings that matter come from a second, adversarial read — see *The critique pass* below. A friendly reviewer finds nothing.
4. **Report** in the format below.
5. **Execute mode:** apply the fixes immediately, smallest diff per finding, behavior unchanged — this raises quality, not functionality. Verify after: re-inspect the UI, walk the flow, run the code and tests. Findings too risky to patch inline (auth-model changes, migration rewrites, large architecture refactors) stay report-only — say so explicitly. **Audit mode:** stop after the report; end with "run `/improve` again in execute mode to apply these."

### Don't flatten creative intent

Accessibility splits in two. The **floor is non-negotiable and never auto-skipped**: semantic elements, a visible focus state (any style), keyboard reachability, and contrast on text the user must *read or operate*. Always fix these.

But low contrast, nonstandard interaction, or restraint on motion can be a **deliberate choice** on expressive surfaces — a moody hero, a faint watermark, decorative ghost text, an art-directed splash. There, don't auto-fix: **flag it as a tradeoff and ask**. "This hero is 2.8:1, below AA — intentional for the mood, or should I lift it?" Distinguish by role: is the element something the user *uses*, or something they *experience*? Fix the floor; flag the expressive choice.

## The critique pass

Don't just list checklist misses. Read the work a second time as a **hostile senior reviewer** and write down **five specific criticisms, each with a location** — `file:line` for code, screen + element for UI. "The table header and body are both 14px/500, so the header disappears" counts; "could be more polished" does not. Vague criticism is a no-op.

This pass is **mode-scaled** — it does not double back on itself like the building loop in `frontend`:

- **Audit mode:** the five located criticisms *are* the report. No fixing, no second pass — cheap by design.
- **Execute mode:** one pass only — critique, fix all five, then verify (re-inspect / run). No forced repeat. This is an audit of presumably-working code, not a first draft; a second loop is build-time discipline that doesn't pay off here.

## Seeing the UI

For `ui`/`ux`, judge the rendered result — but spend tokens like they're scarce. A screenshot is an image: costly and slow. Reach for the cheapest tool that answers the question, in order:

1. **Accessibility-tree / DOM-text snapshot** — Playwright's `ariaSnapshot` or Chrome DevTools' a11y snapshot. Plain text, a fraction of a screenshot's tokens. Answers most of it: hierarchy, labels, states, focus order, what's actually on the page.
2. **One targeted screenshot** — only when the question is genuinely visual (spacing, contrast, alignment, overflow) and the tree can't show it. Capture the specific component, not the full page; one width, then a narrow width only if responsiveness is in question.
3. **No renderer available?** Say so, read the markup, and reason about it at 360px and 1440px — don't pretend you looked.

Use whichever MCP is connected (Chrome DevTools or Playwright); if neither, a small Playwright screenshot script. Don't screenshot what a text snapshot already told you.

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
