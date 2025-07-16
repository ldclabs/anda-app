/// <reference types="@sveltejs/kit" />

// Tauri global type definitions for browser compatibility
declare global {
  interface Window {
    __TAURI_METADATA__?: any
    __TAURI_OS_PLUGIN_INTERNALS__?: any
  }
}

export {}
