#!/usr/bin/env python3
"""UserPromptSubmit hook: stop one prompt when it would resend a large, cold context.

The prompt cache lasts one hour. After that, the next prompt writes the full
context again at the cache-write rate. This hook blocks that prompt once and
prints the cost. Send the prompt again to continue in the same thread.

Tune with TOKEN_GUARD_MIN_TOKENS (default 150000) and TOKEN_GUARD_IDLE_MIN (default 60).
"""
import datetime as dt
import json
import os
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit import rates_for  # noqa: E402

MIN_TOKENS = int(os.environ.get("TOKEN_GUARD_MIN_TOKENS", 150_000))
IDLE_MIN = float(os.environ.get("TOKEN_GUARD_IDLE_MIN", 60))
TAIL_BYTES = 4_000_000


def last_request(path):
    """Return (context tokens, timestamp, model, message id) of the last main-thread request."""
    with open(path, "rb") as f:
        f.seek(0, os.SEEK_END)
        f.seek(max(0, f.tell() - TAIL_BYTES))
        lines = f.read().decode("utf-8", "replace").splitlines()
    for line in reversed(lines):
        if '"usage"' not in line:
            continue
        try:
            o = json.loads(line)
        except ValueError:
            continue
        m = o.get("message") or {}
        u = m.get("usage") or {}
        if o.get("type") != "assistant" or o.get("isSidechain") or m.get("model") == "<synthetic>":
            continue
        ctx = u.get("input_tokens", 0) + u.get("cache_creation_input_tokens", 0) + u.get("cache_read_input_tokens", 0)
        when = dt.datetime.fromisoformat(o["timestamp"].replace("Z", "+00:00")).timestamp()
        return ctx, when, m.get("model"), m.get("id")
    return None


def main():
    try:
        event = json.load(sys.stdin)
        found = last_request(event["transcript_path"])
    except (ValueError, KeyError, OSError):
        return 0
    if not found:
        return 0
    ctx, when, model, mid = found
    idle = (dt.datetime.now().timestamp() - when) / 60
    if ctx < MIN_TOKENS or idle < IDLE_MIN:
        return 0
    # One warning for each cold period: the marker names the last response, which changes after a resend.
    marker = os.path.join(tempfile.gettempdir(), f"token-guard-{event.get('session_id', 'x')}-{mid}")
    if os.path.exists(marker):
        return 0
    open(marker, "w").close()
    cost = ctx * rates_for(model)["cw1h"]
    print(
        f"Token guard: this thread holds {ctx // 1000}k tokens and was idle for {idle:.0f} minutes. "
        f"The cache is cold, so this prompt costs about ${cost:.2f} before any work. "
        "Start a new thread with a handoff (/matt:handoff) to pay less, or send the prompt again to continue here.",
        file=sys.stderr,
    )
    return 2


if __name__ == "__main__":
    sys.exit(main())
