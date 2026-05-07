<script>
  import { onMount } from 'svelte';
  import { hostSuggestion, sortAliases, fillInput } from '../lib/aliases.js';
  import AliasList from './AliasList.svelte';

  const iconUrl = browser.runtime.getURL('icons/icon-192.png');

  let { input, onClose } = $props();

  /** @type {'loading' | 'unconfigured' | 'unauthenticated' | 'ready'} */
  let status = $state('loading');
  let suggestion = $state('');
  let aliases = $state([]);
  let aliasesLoading = $state(true);
  let creating = $state(false);
  let createError = $state('');
  let popTop = $state(0);
  let popLeft = $state(0);

  function reposition() {
    const r = input.getBoundingClientRect();
    let t = r.bottom + 4;
    let l = r.left;
    if (l + 308 > window.innerWidth) l = window.innerWidth - 312;
    if (t + 260 > window.innerHeight) t = r.top - 264;
    popTop = Math.max(4, t);
    popLeft = Math.max(4, l);
  }

  onMount(async () => {
    reposition();
    window.addEventListener('scroll', reposition, { passive: true, capture: true });
    window.addEventListener('resize', reposition, { passive: true });

    let extState;
    try { extState = await browser.runtime.sendMessage({ type: 'getState' }); }
    catch { extState = { configured: false }; }

    if (!extState?.configured) { status = 'unconfigured'; return; }
    if (!extState?.authenticated) { status = 'unauthenticated'; return; }

    status = 'ready';
    suggestion = await hostSuggestion();
    reposition();

    try {
      const res = await browser.runtime.sendMessage({ type: 'listAliases' });
      aliases = res?.ok ? sortAliases(res.aliases ?? []) : [];
    } finally {
      aliasesLoading = false;
    }

    return () => {
      window.removeEventListener('scroll', reposition, { capture: true });
      window.removeEventListener('resize', reposition);
    };
  });

  async function create() {
    const alias = suggestion.trim();
    if (!alias) return;
    creating = true;
    createError = '';
    try {
      const res = await browser.runtime.sendMessage({ type: 'createAlias', alias });
      if (res?.ok && res.alias) {
        fillInput(input, res.alias.alias);
        onClose();
      } else {
        createError = res?.error ?? 'Failed to create alias';
      }
    } catch (e) {
      createError = e.message;
    } finally {
      creating = false;
    }
  }

  function pick(alias) {
    fillInput(input, alias);
    onClose();
  }
</script>

<div class="pop" style="top: {popTop}px; left: {popLeft}px">
  <div class="header">
    <img class="logo" src={iconUrl} alt="" />
    Flaremask
  </div>

  {#if status === 'loading'}
    <div class="hint">Loading…</div>

  {:else if status === 'unconfigured'}
    <p class="msg">
      Not configured yet.<br />
      Click the <strong>Flaremask icon</strong> in your toolbar to set it up.
    </p>

  {:else if status === 'unauthenticated'}
    <p class="msg">
      Sign in to fill aliases.<br />
      Click the <strong>Flaremask icon</strong> in your toolbar to sign in.
    </p>

  {:else}
    <div class="row">
      <input
        class="fi"
        type="text"
        bind:value={suggestion}
        spellcheck="false"
        autocomplete="off"
      />
      <button class="btn" onclick={create} disabled={creating}>
        {creating ? '…' : 'Create & fill'}
      </button>
    </div>

    {#if createError}<div class="err">{createError}</div>{/if}

    <hr />

    <AliasList {aliases} {aliasesLoading} onPick={pick} />
  {/if}
</div>

<style>
  /* CSS custom properties cascade into AliasList.svelte */
  .pop {
    --fm-brand: #f6821f;
    --fm-brand-dark: #e07318;
    --fm-text: #374151;
    --fm-text-muted: #9ca3af;
    --fm-hover-bg: #fff7ed;

    position: fixed;
    background: #fff;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.13);
    padding: 12px;
    width: 300px;
    font-size: 13px;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    color: #111827;
    z-index: 2147483647;
    box-sizing: border-box;
    line-height: 1.4;
  }

  .header {
    display: flex;
    align-items: center;
    gap: 6px;
    font-weight: 600;
    margin-bottom: 10px;
  }
  .logo {
    width: 16px;
    height: 16px;
    border-radius: 3px;
    flex-shrink: 0;
  }

  .msg {
    margin: 0;
    color: #6b7280;
    font-size: 12px;
    line-height: 1.6;
  }
  .msg strong { color: #374151; }

  .row {
    display: flex;
    gap: 6px;
    margin-bottom: 6px;
  }
  .fi {
    flex: 1;
    padding: 5px 8px;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    font-size: 12px;
    font-family: monospace;
    outline: none;
    color: #111827;
    background: #fff;
    box-sizing: border-box;
  }
  .fi:focus {
    border-color: #f6821f;
    box-shadow: 0 0 0 2px rgba(246, 130, 31, 0.2);
  }

  .btn {
    padding: 5px 10px;
    background: #f6821f;
    color: #fff;
    border: none;
    border-radius: 6px;
    font-size: 12px;
    font-family: inherit;
    font-weight: 500;
    cursor: pointer;
    white-space: nowrap;
  }
  .btn:hover { background: #e07318; }
  .btn:disabled { opacity: 0.5; cursor: default; }

  .err {
    color: #ef4444;
    font-size: 11px;
    margin-bottom: 6px;
  }

  hr {
    border: none;
    border-top: 1px solid #f3f4f6;
    margin: 8px 0;
  }

  .hint {
    color: #9ca3af;
    font-size: 12px;
    text-align: center;
    padding: 6px 0;
  }
</style>
