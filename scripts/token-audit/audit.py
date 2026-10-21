#!/usr/bin/env python3
"""Audit token use across Claude Code and Codex transcripts.

Usage: audit.py [--days 14] [--json out.json] [--top 20]
Reads ~/.claude/projects and ~/.codex/sessions. Changes no files except --json.
"""
import argparse
import collections
import datetime as dt
import glob
import hashlib
import json
import os
import re
import statistics
import sys

HOME = os.path.expanduser("~")
CLAUDE_DIR = os.path.join(HOME, ".claude", "projects")
CODEX_DIRS = [os.path.join(HOME, ".codex", "sessions"), os.path.join(HOME, ".codex", "archived_sessions")]
RATES_FILE = os.path.join(HOME, ".t3", "userdata", "usage-model-rates.json")

# USD per token. Used only when the T3 rates file has no entry for the model.
FALLBACK_RATES = {"in": 4e-6, "out": 2e-5, "cw": 5e-6, "cw1h": 8e-6, "cr": 2e-7}

EDIT_TOOLS = {"Edit", "Write", "NotebookEdit", "apply_patch"}
POLL_HINTS = ("status", "wait", "snapshot", "list", "read", "sleep", "screenshot")


def load_rates():
    try:
        return json.load(open(RATES_FILE))["document"]
    except (OSError, KeyError, ValueError):
        return {}


RATES = load_rates()


def rates_for(model):
    r = RATES.get(model) or RATES.get(re.sub(r"\[.*\]$", "", model or "")) or {}
    if not r.get("input_cost_per_token"):
        return FALLBACK_RATES
    cw = r.get("cache_creation_input_token_cost", r["input_cost_per_token"] * 1.25)
    return {
        "in": r["input_cost_per_token"],
        "out": r.get("output_cost_per_token", 0),
        "cw": cw,
        "cw1h": r.get("cache_creation_input_token_cost_above_1hr", cw),
        "cr": r.get("cache_read_input_token_cost", r["input_cost_per_token"] * 0.1),
    }


def ts(s):
    try:
        return dt.datetime.fromisoformat(s.replace("Z", "+00:00")).timestamp()
    except (AttributeError, ValueError):
        return None


def text_len(content):
    if isinstance(content, str):
        return len(content)
    if isinstance(content, list):
        return sum(text_len(c.get("text") or c.get("content") or "") if isinstance(c, dict) else len(str(c)) for c in content)
    return 0


def bash_head(cmd):
    """Reduce a shell command to the program and subcommand that did the work."""
    cmd = re.sub(r"^\s*(cd\s+(\"[^\"]*\"|'[^']*'|\S+)\s*(&&|;)\s*)+", "", cmd.strip())
    cmd = re.sub(r"^(\w+=\S+\s+)+", "", cmd)
    parts = cmd.split()
    if not parts:
        return ""
    head = os.path.basename(parts[0])
    if head in ("git", "gh", "npm", "pnpm", "bun", "yarn", "npx", "cargo", "swift", "xcodebuild", "sqlite3", "docker", "uv") and len(parts) > 1:
        return f"{head} {parts[1]}"
    return head


INLINE_RE = re.compile(r"(python3?|node|bun|ruby|osascript)\s+(-I\s+)?(-\s*<<|-e\b|-c\b|--input-type)|<<\s*['\"]?\w+['\"]?")


class Session:
    def __init__(self, provider, path, project):
        self.provider = provider
        self.path = path
        self.project = project
        self.parent = None
        self.title = ""
        self.prompts = []
        self.models = collections.Counter()
        self.tok = collections.Counter()
        self.cost = 0.0
        self.requests = 0
        self.baseline = None
        self.max_ctx = 0
        self.cache_miss = []
        self.tools = []  # (name, key, detail)
        self.results = {}  # tool_use_id -> (chars, is_error)
        self.skill_chars = collections.Counter()
        self.attach_chars = collections.Counter()
        self.compactions = 0
        self.first_ts = None
        self.last_ts = None

    @property
    def kind(self):
        if "t3code-claude-title" in self.project:
            return "title"
        if self.parent:
            return "subagent"
        first = self.prompts[0].lower() if self.prompts else ""
        if first.startswith(("delegated task", "act as the ")):
            return "delegated"
        if first.startswith("you write "):
            return "utility"
        return "main"


