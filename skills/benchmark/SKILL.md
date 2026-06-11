---
name: benchmark
description: Benchmark models head-to-head or eval skills (with-skill vs bare baseline) on a calibrated mid-weight task — parallel sub-agents do the work, a blind judge scores it, and an HTML report lands in benchmarks/ showing the result first, stats after. Use when the user wants to benchmark, eval, compare, or A/B test models or skills, asks "which model is better at X" or "does this skill actually help", or says /benchmark, /benchmark model, /benchmark skill. Quick mode is the default (one task, minimal questions); "deep" runs more tasks and contenders.
---

# benchmark

A benchmark is only as fair as its blindest part: identical prompts, isolated runs, a judge who doesn't know whose work it's scoring, and stats you actually measured.

## 1. Setup — quick by default

Parse the argument for a target (`model` or `skill`) and a mode (`deep`; quick otherwise). Ask only what's missing:

- No target → one question: benchmark **models** or **skills**?
- **Model · quick** — contenders are whatever the user named, else the session's model plus one sensible rival; one task. **Model · deep** — ask how many and which (sub-agent overrides available: fable, opus, sonnet, haiku); 2–3 tasks.
- **Skill · quick** — one skill, one task, two conditions: *with-skill* vs *bare* on the same model. **Skill · deep** — several skills, or two skills pitted against each other on shared ground.

In quick mode never ask more than one question total — pick a task yourself, state it, and let the user veto.

## 2. Pick a calibrated task

Right-sized means a sub-agent finishes in roughly 2–5 minutes and a few thousand output tokens. Not fizzbuzz (everyone aces it, nothing separates contenders); not "build an app" (slow, expensive, judging turns mushy). Suggest 2–3 matched to the target:

- **UI** — a pricing card, dashboard stat widget, or signup form as a single self-contained HTML file
- **Copy** — landing hero plus three feature blurbs for a stated product
- **Code** — a tricky pure function with edge cases, plus its tests
- **Reasoning** — debug a short broken snippet and explain the root cause

For a skill eval, the task must sit in that skill's wheelhouse — eval `motion` on an animation brief, not a SQL query.

## 3. Run — parallel sub-agents, hermetically

Create `benchmarks/NNNN-<slug>/` — `NNNN` is the next zero-padded number after the highest existing run (`0007-opus-vs-sonnet-ui/`); create `benchmarks/` itself if missing (it's gitignored — runs stay local). Inside the run folder: task prompts in `prompts/`, any helper scripts in `scripts/`, contender artifacts and the report at the root. Nothing lands loose in `benchmarks/` itself.

One sub-agent per cell (model × task, or condition × task), all launched in a single message so they run concurrently:

- **Identical prompt** for every contender in a comparison — the only permitted difference is the model override or the prepended skill content (inline the full SKILL.md into the prompt; sub-agents don't inherit skills).
- Each agent writes to its own file in the run folder (`a.html`, `b.html`, …) and is never told it's being benchmarked or who it's up against.
- Record per run into `stats.json`: exact model ID, effort level, condition, wallclock, output tokens, input + cache-read tokens, turns, cost, artifact size, and the run date — **as the harness reports them**. Anything not reported is `n/a` — never estimate silently.

## 4. Judge blind

A fresh sub-agent gets the outputs relabeled A/B/C in shuffled order, the original task prompt, and 3–4 task-appropriate criteria. It scores each, picks a winner, and explains why — without knowing which model or condition produced what. Reveal the mapping only after the verdict is in.

## 5. Report — result first, always

Every run ends with `<slug>-report.html` in the run folder — no exceptions, quick mode included (the slug in the filename keeps browser tabs tellable apart across runs). In this order:

1. **The work itself** — side-by-side iframes for UI, rendered prose for copy, highlighted code for code. The reader sees what was made before any numbers.
2. **Stats table** — per contender: exact model ID, effort level, condition, wallclock, output tokens, input + cache tokens, turns, cost, artifact size, judge scores per criterion, winner badge. Run date and harness go in the header.
3. **Write-up** — the judge's verdict (now de-anonymized), your own observations, and an honesty note on sample size: a quick run is one task, one shot — indicative, not conclusive.

Open the report (`open <slug>-report.html`) and end by offering the natural next step: a deep run, a rematch on a different task, or a new contender.
