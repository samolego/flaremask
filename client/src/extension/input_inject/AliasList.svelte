<script>
  let { aliases, aliasesLoading, onPick } = $props();
</script>

<div class="list">
  {#if aliasesLoading}
    <div class="hint">Loading aliases…</div>
  {:else if aliases.length === 0}
    <div class="hint">No aliases yet.</div>
  {:else}
    {#each aliases as a (a.alias)}
      <button
        class="item"
        class:root={a.is_root}
        onclick={() => onPick(a.alias)}
        title={a.alias}
      >
        {a.alias}
        {#if a.is_root}<span class="login-label">(login)</span>{/if}
      </button>
    {/each}
  {/if}
</div>

<style>
  .list {
    max-height: 140px;
    overflow-y: auto;
  }

  .item {
    all: unset;
    display: block;
    width: 100%;
    text-align: left;
    padding: 4px 8px;
    border-radius: 5px;
    cursor: pointer;
    font-size: 12px;
    font-family: monospace;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--fm-text);
    box-sizing: border-box;
  }
  .item:hover {
    background: var(--fm-hover-bg);
    color: var(--color-brand);
  }
  .item.root {
    color: var(--fm-text-muted);
    font-size: 11px;
  }
  .item.root:hover {
    background: var(--fm-hover-bg);
    color: var(--color-brand);
  }

  .login-label {
    opacity: 0.55;
  }

  .hint {
    color: var(--fm-text-muted);
    font-size: 12px;
    text-align: center;
    padding: 6px 0;
  }
</style>
