<script lang="ts">
  import AndaPlaceholder from '$lib/components/ui/AndaPlaceholder.svelte'
</script>

<div
  class="relative h-full w-full sm:grid sm:grid-cols-[220px_1fr] md:grid-cols-[280px_1fr] lg:grid-cols-[320px_1fr]"
>
  <div
    class="channels-list border-surface-500/20 h-full w-full bg-white transition-transform duration-300 max-sm:absolute max-sm:top-0 max-sm:bottom-0 max-sm:z-10 sm:border-r dark:bg-neutral-950"
  >
    <AndaPlaceholder><span></span></AndaPlaceholder>
  </div>
  <div class="bg-surface-900/5 h-full">
    <div class="flex h-full flex-col items-center justify-center">
      <AndaPlaceholder><span></span></AndaPlaceholder>
    </div>
  </div>
</div>
