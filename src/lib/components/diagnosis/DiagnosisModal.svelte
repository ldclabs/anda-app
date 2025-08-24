<script lang="ts">
  import { assistantStore } from '$lib/stores/assistant.svelte'
  import type { Conversation, KIPLog } from '$lib/types/assistant'
  import { formatDateTime, ID2Cursor, sleep } from '$lib/utils/helper'
  import { scrollIntoView, scrollOnHooks } from '$lib/utils/window'
  import { Button, Modal, Spinner } from 'flowbite-svelte'
  import { onDestroy, tick } from 'svelte'

  const {
    view = $bindable<{
      open: (view: 'kip' | 'conversation', _id?: number) => void
    }>()
  } = $props()
  // 全局单例状态管理
  let kipLogs = $state<KIPLog[]>([])
  let conversations = $state<Conversation[]>([])
  let prevKipCursor = $state<string | undefined>()
  let prevConversationCursor = $state<string | undefined>()
  let isLoading = $state(false)
  let isLoadingPrev = $state(false)
  let currentView = $state<'kip' | 'conversation'>('kip')
  let isOpen = $state(false)

  view.open = (view: 'kip' | 'conversation', _id?: number) => {
    isOpen = true
    currentView = view
    if (view === 'kip') {
      loadKipLogs(_id)
    } else {
      loadConversations(_id)
    }
  }

  function appendConversations(data: Conversation[]) {
    const list = [...conversations]
    for (const item of data) {
      const idx = list.findIndex((i) => i._id === item._id)
      if (idx === -1) {
        list.push(item)
      } else {
        list[idx] = item
      }
    }
    list.sort((a, b) => a._id - b._id)
    conversations = list
  }

  function appendKipLogs(data: KIPLog[]) {
    const list = [...kipLogs]
    for (const item of data) {
      const idx = list.findIndex((i) => i._id === item._id)
      if (idx === -1) {
        list.push(item)
      } else {
        list[idx] = item
      }
    }
    list.sort((a, b) => a._id - b._id)
    kipLogs = list
  }

  async function loadKipLogs(_id?: number) {
    if (isLoading) return

    isLoading = true
    try {
      const result = await assistantStore.listKipLogs(
        _id ? ID2Cursor(_id + 1) : undefined,
        10
      )
      if (result.result) {
        appendKipLogs(result.result)
        prevKipCursor = result.next_cursor
      }

      if (_id) {
        await tick()
        scrollIntoView(`dm-kiplog-${_id}`, 'instant', 'start')
      }
      await sleep(300)
    } catch (error) {
      console.error('Failed to load KIP logs:', error)
    } finally {
      isLoading = false
    }
  }

  async function loadConversations(_id?: number) {
    if (isLoading) return

    isLoading = true
    try {
      const result = await assistantStore.listConversations(
        _id ? ID2Cursor(_id + 1) : undefined,
        10
      )
      if (result.result) {
        appendConversations(result.result)
        prevConversationCursor = result.next_cursor
      }

      if (_id) {
        await tick()
        scrollIntoView(`dm-conversation-${_id}`, 'instant', 'start')
      }

      await sleep(300)
    } catch (error) {
      console.error('Failed to load conversations:', error)
    } finally {
      isLoading = false
    }
  }

  async function loadPrevKipLogs() {
    if (isLoadingPrev || !prevKipCursor) return false
    isLoadingPrev = true
    try {
      const result = await assistantStore.listKipLogs(prevKipCursor, 10)
      if (result.result) {
        appendKipLogs(result.result)
        prevKipCursor = result.next_cursor
        return !!prevKipCursor
      }
      await sleep(300)
    } catch (error) {
      console.error('Failed to load more KIP logs:', error)
    } finally {
      isLoadingPrev = false
    }
    return false
  }

  async function loadPrevConversations() {
    if (isLoadingPrev || !prevConversationCursor) return false
    isLoadingPrev = true
    try {
      const result = await assistantStore.listConversations(
        prevConversationCursor,
        10
      )
      if (result.result) {
        appendConversations(result.result)
        prevConversationCursor = result.next_cursor
        return !!prevConversationCursor
      }
      await sleep(300)
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
  let abortScroll: (() => void) | undefined

  function mountScroll() {
    if (!scrollContainer || abortScroll) return

    abortScroll = scrollOnHooks(scrollContainer, {
      onTop: () => {
        if (currentView == 'kip') {
          loadPrevKipLogs().then((hasMore) => {
            if (hasMore) {
              tick().then(() => {
                scrollContainer!.scrollTop = 100
              })
            }
          })
        } else {
          loadPrevConversations().then((hasMore) => {
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
          loadConversations()
        }
      }
    })
  }

  $effect(() => {
    isOpen && mountScroll()
  })

  onDestroy(() => {
    abortScroll?.()
    abortScroll = undefined
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
    id="diagnosis-modal"
    class="flex h-full flex-col overflow-y-auto scroll-smooth"
  >
    {#if isLoadingPrev}
      <div class="flex items-center justify-center p-8">
        <Spinner size="4" />
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
      </div>
    {/if}
  </div>
</Modal>
