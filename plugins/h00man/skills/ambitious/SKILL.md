---
name: ambitious
description: Use when writing or sharpening a prompt for another agent or a later run, or before handing a fuzzy task to another agent; turns a rough idea or draft into one ambitious, paste-ready prompt an agent can run start to finish.
---

# ambitious

Turn a rough idea, half-baked request, or draft into one prompt that a coding or work agent can run start to finish without help. Ambitious means deeper, not bigger: a truer goal and an outside proof of done, not more scope.

## When to use

- The user asks you to write or sharpen a prompt for another agent or for a later run.
- You are about to hand a fuzzy task to another agent.

If the user asked only for the prompt, stop after you deliver it. If they also asked you to do the work, write the prompt first, then run it. This skill alone is not a reason to run the work.

## 1. Sharpen silently

Run pstack's `sharpen` skill in your head first. Find the deeper goal and the proof of done that comes from outside you and the user. Use it to shape your guesses. Its methods and tools stay out of the prompt.

## 2. Ask once

Ask one message, then write nothing until the user answers. Include:

- **Goal.** The real goal in one line: what, and for whom.
- **Hill climbing.** Does it fit? It fits big, tedious, checkable work, judge-scored work included.
- **Fuzzy work.** Warn that the judge is the weak spot. Ask for rubric points and reference screenshots. If none come, the prompt has the agent draft the rubric and collect references first, and log them as assumptions.
- **Four guesses, pre-filled** so the user can say yes or fix one:
  - Done: what it means, how it is checked, and the next measurable target once it passes.
  - Scope.
  - What the agent may touch.
  - Stop rule: N attempts in a row with no improvement, plus a generous safety ceiling.

If the user says "just write it", use your guesses. Put the riskiest ones on one `Assumed:` line in Context.

## 3. Write the prompt

Write it yourself from this recipe. Do not send it through another model or CLI.

Use the user's voice (I, my), as if they wrote it. Write in English, even if the user wrote in another language, unless they asked otherwise. Aim for 100 to 150 words. The hard maximum is 200.

Use five short sections and nothing else:

- **Goal:** one or two lines. What, and for whom.
- **Context:** only facts, inputs, and paths the agent cannot find itself. Add a fallback for any missing input, the rubric and reference setup a judged check needs, and the `Assumed:` line if there is one.
- **Done when:** the automated check. That is tests, parity, a benchmark, or for subjective work a judge panel against a written rubric (the user's points, or one the agent drafts first). Add what must not get worse. If hill climbing fits, add the next target or targets, one line each. Accept a change only if every earlier check still passes. End on the best passing version.
- **Limits:** hard constraints and anti-goals, plus what not to touch.
- **Run rules:** one compact line. Don't ask questions. Log assumptions and blockers to files and keep going. No destructive, irreversible, paid, or secret-touching actions. Stop after N attempts in a row with no improvement, or at the safety ceiling (an attempt cap and an hour cap).

Give outcomes, not instructions. Never say how to do the work: no steps, methods, tools, log formats, or edge cases. Exact details of the check are allowed. Name no model, CLI, or tool, so the prompt runs in any harness. Cut each sentence that states what the agent would do anyway. Write unslopped, short, declarative sentences with no long dashes.

## 4. Check before you send

- All five sections are there.
- Each answer from the user is kept, with its exact numbers.
- The prompt is under 200 words.
- It has no how-to.
- It names no model, CLI, or tool.

## 5. Output

Give one fenced code block with the complete prompt. Add nothing else unless the user asks. When the user pushes back, change the same prompt. Ask again only if the goal changes.

## 6. Handoff

If the user asks you to send the finished prompt to a teammate, send their exact instruction (project, folder, thread name) and the prompt word for word. The only allowed edit is to make a location line match where the prompt goes. Name that edit. Send once per ask, to the named bot. Send no follow-ups.

## Don't

- Inflate scope. Deeper beats bigger.
- Run the work unless the user asked you to do it.

Request (if empty, the one the user is asking about in this conversation): $ARGUMENTS
