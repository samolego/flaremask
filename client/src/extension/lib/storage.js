const TOKEN_KEY = 'flaremask_token';
const WORKER_URL_KEY = 'flaremask_worker_url';

export async function getToken() {
  const res = await browser.storage.local.get(TOKEN_KEY);
  return res[TOKEN_KEY] ?? null;
}

export async function saveToken(token) {
  await browser.storage.local.set({ [TOKEN_KEY]: token });
}

export async function clearToken() {
  await browser.storage.local.remove(TOKEN_KEY);
}

export async function getWorkerUrl() {
  const res = await browser.storage.local.get(WORKER_URL_KEY);
  return res[WORKER_URL_KEY] ?? null;
}

export async function saveWorkerUrl(url) {
  await browser.storage.local.set({ [WORKER_URL_KEY]: url.replace(/\/$/, '') });
}
