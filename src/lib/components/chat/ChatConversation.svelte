<script lang="ts">
  import { toChatMessages, type Conversation } from '$lib/types/assistant'
  import ChatMessage from './ChatMessage.svelte'

  // const { conversation }: { conversation: Conversation } = $props() ❌
  // $props() 返回的是“活”的 props 容器，但你对它做了解构成独立变量，这会“截断”后续的变更
  const props = $props<{ conversation: Conversation }>()
  const messages = $derived(toChatMessages(props.conversation))
</script>

<div id={`conversation-${props.conversation._id}`} class="flex flex-col">
  {#each messages as message (message.id)}
    {#if message.role == 'user' || message.role == 'assistant'}
      <ChatMessage {message} />
    {/if}
  {/each}
</div>
