<script lang="ts">
  import { Button } from 'flowbite-svelte'

  let {
    onSend,
    disabled = false
  }: {
    onSend: (message: string) => void
    disabled?: boolean
  } = $props()

  let message = $state('')
  let textareaRef = $state() as HTMLTextAreaElement
  let submitting = $state(false)

  function handleSend() {
    const trimmedMessage = message.trim()
    if (trimmedMessage && !disabled) {
      onSend(trimmedMessage)
      message = ''
      // Reset textarea height
      if (textareaRef) {
        textareaRef.style.height = 'auto'
      }
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (
      !event.shiftKey &&
      !submitting &&
      event.metaKey &&
      (event.keyCode == 13 ||
        (!event.isComposing && ['Enter'].includes(event.code)))
    ) {
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
</script>

<div
  class="grid grid-cols-[1fr_auto] items-end gap-4 bg-transparent py-4 pr-4 pl-8"
>
  <textarea
    id="prompt-input"
    bind:this={textareaRef}
    bind:value={message}
    placeholder="Start typing a prompt"
    rows={1}
    class="min-h-8 resize-none rounded-lg border-0 p-2 placeholder-gray-500 focus:border-0 focus:ring-0 dark:placeholder-gray-400"
    onkeydown={handleKeydown}
    oninput={autoResize}
    {disabled}
  ></textarea>
  <Button
    onclick={handleSend}
    disabled={disabled || !message.trim()}
    class="flex items-center justify-center rounded-4xl border-0 px-4 py-2"
  >
    <span>Run <b>⌘ ↵</b></span>
  </Button>
</div>
