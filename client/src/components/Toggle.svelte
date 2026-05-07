<script>
    /** @type {{ checked: boolean, disabled?: boolean, onchange?: (v: boolean) => void }} */
    let { checked = $bindable(), disabled = false, onchange } = $props();
</script>

<button
    role="switch"
    aria-checked={checked}
    aria-label="Toggle"
    {disabled}
    onclick={() => {
        checked = !checked;
        onchange?.(checked);
    }}
    class="toggle {checked ? 'bg-brand' : 'bg-gray-200'}"
>
    <span class="toggle-thumb {checked ? 'translate-x-4' : 'translate-x-0.5'}"
    ></span>
</button>
