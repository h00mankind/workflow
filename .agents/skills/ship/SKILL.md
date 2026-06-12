---
name: ship
description: Prepare and publish local git changes through branch, commit, push, and pull request with reviewer-ready discipline. Use when the user says "ship it", "commit this", "push this", "open a PR", "create a PR", "publish these changes", or "make a branch". Skip for read-only git inspection, CI debugging, or responding to PR review comments.
---

# ship

Ship like a careful engineer: know the diff, prove it works, publish only the intended change, and leave a PR a reviewer can understand.

## Process

1. **Pin the scope.** Read `git status`, the current branch, unstaged diff, staged diff, and recent commits if needed. State what belongs to this shipment.
2. **Protect user work.** Never stage unrelated files. If the tree contains ambiguous changes, ask before staging or committing them. Keep generated artifacts out unless the task requires them.
3. **Be on the right branch.** Not on one? Create it before committing — never commit new work straight to main.
4. **Verify before commit.** Run the narrowest meaningful tests, lint, typecheck, build, or smoke command. If verification cannot run, say exactly why and do not pretend it passed.
5. **Commit deliberately.** Prefer one cohesive commit per reviewable change. Message format: imperative subject under 72 characters, optional body explaining why. Avoid vague subjects like `update` or `fix stuff`.
6. **Push the branch.** Push the current branch to the expected remote, usually `origin`, and set upstream when needed. Do not force-push unless the user explicitly requests it or the branch is clearly yours and the reason is stated first.
7. **Open the PR.** Use `gh pr create`; without `gh`, push and hand the user the compare URL. Default to a draft PR when confidence is incomplete or checks are still pending; otherwise make it ready for review.
8. **Report the shipment.** Include branch, commit SHA, PR URL, verification command and result, and any residual risk.

## PR shape

Use a compact PR body:

- **Summary** — what changed, in 1-3 bullets.
- **Verification** — commands run and their result.
- **Risk** — migrations, auth, data loss, rollout concerns, or "None identified".

Mention screenshots only when UI changed. Link issues only when they are directly relevant. Do not pad the PR with process narration.

## Boundaries

- Do not merge PRs unless the user explicitly asks.
- Do not amend, rebase, reset, clean, delete branches, or force-push without a clear reason and approval when user work could be affected.
- Do not use `git add .` in a dirty repo unless every changed file was inspected and belongs to the shipment.
- Do not commit secrets, local env files, logs, or dependency/vendor churn unless the task explicitly requires them.
- Do not create feature flags, compatibility shims, or unrelated cleanup just to make the PR feel bigger.

## Example

Good: inspect the diff, stage only `src/auth.ts` and `src/auth.test.ts`, run `npm test -- auth`, commit `Fix expired token refresh`, push `fix/token-refresh`, and open a PR whose body names the behavior and verification.

Bad: run `git add .` on a dirty tree, commit `updates`, skip tests, push to `main`, and open a PR that says "misc changes".
