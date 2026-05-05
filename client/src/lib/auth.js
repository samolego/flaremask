const TOKEN_KEY = 'flaremask_token';

export function getToken() {
  return localStorage.getItem(TOKEN_KEY);
}

export function saveToken(token) {
  localStorage.setItem(TOKEN_KEY, token);
}

export function clearToken() {
  localStorage.removeItem(TOKEN_KEY);
}

export function isTokenValid() {
  const token = getToken();
  if (!token) return false;
  try {
    const { exp } = JSON.parse(atob(token.split('.')[1]));
    return exp * 1000 > Date.now();
  } catch {
    return false;
  }
}

/** Reads /#token=... set by the worker callback, stores it, clears the URL. */
export function consumeTokenFromHash() {
  const params = new URLSearchParams(window.location.hash.slice(1));
  const token = params.get('token');
  if (token) {
    saveToken(token);
    history.replaceState(null, '', '/');
  }
}
