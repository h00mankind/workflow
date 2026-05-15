---
name: chat-handoff
description: Compact the current chat conversation into a portable handoff document — a markdown context block the user can paste into another chat session (same model or a different one like ChatGPT, Gemini, etc.) so the next session picks up where this one left off. Use this whenever the user asks to "hand off this chat", "create a context file", "summarize this conversation for another AI / model / chat", "export this chat", "save context for next time", "continue this in another tool", or anything indicating they want to transfer the working state of this conversation to a fresh session. Designed for chat interfaces, not CLI environments.
---

# Chat Handoff

Write a handoff document so a fresh chat session — any model — can continue the work without reading the full transcript. Output it inline as a single fenced markdown block the user can copy. Nothing else: no preamble, no summary, no "let me know if you want changes" after.

Use these sections, in order: **Task** (the overall goal, 1–3 sentences, usually clearest in the user's early messages) · **Current state** (what's done, in progress, not started) · **Key decisions** (choices locked in — don't let the next session re-litigate them) · **Constraints & preferences** (what the user explicitly wants or doesn't want; pushback moments are gold) · **Artifacts** (inline code/snippets if short; shape and interface if long) · **Open questions** · **Next steps** (actionable, only what the user has endorsed) · **Background** (only what's load-bearing for continuing the work).

Write in third person ("the user", "the previous session") in plain GitHub-flavored markdown — no Claude-specific framing, no UI-specific tags. Bias hard toward compaction: cut pleasantries, abandoned approaches, ambient user-profile facts that didn't shape the work, and anything the next model would already know. If the user passes a target ("handoff for code review", "I'm taking this to GPT-5"), emphasize the relevant section and trim the rest.
