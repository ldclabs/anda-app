<script lang="ts">
  import FadeStaggerSquares from '$lib/components/icons/FadeStaggerSquares.svelte'
  import { assistantStore } from '$lib/stores/assistant.svelte'
  import { t } from '$lib/stores/i18n'
  import {
    lastThought,
    toChatMessages,
    type Conversation
  } from '$lib/types/assistant'
  import { UserHeadsetOutline } from 'flowbite-svelte-icons'
  import ChatMessage from './ChatMessage.svelte'

  // const { conversation }: { conversation: Conversation } = $props() ❌
  // $props() 返回的是“活”的 props 容器，但你对它做了解构成独立变量，这会“截断”后续的变更
  const props = $props<{ conversation: Conversation }>()
  const messages = $derived(toChatMessages(props.conversation))
</script>

<div
  id={`conversation-${props.conversation._id}`}
  class="flex flex-col rounded-lg"
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
  {/if}
</div>
