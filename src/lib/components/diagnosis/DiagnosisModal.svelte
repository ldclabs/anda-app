<script lang="ts">
  import { assistantStore } from '$lib/stores/assistant.svelte'
  import type { Conversation, KIPLogs } from '$lib/types/assistant'
  import { formatDateTime } from '$lib/utils/helper'
  import { Button, Modal, Spinner } from 'flowbite-svelte'
  import { ArrowDownOutline } from 'flowbite-svelte-icons'

  let {
    callback
  }: { callback: { open: (view: 'kip' | 'conversation') => void } } = $props()
  // 全局单例状态管理
  let kipLogs = $state<KIPLogs[]>([])
  let conversations = $state<Conversation[]>([])
  let nextCursor = $state<string | undefined>()
  let isLoading = $state(false)
  let isLoadingMore = $state(false)
  let currentView = $state<'kip' | 'conversation'>('kip')
  let isOpen = $state(false)

  callback.open = (view: 'kip' | 'conversation') => {
    isOpen = true
    currentView = view
    if (view === 'kip') {
      loadKipLogs()
    } else {
      loadConversations()
    }
  }

  async function loadKipLogs() {
    if (isLoading) return
    isLoading = true
    try {
      const result = await assistantStore.listKipLogs(undefined, 10)
      if (result.result) {
        kipLogs = result.result
        nextCursor = result.next_cursor
      }
    } catch (error) {
      console.error('Failed to load KIP logs:', error)
    } finally {
      isLoading = false
    }
  }

  async function loadMoreKipLogs() {
    if (isLoadingMore || !nextCursor) return
    isLoadingMore = true
    try {
      const result = await assistantStore.listKipLogs(nextCursor, 10)
      if (result.result) {
        kipLogs = [...kipLogs, ...result.result]
        nextCursor = result.next_cursor
      }
    } catch (error) {
      console.error('Failed to load more KIP logs:', error)
    } finally {
      isLoadingMore = false
    }
  }

  async function loadConversations() {
    if (isLoading) return
    isLoading = true
    try {
      await assistantStore.loadLatestConversations()
      conversations = assistantStore.conversations
    } catch (error) {
      console.error('Failed to load conversations:', error)
    } finally {
      isLoading = false
    }
  }

  async function loadMoreConversations() {
    if (isLoadingMore) return
    isLoadingMore = true
    try {
      const hasMore = await assistantStore.loadPreviousConversations()
      conversations = assistantStore.conversations
      if (!hasMore) {
        nextCursor = undefined
      }
    } catch (error) {
      console.error('Failed to load more conversations:', error)
    } finally {
      isLoadingMore = false
    }
  }

  function handleViewChange(view: 'kip' | 'conversation') {
    currentView = view
    if (currentView === 'kip' && kipLogs.length === 0) {
      loadKipLogs()
    } else if (currentView === 'conversation' && conversations.length === 0) {
      loadConversations()
    }
  }

  function formatJson(obj: any): string {
    return JSON.stringify(obj, null, 2)
  }

  let scrollContainer: HTMLDivElement

  // function handleScroll() {
  //   if (!scrollContainer) return

  //   const { scrollTop, scrollHeight, clientHeight } = scrollContainer
  //   // 当滚动到距离底部 100px 时加载更多
  //   if (scrollHeight - scrollTop - clientHeight < 100) {
  //     if (currentView === 'kip') {
  //       loadMoreKipLogs()
  //     } else {
  //       loadMoreConversations()
  //     }
  //   }
  // }
</script>

