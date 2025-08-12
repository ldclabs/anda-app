<script lang="ts">
  import { goto } from '$app/navigation'

  setTimeout(() => {
    goto('/assistant', {
      replaceState: true
    })
  }, 2000)
</script>

<main
  class="m-auto flex h-screen w-full max-w-xs flex-col items-center justify-center gap-4 p-4"
>
  <img
    src="/_assets/anda.svg"
    alt="Anda Logo"
    class="border-gray-300 dark:border-gray-700"
  />
</main>

<style>
</style>
