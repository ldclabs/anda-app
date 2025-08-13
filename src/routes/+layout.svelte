<script lang="ts">
  import '$lib/init'
  import { settingsStore } from '$lib/stores/settings.svelte'
  import '../app.css'
  // Initialize Tauri mocks for browser development

  let { children } = $props()

  $effect(() => {
    if (!settingsStore.theme) {
      if (window.matchMedia('(prefers-color-scheme: dark)').matches)
        window.document.documentElement.classList.add('dark')
    } else {
      settingsStore.theme === 'dark'
        ? window.document.documentElement.classList.add('dark')
        : window.document.documentElement.classList.remove('dark')
    }
  })
</script>

{@render children()}
