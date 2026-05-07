<script>
    import AliasTemplateEditor from "../alias/AliasTemplateEditor.svelte";

    /**
     * @type {{
     *   aliasTemplate: string,
     *   workerUrl?: string,
     *   showSiteTag?: boolean,
     *   showWorkerUrl?: boolean,
     *   onSave: () => void,
     * }}
     */
    let {
        aliasTemplate = $bindable(""),
        workerUrl = $bindable(""),
        showSiteTag = false,
        showWorkerUrl = false,
        onSave,
    } = $props();
</script>

<div>
    <p class="mb-1 text-sm font-medium text-gray-800">Alias template</p>
    <p class="mb-2 text-xs text-gray-500">
        Tags are replaced when generating a suggestion.
    </p>
    <AliasTemplateEditor bind:aliasTemplate {showSiteTag} />

    {#if showWorkerUrl}
        <div class="mt-5 border-t border-gray-100 pt-4">
            <label
                for="worker-url"
                class="mb-1 block text-xs font-medium uppercase tracking-wide text-gray-500"
            >
                Worker URL
            </label>
            <input
                id="worker-url"
                bind:value={workerUrl}
                type="text"
                inputmode="url"
                placeholder="mask.example.com"
                class="form-input w-full text-xs font-mono"
            />
        </div>
    {/if}

    <div class="mt-4 flex justify-end">
        <button onclick={onSave} class="btn-primary px-6">Save</button>
    </div>
</div>
