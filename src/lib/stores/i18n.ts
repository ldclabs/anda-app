import { translations } from '$lib/locales'
import { settingsStore } from '$lib/stores/settings.svelte'
import { invoke } from '@tauri-apps/api/core'

export function t(key: string, locale?: string): string {
  const currentLocale = locale || settingsStore.locale || 'en'
  const translation = translations[key]

  if (!translation) {
    console.warn(`Translation key not found: ${key}`)
    return key
  }

  return translation[currentLocale] || translation['en'] || key
}

export async function tBackend(key: string, locale?: string): Promise<string> {
  try {
    const currentLocale = locale || settingsStore.locale || 'en'
    return await invoke('get_translation', { key, locale: currentLocale })
  } catch (error) {
    console.warn(`Backend translation failed for key: ${key}`, error)
    return t(key, locale) // 回退到前端翻译
  }
}

export function tr(key: string) {
  return $derived(t(key, settingsStore.locale))
}
