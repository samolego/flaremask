<script>
    import { onMount, untrack } from "svelte";
    import { AuthError } from "../lib/api.js";
    import { getAliases, sortAliases } from "../lib/aliases.js";
    import {
        resolveTemplate,
        DEFAULT_ALIAS_TEMPLATE,
    } from "../lib/templates.js";
    import {
        Plus,
        Dices,
        Clipboard,
        Check,
        Trash2,
        LoaderCircle,
        TriangleAlert,
    } from "lucide-svelte";

    import Toggle from "./Toggle.svelte";

    let {
        api,
        compact = false,
        initialAlias = "",
        aliasTemplate = DEFAULT_ALIAS_TEMPLATE,
        siteName = "",
    } = $props();

    // Icon size varies between full-page (16) and compact extension popup (14)
    const sz = $derived(compact ? 14 : 16);

    let aliases = $state([]);
    let destination = $state("");
    let loading = $state(true);
    let creating = $state(false);
    let togglingId = $state(null);
    let copiedId = $state(null);
    let error = $state(null);
    let newAlias = $state(untrack(() => initialAlias));

    onMount(load);

    async function load() {
        error = null;
        await getAliases(api, {
            onUpdate(fresh, dest, { fromCache }) {
                aliases = sortAliases(fresh);
                destination = dest;
                loading = false;
            },
            onError(e) {
                if (!(e instanceof AuthError)) error = e.message;
                loading = false;
            },
        });
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
                await navigator.clipboard
                    .writeText(created.alias)
                    .catch(() => {});
            }
            newAlias = "";
        } catch (e) {
            if (!(e instanceof AuthError)) error = e.message;
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
            if (!(e instanceof AuthError)) error = e.message;
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
            if (!(e instanceof AuthError)) error = e.message;
        }
    }

    async function copyAlias(alias) {
        await navigator.clipboard.writeText(alias.alias);
        copiedId = alias.id;
        setTimeout(() => {
            copiedId = null;
        }, 1500);
    }
</script>

{#if error}
    <div
        class="mb-4 flex items-start gap-3 rounded border border-red-200 bg-red-50 px-3 py-2 text-red-700 {compact
            ? 'text-xs'
            : 'text-sm'}"
    >
        <TriangleAlert size={sz} class="mt-0.5 shrink-0" /><span>{error}</span>
    </div>
{/if}

<!-- New alias -->
<form
    onsubmit={(e) => {
        e.preventDefault();
        create();
    }}
    class="flex gap-2 {compact ? 'mb-4' : 'mb-6'}"
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
        onclick={() => (newAlias = resolveTemplate(aliasTemplate, siteName))}
        title="Generate random alias"
        class="btn-outline"
    >
        <Dices size={sz} />
    </button>
    <button type="submit" disabled={creating} class="btn-primary">
        {#if creating}
            <LoaderCircle size={sz} class="animate-spin" />
            {#if !compact}Adding…{/if}
        {:else}
            <Plus size={sz} />
            {#if !compact}Add alias{/if}
        {/if}
    </button>
</form>

<!-- Alias list -->
<div class="card">
    <div class="card-header">
        <h2
            class={compact
                ? "text-xs font-semibold text-gray-700"
                : "section-title"}
        >
            {compact ? "Aliases" : "Email aliases"}
        </h2>
        {#if destination}
            <span class="badge">
                {#if compact}→ {destination}{:else}forwarding to <span
                        class="font-medium text-gray-700">{destination}</span
                    >{/if}
            </span>
        {/if}
    </div>

    {#if loading}
        <div
            class="px-5 py-8 text-center {compact
                ? 'text-xs'
                : 'text-sm'} text-gray-400"
        >
            Loading…
        </div>
    {:else if aliases.length === 0}
        <div
            class="px-5 py-8 text-center {compact
                ? 'text-xs'
                : 'text-sm'} text-gray-400"
        >
            No aliases yet.
        </div>
    {:else}
        <ul class="divide-y divide-gray-100">
            {#each aliases as alias (alias.id)}
                <li
                    class="flex items-center {compact
                        ? 'gap-2 px-3 py-2'
                        : 'gap-4 px-5 py-4'} {alias.is_root
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
                        {#if copiedId === alias.id}
                            <Check size={sz} />
                        {:else}
                            <Clipboard size={sz} />
                        {/if}
                    </button>

                    <span
                        class="min-w-0 flex-1 truncate font-mono {compact
                            ? 'text-xs'
                            : 'text-sm'} {alias.enabled
                            ? 'text-gray-900'
                            : 'text-gray-400'}"
                    >
                        {alias.alias}
                    </span>

                    {#if alias.is_root}
                        <span class="badge-brand"
                            >{compact ? "login" : "login alias"}</span
                        >
                    {:else}
                        <Toggle
                            checked={alias.enabled}
                            disabled={togglingId === alias.id}
                            onchange={() => toggle(alias)}
                        />

                        <button
                            onclick={() => remove(alias)}
                            aria-label="Delete alias"
                            class="btn-danger-icon"
                        >
                            <Trash2 size={sz} />
                        </button>
                    {/if}
                </li>
            {/each}
        </ul>
    {/if}
</div>
