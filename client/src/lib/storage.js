import { DEFAULT_ALIAS_TEMPLATE } from "./templates.js";

const TOKEN_KEY = "flaremask_token";
const WORKER_URL_KEY = "flaremask_worker_url";
const ALIAS_TEMPLATE_KEY = "flaremask_alias_template";
const ALIAS_CACHE_KEY = "flaremask_aliases_cache";

async function get(key, fallback = null) {
  if (__IS_EXTENSION__) {
    const res = await browser.storage.local.get(key);
    return res[key] ?? fallback;
  } else {
    try {
      const raw = localStorage.getItem(key);
      return raw !== null ? JSON.parse(raw) : fallback;
    } catch {
      return fallback;
    }
  }
}

async function save(key, value) {
  if (__IS_EXTENSION__) {
    return browser.storage.local.set({ [key]: value });
  } else {
    try {
      localStorage.setItem(key, JSON.stringify(value));
    } catch {
      /* quota */
    }
  }
}

async function remove(key) {
  if (__IS_EXTENSION__) return browser.storage.local.remove(key);
  else localStorage.removeItem(key);
}

export const getToken = () => get(TOKEN_KEY);
export const saveToken = (token) => save(TOKEN_KEY, token);
export const clearToken = () => remove(TOKEN_KEY);

export const getWorkerUrl = () => get(WORKER_URL_KEY);
export const saveWorkerUrl = (url) =>
  save(WORKER_URL_KEY, url.replace(/\/$/, ""));

export const getAliasTemplate = () =>
  get(ALIAS_TEMPLATE_KEY, DEFAULT_ALIAS_TEMPLATE);
export const saveAliasTemplate = (value) => save(ALIAS_TEMPLATE_KEY, value);

export const loadAliasCache = () => get(ALIAS_CACHE_KEY);
export const saveAliasCache = (data) => save(ALIAS_CACHE_KEY, data);
