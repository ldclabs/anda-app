/**
 * Tauri Mock utilities for browser development
 */

// Check if we're running in a Tauri environment
export function isTauriEnvironment(): boolean {
  return (
    typeof window !== 'undefined' && (window as any).__TAURI__ !== undefined
  )
}

// Mock Tauri OS plugin for browser development
export function mockTauriOS() {
  if (
    typeof window !== 'undefined' &&
    !isTauriEnvironment() &&
    !(window as any).__TAURI_OS_PLUGIN_INTERNALS__
  ) {
    ;(window as any).__TAURI_OS_PLUGIN_INTERNALS__ = {
      type: () => 'web'
    }
  }
}

// Safe OS type detection that returns synchronously
export function safeOsType(): string {
  // Browser environment fallback
  if (typeof navigator !== 'undefined') {
    const userAgent = navigator.userAgent.toLowerCase()
    if (userAgent.includes('mac')) return 'macos'
    if (userAgent.includes('win')) return 'windows'
    if (userAgent.includes('linux')) return 'linux'
    if (userAgent.includes('android')) return 'android'
    if (userAgent.includes('iphone') || userAgent.includes('ipad')) return 'ios'
  }

  return 'web'
}

// Async version for dynamic imports in Tauri environment
export async function safeOsTypeAsync(): Promise<string> {
  if (isTauriEnvironment()) {
    try {
      const { type } = await import('@tauri-apps/plugin-os')
      return type()
    } catch (error) {
      console.warn('Failed to get OS type from Tauri:', error)
      return safeOsType()
    }
  }

  return safeOsType()
}
