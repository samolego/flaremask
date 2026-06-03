<script>
    import { LoaderCircle, TriangleAlert } from "lucide-svelte";
    import { AuthError, RateLimitError } from "../../lib/api.js";

    let { api, compact = false } = $props();

    const sz = $derived(compact ? 14 : 16);

    let sending = $state(false);
    let sent = $state(false);
    let rateLimited = $state(false);
    let error = $state(null);

    async function send() {
        sending = true;
        error = null;
        try {
            await api.sendVerificationEmail();
            sent = true;
        } catch (e) {
            if (e instanceof RateLimitError) {
                rateLimited = true;
            } else if (!(e instanceof AuthError)) {
                error = e.message;
            }
        } finally {
            sending = false;
        }
    }
</script>

<div class="card">
    <div class="flex flex-col items-center gap-4 px-6 py-10 text-center">
        <TriangleAlert size={compact ? 28 : 36} class="text-gray-400" />
        <div>
            <p class="{compact ? 'text-sm' : 'text-base'} font-medium text-gray-700">
                Email address not verified
            </p>
            <p class="mt-1 {compact ? 'text-xs' : 'text-sm'} text-gray-400">
                {#if sent || rateLimited}
                    Check your inbox and click the verification link, then reload this page.
                {:else}
                    Cloudflare needs to verify your email before you can create aliases.
                {/if}
            </p>
            {#if error}
                <p class="mt-2 {compact ? 'text-xs' : 'text-sm'} text-red-600">{error}</p>
            {/if}
        </div>
        {#if !sent && !rateLimited}
            <button onclick={send} disabled={sending} class="btn-primary">
                {#if sending}
                    <LoaderCircle size={sz} class="animate-spin" />
                    Sending…
                {:else}
                    Send verification email
                {/if}
            </button>
        {/if}
    </div>
</div>
