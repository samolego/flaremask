import { DEFAULT_ALIAS_TEMPLATE } from "./utils.js";

const TOKEN_KEY = "flaremask_token";
const WORKER_URL_KEY = "flaremask_worker_url";
const ALIAS_TEMPLATE_KEY = "flaremask_alias_template";
const ALIAS_CACHE_KEY = "flaremask_aliases_cache";

export async function getToken() {
  if (__IS_EXTENSION__) {
    const res = await browser.storage.local.get(TOKEN_KEY);
    return res[TOKEN_KEY] ?? null;
  }
  return Promise.resolve(localStorage.getItem(TOKEN_KEY));
}

export async function saveToken(token) {
  if (__IS_EXTENSION__) {
    await browser.storage.local.set({ [TOKEN_KEY]: token });
    return;
  }
  return Promise.resolve(localStorage.setItem(TOKEN_KEY, token));
}

export async function clearToken() {
  if (__IS_EXTENSION__) {
    await browser.storage.local.remove(TOKEN_KEY);
    return;
  }
  return Promise.resolve(localStorage.removeItem(TOKEN_KEY));
}

export async function getWorkerUrl() {
  if (__IS_EXTENSION__) {
    const res = await browser.storage.local.get(WORKER_URL_KEY);
    return res[WORKER_URL_KEY] ?? null;
  }
  return Promise.resolve(null);
}

export async function saveWorkerUrl(url) {
  if (__IS_EXTENSION__) {
    await browser.storage.local.set({ [WORKER_URL_KEY]: url.replace(/\/$/, "") });
    return;
  }
  return Promise.resolve();
}

export async function getAliasTemplate() {
  if (__IS_EXTENSION__) {
    const res = await browser.storage.local.get(ALIAS_TEMPLATE_KEY);
    return res[ALIAS_TEMPLATE_KEY] ?? DEFAULT_ALIAS_TEMPLATE;
  }
  return Promise.resolve(
    localStorage.getItem(ALIAS_TEMPLATE_KEY) ?? DEFAULT_ALIAS_TEMPLATE,
  );
}

export async function saveAliasTemplate(value) {
  if (__IS_EXTENSION__) {
    await browser.storage.local.set({ [ALIAS_TEMPLATE_KEY]: value });
    return;
  }
  return Promise.resolve(localStorage.setItem(ALIAS_TEMPLATE_KEY, value));
}

export async function loadAliasCache() {
  if (__IS_EXTENSION__) {
    const res = await browser.storage.local.get(ALIAS_CACHE_KEY);
    return res[ALIAS_CACHE_KEY] ?? null;
  }

  try {
    const raw = localStorage.getItem(ALIAS_CACHE_KEY);
    return Promise.resolve(raw ? JSON.parse(raw) : null);
  } catch {
    return Promise.resolve(null);
  }
}

export async function saveAliasCache(data) {
  if (__IS_EXTENSION__) {
    await browser.storage.local.set({ [ALIAS_CACHE_KEY]: data });
    return;
  }

  try {
    return Promise.resolve(
      localStorage.setItem(ALIAS_CACHE_KEY, JSON.stringify(data)),
    );
  } catch {
    return Promise.resolve();
  }
}
