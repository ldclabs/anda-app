<script lang="ts">
  import { page } from '$app/state'
  import { t } from '$lib/stores/i18n'
  import {
    get_secret_setting,
    secretSettingsStore,
    set_secret_setting,
    set_setting,
    settingsStore,
    type ModelProvider,
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

  // 基础状态
  let isLoading = $state(false)
  let notification = $state<{
    type: 'success' | 'error'
    message: string
  } | null>(null)
  let toastStatus = $derived.by(() => !!notification)
  let activeSection = $state(page.url.searchParams.get('section') || 'general')

  // 计算属性
  const navigationItems = $derived([
    { id: 'general', name: t('settings.nav.general'), icon: CogOutline },
    { id: 'appearance', name: t('settings.nav.appearance'), icon: EyeOutline },
    { id: 'network', name: t('settings.nav.network'), icon: GlobeOutline },
    { id: 'ai', name: t('settings.nav.ai'), icon: WandMagicSparklesOutline }
  ])

  const themeOptions = $derived([
    { value: 'light', name: t('settings.theme.light') },
    { value: 'dark', name: t('settings.theme.dark') },
    { value: 'system', name: t('settings.theme.system') }
  ])

  const localeOptions = $derived([
    { value: 'en', name: t('language.english') },
    { value: 'zh', name: t('language.chinese') }
  ])

  const providerOptions = [
    { value: 'gemini', name: 'Google Gemini' },
    { value: 'openai', name: 'OpenAI GPT' }
  ]

  let currentProvider = $state<'gemini' | 'openai'>(
    secretSettingsStore.preferred_provider || 'gemini'
  )

  let providerState = $state<ModelProvider>(
    secretSettingsStore[secretSettingsStore.preferred_provider || 'gemini'] || {
      model: 'gemini-2.5-pro',
      api_key: '',
      api_base: ''
    }
  )

  function setAIProvider() {
    switch (currentProvider) {
      case 'gemini':
        providerState = secretSettingsStore[currentProvider] || {
          model: 'gemini-2.5-pro',
          api_key: '',
          api_base: ''
        }
        break
      case 'openai':
        providerState = secretSettingsStore[currentProvider] || {
          model: 'gpt-5',
          api_key: '',
          api_base: ''
        }
        break
    }
  }

  const hasUnsavedChanges = $derived.by(() => {
    const origin = secretSettingsStore[currentProvider] as ModelProvider
    return (
      currentProvider !== secretSettingsStore.preferred_provider ||
      providerState.model !== origin?.model ||
      providerState.api_key !== origin?.api_key ||
      providerState.api_base !== origin?.api_base
    )
  })

  const isFormValid = $derived.by(() => {
    return (
      providerState.model.trim() !== '' && providerState.api_key.trim() !== ''
    )
  })

  const api_base_placeholder = $derived.by(() => {
    switch (currentProvider) {
      case 'gemini':
        return 'https://generativelanguage.googleapis.com/v1beta/openai'
      case 'openai':
        return 'https://api.openai.com/v1'
      default:
        return ''
    }
  })

  // AI 配置保存
  async function saveAIProvider() {
    try {
      isLoading = true

      // 保存首选提供商
      await set_secret_setting('preferred_provider', currentProvider)
      await set_secret_setting(currentProvider, {
        model: providerState.model.trim(),
        api_key: providerState.api_key.trim(),
        api_base: providerState.api_base?.trim() || undefined
      })

      showNotification('success', t('settings.saved'))
    } catch (error) {
      console.error('Failed to save AI config:', error)
      showNotification('error', t('settings.save_failed'))
    } finally {
      isLoading = false
    }
  }

  // 工具函数
  function showNotification(type: 'success' | 'error', message: string) {
    notification = { type, message }
    setTimeout(() => {
      notification = null
    }, 6000)
  }

  // 设置更新函数
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

  function handleProxyChange(event: Event) {
    const target = event.target as HTMLInputElement
    updateSetting('https_proxy', target.value || '')
  }

  // 组件挂载时初始化
  onMount(async () => {
    try {
      isLoading = true
      await Promise.all([
        get_secret_setting('preferred_provider'),
        get_secret_setting('gemini'),
        get_secret_setting('openai')
      ])

      currentProvider = secretSettingsStore.preferred_provider || 'gemini'
      providerState = secretSettingsStore[
        secretSettingsStore.preferred_provider || 'gemini'
      ] || {
        model: 'gemini-2.5-pro',
        api_key: '',
        api_base: ''
      }
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
            <h2 class="mb-2 text-2xl font-bold text-gray-900 dark:text-white">
              {t('settings.general.title')}
            </h2>
            <p class="text-gray-600 dark:text-gray-400">
              {t('settings.general.description')}
            </p>
          </div>

          <!-- 语言设置 -->
          <div
            class="rounded-xl border border-gray-200 bg-white p-6 dark:border-gray-700 dark:bg-gray-800"
          >
            <div class="flex items-center justify-between">
              <div class="flex-1">
                <h3
                  class="mb-1 text-lg font-medium text-gray-900 dark:text-white"
                >
                  {t('settings.language.title')}
                </h3>
                <p class="text-sm text-gray-500 dark:text-gray-400">
                  {t('settings.language.description')}
                </p>
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
            <h2 class="mb-2 text-2xl font-bold text-gray-900 dark:text-white">
              {t('settings.appearance.title')}
            </h2>
            <p class="text-gray-600 dark:text-gray-400">
              {t('settings.appearance.description')}
            </p>
          </div>

          <!-- 主题设置 -->
          <div
            class="rounded-xl border border-gray-200 bg-white p-6 dark:border-gray-700 dark:bg-gray-800"
          >
            <div class="flex items-center justify-between">
              <div class="flex-1">
                <h3
                  class="mb-1 text-lg font-medium text-gray-900 dark:text-white"
                >
                  {t('settings.theme.title')}
                </h3>
                <p class="text-sm text-gray-500 dark:text-gray-400">
                  {t('settings.theme.description')}
                </p>
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
            <h2 class="mb-2 text-2xl font-bold text-gray-900 dark:text-white">
              {t('settings.network.title')}
            </h2>
            <p class="text-gray-600 dark:text-gray-400">
              {t('settings.network.description')}
            </p>
          </div>

          <!-- HTTPS 代理 -->
          <div
            class="rounded-xl border border-gray-200 bg-white p-6 dark:border-gray-700 dark:bg-gray-800"
          >
            <div class="space-y-4">
              <div>
                <h3
                  class="mb-1 text-lg font-medium text-gray-900 dark:text-white"
                >
                  {t('settings.proxy.title')}
                </h3>
                <p class="text-sm text-gray-500 dark:text-gray-400">
                  {t('settings.proxy.description')}
                </p>
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
            <h2 class="mb-2 text-2xl font-bold text-gray-900 dark:text-white">
              {t('settings.ai.provider.title')}
            </h2>
            <p class="text-gray-600 dark:text-gray-400">
              {t('settings.ai.provider.description')}
            </p>
          </div>

          <!-- AI 提供商选择 -->
          <div
            class="rounded-xl border border-gray-200 bg-white p-6 dark:border-gray-700 dark:bg-gray-800"
          >
            <div class="space-y-4">
              <div>
                <h3
                  class="mb-1 text-lg font-medium text-gray-900 dark:text-white"
                >
                  {t('settings.ai.provider.title')}
                </h3>
                <p class="text-sm text-gray-500 dark:text-gray-400">
                  {t('settings.ai.provider.description')}
                </p>
              </div>
              <Select
                bind:value={currentProvider}
                disabled={isLoading}
                onchange={setAIProvider}
                class="w-full"
              >
                {#each providerOptions as option}
                  <option value={option.value}>{option.name}</option>
                {/each}
              </Select>
            </div>
          </div>

          <!-- 提供商配置 -->
          <div
            class="rounded-xl border border-gray-200 bg-white p-6 dark:border-gray-700 dark:bg-gray-800"
          >
            <div class="space-y-4">
              <!-- 模型输入 -->
              <div>
                <label
                  for="ai_model"
                  class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300"
                >
                  {t('settings.ai.model')} *
                </label>
                <Input
                  type="text"
                  id="ai_model"
                  minlength={3}
                  bind:value={providerState.model}
                  disabled={isLoading}
                  class="w-full"
                  required
                />
              </div>

              <!-- API Key 输入 -->
              <div>
                <label
                  for="ai_api_key"
                  class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300"
                >
                  {t('settings.ai.api_key')} *
                </label>
                <Input
                  type="text"
                  id="ai_api_key"
                  minlength={10}
                  bind:value={providerState.api_key}
                  disabled={isLoading}
                  class="w-full"
                  required
                />
              </div>

              <!-- API Base URL 输入 -->
              <div>
                <label
                  for="ai_api_base"
                  class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300"
                >
                  {t('settings.ai.api_base')} ({t('settings.optional')})
                </label>
                <Input
                  type="text"
                  id="ai_api_base"
                  placeholder={api_base_placeholder}
                  bind:value={providerState.api_base}
                  disabled={isLoading}
                  class="w-full"
                />
              </div>
            </div>
          </div>

          <!-- 操作按钮 -->
          <div class="flex justify-end space-x-3">
            <button
              class="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
              onclick={saveAIProvider}
              disabled={isLoading || !hasUnsavedChanges || !isFormValid}
            >
              {#if isLoading}
                {t('settings.saving')}
              {:else}
                {t('settings.ai.save')}
              {/if}
            </button>
          </div>
        </div>
      {/if}
    </div>

    <!-- 通知 Toast -->
    <Toast
      dismissable={false}
      color={notification?.type === 'success' ? 'green' : 'red'}
      class="mx-auto mt-4"
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
