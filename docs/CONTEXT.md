# Glossary

_Project-specific terms. One concept per line, definition only — no implementation._

| Term | Meaning |
|------|---------|
| Knowledge base | The `docs/` folder — domain glossary, decision records, PRDs — shared by all AI tools working in the repo. |
| Cross-tool context | The shared knowledge base in `docs/` that any agent (Claude, Codex, others) can read and write, in plain markdown. |
| ADR (decision record) | A markdown file in `docs/adr/` capturing a hard-to-reverse, surprising-without-context choice and what was rejected. Settled — not re-litigated. |
| PRD | A planning doc in `docs/plans/<feature>.md`: problem, decisions, slices. Produced by the `plan` skill. |
| Skill | An installable unit of agent instructions, one folder per skill, picked up by the `skills.sh` CLI. |
| Workflow suite | The `idea → variants → plan → frontend/backend → improve → ship` chain of skills designed to flow into each other. |
| Experimental skill | A skill in `experimental/` that's a bet, not a tool. Not installed by default; opt in with `--full-depth`. |
| Fable mode | An operating-mode skill (`frontend-fable-mode`, `code-fable-mode`) meant to push *any* capable model toward frontier-level output. Loadable as a skill or pastable into another model's system prompt. |
| Banned list | A skill's enumerated tells of generic AI output (purple gradients, identical rounded cards, swallowed exceptions, etc.) that the skill must refuse. |
| Self-critique loop | A mandatory hostile pass the skill runs on its own output before returning — used in `frontend`, `backend`, and the fable modes. |
| Benchmark | A reproducible head-to-head run in `benchmarks/<NNNN>-<slug>/` comparing models or skills (with-skill vs bare baseline) and producing an HTML report. |
| HTML artifact | A single self-contained `.html` file produced by the `html` or `name` skills. Saved to `docs/html/` in a repo; themed to the content rather than a shared stylesheet. |
| Idea lifecycle | The status states an idea moves through in `docs/ideas/`: `spark → validated → building → parked → dead`. |
| Vertical slice | A thin end-to-end issue produced by the `plan` skill — one user-visible behavior shipped through every layer, not a horizontal task per layer. |
| Grill mode | The `plan` mode that interviews the user one question at a time to stress-test an idea before writing the PRD. |
| Trust mode | The `plan` mode that skips questions and synthesizes the PRD from existing context. |
| Skill mode | A named variant inside a single skill (e.g. `quick`/`deep` in `name`, `grill`/`trust` in `plan`, `audit`/`execute` in `improve`) — preferred over splitting into sibling skills. |
