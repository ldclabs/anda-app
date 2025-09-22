<script lang="ts">
  import ChatConversation from '$lib/components/chat/ChatConversation.svelte'
  import ChatInput from '$lib/components/chat/ChatInput.svelte'
  import AccountCircleLine from '$lib/components/icons/AccountCircleLine.svelte'
  import Robot2Line from '$lib/components/icons/Robot2Line.svelte'
  import Settings5Line from '$lib/components/icons/Settings5Line.svelte'
  import { assistantStore, assistant_name } from '$lib/stores/assistant.svelte'
  import { authStore } from '$lib/stores/auth.svelte'
  import { t } from '$lib/stores/i18n'
  import { open_settings_window } from '$lib/stores/settings.svelte'
  import { type Resource } from '$lib/types/assistant'
  import { renderMarkdown } from '$lib/utils/markdown'
  import { scrollIntoView, scrollOnHooks } from '$lib/utils/window'
  import { Spinner } from 'flowbite-svelte'
  import { getContext, onMount, tick } from 'svelte'

  let messagesContainer = $state<HTMLElement>()
  let latestConversationId = $state<number>(0)
  let my_name = $state<string | null>(null)

  const [helloContent, _] = renderMarkdown(t('assistant.hello.description'))
  const conversations = $derived(assistantStore.conversations)
  const onSignInClick: () => void = getContext('signInHandler')

  // 自动滚动到底部
  $effect(() => {
    if (assistantStore.latestConversationId > latestConversationId) {
      latestConversationId = assistantStore.latestConversationId
      scrollIntoView(`conversation-${latestConversationId}`, 'smooth', 'start')
    }
  })

  onMount(() => {
    assistant_name().then((name) => {
      my_name = name
    })
    if (!authStore.auth.isAuthenticated()) return

    assistantStore.loadLatestConversations()
    const abortScroll = scrollOnHooks(messagesContainer!, {
      onTop: () => {
        assistantStore.loadPreviousConversations().then((hasMore) => {
          if (hasMore) {
            tick().then(() => {
              messagesContainer!.scrollTop = 100
            })
          }
        })
      }
    })
    return abortScroll
  })

  function handleSendMessage(message: string, resources: Resource[]) {
    assistantStore.chat(message, resources)
  }
</script>

<div
  bind:this={messagesContainer}
  class="relative flex h-full w-full flex-col items-center overflow-y-auto scroll-smooth"
>
  {#if assistantStore.isLoadingPrev}
    <div class="mt-4 flex items-center gap-2">
      <Spinner size="5" />
    </div>
  {/if}
  <div class="my-10 flex w-full max-w-4xl flex-1 flex-col px-4">
    <!-- 消息列表 -->

    {#if conversations.length === 0}
      <div class="flex h-full w-full items-center justify-center">
        <div class="max-w-md">
          <div class="text-primary-500 mb-4 *:m-auto *:size-10"
            ><Robot2Line /></div
          >
          <h2
            class="mb-2 text-center text-xl font-semibold text-gray-900 dark:text-white"
          >
            {my_name
              ? t('assistant.hello.title_with_name', { name: my_name })
              : t('assistant.hello.title')}
          </h2>
          <div class="md-content text-gray-800 dark:text-gray-300">
            {@html helloContent}
          </div>
        </div>
      </div>
    {:else}
      {#each conversations as conversation (conversation._id)}
        <ChatConversation {conversation} />
      {/each}
    {/if}
  </div>

  <div
    class="sticky right-0 bottom-0 left-0 z-10 w-full max-w-4xl transform px-4 pb-4"
  >
    <!-- 输入区域 -->
    <div
      class="relative rounded-4xl border border-white/40 bg-gray-200/60 shadow-lg backdrop-blur-xs dark:bg-gray-800/60"
    >
      <ChatInput
        user={authStore.auth.id}
        onSend={handleSendMessage}
        onStop={() => assistantStore.stop()}
        disabled={assistantStore.isSubmitting || assistantStore.isThinking > 0}
        isRunning={assistantStore.isThinking > 0}
      />
      {#if !assistantStore.isReady}
        <div
          class="absolute inset-0 z-20 flex flex-row items-center justify-center rounded-4xl bg-gray-200/70 backdrop-blur-sm dark:bg-gray-800/70"
        >
          <p class="my-2 text-gray-800 dark:text-gray-200">
            {t('assistant.not_ready')}
          </p>
          <button
            class="text-primary-600 flex items-center rounded-sm p-2 text-base font-normal hover:bg-gray-100 dark:text-white dark:hover:bg-gray-700"
            onclick={() => open_settings_window()}
            ><span class="*:size-5"><Settings5Line /></span>
            <span class="ms-2">{t('settings.title')}</span>
          </button>
        </div>
      {:else if !authStore.auth.isAuthenticated()}
        <div
          class="absolute inset-0 z-20 flex flex-row items-center justify-center rounded-4xl bg-gray-200/70 backdrop-blur-sm dark:bg-gray-800/70"
        >
          <p class="my-2 text-gray-800 dark:text-gray-200">
            {t('assistant.signin_required')}
          </p>
          <button
            class="text-primary-600 flex items-center rounded-sm p-2 text-base font-normal hover:bg-gray-100 disabled:cursor-not-allowed disabled:text-gray-400 dark:text-white dark:hover:bg-gray-700"
            onclick={onSignInClick}
            disabled={authStore.isSigningIn}
            ><span class="*:size-5"><AccountCircleLine /></span>
            <span class="ms-2">{t('app.sign_in')}</span>
            {#if authStore.isSigningIn}
              <Spinner class="ms-2 inline-flex" size="4" />
            {/if}
          </button>
        </div>
      {/if}
    </div>
  </div>
</div>
