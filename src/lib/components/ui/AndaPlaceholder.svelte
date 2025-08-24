<script lang="ts">
  import AndaLogo from '$lib/components/icons/AndaLogo.svelte'

  let { children } = $props()
</script>

<div class="flex w-full">
  <div
    class="m-auto flex flex-col gap-4 text-center opacity-50 [&>:first-child]:size-40"
  >
    <AndaLogo />
    {#if children}
      {@render children()}
    {/if}
  </div>
</div>
