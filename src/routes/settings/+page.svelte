<script lang="ts">
  import { page } from '$app/state'
  import { t } from '$lib/stores/i18n'
  import {
    get_secret_setting,
    get_settings,
    secretSettingsStore,
    set_secret_setting,
    set_setting,
    settingsStore,
    type SecretSettings,
    type Settings
  } from '$lib/stores/settings.svelte'
  import { Input, Select, Toast } from 'flowbite-svelte'
  import {
    CheckCircleOutline,
    CogOutline,
    ExclamationCircleOutline,
    EyeOutline,
    GlobeOutline,
    WandMagicSparklesOutline
  } from 'flowbite-svelte-icons'
  import { onMount } from 'svelte'

  let isLoading = $state(false)

  let notification = $state<{
    type: 'success' | 'error'
    message: string
  } | null>(null)
  let toastStatus = $derived.by(() => !!notification)

  let activeSection = $state(page.url.searchParams.get('section') || 'general')

  const navigationItems = $derived([
    { id: 'general', name: t('settings.nav.general'), icon: CogOutline },
    { id: 'appearance', name: t('settings.nav.appearance'), icon: EyeOutline },
    { id: 'network', name: t('settings.nav.network'), icon: GlobeOutline },
    { id: 'ai', name: t('settings.nav.ai'), icon: WandMagicSparklesOutline }
  ])

  const themeOptions = $derived([
    { value: 'light', name: t('settings.theme.light') },
    { value: 'dark', name: t('settings.theme.dark') },
    { value: '', name: t('settings.theme.system') }
  ])

  const localeOptions = $derived([
    { value: 'en', name: t('language.english') },
    { value: 'zh', name: t('language.chinese') }
  ])

  function showNotification(type: 'success' | 'error', message: string) {
    notification = { type, message }
    setTimeout(() => {
      notification = null
    }, 10000)
  }

  // 更新设置
  async function updateSetting<K extends keyof Settings>(
    key: K,
    value: Settings[K]
  ) {
    try {
      isLoading = true
      await set_setting(key, value)
      showNotification('success', t('settings.saved'))
    } catch (error) {
      console.error('Failed to update setting:', error)
      showNotification('error', t('settings.save_failed'))
    } finally {
      isLoading = false
    }
  }

  // 更新秘密设置
  async function updateSecretSetting<K extends keyof SecretSettings>(
    key: K,
    value: SecretSettings[K]
  ) {
    try {
      isLoading = true
      await set_secret_setting(key, value)
      showNotification('success', t('settings.saved'))
    } catch (error) {
      console.error('Failed to update secret setting:', error)
      showNotification('error', t('settings.save_failed'))
    } finally {
      isLoading = false
    }
  }

  // 处理代理设置变化
  function handleProxyChange(event: Event) {
    const target = event.target as HTMLInputElement
    updateSetting('https_proxy', target.value || undefined)
  }

  // 组件挂载时加载设置
  onMount(async () => {
    try {
      isLoading = true
      await get_settings()
      await get_secret_setting('gemini_api_key')
    } catch (error) {
      console.error('Failed to load settings:', error)
      showNotification('error', t('settings.load_failed'))
    } finally {
      isLoading = false
    }
  })
</script>

<svelte:head>
  <title>{t('settings.title')} - Anda AI</title>
</svelte:head>

