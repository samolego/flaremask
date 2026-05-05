<script>
    import { clearToken, consumeTokenFromHash, isTokenValid, getToken } from "./lib/auth.js";
    import { createApi } from "./lib/api.js";
    import { UserRound } from "lucide-svelte";
    import AliasManager from "./components/AliasManager.svelte";

    consumeTokenFromHash();
    if (!isTokenValid()) clearToken();

    let authenticated = $state(isTokenValid());

    const api = createApi('', getToken, () => { clearToken(); authenticated = false; });

    function logout() {
        clearToken();
        authenticated = false;
    }
</script>

{#if !authenticated}
    <div class="flex min-h-screen items-center justify-center bg-gray-50">
        <div class="card w-full max-w-sm p-8">
            <div class="mb-6 flex items-center gap-2">
                <img src="/icons/icon-192.png" class="h-6 w-6 rounded" alt="Flaremask" />
                <span class="text-lg font-semibold text-gray-900">Flaremask</span>
            </div>
            <p class="mb-6 text-sm text-gray-500">
                Sign in to manage your email aliases.
            </p>
            <a
                href="/auth/login"
                class="btn-primary block w-full justify-center py-2 text-center"
                >Sign in</a
            >
        </div>
    </div>
{:else}
    <nav class="border-b border-gray-200 bg-white">
        <div class="mx-auto flex max-w-3xl items-center justify-between px-4 py-3">
            <div class="flex items-center gap-2">
                <img src="/icons/icon-192.png" class="h-5 w-5 rounded" alt="Flaremask" />
                <span class="font-semibold text-gray-900">Flaremask</span>
            </div>
            <button
                onclick={logout}
                class="flex items-center gap-1.5 text-sm text-gray-500 transition-colors hover:text-gray-800"
            >
                <UserRound size={16} />Sign out
            </button>
        </div>
    </nav>

    <main class="mx-auto max-w-3xl px-4 py-8">
        <AliasManager {api} />
    </main>
{/if}
