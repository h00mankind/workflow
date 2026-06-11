---
name: learn
description: Multi-session learning workspace — teaches the user a topic through small self-contained HTML lessons grounded in a mission, with progress, glossary, and trusted sources tracked in docs/learn/<topic>/. Use when the user wants to learn or be taught something — a language, framework, concept, or practice — or says "teach me", "I want to learn", or "next lesson".
---

# learn

Learning is stateful — the user learns over multiple sessions. State lives in `docs/learn/<topic>/`, created lazily (a file appears when there's something to write in it):

- `MISSION.md` — *why* they're learning this; grounds every lesson. Format: [MISSION-FORMAT.md](MISSION-FORMAT.md).
- `RECORDS.md` — dated, decision-grade insights: what they demonstrably learned, prior knowledge, corrected misconceptions. This is what "what to teach next" is calculated from. Format: [RECORDS-FORMAT.md](RECORDS-FORMAT.md).
- `GLOSSARY.md` — the topic's canonical terms, added only once the user understands them. Format: [GLOSSARY-FORMAT.md](GLOSSARY-FORMAT.md).
- `RESOURCES.md` — curated trusted sources and communities; lesson knowledge comes from here, not parametric guesses. Format: [RESOURCES-FORMAT.md](RESOURCES-FORMAT.md).
- `lessons/NNNN-<slug>.html` — the lessons, numbered.
- `reference/` — cheat sheets, syntax cards, sequences: the compressed essence, designed for quick lookup. Lessons are rarely revisited; references are — keep them current.

## Sessions

- **First session:** no `MISSION.md` → ask why they want to learn this (a couple of questions, not a grill), and find 3–5 high-trust sources for `RESOURCES.md`. Without the mission, lessons drift abstract and there's no way to judge what to teach next.
- **Returning:** read `MISSION.md`, `RECORDS.md`, and `GLOSSARY.md`, then teach the next thing in their zone — challenged just enough, building on the last win.

## Lessons

One self-contained HTML file per lesson — inline CSS/JS, opens directly in a browser, no browser storage, themed to the topic — teaching one tightly-scoped thing tied to the mission. Short — completable in minutes; working memory is small, and each lesson should land one tangible win.

- **Knowledge first**, kept to exactly what the skill needs, drawn from `RESOURCES.md` and cited — for understanding, difficulty is the enemy.
- **Then practice** with a tight feedback loop — a quiz, an in-browser task, or real-world steps with a check. For retention, difficulty is the tool: effortful recall builds long-term memory. Mix in retrieval from earlier lessons (spacing).
- Use `GLOSSARY.md` terms throughout. Quiz answers get the same length and formatting — no clues.
- End with one primary source worth reading and a reminder that follow-up questions are welcome.

## After each lesson

Update the workspace while it's fresh: append qualifying insights to `RECORDS.md`, promote newly-understood terms to `GLOSSARY.md`, move anything worth keeping at hand into `reference/`, and prune `RESOURCES.md` if a source disappointed.
