#!/usr/bin/env node
// Inlines inject.js into the web client inside a T3 Code app bundle, so the
// composer shows usage, context, and tok/s. A Nightly update replaces the
// bundle, so run this again after each update.
//
// Usage:
//   patch.mjs             apply (or re-apply) the patch
//   patch.mjs --restore   remove the patch
//   patch.mjs --status    report whether the bundle is patched
//
// Set T3_APP to target a different bundle path. The app must not be running.
//
// The patch rewrites only the header of app.asar and appends the new
// index.html after the existing data, so every other file keeps its offset.
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { closeSync, createReadStream, createWriteStream, openSync, readFileSync, readSync, renameSync, rmSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { pipeline } from "node:stream/promises";
import { fileURLToPath } from "node:url";

const APP = process.env.T3_APP ?? "/Applications/T3 Code (Nightly).app";
const ASAR = join(APP, "Contents/Resources/app.asar");
const INDEX_PATH = ["apps", "server", "dist", "client", "index.html"];
const START = "<!-- t3-composer-stats:start -->";
const END = "<!-- t3-composer-stats:end -->";
const BLOCK_SIZE = 4 * 1024 * 1024;

const mode = process.argv[2] ?? "--apply";
if (!["--apply", "--restore", "--status"].includes(mode)) {
  console.error("usage: patch.mjs [--restore | --status]");
  process.exit(2);
}

function readArchive() {
  const fd = openSync(ASAR, "r");
  try {
    const sizes = Buffer.alloc(16);
    readSync(fd, sizes, 0, 16, 0);
    const headerSize = sizes.readUInt32LE(4);
    const json = Buffer.alloc(sizes.readUInt32LE(12));
    readSync(fd, json, 0, json.length, 16);
    const header = JSON.parse(json.toString("utf8"));
    const dataStart = 8 + headerSize;
    const entry = INDEX_PATH.reduce((node, name) => node?.files?.[name], header);
    if (!entry?.offset) throw new Error(`${INDEX_PATH.join("/")} is not in ${ASAR}`);
    const html = Buffer.alloc(entry.size);
    readSync(fd, html, 0, entry.size, dataStart + Number(entry.offset));
    return { header, entry, dataStart, dataLength: statSync(ASAR).size - dataStart, html: html.toString("utf8") };
  } finally {
    closeSync(fd);
  }
}

function stripPatch(html) {
  const from = html.indexOf(START);
  const to = html.indexOf(END);
  return from === -1 || to === -1 ? html : html.slice(0, from) + html.slice(to + END.length);
}

function integrity(bytes) {
  const sha = (chunk) => createHash("sha256").update(chunk).digest("hex");
  const blocks = [];
  for (let offset = 0; offset < bytes.length; offset += BLOCK_SIZE) blocks.push(sha(bytes.subarray(offset, offset + BLOCK_SIZE)));
  return { algorithm: "SHA256", hash: sha(bytes), blockSize: BLOCK_SIZE, blocks };
}

function encodeHeader(header) {
  const json = Buffer.from(JSON.stringify(header), "utf8");
  const padded = Math.ceil(json.length / 4) * 4;
  const out = Buffer.alloc(16 + padded);
  out.writeUInt32LE(4, 0);
  out.writeUInt32LE(padded + 8, 4);
  out.writeUInt32LE(padded + 4, 8);
  out.writeUInt32LE(json.length, 12);
  json.copy(out, 16);
  return out;
}

const archive = readArchive();
const patched = archive.html.includes(START);

if (mode === "--status") {
  console.log(`${APP}: ${patched ? "patched" : "not patched"}`);
  process.exit(0);
}
if (mode === "--restore" && !patched) {
  console.log("Not patched. Nothing to restore.");
  process.exit(0);
}

const running = execFileSync("/bin/ps", ["-axo", "command="], { encoding: "utf8", maxBuffer: 64 * 1024 * 1024 });
if (running.split("\n").some((command) => command.startsWith(`${APP}/Contents/`))) {
  console.error(`${APP} is running. Quit it, then run this again.`);
  process.exit(1);
}

let html = stripPatch(archive.html);
if (mode === "--apply") {
  const script = readFileSync(join(dirname(fileURLToPath(import.meta.url)), "inject.js"), "utf8").replaceAll("</script", "<\\/script");
  const at = html.indexOf("</head>");
  if (at === -1) throw new Error("index.html has no </head>");
  html = `${html.slice(0, at)}${START}<script>\n${script}</script>${END}${html.slice(at)}`;
}

const bytes = Buffer.from(html, "utf8");
archive.entry.size = bytes.length;
archive.entry.offset = String(archive.dataLength);
archive.entry.integrity = integrity(bytes);

const temp = `${ASAR}.t3-composer-stats`;
try {
  const out = createWriteStream(temp);
  out.write(encodeHeader(archive.header));
  await pipeline(createReadStream(ASAR, { start: archive.dataStart }), out, { end: false });
  await new Promise((resolve, reject) => out.end(bytes, (error) => (error ? reject(error) : resolve())));
  renameSync(temp, ASAR);
} catch (error) {
  rmSync(temp, { force: true });
  if (error.code === "EPERM" || error.code === "EACCES") {
    console.error(`macOS blocked the write to ${APP}.`);
    console.error("Allow your terminal in System Settings > Privacy & Security > App Management, then run this again.");
    process.exit(1);
  }
  throw error;
}
console.log(mode === "--apply" ? `Patched ${APP}. Start the app to see the stats row.` : `Restored ${APP}.`);
