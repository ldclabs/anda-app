import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export interface Settings {
  locale: string
  theme: 'light' | 'dark' | 'system'
  https_proxy?: string
}

export interface SecretSettings {
  preferred_provider: 'gemini' | 'openai'
  gemini?: ModelProvider // model: 'gemini-2.5-pro'
  openai?: ModelProvider // model: 'gpt-5' or 'gpt-5-2025-08-07' ?
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
  preferred_provider: 'gemini'
} as SecretSettings)

export async function get_settings() {
  const res: Settings = await invoke('get_settings')
  if (!res.theme) {
    res.theme = 'system'
  }
  Object.assign(settingsStore, res)
  console.log('settingsStore', settingsStore)
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

export function open_settings_window(params?: string) {
  invoke('open_settings_window', {
    params
  })
}

async function init() {
  get_settings()
  listen<string>('SettingsChanged', (event) => {
    get_settings()
  })
}

init().catch((err) => {
  console.error('Failed to initialize settings store', err)
})
