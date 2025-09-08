<script lang="ts">
  import type DiagnosticsModal from '$lib/components/diagnostics/DiagnosticsModal.svelte'
  import FadeStaggerSquares from '$lib/components/icons/FadeStaggerSquares.svelte'
  import { assistantStore } from '$lib/stores/assistant.svelte'
  import { t } from '$lib/stores/i18n'
  import {
    lastThought,
    toChatMessages,
    type Conversation
  } from '$lib/types/assistant'
  import { Hr } from 'flowbite-svelte'
  import { InfoCircleOutline, UserHeadsetOutline } from 'flowbite-svelte-icons'
  import { getContext } from 'svelte'
  import ChatMessage from './ChatMessage.svelte'

  // const { conversation }: { conversation: Conversation } = $props() ❌
  // $props() 返回的是“活”的 props 容器，但你对它做了解构成独立变量，这会“截断”后续的变更
  const props = $props<{ conversation: Conversation }>()
  const messages = $derived(toChatMessages(props.conversation))
  const diagnosticsModal = getContext<() => DiagnosticsModal>(
    'diagnosticsModalState'
  )
</script>

<div
  id={`conversation-${props.conversation._id}`}
  class="mt-2 flex flex-col rounded-lg"
>
  {#each messages as message (message.id)}
    {#if message.role == 'user' || message.role == 'assistant'}
      <ChatMessage {message} />
    {/if}
  {/each}
  {#if assistantStore.isThinking == props.conversation._id}
    {@const thought = lastThought(props.conversation)}
    <div
      class="group flex-row} relative flex max-w-4xl rounded-lg border border-transparent p-4"
    >
      <div class="flex w-full flex-col gap-1">
        <div class="w-full px-4 py-2 dark:text-white">
          <div class="flex flex-row items-center gap-2">
            <UserHeadsetOutline size="lg" class="animate-pulse" />
            <span class="text-sm">{t('assistant.thinking')}</span>
            <span class="*:w-12"><FadeStaggerSquares /></span>
          </div>
          <div class="md-content w-full text-pretty break-words">
            {#if thought}
              <p class="animate-pulse truncate text-gray-500/50"
                >{thought.content}</p
              >
            {/if}
          </div>
        </div>
      </div>
    </div>
  {:else}
    <Hr class="mx-4 my-4 h-[1px] w-full bg-gray-500/10">
      <div class="flex flex-row items-center gap-1">
        <button
          class="flex items-center rounded-sm p-1 text-base font-normal text-gray-500 hover:bg-gray-100 dark:text-white dark:hover:bg-gray-700"
          onclick={() =>
            diagnosticsModal().openView('conversation', props.conversation._id)}
          ><InfoCircleOutline size="md" />
        </button>
        <span class="text-xs text-gray-500"
          >{Math.floor(
            (props.conversation.updated_at - props.conversation.created_at) /
              1000
          )}s</span
        >
      </div>
    </Hr>
  {/if}
</div>
