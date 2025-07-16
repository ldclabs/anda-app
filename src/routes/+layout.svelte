<script lang="ts">
  import '$lib/init'
  import '../app.css'
  // Initialize Tauri mocks for browser development

  let { children } = $props()
</script>

{@render children()}
