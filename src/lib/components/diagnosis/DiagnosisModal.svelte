<script lang="ts">
  import { assistantStore } from '$lib/stores/assistant.svelte'
  import type { Conversation, KIPLog } from '$lib/types/assistant'
  import { formatDateTime, ID2Cursor } from '$lib/utils/helper'
  import { scrollIntoView, scrollOnHooks } from '$lib/utils/window'
  import { Button, Modal, Spinner } from 'flowbite-svelte'
  import { onMount, tick } from 'svelte'

  let {
    callback
  }: {
    callback: { open: (view: 'kip' | 'conversation', _id?: number) => void }
  } = $props()
  // 全局单例状态管理
  let kipLogs = $state<KIPLog[]>([])
  let conversations = $state<Conversation[]>([])
  let nextKipCursor = $state<string | undefined>()
  let nextConversationCursor = $state<string | undefined>()
  let isLoading = $state(false)
  let isLoadingPrev = $state(false)
  let currentView = $state<'kip' | 'conversation'>('kip')
  let isOpen = $state(false)

  callback.open = (view: 'kip' | 'conversation', _id?: number) => {
    isOpen = true
    currentView = view
    if (view === 'kip') {
      loadKipLogs(_id)
    } else {
      loadConversations(_id)
    }
  }

  async function loadKipLogs(_id?: number) {
    isLoading = true
    nextKipCursor = undefined
    try {
      const result = await assistantStore.listKipLogs(
        _id ? ID2Cursor(_id + 1) : undefined,
        10
      )
      if (result.result) {
        kipLogs = result.result
        nextKipCursor = result.next_cursor
      }
      await tick()
      scrollIntoView(
        `dm-kiplog-${_id || kipLogs.at(-1)?._id}`,
        'smooth',
        'start'
      )
    } catch (error) {
      console.error('Failed to load KIP logs:', error)
    } finally {
      isLoading = false
    }
  }

  async function loadConversations(_id?: number) {
    isLoading = true
    nextConversationCursor = undefined
    try {
      const result = await assistantStore.listConversations(
        _id ? ID2Cursor(_id + 1) : undefined,
        10
      )
      if (result.result) {
        conversations = result.result
        nextConversationCursor = result.next_cursor
      }

      await tick()
      scrollIntoView(
        `dm-conversation-${_id || conversations.at(-1)?._id}`,
        'smooth',
        'start'
      )
    } catch (error) {
      console.error('Failed to load conversations:', error)
    } finally {
      isLoading = false
    }
  }

  async function loadMoreKipLogs() {
    if (isLoadingPrev || !nextKipCursor) return false
    isLoadingPrev = true
    try {
      const result = await assistantStore.listKipLogs(nextKipCursor, 10)
      if (result.result) {
        kipLogs = [...kipLogs, ...result.result]
        nextKipCursor = result.next_cursor
        return !!nextKipCursor
      }
    } catch (error) {
      console.error('Failed to load more KIP logs:', error)
    } finally {
      isLoadingPrev = false
    }
    return false
  }

  async function loadMoreConversations() {
    if (isLoadingPrev || nextConversationCursor) return false
    isLoadingPrev = true
    try {
      const result = await assistantStore.listConversations(
        nextConversationCursor,
        10
      )
      if (result.result) {
        conversations = [...conversations, ...result.result]
        nextConversationCursor = result.next_cursor
        return !!nextConversationCursor
      }
    } catch (error) {
      console.error('Failed to load more conversations:', error)
    } finally {
      isLoadingPrev = false
    }
    return false
  }

  function handleViewChange(view: 'kip' | 'conversation') {
    currentView = view
    if (currentView === 'kip') {
      loadKipLogs()
    } else if (currentView === 'conversation') {
      loadConversations()
    }
  }

  function formatJson(obj: any): string {
    return JSON.stringify(obj, null, 2)
  }

  let scrollContainer: HTMLDivElement

  onMount(() => {
    const abortScroll = scrollOnHooks(scrollContainer!, {
      onTop: () => {
        if (currentView == 'kip') {
          loadMoreKipLogs().then((hasMore) => {
            if (hasMore) {
              tick().then(() => {
                scrollContainer!.scrollTop = 100
              })
            }
          })
        } else {
          loadMoreConversations().then((hasMore) => {
            if (hasMore) {
              tick().then(() => {
                scrollContainer!.scrollTop = 100
              })
            }
          })
        }
      },
      onBottom: () => {
        if (currentView == 'kip') {
          loadKipLogs()
        } else {
          loadMoreConversations()
        }
      }
    })
    return abortScroll
  })
</script>

<Modal bind:open={isOpen} size="xl" class="h-[90vh] w-full max-w-4xl">
  {#snippet header()}
    <div class="flex items-center justify-between">
      <div class="flex gap-2">
        <Button
          size="sm"
          color={currentView === 'conversation' ? 'primary' : 'alternative'}
          class="ring-0!"
          onclick={() => handleViewChange('conversation')}
        >
          Conversations
        </Button>
        <Button
          size="sm"
          color={currentView === 'kip' ? 'primary' : 'alternative'}
          class="ring-0!"
          onclick={() => handleViewChange('kip')}
        >
          KIP Logs
        </Button>
      </div>
    </div>
  {/snippet}

  <div
    bind:this={scrollContainer}
    class="flex h-full flex-col overflow-y-auto scroll-smooth"
  >
    {#if isLoadingPrev}
      <div class="flex items-center justify-center p-8">
        <Spinner size="6" />
      </div>
    {/if}
    {#if currentView === 'kip'}
      {#if kipLogs.length === 0}
        <div class="p-8 text-center text-gray-500"> No KIP logs found. </div>
      {:else}
        {#each kipLogs as log (log._id)}
          <div
            id={`dm-kiplog-${log._id}`}
            class="mx-2 mb-4 rounded-lg border border-gray-200 bg-white p-4 dark:border-gray-700 dark:bg-gray-800"
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
      <div class="p-8 text-center text-gray-500"> No conversations found. </div>
    {:else}
      {#each conversations as conversation (conversation._id)}
        <div
          id={`dm-conversation-${conversation._id}`}
          class="mx-2 mb-4 rounded-lg border border-gray-200 bg-white p-4 dark:border-gray-700 dark:bg-gray-800"
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

    {#if isLoading}
      <div class="flex items-center justify-center p-4">
        <Spinner size="4" />
        <span class="ml-2 text-sm text-gray-500">Loading more...</span>
      </div>
    {/if}
  </div>
</Modal>
