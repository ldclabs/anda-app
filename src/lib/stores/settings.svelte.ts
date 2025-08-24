import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'

export interface Settings {
  locale: string
  theme: 'light' | 'dark' | 'system'
  https_proxy?: string
}

export interface SecretSettings {
  preferred_provider: 'gemini' | 'openai' | 'deepseek'
  gemini?: ModelProvider // model: 'gemini-2.5-pro'
  openai?: ModelProvider // model: 'gpt-5'
  deepseek?: ModelProvider // model: 'deepseek-reasoner'
}

export interface ModelProvider {
  model: string
  api_key: string
  api_base?: string
}

export const settingsStore = $state({
  locale: 'en',
  theme: 'light'
} as Settings)

export const secretSettingsStore = $state({
  preferred_provider: 'deepseek'
} as SecretSettings)

export async function get_settings() {
  const res: Settings = await invoke('get_settings')
  if (!res.theme) {
    res.theme = 'system'
  }
  Object.assign(settingsStore, res)
}

export async function set_setting(
  key: keyof Settings,
  value: Settings[keyof Settings]
) {
  await invoke('set_setting', {
    key,
    value
  })
}

export async function get_secret_setting(key: keyof SecretSettings) {
  let value = await invoke<SecretSettings[keyof SecretSettings]>(
    'get_secret_setting',
    {
      key
    }
  )
  ;(secretSettingsStore[key] as any) = value
}

export async function set_secret_setting(
  key: keyof SecretSettings,
  value: SecretSettings[keyof SecretSettings]
) {
  const updated = await invoke<boolean>('set_secret_setting', {
    key,
    value
  })

  if (updated) {
    ;(secretSettingsStore[key] as any) = value
  }
}

export function open_settings_window() {
  const webview = new WebviewWindow('settings', {
    url: '/settings?section=ai',
    width: 1024,
    height: 800
  })
  webview.once('tauri://error', function (e) {
    // an error happened creating the webview
    console.error('an error happened creating the webview', e)
  })
}

async function init() {
  listen<string>('SettingsChanged', (event) => {
    get_settings()
  })
  await get_settings()
}

init().catch((err) => {
  console.error('Failed to initialize settings store', err)
})
