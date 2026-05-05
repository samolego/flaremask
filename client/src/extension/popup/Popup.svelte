<script>
    import { onMount } from "svelte";
    import { createApi } from "../../lib/api.js";
    import { isTokenValid, randomAlias } from "../../lib/utils.js";
    import {
        getToken,
        clearToken,
        getWorkerUrl,
        saveWorkerUrl,
    } from "../lib/storage.js";
    import { LoaderCircle, Settings, UserRound } from "lucide-svelte";
    import AliasManager from "../../components/AliasManager.svelte";

    let workerUrl = $state("");
    let configuring = $state(false);
    let urlInput = $state("");

    let authenticated = $state(false);
    let signingIn = $state(false);
    let error = $state(null);
    let suggestedAlias = $state("");

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

        // Pre-fill alias from current tab domain
        try {
            const [tab] = await browser.tabs.query({
                active: true,
                currentWindow: true,
            });
            if (tab?.url) {
                const host = new URL(tab.url).hostname.replace(/^www\./, "");
                suggestedAlias = randomAlias() + "-" + host.split(".")[0];
            }
        } catch {
            /* no tab access */
        }
    });

    async function saveConfig() {
        const url = urlInput.trim();
        if (!url) return;
        await saveWorkerUrl(url);
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
        <div class="mb-4 flex items-center gap-2">
            <span class="h-5 w-5 rounded-full bg-brand"></span>
            <span class="font-semibold text-gray-900">Flaremask</span>
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
            <button type="submit" class="btn-primary justify-center"
                >Save</button
            >
        </form>
    </div>

    <!-- Sign-in screen -->
{:else if !authenticated}
    <div class="flex min-h-50 flex-col justify-center p-6">
        <div class="mb-4 flex items-center gap-2">
            <span class="h-5 w-5 rounded-full bg-brand"></span>
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
                <span class="h-4 w-4 rounded-full bg-brand"></span>
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
                <button
                    onclick={logout}
                    class="flex items-center gap-1 text-xs text-gray-500 hover:text-gray-800"
                    title="Sign out"
                >
                    <UserRound size={14} />Sign out
                </button>
            </div>
        </nav>

        <div class="p-4">
            <AliasManager {api} compact initialAlias={suggestedAlias} />
        </div>
    </div>
{/if}
