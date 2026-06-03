export class AuthError extends Error {}
export class RateLimitError extends Error {}

const EMAILS_ENDPOINT = "emails";

export function createApi(baseUrl, getToken, onAuthError) {
  async function request(path, options = {}) {
    const token = await getToken();
    const res = await fetch(baseUrl + "/api/v1" + path, {
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
        ...options.headers,
      },
      ...options,
    });
    if (res.status === 401) {
      if (onAuthError) onAuthError();
      throw new AuthError();
    }
    if (res.status === 429) throw new RateLimitError((await res.text()) || "Too many requests");
    if (!res.ok) throw new Error((await res.text()) || `HTTP ${res.status}`);
    if (res.status === 204) return null;
    return res.json();
  }

  return {
    listAliases: () => request(`/${EMAILS_ENDPOINT}`),
    createAlias: (alias) =>
      request(`/${EMAILS_ENDPOINT}`, {
        method: "POST",
        body: JSON.stringify({ alias }),
      }),
    updateAlias: (id, patch) =>
      request(`/${EMAILS_ENDPOINT}/${id}`, {
        method: "PATCH",
        body: JSON.stringify(patch),
      }),
    deleteAlias: (id) =>
      request(`/${EMAILS_ENDPOINT}/${id}`, { method: "DELETE" }),
    sendVerificationEmail: () =>
      request(`/${EMAILS_ENDPOINT}/verify`, { method: "POST" }),
  };
}
