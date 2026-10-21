---
name: prompt
description: Write or improve text, image, video, or audio prompts with routing for new generation, editing, and model-specific rules. Use when writing prompts, prompt routing, image prompt, image edit, video prompt, audio prompt, or targeting ChatGPT 2.5, Midjourney, Nano Banana, or Seedance; use html for visual artifacts.
---

# # prompt

**Router-First:** determine the output modality and edit intent before writing the prompt. Route image generation and edits to ChatGPT 2.5 by default unless the user names another model. For deep provider settings and API schemas, load `reference.md`.

## Routing decision tree

1. **Classify modality:**
   - `text` (default): Task, delimited Context, exact Format, positive Constraints.
   - `image`: branch to **New Creation** or **Image Edit**.
   - `video`: branch to **Text-to-Video** or **Image-to-Video**.
   - `audio`: Intent/mood, speech (TTS) or music/SFX.
2. **Classify image intent and target:**
   - **Default model:** ChatGPT 2.5 (`gpt-image-2.5-flare` for fast/volume; `gpt-image-2.5-sunburst` for precision/high quality).
   - **New Creation:** Subject + Action + Environment + Lighting + Framing. Use natural sentences; put text in quotes; set dimensions or `background="transparent"` when needed.
   - **Image Edit:** Name the exact change and what stays unchanged (`Change only X. Preserve Y and Z`). Make one edit per turn. Assign roles to inputs (`Image 1` for identity, `Image 2` for clothing).
   - **Provider overrides:**
     - *Midjourney:* `--v 8.2` only, comma-separated descriptors, `--ar`, `--s`, `--cref`, `--sref`.
     - *Google:* Nano Banana Pro or Nano Banana 2 models. 5-part layered description, `aspectRatio`, exact quotes.
     - *Grok Imagine:* director's brief `[Subject] + [Setting] + [Style] + [Lighting]`.
3. **Classify video intent:**
   - Front-load camera movement in the first 20 words (dolly, tracking, static tripod).
   - Assign roles to references (`@Image1` as start frame, `@Image2` as end frame).

## Choose output mode

- **Quick (default):** return the finished prompt in a copyable `text` block with model/parameter notes below. Do not create files.
- **Structured:** create `docs/prompts/NNNN-<mode>-<slug>/prompt.md`. Inspect `docs/prompts/` for the highest existing four-digit prefix, increment to the next unused sequence (never reuse), and keep source media in `assets/` only when original files must be preserved.

## Structured Markdown format

````markdown
---
title: <short title>
mode: <text|image|video|audio>
target: <chatgpt-2.5|midjourney|nano-banana|grok|seedance|general>
intent: <create|edit|tts|music|general>
created: <YYYY-MM-DD>
---

# Prompt

```text
<finished prompt>
```

## Settings and Usage

- Model: <target model>
- Parameters: <size, quality, aspect ratio, or provider flags>
- Workflow: <generation instructions or edit reference roles>
````

## Guardrails

- Image edit prompts must explicitly separate changes from preserved regions.
- Do not append Midjourney flags (`--ar`, `--v`) when prompting ChatGPT 2.5 or Nano Banana.
- Video prompts must start with camera motion and physical action before stylistic details.
- For transparent image assets, request transparent background and PNG or WebP output.
- Do not generate HTML artifacts here; use the `html` skill for interactive files.

## Example

Task: "edit image to change clothing on the subject":

```text
Edit the image to dress the woman using the provided clothing images.
Do not change her face, facial features, skin tone, body shape, pose, or identity in any way.
Preserve her exact likeness, expression, hairstyle, and proportions.
Replace only the clothing with the beige jacket and white top from the reference images, fitting the garments naturally to her pose.
Match the original lighting, shadows, and color temperature.
Do not change the background, camera angle, or framing.
```

Done means the prompt adheres to the routed target conventions, separates edits from invariants, and is returned as a copyable block or saved at the structured path.
