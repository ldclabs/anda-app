<script lang="ts">
  import { t } from '$lib/stores/i18n'
  import type { ChatMessage } from '$lib/types/assistant'
  import { formatDateTime } from '$lib/utils/helper'
  import { getPlainText, renderMarkdown } from '$lib/utils/markdown'
  import { Clipboard } from 'flowbite-svelte'
  import {
    CheckOutline,
    FileCloneOutline,
    FileCopyAltOutline,
    InfoCircleOutline
  } from 'flowbite-svelte-icons'
  import { getContext, onMount } from 'svelte'

  let { message }: { message: ChatMessage } = $props()
  const [content, hook] = renderMarkdown(message.content)
  const diagnosisModalState: { open: (view: 'kip' | 'conversation') => void } =
    getContext('diagnosisModalState')

  let text = $state('')
  let textSuccess = $state(false)
  let mdSuccess = $state(false)

  function handleDiagnosis() {
    diagnosisModalState.open('kip')
  }

  onMount(async () => {
    await hook()
  })
</script>

<div
  id={`${message.id}`}
  class="group hover:border-secondary-500/50 relative flex max-w-4xl gap-3 rounded-lg border border-transparent p-4 {message.role ===
  'user'
    ? 'flex-row-reverse'
    : 'flex-row'}"
>
  <div
    class="flex flex-col gap-1 {message.role === 'user'
      ? 'max-w-[90%] items-end'
      : 'w-full'}"
  >
    <div
      class="pointer-events-none sticky top-0 z-10 -mt-12 -mr-2 flex flex-row gap-2 rounded-lg bg-white/80 p-2 opacity-0 backdrop-blur-xs transition-opacity duration-200 group-hover:pointer-events-auto group-hover:opacity-100 {message.role ===
      'user'
        ? ''
        : 'self-end'}"
    >
      {#if message.role === 'user'}
        <button
          class="flex items-center rounded-sm p-2 text-base font-normal text-gray-500 hover:bg-gray-100 dark:text-white dark:hover:bg-gray-700"
          onclick={handleDiagnosis}
          ><InfoCircleOutline size="md" />
        </button>
      {/if}
      <Clipboard
        bind:value={text}
        bind:success={textSuccess}
        onclick={() => (text = getPlainText(message.content))}
        size="sm"
        color="alternative"
        class="focus-within:ring-0 focus:ring-0"
      >
        {#if textSuccess}<CheckOutline />{:else}<FileCopyAltOutline />{/if}{t(
          'app.copy_text'
        )}
      </Clipboard>
      <Clipboard
        bind:value={message.content}
        bind:success={mdSuccess}
        size="sm"
        color="alternative"
        class="focus-within:ring-0 focus:ring-0"
      >
        {#if mdSuccess}<CheckOutline />{:else}<FileCloneOutline />{/if}{t(
          'app.copy_origin'
        )}
      </Clipboard>
    </div>
    <div
      class="w-full px-4 py-2 dark:text-white {message.role === 'user'
        ? 'rounded-lg bg-gray-50 text-gray-900 dark:bg-gray-800 '
        : ''}"
    >
      <div class="md-content w-full text-pretty break-words"
        >{@html content}</div
      >
    </div>

    <span class="text-xs text-gray-500 dark:text-gray-400">
      {formatDateTime(message.timestamp)}
    </span>
  </div>
</div>