<Modal bind:open={isOpen} size="xl" class="h-[90vh] w-full max-w-4xl">
  {#snippet header()}
    <div class="flex items-center justify-between">
      <div class="flex gap-2">
        <Button
          size="sm"
          color={currentView === 'kip' ? 'primary' : 'alternative'}
          onclick={() => handleViewChange('kip')}
        >
          KIP Logs
        </Button>
        <Button
          size="sm"
          color={currentView === 'conversation' ? 'primary' : 'alternative'}
          onclick={() => handleViewChange('conversation')}
        >
          Conversations
        </Button>
      </div>
    </div>
  {/snippet}

  <div class="flex h-full flex-col">
    <div
      bind:this={scrollContainer}
      class="flex-1 space-y-4 overflow-y-auto p-4"
    >
      {#if isLoading}
        <div class="flex items-center justify-center p-8">
          <Spinner size="6" />
        </div>
      {:else if currentView === 'kip'}
        {#if kipLogs.length === 0}
          <div class="p-8 text-center text-gray-500"> No KIP logs found. </div>
        {:else}
          {#each kipLogs as log (log._id)}
            <div
              class="rounded-lg border border-gray-200 bg-white p-4 dark:border-gray-700 dark:bg-gray-800"
            >
              <div class="mb-2 flex items-center justify-between">
                <span class="text-sm font-medium text-gray-900 dark:text-white">
                  KIP Log #{log._id}
                </span>
                <span class="text-xs text-gray-500 dark:text-gray-400">
                  {formatDateTime(log.timestamp)}
                </span>
              </div>
              <div class="mb-2">
                <span
                  class="inline-flex items-center rounded-full bg-blue-100 px-2.5 py-0.5 text-xs font-medium text-blue-800 dark:bg-blue-900 dark:text-blue-300"
                >
                  {log.command}
                </span>
                {#if log.conversation}
                  <span
                    class="ml-2 inline-flex items-center rounded-full bg-green-100 px-2.5 py-0.5 text-xs font-medium text-green-800 dark:bg-green-900 dark:text-green-300"
                  >
                    Conversation #{log.conversation}
                  </span>
                {/if}
              </div>
              <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
                <div>
                  <h4
                    class="mb-2 text-sm font-medium text-gray-700 dark:text-gray-300"
                  >
                    Request
                  </h4>
                  <pre
                    class="max-h-48 overflow-auto rounded bg-gray-50 p-3 text-xs text-gray-800 dark:bg-gray-900 dark:text-gray-200"
                    >{formatJson(log.request)}</pre
                  >
                </div>
                <div>
                  <h4
                    class="mb-2 text-sm font-medium text-gray-700 dark:text-gray-300"
                  >
                    Response
                  </h4>
                  <pre
                    class="max-h-48 overflow-auto rounded bg-gray-50 p-3 text-xs text-gray-800 dark:bg-gray-900 dark:text-gray-200"
                    >{formatJson(log.response)}</pre
                  >
                </div>
              </div>
            </div>
          {/each}
        {/if}
      {:else if conversations.length === 0}
        <div class="p-8 text-center text-gray-500">
          No conversations found.
        </div>
      {:else}
        {#each conversations as conversation (conversation._id)}
          <div
            class="rounded-lg border border-gray-200 bg-white p-4 dark:border-gray-700 dark:bg-gray-800"
          >
            <div class="mb-2 flex items-center justify-between">
              <span class="text-sm font-medium text-gray-900 dark:text-white">
                Conversation #{conversation._id}
              </span>
              <span class="text-xs text-gray-500 dark:text-gray-400">
                {formatDateTime(conversation.created_at)}
              </span>
            </div>
            <div class="mb-2">
              <span
                class="inline-flex items-center rounded-full bg-purple-100 px-2.5 py-0.5 text-xs font-medium text-purple-800 dark:bg-purple-900 dark:text-purple-300"
              >
                {conversation.status}
              </span>
              <span class="ml-2 text-xs text-gray-500 dark:text-gray-400">
                {conversation.messages.length} messages
              </span>
            </div>
            <div>
              <h4
                class="mb-2 text-sm font-medium text-gray-700 dark:text-gray-300"
              >
                Conversation Data
              </h4>
              <pre
                class="max-h-96 overflow-auto rounded bg-gray-50 p-3 text-xs text-gray-800 dark:bg-gray-900 dark:text-gray-200"
                >{formatJson(conversation)}</pre
              >
            </div>
          </div>
        {/each}
      {/if}

      {#if isLoadingMore}
        <div class="flex items-center justify-center p-4">
          <Spinner size="4" />
          <span class="ml-2 text-sm text-gray-500">Loading more...</span>
        </div>
      {:else if (currentView === 'kip' && nextCursor) || (currentView === 'conversation' && assistantStore.conversations.length > 0)}
        <div class="flex items-center justify-center p-4">
          <Button
            size="sm"
            color="alternative"
            onclick={currentView === 'kip'
              ? loadMoreKipLogs
              : loadMoreConversations}
          >
            <ArrowDownOutline class="mr-2" />
            Load more
          </Button>
        </div>
      {:else if currentView === 'kip' ? kipLogs.length > 0 : conversations.length > 0}
        <div class="p-4 text-center text-sm text-gray-500">
          No more records.
        </div>
      {/if}
    </div>
  </div>
</Modal>
