<script lang="ts">
  import type { ChatMessage } from '$lib/types/chat'
  import { getPlainText, renderMarkdown } from '$lib/utils/markdown'
  import { Clipboard, Spinner } from 'flowbite-svelte'
  import {
    CheckOutline,
    FileCloneOutline,
    FileCopyAltOutline
  } from 'flowbite-svelte-icons'
  import { onMount } from 'svelte'

  let { message }: { message: ChatMessage } = $props()
  const [content, hook] = renderMarkdown(message.content)
  let text = $state(getPlainText(message.content))

  let textSuccess = $state(false)
  let mdSuccess = $state(false)

  function formatTime(date: Date): string {
    return date.toLocaleTimeString('zh-CN', {
      hour: '2-digit',
      minute: '2-digit'
    })
  }

  onMount(async () => {
    await hook()
  })
</script>

<div
  class="group hover:border-secondary-500/50 relative flex max-w-4xl gap-3 rounded-lg border border-transparent p-4 {message.role ===
  'user'
    ? 'flex-row-reverse'
    : 'flex-row'}"
>
  <div
    class="flex flex-col gap-1 {message.role === 'user'
      ? 'max-w-[90%] items-end'
      : 'w-full items-start'}"
  >
    <div
      class="pointer-events-none sticky top-0 z-10 -mt-12 flex flex-row gap-2 rounded-lg bg-white/80 p-2 opacity-0 shadow-xs backdrop-blur-xs transition-opacity duration-200 group-hover:pointer-events-auto group-hover:opacity-100 {message.role ===
      'user'
        ? '-mr-2'
        : '-ml-2'}"
    >
      <Clipboard
        bind:value={text}
        bind:success={textSuccess}
        size="sm"
        color="alternative"
        class="focus-within:ring-0 focus:ring-0"
      >
        {#if textSuccess}<CheckOutline />{:else}<FileCopyAltOutline />{/if}Copy
        text
      </Clipboard>
      <Clipboard
        bind:value={message.content}
        bind:success={mdSuccess}
        size="sm"
        color="alternative"
        class="focus-within:ring-0 focus:ring-0"
      >
        {#if mdSuccess}<CheckOutline />{:else}<FileCloneOutline />{/if}Copy
        markdown
      </Clipboard>
    </div>
    <div
      class="w-full {message.role === 'user'
        ? 'rounded-lg bg-gray-50 px-4 py-2 text-gray-900 dark:bg-gray-800 dark:text-white'
        : ''}"
    >
      {#if message.isTyping}
        <div class="flex items-center gap-2">
          <Spinner size="4" />
          <span class="text-sm">AI 正在思考...</span>
        </div>
      {:else}
        <div class="chat-content w-full text-pretty break-words"
          >{@html content}</div
        >
      {/if}
    </div>

    {#if !message.isTyping}
      <span class="text-xs text-gray-500 dark:text-gray-400">
        {formatTime(message.timestamp)}
      </span>
    {/if}
  </div>
</div>

<style>
  .chat-content {
    /* 基础排版 */
    line-height: 1.6;
    color: inherit;
  }

  /* 标题样式 */
  .chat-content :global(h1),
  .chat-content :global(h2),
  .chat-content :global(h3),
  .chat-content :global(h4),
  .chat-content :global(h5),
  .chat-content :global(h6) {
    margin: 1.2em 0 0.6em 0;
    font-weight: 600;
    line-height: 1.3;
  }

  .chat-content :global(h1) {
    font-size: 1.5em;
  }
  .chat-content :global(h2) {
    font-size: 1.3em;
  }
  .chat-content :global(h3) {
    font-size: 1.1em;
  }

  /* 段落和文本 */
  .chat-content :global(p) {
    margin: 0.8em 0;
  }

  .chat-content :global(p:first-child) {
    margin-top: 0;
  }

  .chat-content :global(p:last-child) {
    margin-bottom: 0;
  }

  /* 强调文本 */
  .chat-content :global(strong) {
    font-weight: 600;
  }

  .chat-content :global(em) {
    font-style: italic;
  }

  /* 列表样式 */
  .chat-content :global(ul),
  .chat-content :global(ol) {
    margin: 0.8em 0;
    padding-left: 1.5em;
  }

  .chat-content :global(li) {
    margin: 0.3em 0;
  }

  .chat-content :global(ul li) {
    list-style-type: disc;
  }

  .chat-content :global(ol li) {
    list-style-type: decimal;
  }

  /* 引用块 */
  .chat-content :global(blockquote) {
    margin: 1em 0;
    padding: 0.8em 1em;
    border-left: 4px solid #e5e7eb;
    background-color: #f9fafb;
    font-style: italic;
  }

  :global(.dark) .chat-content :global(blockquote) {
    border-left-color: #4b5563;
    background-color: #1f2937;
  }

  /* 行内代码 */
  .chat-content :global(code:not(pre code)) {
    background-color: #f3f4f6;
    color: #dc2626;
    padding: 0.2em 0.4em;
    border-radius: 0.25rem;
    font-size: 0.875em;
    font-family:
      'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas,
      'Courier New', monospace;
  }

  :global(.dark) .chat-content :global(code:not(pre code)) {
    background-color: #374151;
    color: #fca5a5;
  }

  /* 代码块 */
  .chat-content :global(pre:not([class*='language-'])) {
    margin: 1em 0;
    padding: 1em;
    background-color: #f8fafc;
    border: 1px solid #e2e8f0;
    border-radius: 0.5rem;
    overflow-x: auto;
    font-size: 0.875em;
    line-height: 1.5;
  }

  :global(.dark) .chat-content :global(pre:not([class*='language-'])) {
    background-color: #1e293b;
    border-color: #475569;
  }

  .chat-content :global(pre code:not([class*='language-'])) {
    background: none;
    color: inherit;
    padding: 0;
    border-radius: 0;
    font-size: inherit;
  }

  /* 链接 */
  .chat-content :global(a) {
    color: #2563eb;
    text-decoration: underline;
    text-decoration-color: transparent;
    transition: text-decoration-color 0.2s;
  }

  .chat-content :global(a:hover) {
    text-decoration-color: currentColor;
  }

  :global(.dark) .chat-content :global(a) {
    color: #60a5fa;
  }

  /* 表格 */
  .chat-content :global(table) {
    width: 100%;
    margin: 1em 0;
    border-collapse: collapse;
    border: 1px solid #e5e7eb;
    border-radius: 0.5rem;
    overflow: hidden;
  }

  .chat-content :global(th),
  .chat-content :global(td) {
    padding: 0.75em 1em;
    text-align: left;
    border-bottom: 1px solid #e5e7eb;
  }

  .chat-content :global(th) {
    background-color: #f9fafb;
    font-weight: 600;
  }

  :global(.dark) .chat-content :global(table) {
    border-color: #4b5563;
  }

  :global(.dark) .chat-content :global(th),
  :global(.dark) .chat-content :global(td) {
    border-bottom-color: #4b5563;
  }

  :global(.dark) .chat-content :global(th) {
    background-color: #374151;
  }

  /* 分隔线 */
  .chat-content :global(hr) {
    margin: 1.5em 0;
    border: none;
    border-top: 1px solid #e5e7eb;
  }

  :global(.dark) .chat-content :global(hr) {
    border-top-color: #4b5563;
  }

  /* 图片 */
  .chat-content :global(img) {
    max-width: 100%;
    height: auto;
    border-radius: 0.5rem;
    margin: 0.5em 0;
  }
</style>
