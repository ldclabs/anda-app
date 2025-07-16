/**
 * Application initialization script
 * This should be imported at the top of your app to ensure Tauri mocks are available
 */

import { mockTauriOS } from '$lib/utils/tauri-mock'

// Initialize Tauri mocks for browser development
if (typeof window !== 'undefined') {
  mockTauriOS()
}

export default {}
