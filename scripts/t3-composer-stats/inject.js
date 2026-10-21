// Adds a stats row under the T3 Code composer: provider usage windows (5h,
// weekly), context use of the open thread, and the output speed of its last
// turn. patch.mjs inlines this file into the client's index.html.
//
// It reads the JSON frames the client already exchanges with the server, so it
// depends on the wire protocol and on one DOM anchor, not on minified names.
(() => {
  if (window.__t3ComposerStats) return;

  const FORM_SELECTOR = "form[data-chat-composer-form]";
  const CARD_SELECTOR = ".shadow-composer";
  const NEEDLES = ['"usageLimits"', '"provider-turn:', '"provider-thread:', '"turn-item:'];
  const GENERATION_ITEM_TYPES = new Set(["reasoning", "assistant_message"]);
  const STORAGE_KEY = "t3-composer-stats:v1";
  const STORED_THREADS = 300;
  const UUID = /[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/g;

  /** @type {Map<string, {models: Set<string>, windows: any[]}>} */
  const providers = new Map();
  /** @type {Map<string, string>} */
  const threadByRequest = new Map();
  /** @type {Map<string, {provider?: string, context?: {used: number, max: number | null, at: number}, rate?: {value: number, at: number}, items: Map<string, {type: string, start: number, end: number}>}>} */
  const threads = new Map();

  const state = { providers, threads, frames: 0, errors: 0 };
  window.__t3ComposerStats = state;

  // The client caches threads and resumes their streams, so the server does
  // not resend old turns after a restart. Keep the derived values instead.
  try {
    const stored = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "{}");
    for (const id in stored) threads.set(id, { ...stored[id], items: new Map() });
  } catch {
    state.errors += 1;
  }
  let saved = "";
  function save() {
    const recent = [...threads]
      .filter(([, entry]) => entry.context || entry.rate)
      .sort(([, a], [, b]) => (b.context?.at ?? b.rate.at) - (a.context?.at ?? a.rate.at))
      .slice(0, STORED_THREADS)
      .map(([id, { provider, context, rate }]) => [id, { provider, context, rate }]);
    const json = JSON.stringify(Object.fromEntries(recent));
    if (json === saved) return;
    saved = json;
    localStorage.setItem(STORAGE_KEY, json);
  }

  const thread = (id) => {
    let entry = threads.get(id);
    if (!entry) threads.set(id, (entry = { items: new Map() }));
    return entry;
  };
  const time = (value) => (typeof value === "string" ? Date.parse(value) : NaN);

  function readProvider(node) {
    providers.set(node.instanceId, {
      models: new Set((node.models ?? []).map((model) => model.name)),
      windows: node.usageLimits.windows ?? [],
    });
  }

  function readContext(threadId, usage, at) {
    if (!usage || typeof usage.usedTokens !== "number") return;
    const entry = thread(threadId);
    const stamp = Number.isFinite(at) ? at : Date.now();
    if (entry.context && entry.context.at > stamp) return;
    entry.context = { used: usage.usedTokens, max: usage.maxTokens ?? null, at: stamp };
  }

  // Output tokens over the time the model spent generating: from the first
  // reasoning or message item to the end of the turn, less the time tools ran.
  function readTurn(threadId, turn) {
    readContext(threadId, turn.tokenUsage, time(turn.tokenUsage?.updatedAt));
    const tokens = turn.turnTokenUsage?.outputTokens;
    const start = time(turn.startedAt);
    const end = time(turn.completedAt);
    if (turn.status !== "completed" || !tokens || !(end > start)) return;
    const entry = thread(threadId);
    if (entry.rate && entry.rate.at >= end) return;

    let first = Infinity;
    const tools = [];
    for (const item of entry.items.values()) {
      if (item.start < start || item.start > end) continue;
      if (GENERATION_ITEM_TYPES.has(item.type)) first = Math.min(first, item.start);
      else if (item.end > item.start) tools.push([item.start, Math.min(item.end, end)]);
    }
    if (first === Infinity) first = start;

    let toolTime = 0;
    let cursor = first;
    for (const [from, to] of tools.sort((a, b) => a[0] - b[0])) {
      if (to <= cursor) continue;
      toolTime += to - Math.max(from, cursor);
      cursor = to;
    }
    const seconds = (end - first - toolTime) / 1000;
    if (seconds >= 0.25) entry.rate = { value: tokens / seconds, at: end };
    for (const [id, item] of entry.items) if (item.start <= end) entry.items.delete(id);
  }

  function walk(node, threadId) {
    if (Array.isArray(node)) {
      for (const child of node) walk(child, threadId);
      return;
    }
    if (node === null || typeof node !== "object") return;
    if (typeof node.threadId === "string") threadId = node.threadId;

    if (typeof node.instanceId === "string" && node.usageLimits) readProvider(node);
    if (threadId && typeof node.providerInstanceId === "string") {
      thread(threadId).provider = node.providerInstanceId;
    }
    if (typeof node.id === "string") {
      if (node.id.startsWith("provider-thread:")) {
        const owner = node.appThreadId ?? threadId;
        if (owner) readContext(owner, node.contextUsage, time(node.updatedAt));
      } else if (node.id.startsWith("turn-item:") && threadId) {
        const start = time(node.startedAt);
        if (Number.isFinite(start)) {
          const end = time(node.completedAt ?? node.updatedAt);
          thread(threadId).items.set(node.id, { type: node.type, start, end: end || start });
        }
      }
    }
    for (const key in node) {
      const child = node[key];
      if (child !== null && typeof child === "object") walk(child, threadId);
    }
    // After the children, so that the items of a snapshot are known first.
    if (typeof node.id === "string" && node.id.startsWith("provider-turn:") && threadId) {
      readTurn(threadId, node);
    }
  }

  function onReceived(data) {
    if (typeof data !== "string" || !NEEDLES.some((needle) => data.includes(needle))) return;
    try {
      const message = JSON.parse(data);
      for (const frame of Array.isArray(message) ? message : [message]) {
        if (frame?._tag !== "Chunk") continue;
        walk(frame.values, threadByRequest.get(String(frame.requestId)));
      }
      state.frames += 1;
      save();
      render();
    } catch {
      state.errors += 1;
    }
  }

  function onSent(data) {
    if (typeof data !== "string" || !data.includes('"orchestration.subscribeThread"')) return;
    try {
      const request = JSON.parse(data);
      if (request.payload?.threadId) threadByRequest.set(String(request.id), request.payload.threadId);
    } catch {
      state.errors += 1;
    }
  }

  const NativeWebSocket = window.WebSocket;
  window.WebSocket = class extends NativeWebSocket {
    constructor(...args) {
      super(...args);
      this.addEventListener("message", (event) => onReceived(event.data));
    }
    send(data) {
      onSent(data);
      return super.send(data);
    }
  };

  const STYLE = `
    .t3cs-row { position: relative; z-index: 11; display: flex; align-items: center; justify-content: space-between; gap: 16px;
      padding: 8px 18px 0; font-size: 11px; line-height: 16px; white-space: nowrap;
      color: var(--color-muted-foreground, #8b8b8b); font-variant-numeric: tabular-nums; }
    .t3cs-row:empty { display: none; }
    .t3cs-group { display: flex; align-items: center; gap: 14px; min-width: 0; }
    .t3cs-stat { display: inline-flex; align-items: center; gap: 6px; }
    .t3cs-value { color: var(--color-foreground, currentColor); opacity: 0.85; font-weight: 500; }
    .t3cs-bar { position: relative; width: 36px; height: 3px; border-radius: 2px; overflow: hidden;
      background: color-mix(in oklab, currentColor 24%, transparent); }
    .t3cs-bar > i { position: absolute; inset: 0 auto 0 0; border-radius: 2px;
      background: var(--color-foreground, currentColor); opacity: 0.8; }
    .t3cs-stat[data-left] .t3cs-bar > i { opacity: 1; }
    .t3cs-stat[data-left="ok"] .t3cs-bar > i { background: var(--success, #10b981); }
    .t3cs-stat[data-left="low"] .t3cs-bar > i { background: var(--warning, #f59e0b); }
    .t3cs-stat[data-left="critical"] .t3cs-bar > i { background: var(--error, #ef4444); }
    .t3cs-stat[data-left="critical"] .t3cs-value { color: var(--error, #ef4444); opacity: 1; }
  `;

  const escapeHtml = (text) =>
    String(text).replace(/[&<>"]/g, (char) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[char]);

  // Usage left: more than half is ok, less than a fifth is critical.
  const leftLevel = (usedPercent) => (usedPercent >= 80 ? "critical" : usedPercent >= 50 ? "low" : "ok");

  function windowLabel(window, sameKindCount) {
    if (sameKindCount > 1 && window.id !== "five_hour" && window.id !== "seven_day") return window.label;
    if (window.kind === "session" && window.windowDurationMins) {
      const hours = window.windowDurationMins / 60;
      return hours >= 1 ? `${Math.round(hours)}h` : `${window.windowDurationMins}m`;
    }
    return window.kind === "weekly" ? "Week" : window.label;
  }

  function currentThreadId() {
    const ids = `${location.pathname}${location.hash}`.match(UUID) ?? [];
    for (let index = ids.length - 1; index >= 0; index -= 1) if (threads.has(ids[index])) return ids[index];
    return null;
  }

  // A thread knows its provider. A draft does not, so match the model name
  // that the composer's picker shows against each provider's model list.
  function providerFor(form, threadId) {
    const known = threadId && providers.get(threads.get(threadId).provider);
    if (known) return known;
    const labels = new Set([...form.querySelectorAll("button")].map((button) => button.textContent.trim()));
    for (const provider of providers.values()) {
      if (provider.windows.length === 0) continue;
      for (const model of provider.models) if (labels.has(model)) return provider;
    }
    return null;
  }

  function rowHtml(form) {
    const threadId = currentThreadId();
    const provider = providerFor(form, threadId);
    const windows = (provider?.windows ?? []).filter((window) => window.kind === "session" || window.kind === "weekly");
    const count = (kind) => windows.filter((window) => window.kind === kind).length;
    const left = windows.map((window) => {
      const percent = Math.round(Math.max(0, Math.min(100, window.usedPercent)));
      const reset = Number.isFinite(time(window.resetsAt)) ? `Resets ${new Date(window.resetsAt).toLocaleString()}` : "";
      return `<span class="t3cs-stat" title="${escapeHtml(reset)}" data-left="${leftLevel(percent)}">${escapeHtml(
        windowLabel(window, count(window.kind)),
      )}<span class="t3cs-bar"><i style="width:${percent}%"></i></span><span class="t3cs-value">${percent}%</span></span>`;
    });

    const right = [];
    const entry = threadId ? threads.get(threadId) : null;
    if (entry?.context) {
      const { used, max } = entry.context;
      const value = max ? `${Math.round(Math.min(100, (used / max) * 100))}%` : `${Math.round(used / 1000)}k`;
      const title = max ? `${used.toLocaleString()} of ${max.toLocaleString()} tokens` : `${used.toLocaleString()} tokens`;
      right.push(`<span class="t3cs-stat" title="${title}"><span class="t3cs-value">${value}</span>context</span>`);
    }
    if (entry?.rate) {
      right.push(
        `<span class="t3cs-stat" title="Output tokens per second, last turn, tool time excluded"><span class="t3cs-value">${Math.round(entry.rate.value)}</span>tok/s</span>`,
      );
    }
    if (left.length === 0 && right.length === 0) return "";
    return `<span class="t3cs-group">${left.join("")}</span><span class="t3cs-group">${right.join("")}</span>`;
  }

  function render() {
    if (!document.body) return;
    if (!document.getElementById("t3cs-style")) {
      const style = document.createElement("style");
      style.id = "t3cs-style";
      style.textContent = STYLE;
      document.head.append(style);
    }
    for (const row of document.querySelectorAll(".t3cs-row")) if (!row.__form?.isConnected) row.remove();
    for (const form of document.querySelectorAll(FORM_SELECTOR)) {
      // The row goes below the composer card, not in it.
      const card = form.closest(CARD_SELECTOR) ?? form.lastElementChild;
      let row = card.nextElementSibling;
      if (row?.__form !== form) {
        row = document.createElement("div");
        row.className = "t3cs-row";
        row.__form = form;
        card.after(row);
      }
      const html = rowHtml(form);
      if (row.__html !== html) {
        row.innerHTML = html;
        row.__html = html;
      }
    }
  }

  setInterval(render, 1000);
})();
