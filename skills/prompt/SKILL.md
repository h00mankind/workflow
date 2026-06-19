---
name: prompt
description: Generate prompts. The default mode `text` writes a clean, structured LLM/agent prompt from a task (or tidies up a rough one you paste in) — use it whenever the user asks for "a prompt" without naming a medium. Three media modes for specific jobs — `/prompt image` (MidJourney / ChatGPT Image / Nano Banana Pro), `/prompt video` (Jimeng Seedance 2.0), `/prompt audio` (ElevenLabs Music). Output is a self-contained HTML page in `docs/prompts/NNNN-<mode>-<slug>/NNNN-<mode>-<slug>.html` with one-click copy buttons per prompt. Triggers: "write a prompt", "make a prompt", "prompt for", "system prompt", "agent prompt", "structure this prompt", "image prompt", "video prompt", "audio prompt", "seedance", "midjourney", "nano banana", "elevenlabs", "/prompt".
---

# Prompt Generator — Text / Image / Video / Audio

You generate prompts. **`text` is the default** — when the user asks for "a prompt" without naming a medium, write a structured text/LLM prompt. Reach for a media mode only when the job is explicitly visual or audio: `/prompt image`, `/prompt video`, `/prompt audio`. State your mode pick in one line before proceeding; ask only if genuinely ambiguous.

User-specific style rules (tone, ethnicity, illustration default, etc.) belong in the user's own instructions or in `AGENTS.md`, not in this skill. The skill teaches the **tool-side prompting techniques** only — what each model responds to, what to drop, what to emphasize.

---

## OUTPUT FORMAT (always)

Every run produces one file: **`docs/prompts/NNNN-<mode>-<slug>/NNNN-<mode>-<slug>.html`**.

- `NNNN` — zero-padded 4-digit sequence number, auto-incremented. Run this before writing:

  ```bash
  ls docs/prompts 2>/dev/null \
    | grep -oE '^[0-9]{4}' \
    | sort -n | tail -1
  ```

  Take that number, add 1, pad to 4 digits. If nothing exists, start at `0001`. **Never** reuse or back-fill a number — always go to the next free slot.
- `<mode>` — literal `image`, `video`, or `audio`.
- `<slug>` — kebab-case, derived from the user's task. Ask if unclear.
- **The folder and the HTML file share the same name.** Self-describing in browser tabs, recent-files lists, and tools that just show the leaf filename. The folder also holds an `assets/` subfolder for source files when attachments are present.
- The HTML is self-contained: dark UI, one card per prompt, one-click **Copy prompt** button per card, "How to use" panel at the top.
- Use the template at the bottom of this file. Each prompt sits inside its own `<pre>` block; the copy script reads `innerText`.
- Put the raw prompt directly in `<pre>` — no markdown code fences inside, because the copy button puts plain text on the clipboard.
- The page's "Date" chip is the only place a date appears, and only when relevant. The folder name has no date in it — `git log` is the source of truth for when something was made.

---

## `/prompt text` — structured LLM / agent prompt (default)

The default. Turn a task — or a rough prompt the user pastes — into one clean, structured prompt ready to drop into an LLM or agent. If the user pasted a rough prompt, keep their intent and wording where it's good; restructure, don't rewrite from scratch.

