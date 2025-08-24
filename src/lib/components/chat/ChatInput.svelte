<script lang="ts">
  import { t } from '$lib/stores/i18n'
  import { triggerToast } from '$lib/stores/toast.svelte'
  import { type Resource } from '$lib/types/assistant'
  import { fileToBase64Url, formatSize } from '$lib/utils/helper'
  import { isTauriEnvironment, safeOsType } from '$lib/utils/tauri.mock'
  import { sha3_256 } from '@ldclabs/cose-ts/hash'
  import { bytesToBase64Url } from '@ldclabs/cose-ts/utils'
  import { type as osType } from '@tauri-apps/plugin-os'
  import { Button, Spinner } from 'flowbite-svelte'
  import {
    CirclePlusOutline,
    CloseOutline,
    FileImageOutline,
    FileLinesOutline,
    FileMusicOutline,
    FileOutline,
    FileVideoOutline,
    StopSolid
  } from 'flowbite-svelte-icons'
  import mime from 'mime'
  import { onDestroy } from 'svelte'
  import MIMEType from 'whatwg-mimetype'

  const ot = isTauriEnvironment() ? osType() : safeOsType()
  const shortcutLabel = ot === 'macos' ? '⌘ ↵' : 'Ctrl ↵'
  const isSubmitEvent =
    ot === 'macos'
      ? (event: KeyboardEvent) => {
          return (
            !event.shiftKey &&
            !submitting &&
            event.metaKey &&
            (event.keyCode == 13 ||
              (!event.isComposing && ['Enter'].includes(event.code)))
          )
        }
      : (event: KeyboardEvent) => {
          return (
            !event.shiftKey &&
            !submitting &&
            event.ctrlKey &&
            (event.keyCode == 13 ||
              (!event.isComposing && ['Enter'].includes(event.code)))
          )
        }

  let {
    user,
    onSend,
    onStop,
    onInput,
    disabled,
    isRunning
  }: {
    user: string
    onSend: (message: string, resource: Resource[]) => void
    onStop?: () => void
    onInput?: (event: Event) => void
    disabled?: boolean
    isRunning?: boolean
  } = $props()

  const messageCacheKey = `MessageDraft:${user}`

  let textareaRef: HTMLTextAreaElement
  let fileInputRef = $state<HTMLInputElement>()
  let message = $state(localStorage.getItem(messageCacheKey) || '')
  let files = $state<Resource[]>([])
  let submitting = $state(false)

  async function addFile(file: File) {
    if (file.size > 2 * 1024 * 1024) {
      triggerToast({
        type: 'error',
        message: t('assistant.file_size_too_large')
      })
      return
    }

    const bytes = await file.bytes()
    const mimeType = file.type || mime.getType(file.name) || ''
    const tags = []
    if (mimeType) {
      const tag = mime.getExtension(mimeType)
      if (tag) {
        tags.push(tag) // add extension as a tag
      }
      const mi = new MIMEType(mimeType)
      if (mi) {
        tags.push(mi.type) // add media type as a tag
      }
    }
    const hash = bytesToBase64Url(sha3_256(bytes))
    const blob = await fileToBase64Url(file)
    for (const f of files) {
      if (f.hash === hash) {
        return
      }
    }

    files.push({
      _id: 0,
      hash,
      blob,
      tags,
      mime_type: mimeType,
      name: file.name,
      size: file.size,
      metadata: {
        lastModified: file.lastModified,
        relativePath: file.webkitRelativePath
      }
    })
  }

  function removeFile(hash: string) {
    files = files.filter((f) => f.hash !== hash)
  }

  function fileIcon(mime_type: string) {
    if (mime_type.startsWith('text/')) {
      return FileLinesOutline
    } else if (mime_type.startsWith('image/')) {
      return FileImageOutline
    } else if (mime_type.startsWith('video/')) {
      return FileVideoOutline
    } else if (mime_type.startsWith('audio/')) {
      return FileMusicOutline
    }
    return FileOutline
  }

  function handlePaste(event: ClipboardEvent) {
    const files = event.clipboardData?.files || []
    if (files.length > 0) {
      event.preventDefault()
      for (const f of files) {
        addFile(f)
      }
    }
  }

  function openFilePicker() {
    fileInputRef?.click()
  }

  function handleFileChange(event: Event) {
    const input = event.currentTarget as HTMLInputElement
    const files = input?.files
    if (files && files.length > 0) {
      for (const f of files) {
        addFile(f)
      }
    }
    // 清空以便下次选择相同文件也能触发 change
    if (input) input.value = ''
  }

  function handleSend() {
    if (isRunning) {
      onStop?.()
      return
    }

    const trimmedMessage = message.trim()
    if (trimmedMessage && !disabled) {
      onSend(trimmedMessage, $state.snapshot(files))
      message = ''
      files = []
      localStorage.removeItem(messageCacheKey)
      // Reset textarea height
      if (textareaRef) {
        textareaRef.style.height = 'auto'
      }
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (isSubmitEvent(event)) {
      event.preventDefault()
      handleSend()
    }
  }

  function handleInput(event: Event) {
    if (textareaRef) {
      textareaRef.style.height = 'auto'
      // 限制最大高度为视口高度的40%
      const maxHeight = window.innerHeight * 0.4
      const newHeight = Math.min(textareaRef.scrollHeight, maxHeight)
      textareaRef.style.height = newHeight + 'px'
    }
    if (onInput) {
      onInput(event)
    }
  }

  onDestroy(() => {
    message = message.trim()
    if (message) {
      localStorage.setItem(messageCacheKey, message)
    }
  })
</script>

{#snippet fileCard(file: Resource)}
  {@const Icon = fileIcon(file.mime_type!)}
  <div class="grid grid-cols-[1fr_auto] p-1">
    <div class="flex min-w-0 flex-row items-center gap-2">
      <span class="grid place-items-center text-gray-600 dark:text-gray-300">
        <Icon size="md" />
      </span>

      <span>{file.mime_type}</span>
      <span>{formatSize(file.size!)}</span>
      <p class="min-w-0 flex-1 truncate">{file.name}</p>
    </div>
    <Button
      onclick={() => removeFile(file.hash!)}
      color="alternative"
      size="xs"
      class="shadow-0 rounded-full border-0 p-1"
    >
      <CloseOutline size="lg" />
    </Button>
  </div>
{/snippet}

{#if files.length > 0}
  <div class="mt-6 bg-white/90 pr-2 pl-8 text-gray-500 dark:bg-gray-800/90">
    {#each files as file (file.hash)}
      {@render fileCard(file)}
    {/each}
  </div>
{/if}

<div
  class="relative grid grid-cols-[1fr_auto] items-end gap-4 bg-transparent py-4 pr-4 pl-8"
>
  <textarea
    id="prompt-input"
    bind:this={textareaRef}
    bind:value={message}
    placeholder={t('assistant.prompt.placeholder')}
    rows={1}
    class="min-h-8 resize-none rounded-lg border-0 p-2 placeholder-gray-500 focus:border-0 focus:ring-0 dark:bg-gray-800 dark:text-white dark:placeholder-gray-400"
    onkeydown={handleKeydown}
    oninput={handleInput}
    onpaste={handlePaste}
    {disabled}
  ></textarea>
  <input
    id="file-input"
    name="file-input"
    bind:this={fileInputRef}
    type="file"
    multiple
    accept=".txt,.md,.json,.pdf,text/plain,text/markdown,application/json,application/pdf"
    class="sr-only"
    onchange={handleFileChange}
  />

  <div class="flex items-center gap-2">
    <Button
      onclick={openFilePicker}
      {disabled}
      color="alternative"
      class="shadow-0 rounded-full p-1 disabled:text-gray-500/50"
    >
      <CirclePlusOutline size="lg" />
    </Button>
    <Button
      onclick={handleSend}
      disabled={!isRunning && (disabled || !message.trim())}
      class="flex transform-gpu items-center justify-center rounded-4xl border-0 p-2"
    >
      {#if isRunning}
        <div class="relative -m-1 grid size-8 place-items-center">
          <Spinner class="absolute inset-0 size-8 text-gray-500/50" />
          <StopSolid size="md" class="z-10" />
        </div>
      {:else}
        <span class="px-2">{t('assistant.run')} <b>{shortcutLabel}</b></span>
      {/if}
    </Button>
  </div>
</div>
