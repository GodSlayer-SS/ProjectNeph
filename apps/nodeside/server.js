import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import net from "node:net";
import { chromium } from "playwright";

const PIPE_NAME = "\\\\.\\pipe\\NephNodeSide";
const BASE_PROFILE_DIR = path.join(os.homedir(), ".nephis", "browser-profiles");

const PROFILE_MAP = {
  "nephis-research": "nephis-research",
  "nephis-tools": "nephis-tools",
  "nephis-personal": "nephis-personal",
  "nephis-throwaway": "nephis-throwaway"
};

/** @type {Map<string, import("playwright").BrowserContext>} */
const contexts = new Map();

async function ensureContext(profile) {
  const key = PROFILE_MAP[profile] ?? "nephis-research";
  if (contexts.has(key)) return contexts.get(key);
  const dir = path.join(BASE_PROFILE_DIR, key);
  fs.mkdirSync(dir, { recursive: true });
  const ctx = await chromium.launchPersistentContext(dir, { headless: true });
  contexts.set(key, ctx);
  return ctx;
}

async function browserReadPage(params) {
  const profile = params.profile || "nephis-research";
  const url = params.url;
  if (!url) throw new Error("url is required");
  const ctx = await ensureContext(profile);
  const page = await ctx.newPage();
  await page.goto(url, { waitUntil: "domcontentloaded", timeout: 15000 });
  const title = await page.title();
  const text = await page.locator("body").innerText().catch(() => "");
  await page.close();
  return { title, text: String(text).slice(0, 6000), url };
}

async function browserSearch(params) {
  const profile = params.profile || "nephis-research";
  const query = params.query;
  if (!query) throw new Error("query is required");
  const ctx = await ensureContext(profile);
  const page = await ctx.newPage();
  const url = `https://duckduckgo.com/?q=${encodeURIComponent(query)}`;
  await page.goto(url, { waitUntil: "domcontentloaded", timeout: 15000 });
  const title = await page.title();
  const text = await page.locator("body").innerText().catch(() => "");
  await page.close();
  return { title, text: String(text).slice(0, 4000), url };
}

async function browserClick(params) {
  const profile = params.profile || "nephis-tools";
  const url = params.url;
  const selector = params.selector;
  if (!url || !selector) throw new Error("url and selector are required");
  const ctx = await ensureContext(profile);
  const page = await ctx.newPage();
  await page.goto(url, { waitUntil: "domcontentloaded", timeout: 15000 });
  await page.locator(selector).first().click({ timeout: 10000 });
  const title = await page.title();
  const text = await page.locator("body").innerText().catch(() => "");
  await page.close();
  return { ok: true, title, text: String(text).slice(0, 3000), url };
}

async function browserFillForm(params) {
  const profile = params.profile || "nephis-tools";
  const url = params.url;
  const fields = params.fields || {};
  const submitSelector = params.submit_selector || null;
  if (!url) throw new Error("url is required");
  const ctx = await ensureContext(profile);
  const page = await ctx.newPage();
  await page.goto(url, { waitUntil: "domcontentloaded", timeout: 15000 });
  for (const [selector, value] of Object.entries(fields)) {
    await page.locator(selector).first().fill(String(value), { timeout: 10000 });
  }
  if (submitSelector) {
    await page.locator(submitSelector).first().click({ timeout: 10000 });
  }
  const title = await page.title();
  const text = await page.locator("body").innerText().catch(() => "");
  await page.close();
  return { ok: true, title, text: String(text).slice(0, 3000), url };
}

async function dispatch(method, params) {
  switch (method) {
    case "ping":
      return { pong: true };
    case "browser.read_page":
      return await browserReadPage(params || {});
    case "browser.search":
      return await browserSearch(params || {});
    case "browser.click":
      return await browserClick(params || {});
    case "browser.fill_form":
      return await browserFillForm(params || {});
    default:
      throw new Error(`Unknown method: ${method}`);
  }
}

function send(socket, obj) {
  socket.write(`${JSON.stringify(obj)}\n`);
}

const server = net.createServer((socket) => {
  let buf = "";
  socket.setEncoding("utf8");

  socket.on("data", async (chunk) => {
    buf += chunk;
    while (buf.includes("\n")) {
      const idx = buf.indexOf("\n");
      const line = buf.slice(0, idx).trim();
      buf = buf.slice(idx + 1);
      if (!line) continue;
      let req;
      try {
        req = JSON.parse(line);
        const result = await dispatch(req.method, req.params || {});
        send(socket, { jsonrpc: "2.0", id: req.id, result });
      } catch (e) {
        send(socket, {
          jsonrpc: "2.0",
          id: req?.id ?? null,
          error: { code: -32000, message: String(e?.message || e) }
        });
      }
    }
  });
});

try { fs.unlinkSync(PIPE_NAME); } catch {}
server.listen(PIPE_NAME);

