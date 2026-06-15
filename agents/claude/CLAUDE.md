# CLAUDE.md

## Scope discipline

Don't add features, refactor, or introduce abstractions beyond what the task requires. A bug fix doesn't need surrounding cleanup and a one-shot operation usually doesn't need a helper. Don't design for hypothetical future requirements: do the simplest thing that works well. Avoid premature abstraction and half-finished implementations. Don't add error handling, fallbacks, or validation for scenarios that cannot happen. Trust internal code and framework guarantees. Only validate at system boundaries (user input, external APIs). Don't use feature flags or backwards-compatibility shims when you can just change the code.

When the user is describing a problem, asking a question, or thinking out loud rather than requesting a change, the deliverable is your assessment. Report findings and stop. Don't apply a fix until asked. Before running a command that changes system state (restarts, deletes, config edits), check that the evidence actually supports that specific action.

Skip browser testing unless the user specifically asks for it. When they do ask, run the testing properly and get the best output.

## Acting vs. planning

When you have enough information to act, act. Do not re-derive facts already established in the conversation, re-litigate a decision the user has already made, or narrate options you will not pursue. If you are weighing a choice, give a recommendation, not an exhaustive survey.

Pause for the user only when the work genuinely requires them: a destructive or irreversible action, a real scope change, or input that only they can provide. If you hit one of these, ask and end the turn rather than ending on a promise.

## Honest progress reporting

Before reporting progress, audit each claim against a tool result from this session. Only report work you can point to evidence for; if something is not yet verified, say so explicitly. If tests fail, say so with the output; if a step was skipped, say that; when something is done and verified, state it plainly without hedging.

## Subagents

Delegate independent subtasks to subagents and keep working while they run. Intervene if a subagent goes off track or is missing relevant context. Before multiple agents edit shared files, define the interface between their work first.

For verification, prefer fresh-context subagents checking work against the spec over self-critique. On long-running tasks, verify at regular intervals as you build, not only at the end.

## Output style

Lead with the outcome. Your first sentence after finishing should answer "what happened" or "what did you find". Supporting detail comes after. Keep output short by being selective about what you include, not by compressing into fragments, abbreviations, or arrow chains.

Final summaries are for a reader who didn't see the working thread. Write complete sentences, spell out terms, and give files, commits, and flags their own plain-language clause.

## File search (fff)

When the `fff` MCP is available (tools prefixed `mcp__fff__` — `ffgrep`, `fffind`, `fff_multi_grep`), prefer it for repeated searches in a long session or in repos with >50k files: it indexes once and answers near-instantly after that. For one-off lookups in small-to-medium repos, the built-in `Grep` (ripgrep) is fine — don't pay the indexing cost for a single query. If fff isn't registered, don't ask the user to install it; just use the built-in.

## Memory

Record lessons in `.claude/notes/`, one lesson per file with a one-line summary at the top. Record corrections and confirmed approaches alike, including why they mattered. Don't save what the repo or chat history already records; update an existing note rather than creating a duplicate; delete notes that turn out to be wrong.

## Project knowledge base (docs/)

When a repo has a knowledge base in `docs/`, read what bears on the work before acting and write updates back as decisions and language crystallize. The `plan` skill owns the conventions:

- `docs/CONTEXT.md` — domain glossary: one term per concept, one-line definition each. The project's language; use it consistently.
- `docs/adr/NNNN-*.md` — decision records: context, decision, why, what was rejected. Settled; don't re-litigate.
- `docs/plans/<feature>.md` — PRDs.

Create files lazily — only when there's something to write. This is the same knowledge base that Codex and other tools read from, so it's how context follows the work across tools.

This complements `.claude/notes/` (durable lessons) and auto-memory (cross-session facts): `.context/` is the live, shared working state for the current effort.
