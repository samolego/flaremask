import { extractSiteName } from "./templates.js";
import { loadAliasCache, saveAliasCache } from "./storage.js";

export function sortAliases(aliases) {
  const siteName = extractSiteName(location.hostname, document.title);
  return [...aliases].sort((a, b) => {
    const aMatch = a.alias.toLowerCase().includes(siteName) ? 0 : 1;
    const bMatch = b.alias.toLowerCase().includes(siteName) ? 0 : 1;
    if (aMatch !== bMatch) return aMatch - bMatch;
    return (a.is_root ? 1 : 0) - (b.is_root ? 1 : 0);
  });
}

function aliasesChanged(prev, next) {
  if (prev.length !== next.length) return true;
  const prevMap = new Map(prev.map((a) => [a.id, a.enabled]));
  return next.some(
    (a) => !prevMap.has(a.id) || prevMap.get(a.id) !== a.enabled,
  );
}

export async function getAliases(api, { onUpdate, onError } = {}) {
  let cachedAliases = [];
  let cachedDestination = "";

  const cached = await loadAliasCache();
  if (cached) {
    cachedAliases = cached.aliases ?? [];
    cachedDestination = cached.destination ?? "";
    onUpdate?.(cachedAliases, cachedDestination, { fromCache: true });
  }

  try {
    const data = await api.listAliases();
    const fresh = data?.aliases ?? [];
    const freshDest = data?.destination ?? "";

    if (
      !cached ||
      aliasesChanged(cachedAliases, fresh) ||
      freshDest !== cachedDestination
    ) {
      onUpdate?.(fresh, freshDest, { fromCache: false });
    }

    await saveAliasCache(data);
  } catch (e) {
    onError?.(e);
  }
}
