#!/usr/bin/env node
// inspect — local GUI that shows the skills, MCP servers, agents, and
// CLAUDE.md / AGENTS.md files a project has. Read-only. Binds to 127.0.0.1.
//
//   node server.mjs [project-path] [--port 4317] [--no-open]

import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import crypto from 'node:crypto';
import { execFile } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const MAX_CONTENT = 200_000;
const MAX_WALK_DEPTH = 4;

// [project-relative dir, tool that reads it]. "Source" is the repo's own
// authoring folder: present in the repo but not installed for any tool.
const SKILL_DIRS = [
  ['.claude/skills', 'Claude Code'],
  ['.agents/skills', '.agents'],
  ['.opencode/skill', 'opencode'],
  ['.opencode/skills', 'opencode'],
  ['.codex/skills', 'Codex'],
  ['.cursor/skills', 'Cursor'],
  ['.gemini/skills', 'Gemini'],
  ['.github/skills', 'Copilot'],
  ['skills', 'Source'],
];

const AGENT_DIRS = [
  ['.claude/agents', 'Claude Code'],
  ['.opencode/agent', 'opencode'],
  ['.opencode/agents', 'opencode'],
  ['.codex/agents', 'Codex'],
  ['.cursor/agents', 'Cursor'],
  ['.github/agents', 'Copilot'],
  ['agents', 'Source'],
];

// [file, tool, scope, key that holds the server map]
const MCP_FILES = [
  ['.mcp.json', 'Claude Code', 'project', 'mcpServers'],
  ['.cursor/mcp.json', 'Cursor', 'project', 'mcpServers'],
  ['.vscode/mcp.json', 'VS Code', 'project', 'servers'],
  ['.gemini/settings.json', 'Gemini', 'project', 'mcpServers'],
  ['opencode.json', 'opencode', 'project', 'mcp'],
  ['opencode.jsonc', 'opencode', 'project', 'mcp'],
  ['.opencode/opencode.json', 'opencode', 'project', 'mcp'],
];
const CODEX_CONFIG = '.codex/config.toml';
const CLAUDE_SETTINGS = ['.claude/settings.json', '.claude/settings.local.json'];

const INSTRUCTION_TOOLS = {
  'CLAUDE.md': 'Claude Code',
  'CLAUDE.local.md': 'Claude Code',
  'AGENTS.md': 'Codex + others',
  'AGENTS.override.md': 'Codex + others',
};
const SKIP_DIRS = new Set([
  'node_modules', 'dist', 'build', 'out', 'target', 'vendor', 'venv', '__pycache__',
]);

const SECRET_NAME = /token|secret|key|password|passwd|auth|bearer|credential/i;
const REDACTED = '•••';

// ---------- file boundary ----------

function readText(file) {
  try {
    const buf = fs.readFileSync(file);
    if (buf.length <= MAX_CONTENT) return buf.toString('utf8');
    return `${buf.subarray(0, MAX_CONTENT).toString('utf8')}\n\n[truncated at ${MAX_CONTENT} bytes]`;
  } catch {
    return null;
  }
}

// Config files are read whole: a truncated JSON file does not parse.
function readJson(file) {
  try {
    return JSON.parse(stripJsonComments(fs.readFileSync(file, 'utf8')));
  } catch {
    return null;
  }
}

function stripJsonComments(text) {
  let out = '';
  let inString = false;
  for (let i = 0; i < text.length; i++) {
    const c = text[i];
    if (inString) {
      out += c;
      if (c === '\\') out += text[++i] ?? '';
      else if (c === '"') inString = false;
    } else if (c === '"') {
      inString = true;
      out += c;
    } else if (c === '/' && text[i + 1] === '/') {
      while (i < text.length && text[i] !== '\n') i++;
      out += '\n';
    } else if (c === '/' && text[i + 1] === '*') {
      i = text.indexOf('*/', i + 2);
      if (i === -1) break;
      i++;
    } else {
      out += c;
    }
  }
  return out;
}

function listDir(dir) {
  try {
    return fs.readdirSync(dir, { withFileTypes: true });
  } catch {
    return [];
  }
}

function linkTarget(file) {
  try {
    return fs.lstatSync(file).isSymbolicLink() ? fs.readlinkSync(file) : null;
  } catch {
    return null;
  }
}

function isDir(file) {
  try {
    return fs.statSync(file).isDirectory();
  } catch {
    return false;
  }
}

function isObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

// ---------- parsing ----------

