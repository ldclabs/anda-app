<script lang="ts">
  import { t } from '$lib/stores/i18n'
  import { isTauriEnvironment, safeOsType } from '$lib/utils/tauri.mock'
  import { type as osType } from '@tauri-apps/plugin-os'
  import { Button } from 'flowbite-svelte'
  import { onDestroy } from 'svelte'

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
    disabled = false
  }: {
    user: string
    onSend: (message: string) => void
    disabled?: boolean
  } = $props()

  const messageCacheKey = `MessageDraft:${user}`

  let message = $state(localStorage.getItem(messageCacheKey) || '')
  let textareaRef = $state() as HTMLTextAreaElement
  let submitting = $state(false)

  function handleSend() {
    const trimmedMessage = message.trim()
    if (trimmedMessage && !disabled) {
      onSend(trimmedMessage)
      message = ''
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

  function autoResize() {
    if (textareaRef) {
      textareaRef.style.height = 'auto'
      // 限制最大高度为视口高度的40%
      const maxHeight = window.innerHeight * 0.4
      const newHeight = Math.min(textareaRef.scrollHeight, maxHeight)
      textareaRef.style.height = newHeight + 'px'
    }
  }

  onDestroy(() => {
    message = message.trim()
    if (message) {
      localStorage.setItem(messageCacheKey, message)
    }
  })
</script>

<div
  class="grid grid-cols-[1fr_auto] items-end gap-4 bg-transparent py-4 pr-4 pl-8"
>
  <textarea
    id="prompt-input"
    bind:this={textareaRef}
    bind:value={message}
    placeholder={t('assistant.prompt.placeholder')}
    rows={1}
    class="min-h-8 resize-none rounded-lg border-0 p-2 placeholder-gray-500 focus:border-0 focus:ring-0 dark:bg-gray-800 dark:text-white dark:placeholder-gray-400"
    onkeydown={handleKeydown}
    oninput={autoResize}
    {disabled}
  ></textarea>
  <Button
    onclick={handleSend}
    disabled={disabled || !message.trim()}
    class="flex items-center justify-center rounded-4xl border-0 px-4 py-2"
  >
    <span>{t('assistant.run')} <b>{shortcutLabel}</b></span>
  </Button>
</div>
