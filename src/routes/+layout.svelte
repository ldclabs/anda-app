<script lang="ts">
  import '$lib/init'
  import { settingsStore } from '$lib/stores/settings.svelte'
  import { toastStore } from '$lib/stores/toast.svelte'
  import { Toast } from 'flowbite-svelte'
  import {
    CheckCircleSolid,
    ExclamationCircleSolid,
    InfoCircleSolid
  } from 'flowbite-svelte-icons'
  import '../app.css'

  let { children } = $props()

  function toastColor(type: 'success' | 'error' | 'info') {
    return type === 'success' ? 'green' : type === 'error' ? 'red' : 'blue'
  }

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

  function init(ele: HTMLDivElement) {
    const checkState = () => {
      if (ele.childElementCount > 0) {
        try {
          // 重新打开以确保位于 Top Layer 顶部
          try {
            ele.hidePopover()
          } catch {}
          ele.showPopover()
        } catch {}
      } else {
        try {
          ele.hidePopover()
        } catch {}
      }
    }
    // 初始检查
    checkState()

    const observer = new MutationObserver(checkState)
    observer.observe(ele, { childList: true })

    return () => observer.disconnect()
  }
</script>

{@render children()}

<div
  {@attach init}
  popover="manual"
  class="pointer-events-none fixed top-4 right-4 bottom-auto left-auto z-[2147483647] m-0 flex flex-col items-end gap-2 border-0 bg-transparent p-0 outline-none"
>
  {#each toastStore as toast (toast.id)}
    <Toast
      color={toastColor(toast.type)}
      class="pointer-events-auto w-sm max-w-sm bg-gray-100 py-2 dark:bg-gray-800"
      classes={{ content: 'w-3xs text-pretty break-words' }}
      onclick={toast.onclick}
    >
      {#snippet icon()}
        {#if toast.icon}
          {@render toast.icon()}
        {:else if toast.type === 'success'}
          <CheckCircleSolid class="size-5" />
        {:else if toast.type === 'error'}
          <ExclamationCircleSolid class="size-5" />
        {:else}
          <InfoCircleSolid class="size-5" />
        {/if}
      {/snippet}
      {#if toast.content}
        {@render toast.content()}
      {:else if toast.message}
        {toast.message}
      {/if}
    </Toast>
  {/each}
</div>
