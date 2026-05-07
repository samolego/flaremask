import { saveToken } from "./storage.js";

/** Reads /#token=... set by the worker callback, stores it, clears the URL. */
export async function consumeTokenFromHash() {
  const params = new URLSearchParams(window.location.hash.slice(1));
  const token = params.get("token");
  if (token) {
    await saveToken(token);
    history.replaceState(null, "", "/");
  }
}

export function parseTokenPayload(token) {
  try {
    return JSON.parse(atob(token.split(".")[1]));
  } catch {
    return null;
  }
}

export function isTokenValid(token) {
  if (!token) return false;
  const payload = parseTokenPayload(token);
  return payload ? payload.exp * 1000 > Date.now() : false;
}
