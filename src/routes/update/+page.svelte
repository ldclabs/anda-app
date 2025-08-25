<script lang="ts">
  import { t } from '$lib/stores/i18n'
  import { updaterStore } from '$lib/stores/updater.svelte'
  import { renderMarkdown } from '$lib/utils/markdown'
  import { openUrl } from '@tauri-apps/plugin-opener'
  import { Button, Spinner } from 'flowbite-svelte'
  import {
    ArrowUpRightFromSquareOutline,
    RefreshOutline
  } from 'flowbite-svelte-icons'
  import { onMount, tick } from 'svelte'

  // 原始 ESC 控制序列
  const ansiPattern =
    /[\u001B\u009B][[\]()#;?]*(?:\d{1,4}(?:;\d{0,4})*)?[0-9A-ORZcf-nqry=><~]/g
  // 被字符串转义成字面量的 \u001b[...]m
  const escapedAnsiPattern = /\\u001b\[[0-9;]*[A-Za-z]/gi
  // 去除 ANSI 转义序列（同时兼容被转义成字面量 \\u001b 的情况）
  function stripAnsi(text: string): string {
    return text.replace(ansiPattern, '').replace(escapedAnsiPattern, '')
  }

  const releaseNotesHtml = $derived.by(() => {
    const notes = updaterStore.info?.notes || ''
    if (!notes) return ''
    const [html, hook] = renderMarkdown(stripAnsi(notes))
    tick().then(hook)
    return html
  })

  let isChecked = $state(false)
  onMount(async () => {
    await updaterStore.check_update()
    isChecked = true
  })
</script>

<div class="mx-auto max-w-2xl p-6">
  <h1 class="mb-6 text-2xl font-bold">{t('app.check_update')}</h1>

  {#if updaterStore.info}
    <div
      class="mb-6 space-y-2 rounded-lg border border-gray-200 p-4 dark:border-gray-700"
    >
      <div class="text-gray-600 dark:text-gray-300">
        <div
          >{t('app.current_version')}<span class="ml-2 font-mono"
            >{updaterStore.info.current_version}</span
          ></div
        >
        <div>
          <a
            href="https://github.com/ldclabs/anda-app/releases"
            target="_blank"
            rel="noopener noreferrer"
            class="text-primary-500 flex flex-row items-center gap-2"
            onclick={(e) => {
              e.preventDefault()
              console.log('openUrl')
              openUrl('https://github.com/ldclabs/anda-app/releases')
            }}
          >
            {t('app.new_version')}<span class="font-mono"
              >{updaterStore.info.version}</span
            >
            <span class="ml-2">Github</span><ArrowUpRightFromSquareOutline
              size="md"
            />
          </a>
        </div>
      </div>
    </div>

    {#if updaterStore.isDownloading}
      <div
        class="flex items-center gap-3 rounded-lg border border-gray-200 p-4 text-gray-700 dark:border-gray-700 dark:text-gray-200"
      >
        <Spinner size="4" class="text-green-500" />
        <span>
          {t('app.download_update', { version: updaterStore.info.version })}
        </span>
      </div>
    {:else}
      <div class="flex flex-col items-center gap-4">
        <p class="text-sm text-gray-500 dark:text-gray-400">
          {t('app.update_ready')}
        </p>
        <Button
          color="green"
          size="md"
          onclick={() => updaterStore.restartApp()}
          disabled={updaterStore.isRestarting}
        >
          <span class="inline-flex items-center">
            <RefreshOutline class="me-2" size="lg" />
            {t('app.update_restart')}
            {#if updaterStore.isRestarting}
              <Spinner class="ms-3 inline-flex" size="4" />
            {/if}
          </span>
        </Button>
      </div>
    {/if}

    {#if updaterStore.info?.notes}
      <div
        class="mt-6 rounded-lg border border-gray-200 p-4 dark:border-gray-700"
      >
        <h2 class="mb-3 text-lg font-semibold">{t('app.release_notes')}</h2>
        <div class="text-sm leading-6 text-gray-800 dark:text-gray-200">
          {@html releaseNotesHtml}
        </div>
      </div>
    {/if}
  {:else}
    <div
      class="rounded-lg border border-gray-200 p-6 text-gray-700 dark:border-gray-700 dark:text-gray-200"
    >
      <div class="mb-3 flex flex-row items-center gap-3">
        {#if isChecked}
          <span class="leading-none">{t('app.no_update_available')}</span>
        {:else}
          <span class="leading-none">{t('app.checking_update')}</span>
          <Spinner size="4" />
        {/if}
      </div>
      <p class="text-sm text-gray-500 dark:text-gray-400">
        {t('app.checking_update_tip')}
      </p>
    </div>
  {/if}
</div>
