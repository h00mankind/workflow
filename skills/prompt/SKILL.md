---
name: prompt
description: Generate paste-ready prompts in quick mode or save structured Markdown prompt records. Use when the user asks to write, make, structure, or improve a prompt, including text, image, video, audio, system, agent, MidJourney, Seedance, Nano Banana, or ElevenLabs prompts. Use the html skill for visual or interactive artifacts.
---

# Prompt Generator

**Tracer bullet:** produce one clear, paste-ready prompt, then choose the lightest useful output.

## Choose output mode

Use the request to choose the mode:

- **Quick:** one-off task or no request to save. Return the finished prompt in a fenced `text` block so the user can copy it. Do not create files.
- **Structured:** reusable prompt, prompt record, or explicit request to save. Create `docs/prompts/NNNN-<content-mode>-<slug>/prompt.md`.
- **HTML artifact:** do not create it here. Use the `html` skill when the user asks for a visual, interactive, or browser-ready artifact.

If the request does not show which mode is useful, ask:

> Quick copyable prompt, or structured Markdown prompt folder?

## Content modes

- **`text` (default):** Role when useful; Task; delimited Context; exact Format; positive Constraints. Write the prompt itself, not commentary about it.
- **`image`:** Subject + Action + Location/context. Use natural description. For edits, name what changes and what stays the same; make one change per prompt.
- **`video`:** Subject performing Motion in Environment. Give every image or video reference a role. Add camera, style, audio, text, or transitions only when requested.
- **`audio`:** Intent/use case + genre/mood. Add BPM, key, length, timing, or vocal controls only when requested.

Keep prompts portable, specific, and free of padding. Use one prompt unless the user asks for variants.

## Workflow

1. Choose `quick` or `structured` from the request; ask only when the choice is unclear. Then choose the content mode. `text` is the default unless the request names a medium.
2. Clarify only information required to produce the prompt. Preserve good wording from any rough prompt and delimit pasted material.
3. Draft the prompt. Prefer positive instructions; omit model-specific flags and metadata unless requested.
4. Complete the chosen output:
   - **Quick:** return only the finished prompt in a copyable `text` block.
   - **Structured:** find the next unused four-digit sequence, create the folder, write `prompt.md`, and add `assets/` only when source files need to be preserved.
5. Report one line. In structured mode, include the saved file path.

## Structured Markdown format

Use one `prompt.md` per prompt record:

````markdown
---
title: <short title>
mode: <text|image|video|audio>
created: <YYYY-MM-DD>
---

# Prompt

```text
<finished prompt>
```

## How to use

<short target-model or workflow note>
````

Use a longer outer fence when the prompt itself contains triple backticks. Keep notes short and factual. Put copied source files in `assets/` and reference them with relative paths.

## Sequence and naming

- Inspect `docs/prompts/` for the highest existing four-digit folder prefix. Start at `0001` when none exists.
- Never reuse or back-fill a number.
- Use `NNNN-<content-mode>-<slug>` for the folder. Keep the file name `prompt.md`.

## Guardrails

- Text: put pasted material inside a delimiter such as `<input>…</input>`.
- Image: describe what is present; do not add `--ar`, `--v`, `--style`, or `--no` unless asked.
- Video: assign a role to every reference instead of listing filenames only.
- Audio: do not pad a short musical brief with technical controls.
- Do not add HTML, CSS, JavaScript, or an HTML artifact from this skill.

## Example

Task: “prompt to summarize support tickets” — use quick mode for this one-off:

```text
You are a support-operations analyst.

Summarize the support ticket below for an engineer who has not seen it.

<ticket>
{paste ticket here}
</ticket>

Format: Problem, steps already tried, and what the customer wants.
Constraints: stay factual, quote error messages exactly, and keep the summary under 120 words.
```

Done means the quick prompt is self-contained, or the structured `prompt.md` exists at the reported path with valid frontmatter and the finished prompt.
