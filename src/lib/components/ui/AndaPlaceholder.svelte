<script>
  import AndaLogo from '$lib/components/icons/AndaLogo.svelte'

  let { children } = $props()
</script>

<div class="flex h-full w-full">
  <div class="m-auto flex flex-col gap-4 text-center opacity-50 *:size-40">
    <AndaLogo />
    {@render children()}
  </div>
</div>
