import 'webextension-polyfill';
import { saveToken, getWorkerUrl } from '../lib/storage.js';

// Auth is initiated by the popup (which closes when it loses focus).
// We handle launchWebAuthFlow here so it survives the popup being destroyed.
browser.runtime.onMessage.addListener((msg) => {
  if (msg.type === 'auth') {
    return handleAuth(msg.workerUrl);
  }
});

async function handleAuth(workerUrl) {
  const redirectUrl = browser.identity.getRedirectURL();
  const authUrl = `${workerUrl}/auth/login?return_to=${encodeURIComponent(redirectUrl)}`;
  try {
    const resultUrl = await browser.identity.launchWebAuthFlow({
      url: authUrl,
      interactive: true,
    });
    const hash = new URL(resultUrl).hash.slice(1);
    const token = new URLSearchParams(hash).get('token');
    if (token) await saveToken(token);
    return { ok: !!token };
  } catch (e) {
    return { ok: false, error: e.message };
  }
}
