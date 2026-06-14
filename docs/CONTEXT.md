# Glossary

_Project-specific terms. One concept per line, definition only — no implementation._

| Term | Meaning |
|------|---------|
| Knowledge base | The `docs/` folder — domain glossary, decision records, PRDs — shared by all AI tools working in the repo. |
| Decision record (ADR) | A markdown file capturing a hard-to-reverse, surprising-without-context choice and what was rejected. Settled — not re-litigated. |
| Cross-tool context | The shared knowledge base in `docs/` that any agent (Claude, Codex, others) can read and write, in plain markdown. |
| Skill | An installable unit of agent instructions, one folder per skill, picked up by the `skills.sh` CLI. |
