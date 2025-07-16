<script lang="ts">
  import ChatInput from '$lib/components/chat/ChatInput.svelte'
  import ChatMessage from '$lib/components/chat/ChatMessage.svelte'
  import { chatStore } from '$lib/stores/chat.svelte'
  import { Button, Card } from 'flowbite-svelte'
  import { UserHeadsetOutline } from 'flowbite-svelte-icons'
  import { onMount } from 'svelte'

  let messagesContainer: HTMLDivElement

  // 自动滚动到底部
  $effect(() => {
    if (chatStore.currentSession?.messages && messagesContainer) {
      setTimeout(() => {
        messagesContainer.scrollTop = messagesContainer.scrollHeight
      }, 100)
    }
  })

  onMount(() => {
    // 如果没有会话，创建一个默认会话
    if (chatStore.sessions.length === 0) {
      chatStore.createSession('欢迎使用 Anda AI')
      chatStore.addMessage(
        '你好！我是 Anda AI 助手，很高兴为你服务。有什么我可以帮助你的吗？',
        'assistant'
      )
    }
  })

  function handleSendMessage(message: string) {
    chatStore.sendMessage(message)
  }

  function handleClearChat() {
    chatStore.clearCurrentSession()
  }
</script>

<div class="relative flex h-full w-full flex-col items-center overflow-y-auto">
  <div class="my-10 flex w-full max-w-4xl flex-1 flex-col">
    {#if chatStore.currentSession}
      <!-- 消息列表 -->
      <div bind:this={messagesContainer} class="w-full">
        {#if chatStore.currentSession.messages.length === 0}
          <div class="flex h-full w-full items-center justify-center">
            <Card class="max-w-md text-center">
              <UserHeadsetOutline
                size="xl"
                class="text-primary-500 mx-auto mb-4"
              />
              <h2
                class="mb-2 text-xl font-semibold text-gray-900 dark:text-white"
              >
                开始对话
              </h2>
              <p class="text-gray-500 dark:text-gray-400">
                向 AI 助手提问任何问题，我会尽力为你提供帮助。
              </p>
            </Card>
          </div>
        {:else}
          {#each chatStore.currentSession.messages as message (message.id)}
            <ChatMessage {message} />
          {/each}
        {/if}
      </div>
    {:else}
      <!-- 无会话状态 -->
      <div class="flex h-full w-full items-center justify-center">
        <Card class="max-w-md text-center">
          <UserHeadsetOutline size="xl" class="mx-auto mb-4 text-gray-400" />
          <h2 class="mb-2 text-xl font-semibold text-gray-900 dark:text-white">
            选择或创建对话
          </h2>
          <p class="mb-4 text-gray-500 dark:text-gray-400">
            从左侧选择一个对话，或创建新的对话开始聊天。
          </p>
          <Button onclick={() => chatStore.createSession()}>创建新对话</Button>
        </Card>
      </div>
    {/if}
  </div>
  <div
    class="sticky right-0 bottom-0 left-0 z-10 w-full max-w-4xl transform px-4 pb-4"
  >
    <!-- 输入区域 -->
    <div
      class="rounded-4xl border border-white/40 bg-gray-200/60 shadow-lg backdrop-blur-xs dark:bg-gray-800/60"
    >
      <ChatInput onSend={handleSendMessage} disabled={chatStore.isLoading} />
    </div>
  </div>
</div>
