import "webextension-polyfill";
import { saveToken, getToken, getWorkerUrl, loadAliasCache, saveAliasCache } from "../../lib/storage.js";
import { isTokenValid } from "../../lib/auth.js";
import { createApi } from "../../lib/api.js";

browser.runtime.onMessage.addListener((msg) => {
  if (msg.type === "auth") return handleAuth(msg.workerUrl);
  if (msg.type === "getState") return handleGetState();
  if (msg.type === "getCachedAliases") return handleGetCachedAliases();
  if (msg.type === "listAliases") return handleListAliases();
  if (msg.type === "createAlias") return handleCreateAlias(msg.alias);
});

// Register content script whenever host permission is newly granted
browser.permissions.onAdded.addListener(async (perms) => {
  if (perms.origins?.some((o) => o === "<all_urls>" || o === "*://*/*")) {
    await ensureContentScript();
  }
});

// Re-register on service worker startup
browser.runtime.onInstalled.addListener(ensureContentScript);
browser.runtime.onStartup.addListener(ensureContentScript);

async function ensureContentScript() {
  const granted = await browser.permissions.contains({
    origins: ["<all_urls>"],
  });
  if (!granted) return;
  try {
    await browser.scripting.registerContentScripts([
      {
        id: "flaremask-content",
        matches: ["<all_urls>"],
        js: ["content.js"],
        runAt: "document_idle",
        allFrames: false,
      },
    ]);
  } catch {
    // Already registered — that's fine
  }
}

async function handleAuth(workerUrl) {
  const redirectUrl = browser.identity.getRedirectURL();
  const authUrl = `${workerUrl}/auth/login?return_to=${encodeURIComponent(redirectUrl)}`;
  try {
    const resultUrl = await browser.identity.launchWebAuthFlow({
      url: authUrl,
      interactive: true,
    });
    const hash = new URL(resultUrl).hash.slice(1);
    const token = new URLSearchParams(hash).get("token");
    if (token) await saveToken(token);
    return { ok: !!token };
  } catch (e) {
    return { ok: false, error: e.message };
  }
}

async function handleGetState() {
  const workerUrl = await getWorkerUrl();
  if (!workerUrl) return { configured: false, authenticated: false };
  const token = await getToken();
  return { configured: true, authenticated: isTokenValid(token) };
}

async function handleGetCachedAliases() {
  const cached = await loadAliasCache();
  if (!cached) return { ok: false };
  return { ok: true, aliases: cached.aliases ?? [], destination: cached.destination ?? '' };
}

async function handleListAliases() {
  const workerUrl = await getWorkerUrl();
  const token = await getToken();
  if (!workerUrl || !isTokenValid(token))
    return { ok: false, error: "Not authenticated" };
  const api = createApi(workerUrl, () => token, null);
  try {
    const data = await api.listAliases();
    await saveAliasCache(data);
    return { ok: true, aliases: data?.aliases ?? [], destination: data?.destination ?? '' };
  } catch (e) {
    // Fall back to cache on network error
    const cached = await loadAliasCache();
    if (cached) return { ok: true, aliases: cached.aliases ?? [], destination: cached.destination ?? '', fromCache: true };
    return { ok: false, error: e.message };
  }
}

async function handleCreateAlias(alias) {
  const workerUrl = await getWorkerUrl();
  const token = await getToken();
  if (!workerUrl || !isTokenValid(token))
    return { ok: false, error: "Not authenticated" };
  const api = createApi(workerUrl, () => token, null);
  try {
    const created = await api.createAlias(alias);
    return { ok: true, alias: created };
  } catch (e) {
    return { ok: false, error: e.message };
  }
}