<div class="flex h-screen bg-gray-50 dark:bg-gray-900">
  <!-- 左侧导航栏 -->
  <div
    class="flex w-64 flex-col border-r border-gray-200 bg-white dark:border-gray-700 dark:bg-gray-800"
  >
    <!-- 标题 -->
    <div class="border-b border-gray-200 p-6 dark:border-gray-700">
      <h1
        class="flex items-center gap-2 text-xl font-semibold text-gray-900 dark:text-white"
      >
        <CogOutline class="h-5 w-5" />
        {t('settings.title')}
      </h1>
    </div>

    <!-- 导航菜单 -->
    <nav class="flex-1 p-4">
      <ul class="space-y-1">
        {#each navigationItems as item}
          {@const Icon = item.icon}
          <li>
            <button
              class="flex w-full items-center gap-3 rounded-lg px-3 py-2 text-left transition-colors duration-150 {activeSection ===
              item.id
                ? 'bg-blue-50 text-blue-600 dark:bg-blue-900/20 dark:text-blue-400'
                : 'text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700'}"
              onclick={() => (activeSection = item.id)}
            >
              <Icon class="h-4 w-4" />
              {item.name}
            </button>
          </li>
        {/each}
      </ul>
    </nav>
  </div>

  <!-- 右侧内容区域 -->
  <div class="flex-1 overflow-auto">
    <div class="relative mx-auto max-w-2xl p-8">
      <!-- 通用设置 -->
      {#if activeSection === 'general'}
        <div class="space-y-8">
          <div>
            <h2 class="mb-2 text-2xl font-bold text-gray-900 dark:text-white"
              >{t('settings.general.title')}</h2
            >
            <p class="text-gray-600 dark:text-gray-400"
              >{t('settings.general.description')}</p
            >
          </div>

          <!-- 语言设置 -->
          <div
            class="rounded-xl border border-gray-200 bg-white p-6 dark:border-gray-700 dark:bg-gray-800"
          >
            <div class="flex items-center justify-between">
              <div class="flex-1">
                <h3
                  class="mb-1 text-lg font-medium text-gray-900 dark:text-white"
                  >{t('settings.language.title')}</h3
                >
                <p class="text-sm text-gray-500 dark:text-gray-400"
                  >{t('settings.language.description')}</p
                >
              </div>
              <div class="w-32">
                <Select
                  bind:value={settingsStore.locale}
                  onchange={() => updateSetting('locale', settingsStore.locale)}
                  disabled={isLoading}
                  class="text-sm"
                >
                  {#each localeOptions as option}
                    <option value={option.value}>{option.name}</option>
                  {/each}
                </Select>
              </div>
            </div>
          </div>
        </div>
      {/if}

      <!-- 外观设置 -->
      {#if activeSection === 'appearance'}
        <div class="space-y-8">
          <div>
            <h2 class="mb-2 text-2xl font-bold text-gray-900 dark:text-white"
              >{t('settings.appearance.title')}</h2
            >
            <p class="text-gray-600 dark:text-gray-400"
              >{t('settings.appearance.description')}</p
            >
          </div>

          <!-- 主题设置 -->
          <div
            class="rounded-xl border border-gray-200 bg-white p-6 dark:border-gray-700 dark:bg-gray-800"
          >
            <div class="flex items-center justify-between">
              <div class="flex-1">
                <h3
                  class="mb-1 text-lg font-medium text-gray-900 dark:text-white"
                  >{t('settings.theme.title')}</h3
                >
                <p class="text-sm text-gray-500 dark:text-gray-400"
                  >{t('settings.theme.description')}</p
                >
              </div>
              <div class="w-32">
                <Select
                  bind:value={settingsStore.theme}
                  onchange={() => updateSetting('theme', settingsStore.theme)}
                  disabled={isLoading}
                  class="text-sm"
                >
                  {#each themeOptions as option}
                    <option value={option.value}>{option.name}</option>
                  {/each}
                </Select>
              </div>
            </div>
          </div>
        </div>
      {/if}

      <!-- 网络设置 -->
      {#if activeSection === 'network'}
        <div class="space-y-8">
          <div>
            <h2 class="mb-2 text-2xl font-bold text-gray-900 dark:text-white"
              >{t('settings.network.title')}</h2
            >
            <p class="text-gray-600 dark:text-gray-400"
              >{t('settings.network.description')}</p
            >
          </div>

          <!-- HTTPS 代理 -->
          <div
            class="rounded-xl border border-gray-200 bg-white p-6 dark:border-gray-700 dark:bg-gray-800"
          >
            <div class="space-y-4">
              <div>
                <h3
                  class="mb-1 text-lg font-medium text-gray-900 dark:text-white"
                  >{t('settings.proxy.title')}</h3
                >
                <p class="text-sm text-gray-500 dark:text-gray-400"
                  >{t('settings.proxy.description')}</p
                >
              </div>
              <Input
                type="text"
                placeholder={t('settings.proxy.placeholder')}
                value={settingsStore.https_proxy || ''}
                onchange={handleProxyChange}
                disabled={isLoading}
                class="w-full"
              />
            </div>
          </div>
        </div>
      {/if}

      <!-- AI 配置 -->
      {#if activeSection === 'ai'}
        <div class="space-y-8">
          <div>
            <h2 class="mb-2 text-2xl font-bold text-gray-900 dark:text-white"
              >{t('settings.ai.title')}</h2
            >
            <p class="text-gray-600 dark:text-gray-400"
              >{t('settings.ai.description')}</p
            >
          </div>

          <!-- Gemini API Key -->
          <div
            class="rounded-xl border border-gray-200 bg-white p-6 dark:border-gray-700 dark:bg-gray-800"
          >
            <div class="space-y-4">
              <div>
                <h3
                  class="mb-1 text-lg font-medium text-gray-900 dark:text-white"
                  >{t('settings.gemini.title')}</h3
                >
                <p class="text-sm text-gray-500 dark:text-gray-400"
                  >{t('settings.gemini.description')}</p
                >
              </div>
              <Input
                type="text"
                placeholder={t('settings.gemini.placeholder')}
                value={secretSettingsStore.gemini_api_key || ''}
                onchange={(e) =>
                  updateSecretSetting(
                    'gemini_api_key',
                    (e.target as HTMLInputElement).value
                  )}
                disabled={isLoading}
                class="w-full"
              />
            </div>
          </div>

          <!-- OpenAI API Key (GPT-5) -->
          <div
            class="rounded-xl border border-gray-200 bg-white p-6 dark:border-gray-700 dark:bg-gray-800"
          >
            <div class="space-y-4">
              <div>
                <h3
                  class="mb-1 text-lg font-medium text-gray-900 dark:text-white"
                  >{t('settings.openai.title')}</h3
                >
                <p class="text-sm text-gray-500 dark:text-gray-400"
                  >{t('settings.openai.description')}</p
                >
              </div>
              <Input
                type="text"
                placeholder={t('settings.openai.placeholder')}
                value={secretSettingsStore.openai_api_key || ''}
                onchange={(e) =>
                  updateSecretSetting(
                    'openai_api_key',
                    (e.target as HTMLInputElement).value
                  )}
                disabled={isLoading}
                class="w-full"
              />
            </div>
          </div>
        </div>
      {/if}
    </div>

    <Toast
      dismissable={false}
      color={notification?.type === 'success' ? 'green' : 'red'}
      class="mx-auto mt-10"
      bind:toastStatus
    >
      {#snippet icon()}
        {#if notification!.type === 'success'}
          <CheckCircleOutline class="h-4 w-4" />
        {:else}
          <ExclamationCircleOutline class="h-4 w-4" />
        {/if}
      {/snippet}
      {notification?.message}
    </Toast>
  </div>
</div>

<style>
  /* 自定义滚动条样式 */
  :global(.overflow-auto::-webkit-scrollbar) {
    width: 6px;
  }

  :global(.overflow-auto::-webkit-scrollbar-track) {
    background: transparent;
  }

  :global(.overflow-auto::-webkit-scrollbar-thumb) {
    background: rgba(156, 163, 175, 0.5);
    border-radius: 3px;
  }

  :global(.overflow-auto::-webkit-scrollbar-thumb:hover) {
    background: rgba(156, 163, 175, 0.7);
  }
</style>