**Structure (in this order, omit a part only when it truly doesn't apply):**

1. **Role / persona** — who the model should act as, when it sharpens the output (`"You are a senior copy editor."`). Skip for simple one-shot asks.
2. **Task** — the single clear instruction, leading verb first (`"Rewrite the text below to …"`).
3. **Context** — the material to work on and any background the model needs. Mark pasted-in material with a delimiter (`<text>…</text>`, triple backticks) so instruction and data never blur.
4. **Format** — exactly what the output should look like (JSON shape, headings, length, "answer with only the rewritten text").
5. **Constraints** — what to do and not do, positively framed (`"keep it under 60 words"`, `"use British spelling"`).

**Rules:**

- One prompt per run unless the user asks for variants. Write it as the finished prompt itself — second person to the target model, no meta-commentary.
- Positive instructions over negative where possible (`"write plainly"` beats `"don't be verbose"`).
- Put any example input/output *inside* the prompt as a labelled example when the format is non-obvious — one good example beats a paragraph describing the format.
- No model-specific tags or fluff. The prompt should be portable across chat models.
- Done when the prompt can be pasted into a fresh model with no other context and produce the intended output.

### Text prompt template

```
[Role — optional, one line]

[Task — one clear instruction, verb first.]

[Context / input — delimited if it's pasted material:]
<input>
…
</input>

Format: [exactly what the output should look like.]
Constraints: [bullets or one line — positively framed.]
```

### Text prompt example

Task given: *"prompt to summarize support tickets"*

```
You are a support-operations analyst.

Summarize the customer support ticket below for an engineer who has not seen it.

<ticket>
{paste ticket here}
</ticket>

Format: three short sections — **Problem** (one sentence), **Steps already tried**, **What the customer wants**.
Constraints: stay factual, quote error messages verbatim, keep the whole summary under 120 words.
```

---

## `/prompt image` — MidJourney / ChatGPT Image / Nano Banana Pro

**Tool flow:** MidJourney for initial generation; ChatGPT Image or Nano Banana Pro for editing or iterating on an existing image.

**Core structure:** `[Subject] + [Action] + [Location/context]` — tight, descriptive, and weighted on what the image actually shows.

### MidJourney prompting

- Natural descriptive phrasing wins over keyword stuffing. Describe a scene like you'd describe it to a person.
- Sentence form works for a single coherent shot; comma-separated phrase chunks layer in detail.
- Mention medium early when you want a specific look (`"editorial illustration of …"`, `"oil painting of …"`, `"35mm photograph of …"`).
- Reference a known style only when it genuinely fits — era, illustrator name, magazine genre, film.
- Positive framing only — describe what is there, never what isn't (`"empty street"`, not `"no cars"`).
- For consistent characters across a series, repeat a small set of identifying descriptors (outfit, hair, defining feature) in every prompt.
- Skip `--ar`, `--v`, `--style`, `--no` unless the user explicitly asks for them. The skill keeps prompts clean.

### ChatGPT Image / Nano Banana Pro editing

- Treat the prompt as an *instruction over the existing image*, not a fresh scene: `"change the woman's jacket to a deep red linen blazer, keep everything else identical"`.
- One change per prompt — sequential edits beat one mega-prompt.
- Explicitly anchor what to preserve (`"same character, same pose, same lighting"`) — these models hold scene state better with explicit anchoring.
- When the source has multiple subjects, name the one being edited (`"the woman on the left"`) so the edit lands where you want it.

### Image prompt template

```
[Subject — who they are, age/role/expression], [Action — what they're doing], [Location — where, with one or two grounding details][, optional medium/style]
```

### Image prompt examples

```
A young woman in a faded denim apron kneeling to feed a stray tabby cat outside a small bakery, warm morning light spilling onto the sidewalk
```

```
Three construction workers sharing lunch on a scaffold platform, laughing as one points at something in the sky, half-finished city skyline behind, editorial illustration
```

```
An elderly fisherman mending nets at the prow of a wooden boat, calm harbor water at dawn, distant silhouettes of other boats, oil painting style
```

---

## `/prompt video` — Jimeng Seedance 2.0 / Seedance 2.0 Fast

**Required:** Subject + Motion (WHO performs WHAT action). Keep Environment as the anchor.

**Drop unless asked:** aesthetics, camera moves, audio direction, transitions, on-screen text.

**Structure:** `[Subject] performing [Motion/Action] in [Environment/Location].`

**Reference syntax:** `Image 1, Image 2 … Image N` for image refs; `Video 1, Video 2 … Video N` for video refs. Each reference must be assigned a role in the prompt — first frame, character, scene, camera, action, audio, etc. — never just listed.

### Seedance 2.0 feature formulas (use only when the user explicitly asks)

| Feature | Formula |
|---|---|
| **Text / Slogan** | `[Text Content] + [Timing] + [Position] + [Appearance Method], [Style]` |
| **Subtitles** | `Subtitles appear at the bottom of the screen, synchronized with the audio.` |
| **Speech bubbles** | `[Character] says: "…", speech bubbles appear around the character.` |
| **Image reference** | `Reference / Extract / Combine + [Image N]'s [Subject], generate [Scene], maintaining consistent [Subject] features.` |
| **Video reference (action / camera / effects)** | `Reference [Video N]'s [Description], generate [Scene], maintaining consistent details.` |
| **Editing — Add / Remove / Modify** | `Remove [Element] from [Video N], keep everything else unchanged.` |
| **Extension** | `Extend [Video N] forward / backward + [Description of extended content].` |
| **Track completion** | Up to 3 videos, total ≤ 15 seconds. |

Use common characters in any on-screen text; avoid rare characters or special symbols.

### Seedance constraints

- ≤ 9 images, ≤ 3 videos, ≤ 3 audios, ≤ 12 total files per generation.
- Image formats: jpeg, png, webp, bmp, tiff, gif. Max 30 MB each.
- Video formats: mp4, mov. Max 50 MB each, 2–15s total reference duration.
- Audio formats: mp3, wav. Max 15 MB each, ≤ 15s total reference duration.
- Output duration: 4–15s, user-selectable.
- Resolution: 480p (640×640) to 720p (834×1112).
- **No realistic human faces** in uploaded images/videos — the platform blocks them. Illustrated characters are fine.

### Video prompt template

```
[Subject — who they are] performing [Motion/Action], in [Environment/Location].
```

### Video prompt example

```
A young baker, sleeves rolled up, kneading dough on a flour-dusted wooden counter, in a small early-morning bakery with steam rising from a kettle behind her.
```

### Speech-bubble pattern (e.g. heart-bubble illustrations)

When a speech bubble is the visual hero, give it three distinct beats:

1. **Bubble enters** — fade in, settle, soft pulse.
2. **Contents come alive** — hearts/icons inside glow in sequence; one or two release outward.
3. **Final beat** — remaining contents pulse together once, released ones dissolve into light specks.

Keep the camera static. Only the bubble, its contents, and micro-gestures (blinks, hair drift, blush) should move. This protects the painted look of the source illustration.

---

## `/prompt audio` — ElevenLabs Music (Eleven Music)

**Required focus:** Intent + Genre/Mood — what the track is for, and how it should feel.

**Style:** plain-language, flowing description. Short and evocative often beats long and detailed — the model interprets freely. Combine abstract mood descriptors with detailed musical language when you want tighter control.

**Drop unless asked:** BPM, key, length, timing cues. When the user does want them, fold them into the prose, don't list as tags.

### Three prompt structures

| Structure | Shape |
|---|---|
| **Intent-led** | `[Use case / intent] + [genre & mood] + [optional musical detail]` |
| **Voiceover / ad** | `[Track purpose] + [tone] + "Voiceover only" + [script in quotes] + [brand mention if needed]` |
| **Detailed musical** | `[genre fusion] + [feel / performance quality] + [optional key / BPM] + [optional vocal entry timing + lyrics]` |

### Optional controls (use only when explicitly requested)

- **Solo instrument:** prefix `solo` — `"solo electric guitar"`, `"solo piano in C minor"`.
- **A cappella vocals:** prefix `a cappella` — `"a cappella female vocals"`, `"a cappella male chorus"`.
- **Tempo / key:** `"130 BPM"`, `"in A minor"` — fold into the sentence.
- **Vocal delivery:** `"raw"`, `"live"`, `"glitching"`, `"breathy"`, `"aggressive"`.
- **Multiple vocalists:** `"two singers harmonizing in C"`.
- **Instrumental only:** add `"instrumental only"` — most prompts include lyrics by default.
- **Length:** `"60 seconds"`, or rely on auto.
- **Lyric timing:** `"lyrics begin at 15 seconds"`, `"instrumental only after 1:45"`.
- **Custom lyrics:** provide them; the model places vocals based on prompt length.
- **Multilingual:** specify language, or follow up with `"make it Japanese"` / `"translate to Spanish"`.

For section structure, lyric placement, and multi-vocalist arrangements, use a **composition plan** (sections with global/local styles and formatted lyrics) instead of a simple text prompt — only when explicitly requested. Composition plans lock to one style.

### Audio prompt template

```
[Intent / use case] — [genre and mood][, optional musical detail][, optional vocal direction].
```

### Audio prompt examples

```
Background music for a small bakery's social ad — warm acoustic indie folk with light brushed drums, hopeful and unhurried, soft female humming entering halfway through.
```

```
Score for a working-class hero montage — uplifting modern orchestral with rising strings and a steady piano pulse, a single quiet female vocal entering on the final third with the line "the morning still belongs to us".
```

```
Voiceover ad for a Japanese tea brand — calm minimal piano underscore, soft and grounded. Voiceover only: "A small cup. A long morning. Still time." Brand stinger at the end.
```

---

## WORKFLOW (every run)

1. **Pick mode.** Default to `text`. Use a media mode only when the job is explicitly visual or audio. State the pick in one line; ask only if genuinely ambiguous.
2. **Clarify the task / intent.** For `text`: the goal, the target model's job, any pasted material. For media: subject, action, location (visual) or intent + genre/mood (audio); list any references and map each to a role.
3. **Draft prompts.** One per task/scene/track. Keep tight — no padding, no negation, no metadata unless asked.
4. **Pick the folder.** Auto-increment the next `NNNN` (see Output Format above), then write to `docs/prompts/NNNN-<mode>-<slug>/NNNN-<mode>-<slug>.html`.
5. **Render the HTML** using the template below. Each prompt sits inside its own `<pre id="prompt-N">` block. The copy button reads `innerText`.
6. **Report** one line: file path, prompt count, mode.

---

## ATTACHMENT HANDLING

When the user provides source files (reference images, source videos, audio clips):

1. **Copy them into the output folder** under `assets/` — `docs/prompts/NNNN-<mode>-<slug>/assets/<original-filename>`. Use `cp`, never move the originals.
2. **Reference them with relative paths** in the HTML (`assets/JD_June5_001.jpg`) so the page works when opened directly with `file://`.
3. **Render a preview** beside the prompt — `<img>` for images, `<video controls>` for video, `<audio controls>` for audio.
4. **Use the two-column card** template variant (preview left, prompt + copy button right). On narrow viewports the columns stack.

When there's no attachment, use the simple single-column card (preview block omitted entirely).

---

## HTML TEMPLATE — editorial design (use verbatim)

Single file at `docs/prompts/NNNN-<mode>-<slug>/NNNN-<mode>-<slug>.html`.

**Design system (do not deviate):**

- **Aesthetic:** print/editorial. Warm paper background, generous margins, asymmetric hierarchy, serif display headings, sans body, mono prompts. Reads like a designer's worksheet, not a dashboard.
- **Type:** Fraunces (display) + Inter (body) + JetBrains Mono (prompt code) — all from Google Fonts.
- **Layout:** centered column, max width 760px on page, generous 96px top padding, 120px bottom. Each prompt card is two-column on desktop (left: 200px attachment thumb sticky, right: prompt + actions); single column under 720px. No card backgrounds — just hairline rules and whitespace.
- **Per-mode accent (set on `:root` via `data-mode`):**
  - `text` → `#9a6b1f` (ochre)
  - `image` → `#d65a3e` (warm coral)
  - `video` → `#3a4cd6` (electric indigo)
  - `audio` → `#2d8c5b` (acid green)
- **Set `<html data-mode="text|image|video|audio">`** so the accent CSS variable resolves correctly.
- **No emojis. No gradients. No drop shadows except on the copy button.** The discipline is the point.

```html
<!doctype html>
<html lang="en" data-mode="{text | image | video | audio}">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width,initial-scale=1" />
<title>{NNNN} · {TITLE}</title>
<link rel="preconnect" href="https://fonts.googleapis.com" />
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Fraunces:opsz,wght@9..144,400;9..144,500;9..144,600&family=Inter:wght@400;500;600&family=JetBrains+Mono:wght@400;500&display=swap" />
<style>
  :root {
    --paper: #f4ecde;
    --paper-2: #efe6d4;
    --ink: #1a1410;
    --ink-2: #3a302a;
    --muted: #80766a;
    --muted-2: #b6ac9c;
    --line: #d8cdb8;
    --line-strong: #a89c83;
    --accent: #1a1410;
    --ok: #2d8c5b;
  }
  html[data-mode="text"]  { --accent: #9a6b1f; }
  html[data-mode="image"] { --accent: #d65a3e; }
  html[data-mode="video"] { --accent: #3a4cd6; }
  html[data-mode="audio"] { --accent: #2d8c5b; }

  * { box-sizing: border-box; }
  html, body { margin: 0; padding: 0; background: var(--paper); color: var(--ink); }
  body {
    font-family: "Inter", ui-sans-serif, system-ui, sans-serif;
    line-height: 1.6;
    font-size: 15px;
    -webkit-font-smoothing: antialiased;
    text-rendering: optimizeLegibility;
    background:
      repeating-linear-gradient(0deg, transparent, transparent 31px, rgba(168,156,131,.06) 31px, rgba(168,156,131,.06) 32px),
      var(--paper);
    min-height: 100vh;
  }
  main { max-width: 760px; margin: 0 auto; padding: 96px 32px 120px; }

  /* page header — print masthead */
  header.page { margin-bottom: 64px; }
  header.page .topline {
    display: flex; justify-content: space-between; align-items: baseline;
    font-family: "JetBrains Mono", ui-monospace, monospace;
    font-size: 11px; letter-spacing: .14em; text-transform: uppercase;
    color: var(--muted); border-bottom: 1px solid var(--line-strong); padding-bottom: 10px;
    margin-bottom: 40px;
  }
  header.page .topline .no { color: var(--accent); font-weight: 500; }
  header.page h1 {
    font-family: "Fraunces", "Times New Roman", serif;
    font-optical-sizing: auto;
    font-weight: 500;
    font-size: clamp(48px, 8vw, 84px);
    line-height: .98;
    letter-spacing: -0.025em;
    margin: 0 0 24px;
    color: var(--ink);
  }
  header.page h1 .period { color: var(--accent); }
  header.page .sub {
    font-family: "Fraunces", serif; font-style: italic;
    font-size: 19px; line-height: 1.5;
    color: var(--ink-2); max-width: 52ch; margin: 0 0 32px;
    font-weight: 400;
  }
  header.page .meta {
    display: grid; grid-template-columns: repeat(2, minmax(0,1fr));
    gap: 4px 28px;
    font-family: "JetBrains Mono", ui-monospace, monospace;
    font-size: 11.5px; letter-spacing: .04em;
    color: var(--ink-2);
    border-top: 1px solid var(--line); padding-top: 18px;
    max-width: 520px;
  }
  header.page .meta .row { display: flex; justify-content: space-between; gap: 12px; padding: 3px 0; }
  header.page .meta .row b { color: var(--muted); font-weight: 400; text-transform: uppercase; letter-spacing: .12em; }
  header.page .meta .row span { color: var(--ink); text-align: right; }

  /* how to use — folded note */
  details.howto {
    margin: 48px 0 0;
    border-top: 1px solid var(--line);
    padding: 14px 0 0;
  }
  details.howto summary {
    cursor: pointer;
    font-family: "JetBrains Mono", ui-monospace, monospace;
    font-size: 11.5px; letter-spacing: .14em; text-transform: uppercase;
    color: var(--muted);
    list-style: none;
    display: flex; align-items: center; gap: 10px;
    transition: color .15s;
  }
  details.howto summary:hover { color: var(--ink); }
  details.howto summary::-webkit-details-marker { display: none; }
  details.howto summary::before {
    content: "+"; color: var(--accent); font-weight: 500;
    width: 12px; display: inline-block; text-align: center;
    transition: transform .2s ease;
  }
  details.howto[open] summary::before { content: "−"; }
  details.howto ol { margin: 16px 0 0; padding-left: 24px; color: var(--ink-2); font-size: 14px; line-height: 1.7; }
  details.howto li { margin: 4px 0; }
  details.howto code {
    background: var(--paper-2); padding: 1px 6px; border-radius: 3px;
    font-family: "JetBrains Mono", ui-monospace, monospace;
    font-size: 12px; color: var(--ink);
    border: 1px solid var(--line);
  }
  details.howto .path { color: var(--muted); font-size: 12.5px; margin-top: 14px; font-family: "JetBrains Mono", ui-monospace, monospace; }

  /* section rule between header and prompts */
  .prompts-start {
    display: flex; align-items: center; gap: 16px;
    margin: 64px 0 40px;
    font-family: "JetBrains Mono", ui-monospace, monospace;
    font-size: 11px; letter-spacing: .18em; text-transform: uppercase;
    color: var(--muted);
  }
  .prompts-start::before, .prompts-start::after {
    content: ""; flex: 1; height: 1px; background: var(--line-strong);
  }

  /* prompt — no card, just typography and rules */
  article.prompt {
    margin: 0 0 72px;
    padding-top: 32px;
    border-top: 1px solid var(--line);
    position: relative;
  }
  article.prompt:first-of-type { border-top: 0; padding-top: 0; }

  article.prompt .card-head {
    display: flex; flex-direction: column; gap: 6px;
    margin-bottom: 20px;
  }
  article.prompt .card-head .num {
    font-family: "JetBrains Mono", ui-monospace, monospace;
    font-size: 11px; letter-spacing: .14em; text-transform: uppercase;
    color: var(--accent); font-weight: 500;
  }
  article.prompt .card-head h2 {
    margin: 0;
    font-family: "Fraunces", serif; font-weight: 500;
    font-size: 28px; line-height: 1.2;
    letter-spacing: -0.012em;
    color: var(--ink);
  }
  article.prompt .card-head .desc {
    font-family: "Fraunces", serif; font-style: italic;
    color: var(--muted); font-size: 15px; line-height: 1.5;
    margin: 6px 0 0; max-width: 56ch;
  }

  /* body: two-column when attachment present, otherwise single column */
  article.prompt .card-body { display: block; }
  article.prompt.has-attachment .card-body {
    display: grid; grid-template-columns: 200px 1fr;
    gap: 28px; align-items: start;
  }
  @media (max-width: 720px) {
    article.prompt.has-attachment .card-body { grid-template-columns: 1fr; gap: 20px; }
  }

  .attachment {
    position: sticky; top: 24px;
  }
  .attachment img,
  .attachment video {
    display: block; width: 100%; height: auto;
    border: 1px solid var(--line-strong);
    background: var(--paper-2);
  }
  .attachment audio { display: block; width: 100%; }
  .attachment .att-meta {
    display: flex; justify-content: space-between; align-items: center;
    margin-top: 8px;
    font-family: "JetBrains Mono", ui-monospace, monospace;
    font-size: 10.5px; letter-spacing: .06em;
    color: var(--muted);
  }
  .attachment .att-meta a {
    color: var(--accent); text-decoration: none;
    border-bottom: 1px solid var(--accent);
    padding-bottom: 1px;
    transition: opacity .15s;
  }
  .attachment .att-meta a:hover { opacity: .7; }

  .prompt-col { display: flex; flex-direction: column; gap: 16px; min-width: 0; }

  pre.body {
    background: var(--paper-2);
    border: 1px solid var(--line);
    border-left: 3px solid var(--accent);
    border-radius: 0;
    padding: 18px 20px;
    margin: 0;
    overflow-x: auto;
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
    font-size: 12.5px; line-height: 1.7;
    color: var(--ink-2);
    white-space: pre-wrap; word-break: normal;
    max-height: 520px; overflow-y: auto;
  }
  pre.body::-webkit-scrollbar { width: 8px; height: 8px; }
  pre.body::-webkit-scrollbar-thumb { background: var(--line-strong); border-radius: 0; }
  pre.body::-webkit-scrollbar-track { background: transparent; }

  .actions { display: flex; gap: 14px; flex-wrap: wrap; align-items: center; justify-content: flex-end; }
  button.copy {
    background: var(--ink);
    color: var(--paper);
    border: 1px solid var(--ink);
    border-radius: 0;
    font-family: "JetBrains Mono", ui-monospace, monospace;
    font-size: 11px; font-weight: 500;
    letter-spacing: .14em; text-transform: uppercase;
    padding: 10px 18px;
    cursor: pointer;
    display: inline-flex; align-items: center; gap: 8px;
    box-shadow: 3px 3px 0 0 var(--accent);
    transition: transform .12s ease, box-shadow .12s ease, background .15s, color .15s;
  }
  button.copy:hover { transform: translate(-1px, -1px); box-shadow: 4px 4px 0 0 var(--accent); }
  button.copy:active { transform: translate(2px, 2px); box-shadow: 1px 1px 0 0 var(--accent); }
  button.copy.copied { background: var(--ok); border-color: var(--ok); box-shadow: 3px 3px 0 0 var(--ink); }
  button.copy svg { width: 12px; height: 12px; }
  .word-count {
    color: var(--muted);
    font-family: "JetBrains Mono", ui-monospace, monospace;
    font-size: 10.5px; letter-spacing: .08em;
  }

  footer.page {
    margin-top: 96px; padding-top: 18px;
    border-top: 1px solid var(--line-strong);
    color: var(--muted);
    font-family: "JetBrains Mono", ui-monospace, monospace;
    font-size: 10.5px; letter-spacing: .14em; text-transform: uppercase;
    display: flex; justify-content: space-between; gap: 16px; flex-wrap: wrap;
  }
  footer.page code {
    font-family: inherit; background: none; padding: 0;
    color: var(--ink);
  }

  /* selection */
  ::selection { background: var(--accent); color: var(--paper); }
</style>
</head>
<body>
<main>

  <header class="page">
    <div class="topline">
      <span>{NNNN} · {MODE} · {SLUG}</span>
      <span class="no">№{NNNN}</span>
    </div>
    <h1>{TITLE}<span class="period">.</span></h1>
    <p class="sub">{ONE-LINE SUBTITLE — what this batch is for, written like a magazine standfirst}</p>
    <div class="meta">
      <div class="row"><b>No.</b> <span>{NNNN}</span></div>
      <div class="row"><b>Mode</b> <span>{text | image | video | audio}</span></div>
      <div class="row"><b>Tool</b> <span>{LLM / agent · MidJourney / Seedance 2.0 / ElevenLabs Music}</span></div>
      <div class="row"><b>Count</b> <span>{N} prompts</span></div>
    </div>
  </header>

  <details class="howto">
    <summary>How to use</summary>
    <ol>
      <li>Open the target tool ({your LLM / agent · or Seedance / MidJourney / ElevenLabs Music}).</li>
      <li>Upload the source file shown beside each prompt (if any).</li>
      <li>Click <b>Copy prompt</b>, paste, generate.</li>
    </ol>
    <div class="path">docs/prompts/{NNNN}-{mode}-{slug}/</div>
  </details>

  <div class="prompts-start"><span>The prompts</span></div>

  <!--
    === CARD VARIANTS ===
    With attachment:    <article class="prompt has-attachment">
    Without attachment: <article class="prompt">
  -->

  <!-- WITH attachment -->
  <article class="prompt has-attachment">
    <div class="card-head">
      <span class="num">№ 01</span>
      <h2>{SHORT LABEL}</h2>
      <p class="desc">{ONE-LINE CONTEXT}</p>
    </div>
    <div class="card-body">
      <div class="attachment">
        <img src="assets/{FILENAME}.jpg" alt="Source for prompt 01" loading="lazy" />
        <!-- <video src="assets/{FILENAME}.mp4" controls preload="metadata"></video> -->
        <!-- <audio src="assets/{FILENAME}.mp3" controls preload="metadata"></audio> -->
        <div class="att-meta">
          <span>{FILENAME}.jpg</span>
          <a href="assets/{FILENAME}.jpg" download>Download</a>
        </div>
      </div>
      <div class="prompt-col">
        <pre class="body" id="prompt-1">{RAW PROMPT TEXT — no markdown fences}</pre>
        <div class="actions">
          <span class="word-count" data-source="prompt-1"></span>
          <button class="copy" data-target="prompt-1">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
            Copy prompt
          </button>
        </div>
      </div>
    </div>
  </article>

  <!-- WITHOUT attachment -->
  <article class="prompt">
    <div class="card-head">
      <span class="num">№ 02</span>
      <h2>{SHORT LABEL}</h2>
      <p class="desc">{ONE-LINE CONTEXT}</p>
    </div>
    <div class="card-body">
      <div class="prompt-col">
        <pre class="body" id="prompt-2">{RAW PROMPT TEXT}</pre>
        <div class="actions">
          <span class="word-count" data-source="prompt-2"></span>
          <button class="copy" data-target="prompt-2">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
            Copy prompt
          </button>
        </div>
      </div>
    </div>
  </article>

  <footer class="page">
    <span>Built with the <code>prompt</code> skill</span>
    <span>{N} · {MODE} · №{NNNN}</span>
  </footer>

</main>

<script>
  document.querySelectorAll('button.copy').forEach(btn => {
    btn.addEventListener('click', async () => {
      const target = document.getElementById(btn.dataset.target);
      if (!target) return;
      const text = target.innerText.trim();
      try {
        await navigator.clipboard.writeText(text);
      } catch (e) {
        const range = document.createRange();
        range.selectNode(target);
        window.getSelection().removeAllRanges();
        window.getSelection().addRange(range);
        document.execCommand('copy');
        window.getSelection().removeAllRanges();
      }
      const original = btn.innerHTML;
      btn.classList.add('copied');
      btn.innerHTML = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg> Copied';
      setTimeout(() => { btn.classList.remove('copied'); btn.innerHTML = original; }, 1600);
    });
  });
  document.querySelectorAll('.word-count').forEach(el => {
    const src = document.getElementById(el.dataset.source);
    if (!src) return;
    const words = src.innerText.trim().split(/\s+/).filter(Boolean).length;
    el.textContent = `${words} words`;
  });
</script>

</body>
</html>
```

---

## COMMON MISTAKES TO AVOID

- Writing a video prompt that's a list of camera moves with no clear subject and action.
- Putting `--ar`, `--v`, `--style`, `--no` into image prompts unless the user asked.
- Negating instead of describing (`"no cars"` vs `"empty street"`).
- Padding audio prompts with BPM/key/length when the user didn't ask.
- Writing prompts inside markdown code fences in the HTML — put the raw prompt in `<pre>` so the copy button reads it as plain text.
- Wrong folder format. Always `docs/prompts/NNNN-<mode>-<slug>/NNNN-<mode>-<slug>.html` with auto-incremented `NNNN`.
- Listing references in the video prompt without assigning each a role (first frame, character, scene, camera, action, audio).
- For `text`: blurring instruction and pasted material — always delimit the input. And writing *about* the prompt instead of writing the prompt itself.