def parse_claude(path, cutoff):
    rel = os.path.relpath(path, CLAUDE_DIR).split(os.sep)
    s = Session("claude", path, rel[0])
    if "subagents" in rel:
        s.parent = rel[1]
    seen = {}
    prev_ctx = 0
    prev_ts = None
    pending_skill = None
    for line in open(path, errors="replace"):
        try:
            o = json.loads(line)
        except ValueError:
            continue
        t = o.get("type")
        if t == "ai-title":
            s.title = o.get("aiTitle") or s.title
            continue
        when = ts(o.get("timestamp"))
        if when is not None and when < cutoff:
            continue
        if when:
            s.first_ts = s.first_ts or when
            s.last_ts = when
        if t == "attachment":
            a = o.get("attachment") or {}
            s.attach_chars[a.get("type", "?")] += len(json.dumps(a))
        elif t == "user":
            if o.get("isCompactSummary"):
                s.compactions += 1
            content = (o.get("message") or {}).get("content")
            if isinstance(content, str):
                content = [{"type": "text", "text": content}]
            for b in content or []:
                if not isinstance(b, dict):
                    continue
                if b.get("type") == "tool_result":
                    s.results[b.get("tool_use_id")] = (text_len(b.get("content")), bool(b.get("is_error")))
                elif b.get("type") == "text":
                    txt = b.get("text") or ""
                    if txt.startswith("Base directory for this skill"):
                        s.skill_chars[pending_skill or "?"] += len(txt)
                    elif not o.get("isMeta") and not txt.startswith("<") and not s.parent:
                        s.prompts.append(txt[:300])
        elif t == "assistant":
            m = o.get("message") or {}
            u = m.get("usage") or {}
            mid = m.get("id") or o.get("uuid")
            for b in m.get("content") or []:
                if b.get("type") == "tool_use":
                    inp = b.get("input") or {}
                    name = b.get("name")
                    if name == "Skill":
                        pending_skill = inp.get("skill")
                    s.tools.append((name, b.get("id"), inp))
            model = m.get("model") or "?"
            if model == "<synthetic>":
                continue
            out = u.get("output_tokens", 0)
            if mid in seen:
                # One API response is stored as several records; only output can grow.
                extra = max(0, out - seen[mid])
                seen[mid] = max(seen[mid], out)
                s.tok["out"] += extra
                s.cost += extra * rates_for(model)["out"]
                continue
            seen[mid] = out
            r = rates_for(model)
            # The advisor runs inside the request and is billed at its own model, uncached.
            for it in u.get("iterations") or []:
                if it.get("type") == "advisor_message":
                    ar = rates_for(it.get("model") or model)
                    s.tok["advisor_calls"] += 1
                    s.tok["advisor_in"] += it.get("input_tokens", 0)
                    adv = it.get("input_tokens", 0) * ar["in"] + it.get("output_tokens", 0) * ar["out"]
                    s.tok["cost_advisor"] += adv
                    s.cost += adv
            cc = u.get("cache_creation") or {}
            cw1h = cc.get("ephemeral_1h_input_tokens", 0)
            cw = u.get("cache_creation_input_tokens", 0)
            cw5 = cw - cw1h
            cr = u.get("cache_read_input_tokens", 0)
            inp = u.get("input_tokens", 0)
            s.models[model] += 1
            s.requests += 1
            s.tok["in"] += inp
            s.tok["cw"] += cw
            s.tok["cr"] += cr
            s.tok["out"] += out
            s.tok["think"] += (u.get("output_tokens_details") or {}).get("thinking_tokens", 0)
            cost = inp * r["in"] + cw5 * r["cw"] + cw1h * r["cw1h"] + cr * r["cr"] + out * r["out"]
            s.cost += cost
            s.tok["cost_cr"] += cr * r["cr"]
            s.tok["cost_cw"] += cw5 * r["cw"] + cw1h * r["cw1h"]
            s.tok["cost_out"] += out * r["out"]
            ctx = inp + cw + cr
            blocks = [b.get("type") for b in m.get("content") or []]
            if blocks and "text" not in blocks and out < 400:
                s.tok["tool_only_requests"] += 1
                s.tok["cost_tool_only"] += cost
            if s.baseline is None:
                s.baseline = ctx
            elif prev_ctx > 30000 and cr < prev_ctx * 0.5 and cw > 20000:
                gap = (when - prev_ts) if when and prev_ts else 0
                s.cache_miss.append((cw, gap, cw5 * r["cw"] + cw1h * r["cw1h"]))
            s.max_ctx = max(s.max_ctx, ctx)
            prev_ctx, prev_ts = ctx, when
    return s


