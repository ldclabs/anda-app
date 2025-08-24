<script lang="ts">
  import { t } from '$lib/stores/i18n'
  import { type Resource } from '$lib/types/assistant'
  import { fileToBase64Url, formatSize } from '$lib/utils/helper'
  import { isTauriEnvironment, safeOsType } from '$lib/utils/tauri.mock'
  import { sha3_256 } from '@ldclabs/cose-ts/hash'
  import { bytesToBase64Url } from '@ldclabs/cose-ts/utils'
  import { type as osType } from '@tauri-apps/plugin-os'
  import { Button } from 'flowbite-svelte'
  import {
    CirclePlusOutline,
    CloseOutline,
    FileImageOutline,
    FileLinesOutline,
    FileMusicOutline,
    FileOutline,
    FileVideoOutline
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
    onInput,
    disabled = false
  }: {
    user: string
    onSend: (message: string, resource: Resource[]) => void
    onInput?: (event: Event) => void
    disabled?: boolean
  } = $props()

  const messageCacheKey = `MessageDraft:${user}`

  let textareaRef: HTMLTextAreaElement
  let fileInputRef = $state<HTMLInputElement>()
  let message = $state(localStorage.getItem(messageCacheKey) || '')
  let files = $state<Resource[]>([])
  let submitting = $state(false)

  async function addFile(file: File) {
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
  <div class="flex items-center gap-2 p-1">
    <span class="grid place-items-center text-gray-600 dark:text-gray-300">
      <Icon size="md" />
    </span>

    <span>{file.mime_type}</span>
    <span>{formatSize(file.size!)}</span>
    <span class="truncate">{file.name}</span>
    <Button
      onclick={() => removeFile(file.hash!)}
      color="alternative"
      size="xs"
      class="shadow-0 ml-4 rounded-full border-0 p-1"
    >
      <CloseOutline size="lg" />
    </Button>
  </div>
{/snippet}

<div class="mt-6 bg-white/90 px-8 text-gray-500 dark:bg-gray-800/90">
  {#each files as file (file.hash)}
    {@render fileCard(file)}
  {/each}
</div>
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
      color="alternative"
      class="shadow-0 rounded-full p-1"
    >
      <CirclePlusOutline size="lg" />
    </Button>
    <Button
      onclick={handleSend}
      disabled={disabled || !message.trim()}
      class="flex items-center justify-center rounded-4xl border-0 px-4 py-2"
    >
      <span>{t('assistant.run')} <b>{shortcutLabel}</b></span>
    </Button>
  </div>
</div>
