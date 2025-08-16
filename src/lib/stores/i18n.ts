import { translations } from '$lib/locales'
import { settingsStore } from '$lib/stores/settings.svelte'
import { invoke } from '@tauri-apps/api/core'

// 插值函数：支持 %{key} 和 {key}
function interpolate(
  template: string,
  params: Record<string, string | number>
): string {
  let result = template
  // 支持 %{key}
  result = result.replace(/%\{([\w.]+)\}/g, (_, key) => {
    const v = (params as any)[key]
    return v !== undefined && v !== null ? String(v) : `%{${key}}`
  })
  // 兼容 {key}
  result = result.replace(/\{([\w.]+)\}/g, (_, key) => {
    const v = (params as any)[key]
    return v !== undefined && v !== null ? String(v) : `{${key}}`
  })
  return result
}

// 支持重载：t(key, locale) / t(key, params) / t(key, params, locale)
export function t(key: string, locale?: string): string
export function t(
  key: string,
  params: Record<string, string | number>,
  locale?: string
): string
export function t(
  key: string,
  paramsOrLocale?: Record<string, string | number> | string,
  locale?: string
): string {
  const currentLocale =
    (typeof paramsOrLocale === 'string' ? paramsOrLocale : locale) ||
    settingsStore.locale ||
    'en'
  const translation = translations[key]

  if (!translation) {
    console.warn(`Translation key not found: ${key}`)
    return key
  }

  const raw = translation[currentLocale] || translation['en'] || key
  if (paramsOrLocale && typeof paramsOrLocale === 'object') {
    return interpolate(raw, paramsOrLocale)
  }
  return raw
}

export async function tBackend(key: string, locale?: string): Promise<string> {
  try {
    const currentLocale = locale || settingsStore.locale || 'en'
    return await invoke('get_translation', { key, locale: currentLocale })
  } catch (error) {
    console.warn(`Backend translation failed for key: ${key}`, error)
    return t(key, locale)
  }
}

export function tr(key: string) {
  return $derived(t(key, settingsStore.locale))
}
