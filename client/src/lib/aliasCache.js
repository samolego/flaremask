const CACHE_KEY = 'flaremask_aliases_cache';

export function loadAliasCache() {
    try {
        const raw = localStorage.getItem(CACHE_KEY);
        return raw ? JSON.parse(raw) : null;
    } catch {
        return null;
    }
}

export function saveAliasCache(data) {
    try {
        localStorage.setItem(CACHE_KEY, JSON.stringify(data));
    } catch { /* storage full or unavailable */ }
}
