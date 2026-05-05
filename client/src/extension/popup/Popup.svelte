<script>
    import { onMount } from "svelte";
    import { createApi } from "../../lib/api.js";
    import { isTokenValid, randomAlias } from "../../lib/utils.js";
    import {
        getToken,
        clearToken,
        getWorkerUrl,
        saveWorkerUrl,
        getUseSiteUrl,
        saveUseSiteUrl,
    } from "../lib/storage.js";
    import { LoaderCircle, LogOut, Settings } from "lucide-svelte";
    import AliasManager from "../../components/AliasManager.svelte";

    let workerUrl = $state("");
    let configuring = $state(false);
    let urlInput = $state("");

    let authenticated = $state(false);
    let signingIn = $state(false);
    let error = $state(null);
    let suggestedAlias = $state("");
    let permissionGranted = $state(/** @type {boolean|null} */ (null));
    let useSiteUrl = $state(true);

    let api = $state(null);

    onMount(async () => {
        workerUrl = (await getWorkerUrl()) ?? "";
        if (!workerUrl) {
            configuring = true;
            return;
        }
        const token = await getToken();
        authenticated = isTokenValid(token);
        api = createApi(
            workerUrl,
            () => token,
            async () => {
                await clearToken();
                authenticated = false;
            },
        );

        permissionGranted = await browser.permissions.contains({
            origins: ["<all_urls>"],
        });

        useSiteUrl = await getUseSiteUrl();

        // Pre-fill alias from current tab domain
        try {
            const [tab] = await browser.tabs.query({
                active: true,
                currentWindow: true,
            });
            if (tab?.url) {
                const host = new URL(tab.url).hostname.replace(/^www\./, "");
                const site = host.split(".")[0];
                suggestedAlias = useSiteUrl
                    ? randomAlias() + "-" + site
                    : randomAlias();
            }
        } catch {
            /* no tab access */
        }
    });

    async function requestSitePermission() {
        // Fire the request then close — Firefox shows its own native permission UI
        browser.permissions.request({ origins: ["<all_urls>"] });
        window.close();
    }

    async function saveConfig() {
        const url = urlInput.trim();
        if (!url) return;
        await saveWorkerUrl(url);
        await saveUseSiteUrl(useSiteUrl);
        workerUrl = url;
        configuring = false;
        api = createApi(workerUrl, getToken, async () => {
            await clearToken();
            authenticated = false;
        });
    }

    async function signIn() {
        signingIn = true;
        error = null;
        try {
            const res = await browser.runtime.sendMessage({
                type: "auth",
                workerUrl,
            });
            if (res?.ok) {
                const token = await getToken();
                authenticated = isTokenValid(token);
            } else {
                error = res?.error ?? "Sign-in failed";
            }
        } catch (e) {
            error = "Sign-in failed: " + e.message;
        } finally {
            signingIn = false;
        }
    }

    async function logout() {
        await clearToken();
        authenticated = false;
        api = null;
    }
</script>

<!-- Configure screen -->
{#if configuring}
    <div class="flex min-h-50 flex-col justify-center p-6">
        <div class="mb-4 flex items-center justify-between">
            <div class="flex items-center gap-2">
                <img src="../icons/icon-192.png" class="h-5 w-5 rounded" alt="Flaremask" />
                <span class="font-semibold text-gray-900">Flaremask</span>
            </div>
            {#if authenticated}
                <button
                    onclick={logout}
                    class="btn-icon text-xs text-gray-400 hover:text-red-500"
                    title="Sign out"
                >
                    <LogOut size={13} />Sign out
                </button>
            {/if}
        </div>
        <p class="mb-4 text-sm text-gray-500">
            Enter your worker URL to get started.
        </p>
        <form
            onsubmit={(e) => {
                e.preventDefault();
                saveConfig();
            }}
            class="flex flex-col gap-2"
        >
            <input
                bind:value={urlInput}
                type="url"
                placeholder="https://mask.example.com"
                required
                class="form-input"
            />
            <label class="flex cursor-pointer items-center gap-2 py-1 text-sm text-gray-600">
                <input
                    type="checkbox"
                    bind:checked={useSiteUrl}
                    class="accent-brand h-3.5 w-3.5 cursor-pointer rounded"
                />
                Include site name in suggestions
            </label>
            <button type="submit" class="btn-primary justify-center"
                >Save</button
            >
        </form>
    </div>

    <!-- Sign-in screen -->
{:else if !authenticated}
    <div class="flex min-h-50 flex-col justify-center p-6">
        <div class="mb-4 flex items-center gap-2">
            <img src="../icons/icon-192.png" class="h-5 w-5 rounded" alt="Flaremask" />
            <span class="font-semibold text-gray-900">Flaremask</span>
        </div>
        <p class="mb-4 text-sm text-gray-500">
            Sign in to manage your email aliases.
        </p>
        <button
            onclick={signIn}
            disabled={signingIn}
            class="btn-primary justify-center"
        >
            {#if signingIn}<LoaderCircle size={16} class="animate-spin" />{/if}
            {signingIn ? "Signing in…" : "Sign in"}
        </button>
        <button
            onclick={() => {
                configuring = true;
                urlInput = workerUrl;
            }}
            class="btn-icon mt-3 self-end text-xs text-gray-400"
        >
            <Settings size={13} />
            {workerUrl}
        </button>
    </div>

    <!-- Main UI -->
{:else}
    <div class="w-90">
        <nav
            class="flex items-center justify-between border-b border-gray-200 px-4 py-2"
        >
            <div class="flex items-center gap-2">
                <img src="../icons/icon-192.png" class="h-4 w-4 rounded" alt="Flaremask" />
                <span class="text-sm font-semibold text-gray-900"
                    >Flaremask</span
                >
            </div>
            <div class="flex items-center gap-2">
                <button
                    onclick={() => {
                        configuring = true;
                        urlInput = workerUrl;
                    }}
                    class="btn-icon"
                    title="Settings"
                >
                    <Settings size={14} />
                </button>
            </div>
        </nav>

        <div class="p-4">
            {#if permissionGranted === false}
                <div
                    class="mb-3 flex items-start gap-2 rounded-lg border border-amber-200 bg-amber-50 p-3"
                >
                    <span class="mt-0.5 text-amber-500">⚠</span>
                    <div class="flex-1 text-xs text-amber-800">
                        <p class="mb-1.5 font-medium">
                            Website access not granted
                        </p>
                        <button
                            onclick={requestSitePermission}
                            class="rounded bg-amber-500 px-2.5 py-1 text-xs font-medium text-white hover:bg-amber-600"
                        >
                            Enable on all websites
                        </button>
                    </div>
                </div>
            {/if}
            <AliasManager {api} compact initialAlias={suggestedAlias} />
        </div>
    </div>
{/if}
