import { resolveTemplate, extractSiteName } from "../../lib/utils.js";
import { DEFAULT_ALIAS_TEMPLATE } from "./storage.js";

export async function hostSuggestion() {
  const res = await browser.storage.local.get('flaremask_alias_template');
  const template = res['flaremask_alias_template'] ?? DEFAULT_ALIAS_TEMPLATE;
  const siteName = extractSiteName(location.hostname, document.title);
  return resolveTemplate(template, siteName);
}

export function sortAliases(aliases) {
  const siteName = extractSiteName(location.hostname, document.title);
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
  const setter = Object.getOwnPropertyDescriptor(
    HTMLInputElement.prototype,
    "value",
  )?.set;
  if (setter) setter.call(input, value);
  else input.value = value;
  input.dispatchEvent(new Event("input", { bubbles: true }));
  input.dispatchEvent(new Event("change", { bubbles: true }));
}
