<script lang="ts">
  import ExternalLinkLine from '$lib/components/icons/ExternalLinkLine.svelte'
  import RefreshLine from '$lib/components/icons/RefreshLine.svelte'
  import { t } from '$lib/stores/i18n'
  import { updaterStore } from '$lib/stores/updater.svelte'
  import { renderMarkdown } from '$lib/utils/markdown'
  import { osType } from '$lib/utils/tauri.mock'
  import { openUrl } from '@tauri-apps/plugin-opener'
  import { Button, Spinner } from 'flowbite-svelte'
  import { onMount, tick } from 'svelte'

  const ot = osType()
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

<div
  class="mx-auto min-h-screen max-w-2xl p-6 dark:bg-gray-800 dark:text-gray-200"
>
  <h1 class="mb-6 text-2xl font-bold">{t('app.check_update')}</h1>
  {#if updaterStore.info}
    <div
      class="mb-6 space-y-2 rounded-lg border border-gray-200 p-4 dark:border-gray-700"
    >
      <div class="text-gray-600 dark:text-gray-300">
        <div
          >{t('app.current_version', {
            version: updaterStore.info.current_version
          })}</div
        >
        <div>
          <a
            href="https://github.com/ldclabs/anda-app/releases"
            target="_blank"
            rel="noopener noreferrer"
            class="text-primary-500 flex flex-row items-center gap-2"
            onclick={(e) => {
              e.preventDefault()
              openUrl('https://github.com/ldclabs/anda-app/releases')
            }}
          >
            {t('app.new_version', { version: updaterStore.info.version })}
            <span class="ml-2">Github</span>
            <span class="*:size-5"><ExternalLinkLine /></span>
          </a>
        </div>
      </div>
    </div>

    {#if ot === 'macos'}
      <!-- macOS sandbox: 引导用户前往 GitHub 下载并手动安装 -->
      <div
        class="flex flex-col items-center gap-4 rounded-lg border border-gray-200 p-4 text-gray-700 dark:border-gray-700 dark:text-gray-200"
      >
        <p class="text-center text-sm text-gray-500 dark:text-gray-400">
          {t('app.update_manual_mac')}
        </p>
        <Button
          color="green"
          size="md"
          class="flex flex-row items-center gap-2"
          onclick={() =>
            openUrl('https://github.com/ldclabs/anda-app/releases')}
        >
          <span class="">{t('app.go_to_download')}</span>
          <span class="*:size-5"><ExternalLinkLine /></span>
        </Button>
      </div>
    {:else if updaterStore.isDownloading}
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
          class="flex flex-row items-center gap-2"
          onclick={() => updaterStore.restartApp()}
          disabled={updaterStore.isRestarting}
        >
          <span class="*:size-5"><RefreshLine /></span>
          <span>
            {t('app.update_restart')}
          </span>
          {#if updaterStore.isRestarting}
            <Spinner size="4" />
          {/if}
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