TOOLS_RE = re.compile(r"tools\.(\w+)\(")
CMD_RE = re.compile(r'cmd:\s*"((?:[^"\\]|\\.)*)"')


def parse_codex(path, cutoff):
    s = Session("codex", path, "?")
    model = "?"
    last_total = None
    calls = {}
    for line in open(path, errors="replace"):
        try:
            o = json.loads(line)
        except ValueError:
            continue
        t = o.get("type")
        p = o.get("payload") or {}
        when = ts(o.get("timestamp"))
        if when:
            s.first_ts = s.first_ts or when
            s.last_ts = when
        if t == "session_meta":
            s.project = (p.get("cwd") or "?").replace(HOME, "~")
            src = p.get("source")
            if isinstance(src, dict) and "subagent" in json.dumps(src):
                s.parent = "codex-subagent"
            continue
        if t == "turn_context":
            model = p.get("model") or model
            continue
        pt = p.get("type") if isinstance(p, dict) else None
        if t == "event_msg" and pt == "token_count":
            info = p.get("info") or {}
            last = info.get("last_token_usage") or {}
            last_total = info.get("total_token_usage") or last_total
            if last:
                s.requests += 1
                s.models[model] += 1
                ctx = last.get("input_tokens", 0)
                if s.baseline is None:
                    s.baseline = ctx
                s.max_ctx = max(s.max_ctx, ctx)
                r = rates_for(model)
                cached = last.get("cached_input_tokens", 0)
                fresh = max(0, ctx - cached)
                out = last.get("output_tokens", 0)
                s.tok["in"] += fresh
                s.tok["cr"] += cached
                s.tok["out"] += out
                s.tok["think"] += last.get("reasoning_output_tokens", 0)
                s.cost += fresh * r["in"] + cached * r["cr"] + out * r["out"]
                s.tok["cost_cr"] += cached * r["cr"]
                s.tok["cost_cw"] += fresh * r["in"]
                s.tok["cost_out"] += out * r["out"]
        elif t == "event_msg" and pt == "item_completed":
            item = p.get("item") or {}
            if item.get("type") == "UserMessage":
                txt = " ".join(c.get("text", "") for c in item.get("content") or [] if isinstance(c, dict))
                if txt and not txt.startswith("<"):
                    s.prompts.append(txt[:300])
        elif t == "event_msg" and pt == "context_compacted":
            s.compactions += 1
        elif t == "response_item" and pt == "function_call":
            name = p.get("name")
            if p.get("namespace"):
                name = f"{p['namespace']}__{name}"
            try:
                args = json.loads(p.get("arguments") or "{}")
            except ValueError:
                args = {"raw": p.get("arguments")}
            calls[p.get("call_id")] = name
            s.tools.append((name, p.get("call_id"), args))
        elif t == "response_item" and pt == "custom_tool_call":
            code = p.get("input") or ""
            inner = TOOLS_RE.findall(code) or [p.get("name")]
            cmds = CMD_RE.findall(code)
            for i, name in enumerate(inner):
                inp = {"code": code}
                if name == "exec_command" and cmds:
                    inp["cmd"] = cmds.pop(0).encode().decode("unicode_escape", "replace")
                s.tools.append((name, p.get("call_id") if i == 0 else None, inp))
        elif t == "response_item" and pt in ("function_call_output", "custom_tool_call_output"):
            out = p.get("output")
            chars = text_len(out)
            m = re.search(r"original token count: (\d+)", json.dumps(out)[:600])
            if m:
                chars = max(chars, int(m.group(1)) * 4)
            s.results[p.get("call_id")] = (chars, False)
    if not s.title and s.prompts:
        s.title = s.prompts[0][:70]
    return s


