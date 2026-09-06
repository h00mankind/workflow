---
name: learn
description: Use to teach a topic, start a learning workspace, or continue lessons, including "teach me", "I want to learn", and "next lesson".
---
# learn

Learning is stateful — the user learns over multiple sessions. State lives in `docs/learn/<topic>/`, created lazily (a file appears when there's something to write in it):

- `MISSION.md` — *why* they're learning this; grounds every lesson. Format: [MISSION-FORMAT.md](MISSION-FORMAT.md).
- `RECORDS.md` — dated, decision-grade insights: what they demonstrably learned, prior knowledge, corrected misconceptions. This is what "what to teach next" is calculated from. Format: [RECORDS-FORMAT.md](RECORDS-FORMAT.md).
- `GLOSSARY.md` — the topic's canonical terms, added only once the user understands them. Format: [GLOSSARY-FORMAT.md](GLOSSARY-FORMAT.md).
- `RESOURCES.md` — curated trusted sources and communities; lesson knowledge comes from here, not parametric guesses. Format: [RESOURCES-FORMAT.md](RESOURCES-FORMAT.md).
- `lessons/NNNN-<slug>.html` — the lessons, numbered.
- `reference/` — cheat sheets, syntax cards, sequences: the compressed essence, designed for quick lookup. Lessons are rarely revisited; references are — keep them current.
- `assets/` — reusable components shared across lessons (a course stylesheet, quiz widget, diagram helper). See [Assets](#assets).
- `NOTES.md` — a scratchpad for how the user wants to be taught: pace, format, things to keep in mind. Read it before designing a lesson.

## Sessions

- **First session:** no `MISSION.md` → ask why they want to learn this (a couple of questions, not a grill), and find 3–5 high-trust sources for `RESOURCES.md`. Without the mission, lessons drift abstract and there's no way to judge what to teach next.
- **Returning:** read `MISSION.md`, `RECORDS.md`, and `GLOSSARY.md`, then teach the next thing in their zone — challenged just enough, building on the last win.
- **Mission drift is normal** as skills grow. When it shifts, confirm with the user, update `MISSION.md`, and log the change in `RECORDS.md` — never silently re-aim the course.

## Lessons

One HTML file per lesson — opens directly in a browser, no browser storage, linking the shared course assets (see below) — teaching one tightly-scoped thing tied to the mission. Make it **beautiful and printable** (think Tufte): clean typography, readable layout — the user returns to these to review. Short — completable in minutes; working memory is small, and each lesson should land one tangible win.

Aim for **storage strength** (long-term retention), not **fluency** (in-the-moment recall). Fluency feels like mastery and isn't — the goal is what survives a week, so design for *desirable difficulty*.

- **Knowledge first**, kept to exactly what the skill needs, drawn from `RESOURCES.md` and cited — for understanding, difficulty is the enemy.
- **Then practice** with a tight feedback loop — a quiz, an in-browser task, or real-world steps with a check, ideally graded automatically. For retention, difficulty is the tool: effortful recall builds storage strength. Use the three levers — **retrieval** (recall from memory), **spacing** (pull from earlier lessons), and **interleaving** (mix related topics in one practice set — skills practice only).
- Use `GLOSSARY.md` terms throughout. Quiz answers get the same length and formatting — no clues.
- **Wisdom comes from the real world**, not a lesson. When a question needs judgment over knowledge, answer it, then point the user to a high-reputation community (forum, subreddit, local group) where they can test the skill for real — unless they've said they don't want one.
- End with one primary source worth reading and a reminder that follow-up questions are welcome.

## Assets

Reuse is the default, not the exception. Before authoring a lesson, read `assets/` and build from what's there; when a lesson needs something new and reusable — a quiz widget, a simulator, a diagram helper — write it into `assets/` and link it, never inline code a later lesson would duplicate. A **shared course stylesheet** is the first asset every workspace earns: every lesson links it, so the course reads as one consistent thing rather than a pile of one-offs. The library grows with the course.

## After each lesson

Update the workspace while it's fresh: append qualifying insights to `RECORDS.md`, promote newly-understood terms to `GLOSSARY.md`, move anything worth keeping at hand into `reference/`, note any teaching preference the user voiced in `NOTES.md`, and prune `RESOURCES.md` if a source disappointed.
