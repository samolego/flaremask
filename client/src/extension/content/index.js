import 'webextension-polyfill';
import { mount, unmount } from 'svelte';
import Button from './Button.svelte';
import Popover from './Popover.svelte';

const EMAIL_SELECTOR = 'input[type="email"], input[autocomplete~="email"], input[name="email"]';

// Singleton popover — only one open at a time
let activeComponent = null;
let activeHost = null;
let activeInput = null;

function closePopover() {
  if (activeComponent) { unmount(activeComponent); activeComponent = null; }
  activeHost?.remove();
  activeHost = null;
  activeInput = null;
}

function togglePopover(input) {
  if (activeInput === input) { closePopover(); return; }
  closePopover();
  activeInput = input;
  activeHost = document.createElement('div');
  activeHost.setAttribute('data-flaremask-pop', '');
  document.body.appendChild(activeHost);
  activeComponent = mount(Popover, { target: activeHost, props: { input, onClose: closePopover } });
}

// Per-input button injection
const buttonInstances = new WeakMap();

function injectButton(input) {
  if (buttonInstances.has(input)) return;
  const host = document.createElement('div');
  document.body.appendChild(host);
  const comp = mount(Button, {
    target: host,
    props: {
      input,
      onToggle: () => togglePopover(input),
      onRemove: () => { unmount(comp); host.remove(); buttonInstances.delete(input); },
    },
  });
  buttonInstances.set(input, { host, comp });
}

function scan(root = document) {
  root.querySelectorAll(EMAIL_SELECTOR).forEach(injectButton);
}

new MutationObserver((mutations) => {
  for (const m of mutations) {
    for (const node of m.addedNodes) {
      if (node.nodeType !== 1) continue;
      if (node.matches?.(EMAIL_SELECTOR)) injectButton(node);
      node.querySelectorAll?.(EMAIL_SELECTOR).forEach(injectButton);
    }
  }
}).observe(document.documentElement, { childList: true, subtree: true });

document.addEventListener('click', (e) => {
  if (!e.target.closest?.('[data-flaremask-btn]') && !e.target.closest?.('[data-flaremask-pop]')) {
    closePopover();
  }
}, true);

document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') closePopover();
});

scan();
