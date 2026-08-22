#!/usr/bin/env python3
"""Extract a one-line summary of Claude's last turn from a transcript.

Usage: last-summary.py <transcript_path> [max_len]

Reads the JSONL transcript Claude Code writes per session, finds the last
assistant message that contains a text block, and condenses its first
meaningful line into a short single-line string suitable for a notification
body. Prints nothing (exit 0) when no summary can be derived, so the caller
can fall back to its own label.
"""
import json
import re
import sys


def last_assistant_text(path):
    """Return the text of the final assistant message that has a text block."""
    try:
        with open(path, "r", encoding="utf-8") as fh:
            lines = fh.readlines()
    except OSError:
        return ""
    for line in reversed(lines):
        line = line.strip()
        if not line:
            continue
        try:
            obj = json.loads(line)
        except json.JSONDecodeError:
            continue
        if obj.get("type") != "assistant":
            continue
        content = obj.get("message", {}).get("content", [])
        texts = [
            c.get("text", "")
            for c in content
            if isinstance(c, dict) and c.get("type") == "text"
        ]
        text = "\n".join(t for t in texts if t).strip()
        if text:
            return text
    return ""


def strip_markdown(text):
    """Flatten markdown to plain prose: drop emphasis, code ticks, links, headers."""
    text = re.sub(r"`{1,3}([^`]*)`{1,3}", r"\1", text)      # `code` / ```code```
    text = re.sub(r"\*\*([^*]+)\*\*", r"\1", text)           # **bold**
    text = re.sub(r"\*([^*]+)\*", r"\1", text)               # *italic*
    text = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", text)     # [text](url)
    text = re.sub(r"^#{1,6}\s*", "", text, flags=re.MULTILINE)  # # headers
    text = re.sub(r"^[-*]\s+", "", text, flags=re.MULTILINE)    # - bullets
    return text


def first_sentence(text, max_len):
    """First sentence or line of text, trimmed to max_len with an ellipsis."""
    # Take the first non-empty line; lead-in lines like "Done." are kept.
    for raw in text.splitlines():
        line = raw.strip()
        if line:
            break
    else:
        return ""
    # Prefer cutting at the first sentence boundary if it leaves something useful.
    m = re.search(r"(.+?[.!?])(\s|$)", line)
    summary = m.group(1) if m and len(m.group(1)) >= 15 else line
    summary = re.sub(r"\s+", " ", summary).strip()
    if len(summary) > max_len:
        summary = summary[: max_len - 1].rstrip() + "…"
    return summary


def main():
    if len(sys.argv) < 2:
        return
    path = sys.argv[1]
    max_len = int(sys.argv[2]) if len(sys.argv) > 2 else 100
    text = last_assistant_text(path)
    if not text:
        return
    summary = first_sentence(strip_markdown(text), max_len)
    if summary:
        print(summary)


if __name__ == "__main__":
    main()