function parseFrontmatter(text) {
  const match = /^---\r?\n([\s\S]*?)\r?\n---\r?\n?/.exec(text);
  if (!match) return {};
  const meta = {};
  let key = null;
  for (const line of match[1].split(/\r?\n/)) {
    const pair = /^([A-Za-z0-9_-]+):\s*(.*)$/.exec(line);
    if (pair) {
      key = pair[1];
      const value = pair[2].trim();
      meta[key] = /^[>|][+-]?$/.test(value) ? '' : value.replace(/^(["'])(.*)\1$/, '$2');
    } else if (key && /^\s+\S/.test(line)) {
      meta[key] = `${meta[key]} ${line.trim()}`.trim();
    }
  }
  return meta;
}

// Reads only the [mcp_servers.*] tables of a Codex config.toml.
function parseCodexMcp(text) {
  const servers = {};
  let current = null;
  for (const raw of text.split(/\r?\n/)) {
    const line = raw.trim();
    const section = /^\[(.+)\]$/.exec(line);
    if (section) {
      const m = /^mcp_servers\.("?)([^".]+)\1(?:\.(env|http_headers))?$/.exec(section[1]);
      current = m ? { server: (servers[m[2]] ??= {}), sub: m[3] } : null;
      continue;
    }
    const pair = current && /^([A-Za-z0-9_-]+)\s*=\s*(.+)$/.exec(line);
    if (!pair) continue;
    const value = parseTomlValue(pair[2]);
    if (current.sub) (current.server[current.sub] ??= {})[pair[1]] = value;
    else current.server[pair[1]] = value;
  }
  return servers;
}

function parseTomlValue(text) {
  if (text.startsWith('{')) {
    // Inline table: keep the keys only. The values can be secrets.
    return Object.fromEntries([...text.matchAll(/([A-Za-z0-9_-]+)\s*=/g)].map((m) => [m[1], '']));
  }
  try {
    return JSON.parse(text);
  } catch {
    return text;
  }
}

function redactUrl(value) {
  try {
    const url = new URL(value);
    const query = [...url.searchParams.keys()].map((k) => `${k}=${REDACTED}`).join('&');
    return `${url.origin}${url.pathname}${query ? `?${query}` : ''}`;
  } catch {
    return value;
  }
}

function redactArgs(args) {
  return args.map((arg, i) => {
    const text = String(arg);
    const prev = i > 0 ? String(args[i - 1]) : '';
    if (prev.startsWith('-') && SECRET_NAME.test(prev) && !text.startsWith('-')) return REDACTED;
    const assign = /^(--?[\w-]+|[A-Za-z0-9_]+)=/.exec(text);
    if (assign && SECRET_NAME.test(assign[1])) return `${assign[1]}=${REDACTED}`;
    if (/^https?:\/\//.test(text)) return redactUrl(text);
    return text;
  });
}

const keysOf = (value) => (isObject(value) ? Object.keys(value) : []);

// ---------- scanners ----------

function hashOf(text) {
  return crypto.createHash('sha1').update(text).digest('hex');
}

function listFiles(dir, prefix = '', depth = 0) {
  const out = [];
  for (const entry of listDir(dir)) {
    if (entry.name === '.DS_Store') continue;
    const rel = prefix + entry.name;
    if (isDir(path.join(dir, entry.name))) {
      if (depth < 2) out.push(...listFiles(path.join(dir, entry.name), `${rel}/`, depth + 1));
      else out.push(`${rel}/`);
    } else {
      out.push(rel);
    }
  }
  return out.sort();
}

function scanSkills(root) {
  const lock = readJson(path.join(root, 'skills-lock.json'));
  const locked = isObject(lock?.skills) ? lock.skills : {};
  const byName = new Map();

  for (const [relDir, tool] of SKILL_DIRS) {
    for (const entry of listDir(path.join(root, relDir))) {
      const rel = `${relDir}/${entry.name}`;
      const abs = path.join(root, rel);
      const link = linkTarget(abs);
      const text = readText(path.join(abs, 'SKILL.md'));
      const broken = link !== null && !fs.existsSync(abs);
      if (text === null && !broken) continue;

      const skill = byName.get(entry.name) ?? { name: entry.name, locations: [] };
      byName.set(entry.name, skill);
      skill.locations.push({
        tool,
        path: rel,
        linkTarget: link,
        broken,
        hash: text === null ? null : hashOf(text),
      });
      if (text !== null && skill.content === undefined) {
        skill.content = text;
        skill.description = parseFrontmatter(text).description ?? '';
        skill.files = listFiles(abs);
      }
    }
  }

  return [...byName.values()]
    .map((skill) => {
      const hashes = new Set(skill.locations.map((l) => l.hash).filter(Boolean));
      const lockEntry = locked[skill.name];
      return {
        ...skill,
        content: skill.content ?? '',
        description: skill.description ?? '',
        files: skill.files ?? [],
        installed: skill.locations.some((l) => l.tool !== 'Source' && !l.broken),
        differs: hashes.size > 1,
        source: isObject(lockEntry) ? String(lockEntry.source ?? '') : '',
      };
    })
    .sort((a, b) => a.name.localeCompare(b.name));
}

function walkFiles(dir, exts, depth = 0) {
  const out = [];
  for (const entry of listDir(dir)) {
    const abs = path.join(dir, entry.name);
    if (isDir(abs)) {
      if (depth < 3) out.push(...walkFiles(abs, exts, depth + 1));
    } else if (exts.includes(path.extname(entry.name))) {
      out.push(abs);
    }
  }
  return out;
}

function scanAgents(root) {
  const agents = [];
  for (const [relDir, tool] of AGENT_DIRS) {
    for (const abs of walkFiles(path.join(root, relDir), ['.md', '.toml'])) {
      const content = readText(abs);
      if (content === null) continue;
      const isToml = abs.endsWith('.toml');
      const meta = isToml ? {} : parseFrontmatter(content);
      // The repo's authoring folder also holds READMEs. Keep real agents only.
      if (tool === 'Source' && !meta.description) continue;
      agents.push({
        name: meta.name || path.basename(abs, path.extname(abs)),
        description: meta.description ?? '',
        model: meta.model ?? '',
        tools: meta.tools ?? '',
        tool,
        path: path.relative(root, abs),
        linkTarget: linkTarget(abs),
        content,
      });
    }
  }
  return agents.sort((a, b) => a.name.localeCompare(b.name) || a.path.localeCompare(b.path));
}

function toServer(name, config, origin) {
  const command = Array.isArray(config.command)
    ? config.command
    : [config.command, ...(Array.isArray(config.args) ? config.args : [])].filter(Boolean);
  const url = config.url ?? config.serverUrl ?? config.httpUrl ?? '';
  return {
    name,
    ...origin,
    transport: typeof config.type === 'string' ? config.type : url ? 'http' : 'stdio',
    command: redactArgs(command).join(' '),
    url: url ? redactUrl(String(url)) : '',
    envKeys: keysOf(config.env ?? config.environment),
    headerKeys: keysOf(config.headers ?? config.http_headers),
    status: config.enabled === false || config.disabled === true ? 'disabled' : 'enabled',
  };
}

function scanMcp(root) {
  const servers = [];
  const add = (map, origin) => {
    for (const [name, config] of Object.entries(isObject(map) ? map : {})) {
      if (isObject(config)) servers.push(toServer(name, config, origin));
    }
  };

  for (const [file, tool, scope, key] of MCP_FILES) {
    add(readJson(path.join(root, file))?.[key], { tool, scope, file });
  }
  const codex = readText(path.join(root, CODEX_CONFIG));
  if (codex !== null) add(parseCodexMcp(codex), { tool: 'Codex', scope: 'project', file: CODEX_CONFIG });

  // Claude Code keeps local-scope servers and approvals in ~/.claude.json,
  // keyed by project path.
  const userConfig = readJson(path.join(os.homedir(), '.claude.json'));
  const local = userConfig?.projects?.[root] ?? {};
  add(local.mcpServers, { tool: 'Claude Code', scope: 'local', file: '~/.claude.json' });

  const settings = CLAUDE_SETTINGS.map((f) => readJson(path.join(root, f)) ?? {});
  const listed = (key) => new Set([local, ...settings].flatMap((s) => (Array.isArray(s[key]) ? s[key] : [])));
  const approved = listed('enabledMcpjsonServers');
  const rejected = listed('disabledMcpjsonServers');
  const switchedOff = listed('disabledMcpServers');
  const approveAll = settings.some((s) => s.enableAllProjectMcpServers === true);

  for (const server of servers) {
    if (server.tool !== 'Claude Code') continue;
    if (switchedOff.has(server.name)) server.status = 'disabled';
    else if (server.scope !== 'project') continue;
    else if (rejected.has(server.name)) server.status = 'disabled';
    else if (!approveAll && !approved.has(server.name)) server.status = 'pending';
  }

  // One entry per server name. Each tool that defines it is one definition.
  const byName = new Map();
  for (const { name, ...definition } of servers) {
    if (!byName.has(name)) byName.set(name, { name, definitions: [] });
    byName.get(name).definitions.push(definition);
  }
  return [...byName.values()].sort((a, b) => a.name.localeCompare(b.name));
}

function scanInstructions(root) {
  const found = [];
  const add = (abs, tool, scope) => {
    const content = readText(abs);
    if (content === null) return;
    found.push({
      path: path.relative(root, abs),
      tool,
      scope,
      linkTarget: linkTarget(abs),
      imports: [...content.matchAll(/(?:^|\s)@((?:~|\.{1,2})?\/?[\w./-]+\.md)\b/g)].map((m) => m[1]),
      content,
    });
  };

  const walk = (dir, depth) => {
    for (const entry of listDir(dir)) {
      const abs = path.join(dir, entry.name);
      const tool = INSTRUCTION_TOOLS[entry.name];
      if (tool) {
        add(abs, tool, depth === 0 ? 'root' : 'nested');
      } else if (entry.name === '.claude' && entry.isDirectory()) {
        add(path.join(abs, 'CLAUDE.md'), 'Claude Code', depth === 0 ? 'root' : 'nested');
        for (const rule of walkFiles(path.join(abs, 'rules'), ['.md'])) add(rule, 'Claude Code', 'rule');
      } else if (
        entry.isDirectory() &&
        depth < MAX_WALK_DEPTH &&
        !entry.name.startsWith('.') &&
        !SKIP_DIRS.has(entry.name)
      ) {
        walk(abs, depth + 1);
      }
    }
  };
  walk(root, 0);
  return found.sort((a, b) => a.path.localeCompare(b.path));
}

function expandHome(input) {
  return input === '~' || input.startsWith('~/') ? path.join(os.homedir(), input.slice(1)) : input;
}

function scan(input) {
  const root = path.resolve(expandHome(input));
  if (!isDir(root)) return null;
  return {
    root,
    name: path.basename(root),
    isGitRepo: fs.existsSync(path.join(root, '.git')),
    skills: scanSkills(root),
    mcp: scanMcp(root),
    agents: scanAgents(root),
    instructions: scanInstructions(root),
    checked: {
      skills: SKILL_DIRS.map(([dir]) => `${dir}/`),
      mcp: [...MCP_FILES.map(([file]) => file), CODEX_CONFIG, '~/.claude.json (this project)'],
      agents: AGENT_DIRS.map(([dir]) => `${dir}/`),
      instructions: [...Object.keys(INSTRUCTION_TOOLS), '.claude/CLAUDE.md', '.claude/rules/'],
    },
  };
}

// Folder suggestions for the path field.
function suggestDirs(input) {
  const typed = expandHome(input);
  const [dir, prefix] = typed.endsWith('/') ? [typed, ''] : [path.dirname(typed), path.basename(typed)];
  return listDir(dir)
    .filter((e) => e.name.startsWith(prefix) && (prefix.startsWith('.') || !e.name.startsWith('.')))
    .filter((e) => isDir(path.join(dir, e.name)))
    .slice(0, 40)
    .map((e) => path.join(dir, e.name));
}

// ---------- server ----------

function parseArgs(argv) {
  const args = { project: process.cwd(), port: 4317, open: true };
  for (let i = 0; i < argv.length; i++) {
    if (argv[i] === '--port') args.port = Number(argv[++i]);
    else if (argv[i] === '--no-open') args.open = false;
    else args.project = argv[i];
  }
  return args;
}

const args = parseArgs(process.argv.slice(2));
if (!Number.isInteger(args.port) || args.port < 1 || args.port > 65535) {
  console.error('inspect: --port must be a number from 1 to 65535');
  process.exit(1);
}

function sendJson(res, status, body) {
  res.writeHead(status, { 'content-type': 'application/json; charset=utf-8', 'cache-control': 'no-store' });
  res.end(JSON.stringify(body));
}

const server = http.createServer((req, res) => {
  const { port } = server.address();
  // Refuse other hostnames, so a web page cannot reach this server through
  // DNS rebinding.
  if (![`127.0.0.1:${port}`, `localhost:${port}`].includes(req.headers.host)) {
    return sendJson(res, 403, { error: 'Forbidden host' });
  }
  const url = new URL(req.url, `http://${req.headers.host}`);

  if (url.pathname === '/') {
    res.writeHead(200, { 'content-type': 'text/html; charset=utf-8', 'cache-control': 'no-store' });
    return res.end(fs.readFileSync(path.join(HERE, 'index.html')));
  }
  if (url.pathname === '/api/scan') {
    const input = url.searchParams.get('path') || args.project;
    const result = scan(input);
    return result
      ? sendJson(res, 200, result)
      : sendJson(res, 404, { error: `No folder at ${input}. Check the path, then scan again.` });
  }
  if (url.pathname === '/api/dirs') {
    return sendJson(res, 200, suggestDirs(url.searchParams.get('path') ?? ''));
  }
  sendJson(res, 404, { error: 'Not found' });
});

let attempts = 0;
server.on('error', (err) => {
  if (err.code === 'EADDRINUSE' && attempts++ < 10) return server.listen(++args.port, '127.0.0.1');
  console.error(`inspect: ${err.message}`);
  process.exit(1);
});

server.on('listening', () => {
  const address = `http://127.0.0.1:${server.address().port}`;
  console.log(`inspect: ${address}  (project: ${path.resolve(expandHome(args.project))})`);
  console.log('inspect: press Ctrl+C to stop');
  if (!args.open) return;
  const opener = { darwin: 'open', win32: 'explorer' }[process.platform] ?? 'xdg-open';
  execFile(opener, [address], () => {});
});

server.listen(args.port, '127.0.0.1');
