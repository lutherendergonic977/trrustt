<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  let greeting = $state('');

  async function greet() {
    greeting = await invoke('greet', { name: 'Power BI User' });
  }

  $effect(() => {
    greet();
  });
</script>

<main class="flex flex-col items-center justify-center min-h-screen p-8">
  <div class="text-center">
    <h1 class="text-4xl font-bold text-[var(--color-primary)] mb-4">
      🔷 trRUSTt
    </h1>
    <p class="text-xl text-gray-600 dark:text-gray-400 mb-8">
      trRUSTt your data. One binary. Infinite dashboards.
    </p>
    <p class="text-lg">{greeting || 'Connecting...'}</p>
  </div>
</main>
