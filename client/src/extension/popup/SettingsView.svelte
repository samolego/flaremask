<script>
    import { LogOut, ArrowLeft } from "lucide-svelte";
    import SettingsPanel from "../../components/settings/SettingsPanel.svelte";

    let {
        workerUrl = $bindable(""),
        aliasTemplate = $bindable(""),
        authenticated,
        onSave,
        onBack,
        onLogout,
    } = $props();
</script>

<div class="flex min-h-50 flex-col p-4">
    <div class="mb-4 flex items-center justify-between">
        <div class="flex items-center gap-2">
            <button onclick={onBack} class="btn-icon -ml-1" title="Back">
                <ArrowLeft size={15} />
            </button>
            <img
                src="../icons/icon-192.png"
                class="h-4 w-4 rounded"
                alt="Flaremask"
            />
            <span class="text-sm font-semibold text-gray-900">Settings</span>
        </div>
        {#if authenticated}
            <button
                onclick={onLogout}
                class="btn-icon flex items-center gap-1 text-xs text-gray-400 hover:text-red-500"
                title="Sign out"
            >
                <LogOut size={13} />Sign out
            </button>
        {/if}
    </div>

    <SettingsPanel
        bind:aliasTemplate
        bind:workerUrl
        showSiteTag={true}
        showWorkerUrl={true}
        {onSave}
    />
</div>
