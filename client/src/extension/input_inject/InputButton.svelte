<script>
  import { onMount } from 'svelte';

  const BTN_SIZE = 18;

  let { input, onToggle, onRemove } = $props();
  const iconUrl = browser.runtime.getURL('icons/icon-192.png');

  let hovered = $state(false);
  let top = $state(0);
  let left = $state(0);
  let visible = $state(false);

  function reposition() {
    if (!document.body?.contains(input)) { onRemove?.(); return; }
    const r = input.getBoundingClientRect();
    if (r.width === 0) { visible = false; return; }
    visible = true;
    left = r.right - BTN_SIZE - 4;
    top = r.top + (r.height - BTN_SIZE) / 2;
  }

  onMount(() => {
    const existingPR = parseFloat(getComputedStyle(input).paddingRight) || 0;
    input.style.paddingRight = `${existingPR + BTN_SIZE + 6}px`;
    input.style.boxSizing = 'border-box';

    reposition();
    window.addEventListener('scroll', reposition, { passive: true, capture: true });
    window.addEventListener('resize', reposition, { passive: true });

    const ro = new ResizeObserver(reposition);
    ro.observe(document.documentElement);

    const obs = new MutationObserver(() => {
      if (!document.body?.contains(input)) onRemove?.();
    });
    obs.observe(document.body, { childList: true, subtree: true });

    return () => {
      window.removeEventListener('scroll', reposition, { capture: true });
      window.removeEventListener('resize', reposition);
      ro.disconnect();
      obs.disconnect();
    };
  });
</script>

{#if visible}
  <button
    data-flaremask-btn
    type="button"
    title="Fill with Flaremask alias"
    style="top: {top}px; left: {left}px; opacity: {hovered ? 1 : 0.55};"
    onmouseenter={() => (hovered = true)}
    onmouseleave={() => (hovered = false)}
    onclick={(e) => { e.preventDefault(); e.stopPropagation(); onToggle(); }}
  >
    <img src={iconUrl} alt="Flaremask" />
  </button>
{/if}

<style>
  button {
    all: unset;
    position: fixed;
    width: 18px;
    height: 18px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2147483646;
    border-radius: 3px;
    transition: opacity 0.15s;
    box-sizing: border-box;
  }
  button img {
    width: 18px;
    height: 18px;
    border-radius: 3px;
    display: block;
  }
</style>
