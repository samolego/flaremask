export const DEFAULT_ALIAS_TEMPLATE = __IS_EXTENSION__
  ? `{site}.{adj}-{noun}-{rnd}`
  : `{adj}-{noun}-{rnd}`;

export const ADJECTIVES = [
  "swift",
  "quiet",
  "brave",
  "calm",
  "bold",
  "keen",
  "sage",
  "pure",
  "dark",
  "pale",
  "wild",
  "bright",
  "crisp",
  "noble",
  "stark",
  "dusty",
  "misty",
  "silky",
  "vivid",
  "rusty",
];

export const NOUNS = [
  "fox",
  "river",
  "cloud",
  "stone",
  "flame",
  "ridge",
  "dusk",
  "pine",
  "wave",
  "frost",
  "hawk",
  "creek",
  "bloom",
  "cliff",
  "ember",
  "gale",
  "moss",
  "reed",
  "thorn",
  "vale",
];

/**
 * Resolve an alias template string.
 * Supported tags: {site}, {noun}, {adj}, {rnd}
 * @param {string} template
 * @param {string} [siteName]
 * @returns {string}
 */
export function resolveTemplate(template, siteName = "") {
  const adj = () => ADJECTIVES[Math.floor(Math.random() * ADJECTIVES.length)];
  const noun = () => NOUNS[Math.floor(Math.random() * NOUNS.length)];
  const rnd = () => String(Math.floor(Math.random() * 9000 + 1000));
  const site =
    siteName
      .toLowerCase()
      .replace(/[^a-z0-9]/g, "-")
      .replace(/^-+|-+$/g, "") || "";

  return template
    .replace(/\{site\}/g, site)
    .replace(/\{adj\}/g, adj)
    .replace(/\{noun\}/g, noun)
    .replace(/\{rnd\}/g, rnd)
    .toLowerCase()
    .replace(/[^a-z0-9._-]/g, "-")
    .replace(/-{2,}/g, "-")
    .replace(/^[-_.]+|[-_.]+$/g, "");
}

/**
 * Extract a human-readable site name from a hostname and optional page title.
 *
 * Algorithm:
 *  1. Split the page title into words; return the longest word that appears
 *     verbatim inside the hostname (case-insensitive). Ignores short words (<3 chars).
 *  2. Fallback: second-to-top-level domain segment, e.g. "mysite" from
 *     "url.mysite.com".
 *
 * @param {string} hostname
 * @param {string} [title]
 * @returns {string}
 */
export function extractSiteName(hostname, title = "") {
  const host = hostname.replace(/^www\./, "").toLowerCase();

  // Second-to-top-level domain: "mysite" from "url.mysite.com"
  const parts = host.split(".");
  const name = parts.length >= 2 ? parts[parts.length - 2] : parts[0];

  if (title) {
    const words = title
      .toLowerCase()
      .split(/[\s\-_.,|:()/\\]+/)
      .filter((w) => w.length >= 3);

    let best = "";
    for (const word of words) {
      if (host.includes(word) && word.length > best.length && best !== name) {
        best = word;
      }
    }
    if (best) return best;
  }

  return name;
}
