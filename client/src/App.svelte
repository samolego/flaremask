<script>
    import { onMount } from "svelte";
    import { consumeTokenFromHash, isTokenValid } from "./lib/auth.js";
    import {
        clearToken,
        getAliasTemplate,
        getToken,
        saveAliasTemplate,
    } from "./lib/storage.js";
    import { createApi } from "./lib/api.js";
    import { LogOut, Settings } from "lucide-svelte";
    import AliasManager from "./components/AliasManager.svelte";
    import SettingsPanel from "./components/SettingsPanel.svelte";

    consumeTokenFromHash();
    let authenticated = $state(false);
    let loading = $state(true);
    let showSettings = $state(false);
    let aliasTemplate = $state("");
    let editTemplate = $state("");

    const api = createApi("", getToken, async () => {
        await clearToken();
        authenticated = false;
    });

    onMount(async () => {
        const token = await getToken();
        authenticated = isTokenValid(token);
        if (!authenticated) await clearToken();
        aliasTemplate = await getAliasTemplate();
        loading = false;
    });

    async function logout() {
        await clearToken();
        authenticated = false;
    }

    function openSettings() {
        editTemplate = aliasTemplate;
        showSettings = true;
    }

    async function saveSettings() {
        aliasTemplate = editTemplate;
        await saveAliasTemplate(aliasTemplate);
        showSettings = false;
    }
</script>

{#if loading}
    <div class="flex min-h-screen items-center justify-center bg-gray-50">
        <div class="text-sm text-gray-400">Loading…</div>
    </div>
{:else if !authenticated}
    <div class="flex min-h-screen items-center justify-center bg-gray-50">
        <div class="card w-full max-w-sm p-8">
            <div class="mb-6 flex items-center gap-2">
                <img
                    src="/icons/icon-192.png"
                    class="h-6 w-6 rounded"
                    alt="Flaremask"
                />
                <span class="text-lg font-semibold text-gray-900"
                    >Flaremask</span
                >
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
        <div
            class="mx-auto flex max-w-3xl items-center justify-between px-4 py-3"
        >
            <div class="flex items-center gap-2">
                <img
                    src="/icons/icon-192.png"
                    class="h-5 w-5 rounded"
                    alt="Flaremask"
                />
                <span class="font-semibold text-gray-900">Flaremask</span>
            </div>
            <div class="flex items-center gap-1">
                <button
                    onclick={() =>
                        showSettings ? (showSettings = false) : openSettings()}
                    class="btn-icon"
                    title="Settings"
                    aria-label="Settings"
                >
                    <Settings size={16} />
                </button>
                <button
                    onclick={logout}
                    class="btn-icon flex items-center gap-1 text-xs text-gray-400 hover:text-red-500"
                    title="Sign out"
                >
                    <LogOut size={13} />Sign out
                </button>
            </div>
        </div>
    </nav>

    <main class="mx-auto max-w-3xl px-4 py-8">
        {#if showSettings}
            <div class="card mb-6 p-5">
                <h2 class="section-title mb-4">Settings</h2>
                <SettingsPanel
                    bind:aliasTemplate={editTemplate}
                    showSiteTag={false}
                    showWorkerUrl={false}
                    onSave={saveSettings}
                />
            </div>
        {/if}

        <AliasManager {api} {aliasTemplate} />
    </main>
{/if}
