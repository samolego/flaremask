import { getToken, clearToken } from './auth.js';

async function request(path, options = {}) {
  const res = await fetch('/api/v1' + path, {
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${getToken()}`,
      ...options.headers,
    },
    ...options,
  });
  if (res.status === 401) {
    clearToken();
    throw new AuthError();
  }
  if (!res.ok) throw new Error(await res.text() || `HTTP ${res.status}`);
  if (res.status === 204) return null;
  return res.json();
}

export class AuthError extends Error {}

export const api = {
  listAliases: () => request('/emails'),
  createAlias: (alias) => request('/emails', { method: 'POST', body: JSON.stringify({ alias }) }),
  updateAlias: (id, patch) => request(`/emails/${id}`, { method: 'PATCH', body: JSON.stringify(patch) }),
  deleteAlias: (id) => request(`/emails/${id}`, { method: 'DELETE' }),
};
