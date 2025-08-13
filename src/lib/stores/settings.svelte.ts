import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export interface Settings {
  locale: string
  theme: 'light' | 'dark' | null
  https_proxy?: string
}

export interface SecretSettings {
  gemini_api_key?: string
  openai_api_key?: string
}

export const settingsStore = $state({
  locale: 'en',
  theme: 'light'
} as Settings)

export const secretSettingsStore = $state({
  gemini_api_key: '',
  openai_api_key: ''
} as SecretSettings)

export async function get_settings() {
  let res: Settings = await invoke('get_settings')
  Object.assign(settingsStore, res)
}

export async function set_setting(
  key: keyof Settings,
  value: Settings[keyof Settings]
) {
  let res: Settings = await invoke('set_setting', {
    key,
    value
  })
  Object.assign(settingsStore, res)
}

export async function get_secret_setting(key: keyof SecretSettings) {
  let value: string = await invoke('get_secret_setting', {
    key
  })
  secretSettingsStore[key] = value
}

export async function set_secret_setting(
  key: keyof SecretSettings,
  value: SecretSettings[keyof SecretSettings]
) {
  await invoke('set_secret_setting', {
    key,
    value
  })
  secretSettingsStore[key] = value
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
