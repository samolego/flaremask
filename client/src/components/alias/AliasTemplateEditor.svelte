<script>
    import { DEFAULT_ALIAS_TEMPLATE } from "../../lib/templates.js";

    /** @type {{ aliasTemplate: string, showSiteTag?: boolean }} */
    let { aliasTemplate = $bindable(""), showSiteTag = true } = $props();

    const BASE_TAGS = ["{noun}", "{adj}", "{rnd}"];
    const TAGS = $derived(showSiteTag ? ["{site}", ...BASE_TAGS] : BASE_TAGS);

    let templateInput = $state(/** @type {HTMLInputElement|null} */ (null));

    function insertTag(tag) {
        if (!templateInput) {
            aliasTemplate += tag;
            return;
        }
        const s = templateInput.selectionStart ?? aliasTemplate.length;
        const e = templateInput.selectionEnd ?? s;
        aliasTemplate =
            aliasTemplate.slice(0, s) + tag + aliasTemplate.slice(e);
        const pos = s + tag.length;
        setTimeout(() => {
            templateInput?.focus();
            templateInput?.setSelectionRange(pos, pos);
        }, 0);
    }
</script>

<div>
    <input
        bind:this={templateInput}
        bind:value={aliasTemplate}
        type="text"
        spellcheck="false"
        autocomplete="off"
        placeholder={DEFAULT_ALIAS_TEMPLATE}
        class="form-input w-full mb-2 font-mono text-sm"
    />
    <div class="flex flex-wrap gap-1.5">
        {#each TAGS as tag}
            <button
                type="button"
                onclick={() => insertTag(tag)}
                class="rounded border border-gray-200 bg-gray-50 px-2 py-0.5 font-mono text-xs text-gray-600 transition-colors hover:border-brand hover:bg-brand/5 hover:text-brand"
                >{tag}</button
            >
        {/each}
    </div>
</div>
