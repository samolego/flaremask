<script>
    import {
        clearToken,
        consumeTokenFromHash,
        isTokenValid,
    } from "./lib/auth.js";
    import { api, AuthError } from "./lib/api.js";
    import {
        Plus,
        Dices,
        Clipboard,
        Check,
        UserRound,
        Trash2,
        LoaderCircle,
    } from "lucide-svelte";

    const ADJECTIVES = [
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
    ];
    const NOUNS = [
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
    ];

    function randomAlias() {
        const adj = ADJECTIVES[Math.floor(Math.random() * ADJECTIVES.length)];
        const noun = NOUNS[Math.floor(Math.random() * NOUNS.length)];
        newAlias = `${adj}-${noun}-${Math.floor(Math.random() * 90 + 10)}`;
    }

    consumeTokenFromHash();
    if (!isTokenValid()) clearToken();

    let authenticated = $state(isTokenValid());
    let aliases = $state([]);
    let destination = $state("");
    let loading = $state(false);
    let creating = $state(false);
    let togglingId = $state(null);
    let copiedId = $state(null);
    let error = $state(null);
    let newAlias = $state("");

    function logout() {
        clearToken();
        authenticated = false;
    }

    async function load() {
        loading = true;
        error = null;
        try {
            const data = await api.listAliases();
            aliases = data?.aliases ?? [];
            destination = data?.destination ?? "";
        } catch (e) {
            if (e instanceof AuthError) {
                authenticated = false;
                return;
            }
            error = e.message;
        } finally {
            loading = false;
        }
    }

    async function create() {
        const alias = newAlias.trim();
        if (!alias) return;
        creating = true;
        error = null;
        try {
            const created = await api.createAlias(alias);
            if (created) {
                aliases = [created, ...aliases];
                await navigator.clipboard.writeText(created.alias).catch(() => {});
            }
            newAlias = "";
        } catch (e) {
            if (e instanceof AuthError) {
                authenticated = false;
                return;
            }
            error = e.message;
        } finally {
            creating = false;
        }
    }

    async function toggle(alias) {
        togglingId = alias.id;
        error = null;
        try {
            const updated = await api.updateAlias(alias.id, {
                enabled: !alias.enabled,
            });
            if (updated)
                aliases = aliases.map((a) => (a.id === alias.id ? updated : a));
        } catch (e) {
            if (e instanceof AuthError) {
                authenticated = false;
                return;
            }
            error = e.message;
        } finally {
            togglingId = null;
        }
    }

    async function remove(alias) {
        if (!confirm(`Delete ${alias.alias}?`)) return;
        error = null;
        try {
            await api.deleteAlias(alias.id);
            aliases = aliases.filter((a) => a.id !== alias.id);
        } catch (e) {
            if (e instanceof AuthError) {
                authenticated = false;
                return;
            }
            error = e.message;
        }
    }

    async function copyAlias(alias) {
        await navigator.clipboard.writeText(alias.alias);
        copiedId = alias.id;
        setTimeout(() => {
            copiedId = null;
        }, 1500);
    }

    $effect(() => {
        if (authenticated) load();
    });
</script>

{#if !authenticated}
    <div class="flex min-h-screen items-center justify-center bg-gray-50">
        <div class="card w-full max-w-sm p-8">
            <div class="mb-6 flex items-center gap-2">
                <span class="h-6 w-6 rounded-full bg-brand"></span>
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
                <span class="h-5 w-5 rounded-full bg-brand"></span>
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
        {#if error}
            <div
                class="mb-6 flex items-start gap-3 rounded border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700"
            >
                <span class="mt-0.5">⚠</span><span>{error}</span>
            </div>
        {/if}

        <!-- New alias -->
        <div class="card mb-6 p-5">
            <h2 class="section-title mb-3">New alias</h2>
            <form
                onsubmit={(e) => {
                    e.preventDefault();
                    create();
                }}
                class="flex gap-2"
            >
                <input
                    bind:value={newAlias}
                    type="text"
                    placeholder="alias-name"
                    autocomplete="off"
                    autocapitalize="none"
                    required
                    class="form-input"
                />
                <button
                    type="button"
                    onclick={randomAlias}
                    title="Generate random alias"
                    class="btn-outline"
                >
                    <Dices size={16} />
                </button>
                <button type="submit" disabled={creating} class="btn-primary">
                    {#if creating}
                        <LoaderCircle size={16} class="animate-spin" />Adding…
                    {:else}
                        <Plus size={16} />Add alias
                    {/if}
                </button>
            </form>
        </div>

        <!-- Alias list -->
        <div class="card">
            <div class="card-header">
                <h2 class="section-title">Email aliases</h2>
                {#if destination}
                    <span class="badge"
                        >forwarding to <span class="font-medium text-gray-700"
                            >{destination}</span
                        ></span
                    >
                {/if}
            </div>

            {#if loading}
                <div class="px-5 py-8 text-center text-sm text-gray-400">
                    Loading…
                </div>
            {:else if aliases.length === 0}
                <div class="px-5 py-8 text-center text-sm text-gray-400">
                    No aliases yet.
                </div>
            {:else}
                <ul class="divide-y divide-gray-100">
                    {#each aliases as alias (alias.id)}
                        <li
                            class="flex items-center gap-4 px-5 py-4 {alias.is_root
                                ? 'opacity-70'
                                : ''}"
                        >
                            <button
                                onclick={() => copyAlias(alias)}
                                aria-label="Copy alias"
                                title="Copy to clipboard"
                                class={copiedId === alias.id
                                    ? "text-green-500"
                                    : "btn-icon"}
                            >
                                {#if copiedId === alias.id}<Check
                                        size={16}
                                    />{:else}<Clipboard size={16} />{/if}
                            </button>

                            <span
                                class="min-w-0 flex-1 truncate font-mono text-sm {alias.enabled
                                    ? 'text-gray-900'
                                    : 'text-gray-400'}"
                            >
                                {alias.alias}
                            </span>

                            {#if alias.is_root}
                                <span class="badge-brand">login alias</span>
                            {:else}
                                <button
                                    role="switch"
                                    aria-checked={alias.enabled}
                                    onclick={() => toggle(alias)}
                                    disabled={togglingId === alias.id}
                                    class="toggle {alias.enabled
                                        ? 'bg-brand'
                                        : 'bg-gray-200'}"
                                >
                                    {#if togglingId === alias.id}
                                        <LoaderCircle
                                            size={14}
                                            class="absolute inset-0 m-auto animate-spin text-white"
                                        />
                                    {:else}
                                        <span
                                            class="toggle-thumb {alias.enabled
                                                ? 'translate-x-4'
                                                : 'translate-x-0.5'}"
                                        ></span>
                                    {/if}
                                </button>

                                <button
                                    onclick={() => remove(alias)}
                                    aria-label="Delete alias"
                                    class="btn-danger-icon"
                                >
                                    <Trash2 size={16} />
                                </button>
                            {/if}
                        </li>
                    {/each}
                </ul>
            {/if}
        </div>
    </main>
{/if}
