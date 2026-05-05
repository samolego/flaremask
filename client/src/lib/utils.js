export const ADJECTIVES = [
    'swift', 'quiet', 'brave', 'calm', 'bold',
    'keen', 'sage', 'pure', 'dark', 'pale',
];

export const NOUNS = [
    'fox', 'river', 'cloud', 'stone', 'flame',
    'ridge', 'dusk', 'pine', 'wave', 'frost',
];

export function randomAlias() {
    const adj = ADJECTIVES[Math.floor(Math.random() * ADJECTIVES.length)];
    const noun = NOUNS[Math.floor(Math.random() * NOUNS.length)];
    return `${adj}-${noun}-${Math.floor(Math.random() * 90 + 10)}`;
}

/** Decode a JWT payload without verification. Returns null on failure. */
export function parseTokenPayload(token) {
    try {
        return JSON.parse(atob(token.split('.')[1]));
    } catch {
        return null;
    }
}

export function isTokenValid(token) {
    if (!token) return false;
    const payload = parseTokenPayload(token);
    return payload ? payload.exp * 1000 > Date.now() : false;
}
