<script>
    import Toggle from "../Toggle.svelte";

    /** @type {{ label: string, description?: string, checked: boolean, onchange?: (v: boolean) => void }} */
    let { label, description, checked = $bindable(), onchange } = $props();
</script>

<div class="flex items-center justify-between gap-4 py-2.5">
    <div class="flex-1">
        <p class="text-sm font-medium text-gray-800">{label}</p>
        {#if description}
            <p class="mt-0.5 text-xs text-gray-500">{description}</p>
        {/if}
    </div>
    <Toggle bind:checked {onchange} />
</div>
