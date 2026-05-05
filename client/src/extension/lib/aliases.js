import { randomAlias } from '../../lib/utils.js';

export async function hostSuggestion() {
  const res = await browser.storage.local.get('flaremask_use_site_url');
  const useSiteUrl = res['flaremask_use_site_url'] ?? true;
  if (!useSiteUrl) return randomAlias();
  const name =
    location.hostname
      .replace(/^www\./, '')
      .split('.')[0]
      .toLowerCase()
      .replace(/[^a-z0-9]/g, '-')
      .replace(/^-+|-+$/g, '') || 'mail';
  return randomAlias() + '-' + name;
}

export function sortAliases(aliases) {
  const siteName = location.hostname
    .replace(/^www\./, '')
    .split('.')[0]
    .toLowerCase();
  return [...aliases].sort((a, b) => {
    const aMatch = a.alias.toLowerCase().includes(siteName) ? 0 : 1;
    const bMatch = b.alias.toLowerCase().includes(siteName) ? 0 : 1;
    if (aMatch !== bMatch) return aMatch - bMatch;
    // roots sink to the bottom within each group
    return (a.is_root ? 1 : 0) - (b.is_root ? 1 : 0);
  });
}

/** Fill an input in a way that React/Vue/Svelte forms detect. */
export function fillInput(input, value) {
  const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set;
  if (setter) setter.call(input, value);
  else input.value = value;
  input.dispatchEvent(new Event('input', { bubbles: true }));
  input.dispatchEvent(new Event('change', { bubbles: true }));
}
