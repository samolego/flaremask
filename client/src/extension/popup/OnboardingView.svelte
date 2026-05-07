<script>
  let { urlInput = $bindable(''), onSave } = $props();
</script>

<div class="flex min-h-50 flex-col justify-center p-6">
  <div class="mb-4 flex items-center gap-2">
    <img src="../icons/icon-192.png" class="h-5 w-5 rounded" alt="Flaremask" />
    <span class="font-semibold text-gray-900">Flaremask</span>
  </div>
  <p class="mb-4 text-sm text-gray-500">Enter your worker URL to get started.</p>
  <form
    onsubmit={(e) => { e.preventDefault(); onSave(); }}
    class="flex flex-col gap-3"
  >
    <input
      bind:value={urlInput}
      type="url"
      placeholder="https://mask.example.com"
      required
      class="form-input"
    />
    <button type="submit" class="btn-primary justify-center">Continue</button>
  </form>
</div>
