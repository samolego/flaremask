<script>
    import { onMount } from "svelte";
    import { createApi } from "../../lib/api.js";
    import {
        resolveTemplate,
        extractSiteName,
        DEFAULT_ALIAS_TEMPLATE,
    } from "../../lib/templates.js";
    import { isTokenValid } from "../../lib/auth.js";
    import {
        getToken,
        clearToken,
        getWorkerUrl,
        saveWorkerUrl,
        getAliasTemplate,
        saveAliasTemplate,
    } from "../../lib/storage.js";
    import { LoaderCircle, Settings } from "lucide-svelte";
    import AliasManager from "../../components/alias/AliasManager.svelte";
    import OnboardingView from "./OnboardingView.svelte";
    import SettingsView from "./SettingsView.svelte";

    /** @type {'main' | 'onboarding' | 'settings' | 'signin'} */
    let screen = $state("main");

    let workerUrl = $state("");
    let urlInput = $state("");
    let aliasTemplate = $state(DEFAULT_ALIAS_TEMPLATE);

    let authenticated = $state(false);
    let signingIn = $state(false);
    let error = $state(null);
    let suggestedAlias = $state("");
    let siteName = $state("");
    let permissionGranted = $state(/** @type {boolean|null} */ (null));

    let api = $state(null);
    let loading = $state(true);

    /** Ensure the URL has a https:// prefix. */
    function normalizeUrl(raw) {
        const s = raw.trim();
        if (!s) return s;
        if (/^https?:\/\//i.test(s)) return s;
        return "https://" + s;
    }

    onMount(async () => {
        workerUrl = (await getWorkerUrl()) ?? "";
        aliasTemplate = await getAliasTemplate();

        if (!workerUrl) {
            loading = false;
            screen = "onboarding";
            return;
        }

        const token = await getToken();
        authenticated = isTokenValid(token);
        if (!authenticated) {
            loading = false;
            screen = "signin";
            return;
        }

        api = createApi(
            workerUrl,
            () => token,
            async () => {
                await clearToken();
                authenticated = false;
                screen = "signin";
            },
        );

        permissionGranted = await browser.permissions.contains({
            origins: ["<all_urls>"],
        });

        try {
            const [tab] = await browser.tabs.query({
                active: true,
                windowType: "normal",
            });
            if (tab?.url) {
                siteName = extractSiteName(
                    new URL(tab.url).hostname,
                    tab.title ?? "",
                );
                suggestedAlias = resolveTemplate(aliasTemplate, siteName);
            }
        } catch {
            /* no tab access */
        }

        loading = false;
    });

    async function finishOnboarding() {
        const url = normalizeUrl(urlInput);
        if (!url) return;
        urlInput = url;
        await saveWorkerUrl(url);
        workerUrl = url;
        api = createApi(workerUrl, getToken, async () => {
            await clearToken();
            authenticated = false;
            screen = "signin";
        });
        const token = await getToken();
        authenticated = isTokenValid(token);
        screen = authenticated ? "main" : "signin";
    }

    async function saveSettings() {
        workerUrl = normalizeUrl(workerUrl);
        await saveWorkerUrl(workerUrl);
        await saveAliasTemplate(aliasTemplate);
        api = createApi(workerUrl, getToken, async () => {
            await clearToken();
            authenticated = false;
            screen = "signin";
        });
        screen = authenticated ? "main" : "signin";
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
                if (authenticated) {
                    api = createApi(
                        workerUrl,
                        () => token,
                        async () => {
                            await clearToken();
                            authenticated = false;
                            screen = "signin";
                        },
                    );
                    screen = "main";
                } else {
                    error = "Sign-in failed";
                }
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
        screen = "signin";
    }

    async function requestSitePermission() {
        browser.permissions.request({ origins: ["<all_urls>"] });
        window.close();
    }
</script>

{#if loading}
    <div class="flex min-h-32 items-center justify-center">
        <LoaderCircle size={20} class="animate-spin text-brand" />
    </div>
{:else if screen === "onboarding"}
    <OnboardingView bind:urlInput onSave={finishOnboarding} />
{:else if screen === "settings"}
    <SettingsView
        bind:workerUrl
        bind:aliasTemplate
        {authenticated}
        onSave={saveSettings}
        onBack={() => (screen = authenticated ? "main" : "signin")}
        onLogout={logout}
    />
{:else if screen === "signin"}
    <div class="flex min-h-50 flex-col justify-center p-6">
        <div class="mb-4 flex items-center gap-2">
            <img
                src="../icons/icon-192.png"
                class="h-5 w-5 rounded"
                alt="Flaremask"
            />
            <span class="font-semibold text-gray-900">Flaremask</span>
        </div>
        <p class="mb-4 text-sm text-gray-500">
            Sign in to manage your email aliases.
        </p>
        {#if error}
            <p
                class="mb-3 rounded border border-red-200 bg-red-50 px-3 py-2 text-xs text-red-700"
            >
                {error}
            </p>
        {/if}
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
                screen = "settings";
            }}
            class="mt-3 flex w-full items-center justify-center gap-1.5 rounded py-1.5 text-xs text-gray-400 transition-colors hover:bg-gray-100 hover:text-gray-600"
        >
            <Settings size={13} />
            {workerUrl}
        </button>
    </div>
{:else}
    <div class="w-90">
        <nav
            class="flex items-center justify-between border-b border-gray-200 px-4 py-2"
        >
            <div class="flex items-center gap-2">
                <img
                    src="../icons/icon-192.png"
                    class="h-4 w-4 rounded"
                    alt="Flaremask"
                />
                <span class="text-sm font-semibold text-gray-900"
                    >Flaremask</span
                >
            </div>
            <button
                onclick={() => {
                    screen = "settings";
                    urlInput = workerUrl;
                }}
                class="btn-icon"
                title="Settings"
            >
                <Settings size={14} />
            </button>
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
            <AliasManager
                {api}
                {aliasTemplate}
                {siteName}
                compact
                initialAlias={suggestedAlias}
            />
        </div>
    </div>
{/if}