def collect(days):
    cutoff = dt.datetime.now().timestamp() - days * 86400
    sessions = []
    for path in glob.glob(os.path.join(CLAUDE_DIR, "**", "*.jsonl"), recursive=True):
        if os.path.getmtime(path) >= cutoff:
            sessions.append(parse_claude(path, cutoff))
    for d in CODEX_DIRS:
        for path in glob.glob(os.path.join(d, "**", "*.jsonl"), recursive=True):
            if os.path.getmtime(path) >= cutoff:
                sessions.append(parse_codex(path, cutoff))
    return [s for s in sessions if s.requests]


def tool_server(name):
    m = re.match(r"mcp__(.+?)__", name or "")
    return m.group(1) if m else None


def call_key(name, inp):
    return hashlib.md5((str(name) + json.dumps(inp, sort_keys=True, default=str)).encode()).hexdigest()


def analyze(sessions, top):
    rep = {}
    total = sum(s.cost for s in sessions)
    rep["total_cost"] = total
    rep["sessions"] = len(sessions)

    by = lambda keyf: collections.defaultdict(lambda: collections.Counter())
    by_model = by(None)
    for s in sessions:
        # A session can switch models; attribute by share of requests.
        n = sum(s.models.values()) or 1
        for m, c in s.models.items():
            row = by_model[(s.provider, m)]
            row["requests"] += c
            row["cost"] += s.cost * c / n
            for k in ("in", "cw", "cr", "out", "think"):
                row[k] += s.tok[k] * c / n
    rep["by_model"] = {f"{p}/{m}": dict(v) for (p, m), v in sorted(by_model.items(), key=lambda kv: -kv[1]["cost"])}

    by_kind = collections.defaultdict(lambda: collections.Counter())
    for s in sessions:
        row = by_kind[f"{s.provider}/{s.kind}"]
        row["sessions"] += 1
        row["cost"] += s.cost
        row["requests"] += s.requests
    rep["by_kind"] = {k: dict(v) for k, v in by_kind.items()}

    by_proj = collections.Counter()
    for s in sessions:
        by_proj[re.sub(r"^-Users-h0-0man-Documents-|^~/Documents/", "", s.project)[:60]] += s.cost
    rep["by_project"] = by_proj.most_common(15)

    rep["cost_split"] = {
        "cache_read": sum(s.tok["cost_cr"] for s in sessions),
        "cache_write_or_fresh_input": sum(s.tok["cost_cw"] for s in sessions),
        "output": sum(s.tok["cost_out"] for s in sessions),
        "advisor": sum(s.tok["cost_advisor"] for s in sessions),
    }
    rep["advisor"] = {
        "calls": sum(s.tok["advisor_calls"] for s in sessions),
        "uncached_input_tokens": sum(s.tok["advisor_in"] for s in sessions),
        "cost": sum(s.tok["cost_advisor"] for s in sessions),
    }
    rep["tool_only_requests"] = {
        "requests": sum(s.tok["tool_only_requests"] for s in sessions if s.provider == "claude"),
        "of": sum(s.requests for s in sessions if s.provider == "claude"),
        "cost": sum(s.tok["cost_tool_only"] for s in sessions),
        "cost_on_opus_main": sum(s.tok["cost_tool_only"] for s in sessions if s.kind == "main" and "opus" in s.models.most_common(1)[0][0]),
    }

    rep["top_sessions"] = [
        {
            "cost": round(s.cost, 2), "provider": s.provider, "kind": s.kind,
            "model": s.models.most_common(1)[0][0], "requests": s.requests,
            "max_ctx": s.max_ctx, "compactions": s.compactions,
            "project": s.project[-40:], "title": (s.title or (s.prompts[0] if s.prompts else ""))[:80],
            "file": os.path.basename(s.path),
        }
        for s in sorted(sessions, key=lambda s: -s.cost)[:top]
    ]

    base = collections.defaultdict(list)
    for s in sessions:
        if s.baseline:
            base[f"{s.provider}/{s.kind}"].append(s.baseline)
    rep["baseline_ctx"] = {k: {"n": len(v), "median": int(statistics.median(v)), "max": max(v)} for k, v in base.items()}

    miss = [m for s in sessions for m in s.cache_miss]
    rep["cache_miss"] = {
        "events": len(miss),
        "tokens": sum(m[0] for m in miss),
        "cost": sum(m[2] for m in miss),
        "after_idle_over_1h": sum(1 for m in miss if m[1] > 3600),
        "after_idle_5m_to_1h": sum(1 for m in miss if 300 < m[1] <= 3600),
        "no_idle": sum(1 for m in miss if m[1] <= 300),
    }

    # Long sessions pay cache_read on the whole context for every request.
    ctx_buckets = collections.Counter()
    for s in sessions:
        b = "<100k" if s.max_ctx < 100_000 else "100-200k" if s.max_ctx < 200_000 else "200-400k" if s.max_ctx < 400_000 else ">400k"
        ctx_buckets[b] += s.cost
    rep["cost_by_max_context"] = dict(ctx_buckets)

    tool = collections.defaultdict(lambda: collections.Counter())
    biggest = []
    dup = collections.Counter()
    dup_label = {}
    reads = collections.Counter()
    bash = collections.Counter()
    bash_chars = collections.Counter()
    inline = collections.Counter()
    inline_chars = 0
    skills = collections.Counter()
    skill_chars = collections.Counter()
    agents = collections.Counter()
    runs = collections.Counter()
    scratch_writes = collections.Counter()
    for s in sessions:
        skill_chars.update(s.skill_chars)
        seen_keys = collections.Counter()
        prev, run = None, 0
        for name, tid, inp in s.tools:
            chars, err = s.results.get(tid, (0, False))
            row = tool[f"{s.provider}:{name}"]
            row["calls"] += 1
            row["chars"] += chars
            row["errors"] += err
            if chars > 40000:
                biggest.append((chars, name, json.dumps(inp, default=str)[:140], os.path.basename(s.path)[:13]))
            k = call_key(name, inp)
            seen_keys[k] += 1
            if seen_keys[k] > 1:
                dup[k] += 1
                dup_label[k] = (name, json.dumps(inp, default=str)[:110], chars)
            if name == "Read":
                reads[(s.path, inp.get("file_path"))] += 1
            if name in ("Write",) and re.search(r"/tmp/|/scratch|/T/|\.t3/", str(inp.get("file_path"))):
                scratch_writes[os.path.splitext(str(inp.get("file_path")))[1]] += 1
            cmd = inp.get("command") or inp.get("cmd") if isinstance(inp, dict) else None
            if isinstance(cmd, list):
                cmd = " ".join(map(str, cmd))
            if name in ("Bash", "exec_command", "shell") and cmd:
                h = bash_head(cmd)
                bash[h] += 1
                bash_chars[h] += chars
                if INLINE_RE.search(cmd) and len(cmd) > 200:
                    inline[h] += 1
                    inline_chars += len(cmd)
            if name == "Skill":
                skills[inp.get("skill")] += 1
            if name == "Agent":
                agents[(inp.get("subagent_type") or "general-purpose", inp.get("model") or "inherit")] += 1
            label = name if name not in ("Bash", "exec_command") else f"{name}:{bash_head(cmd or '')}"
            if label == prev:
                run += 1
            else:
                if run >= 5:
                    runs[prev] += run
                prev, run = label, 1
        if run >= 5:
            runs[prev] += run

    rep["tools_by_result_chars"] = [
        {"tool": k, "calls": v["calls"], "result_tokens_est": v["chars"] // 4, "avg_tokens": v["chars"] // 4 // max(1, v["calls"]), "errors": v["errors"]}
        for k, v in sorted(tool.items(), key=lambda kv: -kv[1]["chars"])[:top + 10]
    ]
    rep["tools_by_errors"] = [
        {"tool": k, "calls": v["calls"], "errors": v["errors"]}
        for k, v in sorted(tool.items(), key=lambda kv: -kv[1]["errors"])[:12] if v["errors"]
    ]
    servers = collections.defaultdict(lambda: collections.Counter())
    for k, v in tool.items():
        srv = tool_server(k.split(":", 1)[1])
        if srv:
            servers[srv].update(v)
    rep["mcp_servers"] = {k: {"calls": v["calls"], "result_tokens_est": v["chars"] // 4, "errors": v["errors"]} for k, v in sorted(servers.items(), key=lambda kv: -kv[1]["chars"])}
    rep["biggest_results"] = [{"tokens_est": c // 4, "tool": n, "input": i, "session": f} for c, n, i, f in sorted(biggest, reverse=True)[:top]]
    rep["duplicate_calls"] = {
        "extra_calls": sum(dup.values()),
        "wasted_tokens_est": sum(dup[k] * dup_label[k][2] for k in dup) // 4,
        "top": [{"extra": c, "tool": dup_label[k][0], "input": dup_label[k][1]} for k, c in dup.most_common(top)],
    }
    rereads = {k: v for k, v in reads.items() if v >= 4}
    rep["rereads"] = {
        "files_read_4plus_in_one_session": len(rereads),
        "top": [{"reads": v, "file": str(k[1])[-70:]} for k, v in sorted(rereads.items(), key=lambda kv: -kv[1])[:top]],
    }
    rep["bash_heads"] = [{"cmd": k, "calls": v, "result_tokens_est": bash_chars[k] // 4} for k, v in bash.most_common(top + 15)]
    rep["inline_scripts"] = {"count": sum(inline.values()), "source_tokens_est": inline_chars // 4, "by_head": inline.most_common(10), "scratch_file_writes": dict(scratch_writes)}
    rep["skills"] = [{"skill": k, "invocations": v, "injected_tokens_est": skill_chars[k] // 4} for k, v in skills.most_common(top)]
    rep["agent_spawns"] = [{"type": k[0], "model": k[1], "count": v} for k, v in agents.most_common(top)]
    rep["same_tool_runs_5plus"] = [{"tool": k, "calls_in_runs": v} for k, v in runs.most_common(top)]
    rep["poll_like_runs"] = [r for r in rep["same_tool_runs_5plus"] if any(h in r["tool"].lower() for h in POLL_HINTS)]

    att = collections.Counter()
    for s in sessions:
        att.update(s.attach_chars)
    rep["attachments_tokens_est"] = {k: v // 4 for k, v in att.most_common(12)}

    prompts = collections.Counter()
    for s in sessions:
        if s.kind != "main":
            continue
        for p in s.prompts:
            norm = re.sub(r"\s+", " ", re.sub(r"[/~][\w./-]+|\d+", "#", p.lower())).strip()[:50]
            if len(norm) > 12:
                prompts[norm] += 1
    rep["repeated_prompts"] = [{"n": v, "prompt": k} for k, v in prompts.most_common(top + 10) if v >= 3]

    # Sessions where a top-tier model did no edits and wrote little: candidates for a cheaper model.
    cheap = collections.Counter()
    cheap_n = collections.Counter()
    for s in sessions:
        model = s.models.most_common(1)[0][0]
        if not re.search(r"opus|fable|gpt-6\.1-sol", model):
            continue
        edits = sum(1 for n, _, _ in s.tools if n in EDIT_TOOLS)
        shape = "title" if s.kind == "title" else "no-tools" if not s.tools else "read-only" if not edits else "light-edit" if s.tok["out"] < 15000 else "heavy"
        cheap[f"{s.kind}/{model}/{shape}"] += s.cost
        cheap_n[f"{s.kind}/{model}/{shape}"] += 1
    rep["top_tier_session_shapes"] = [{"shape": k, "sessions": cheap_n[k], "cost": round(v, 2)} for k, v in cheap.most_common(25)]
    return rep


def show(rep):
    def money(x):
        return f"${x:,.2f}"

    print(f"SESSIONS {rep['sessions']}  EST. API-RATE COST {money(rep['total_cost'])}")
    print("\nCOST SPLIT", {k: money(v) for k, v in rep["cost_split"].items()})
    print("\nBY MODEL")
    for k, v in rep["by_model"].items():
        print(f"  {money(v['cost']):>11}  {k:34} req={int(v['requests']):6}  cache_read={int(v['cr'])/1e6:8.1f}M  write={int(v['cw'])/1e6:6.1f}M  in={int(v['in'])/1e6:6.1f}M  out={int(v['out'])/1e6:5.2f}M")
    print("\nBY KIND")
    for k, v in sorted(rep["by_kind"].items(), key=lambda kv: -kv[1]["cost"]):
        print(f"  {money(v['cost']):>11}  {k:18} sessions={v['sessions']:4} requests={v['requests']}")
    print("\nBY PROJECT")
    for k, v in rep["by_project"]:
        print(f"  {money(v):>11}  {k}")
    print("\nCOST BY MAX CONTEXT", {k: money(v) for k, v in rep["cost_by_max_context"].items()})
    print("BASELINE CONTEXT (first request)", rep["baseline_ctx"])
    for key in ("cache_miss", "advisor", "tool_only_requests"):
        print(key.upper(), {k: (money(v) if k.startswith("cost") else v) for k, v in rep[key].items()})
    for key in ("top_sessions", "top_tier_session_shapes", "tools_by_result_chars", "tools_by_errors", "biggest_results", "bash_heads", "skills", "agent_spawns", "same_tool_runs_5plus", "repeated_prompts"):
        print(f"\n{key.upper()}")
        for row in rep[key]:
            print("  " + "  ".join(f"{k}={v}" for k, v in row.items()))
    for key in ("mcp_servers", "duplicate_calls", "rereads", "inline_scripts", "attachments_tokens_est"):
        print(f"\n{key.upper()}")
        val = rep[key]
        for k, v in val.items():
            if isinstance(v, list):
                print(f"  {k}:")
                for row in v:
                    print("    " + (("  ".join(f"{a}={b}" for a, b in row.items())) if isinstance(row, dict) else str(row)))
            else:
                print(f"  {k}: {v}")


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--days", type=float, default=14)
    ap.add_argument("--top", type=int, default=20)
    ap.add_argument("--json", help="also write the full report to this path")
    a = ap.parse_args()
    rep = analyze(collect(a.days), a.top)
    if a.json:
        json.dump(rep, open(a.json, "w"), indent=1, default=str)
    show(rep)


if __name__ == "__main__":
    sys.exit(main())
