import { resolveTemplate, extractSiteName } from "../../lib/templates.js";
import { getAliasTemplate } from "../../lib/storage.js";

export { sortAliases } from "../../lib/aliases.js";

export async function hostSuggestion() {
  const template = await getAliasTemplate();
  const siteName = extractSiteName(location.hostname, document.title);
  return resolveTemplate(template, siteName);
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
