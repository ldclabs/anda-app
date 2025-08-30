import { osType } from '$lib/utils/tauri.mock'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { toastRun } from './toast.svelte'

const ot = osType()

export interface UpdateInfo {
  // 应用当前版本
  current_version: string
  // 最新版本
  version: string
  // 最新版本更新说明
  notes: string
  // 是否已经下载安装包
  ready: boolean
}

export async function update_supported(): Promise<boolean> {
  return await invoke('update_supported')
}

export async function check_update(): Promise<UpdateInfo | null> {
  return await invoke('check_update')
}

export async function restart(): Promise<void> {
  await invoke('restart')
}

export function open_update_window() {
  const webview = new WebviewWindow('update', {
    url: '/update',
    width: 480,
    height: 320
  })
  webview.once('tauri://error', function (e) {
    // an error happened creating the webview
    console.error('an error happened creating the webview', e)
  })
}

class UpdaterStore {
  static async init() {
    const supported = await update_supported()

    const view = getCurrentWebview()
    if (
      supported &&
      view.label == 'main' &&
      (ot == 'macos' || ot == 'windows' || ot == 'linux')
    ) {
      updaterStore.checkUpdateInternal()
    }
  }

  private _info = $state<UpdateInfo | null>(null)
  private _isDownloading = $derived<boolean>(!!this._info && !this._info?.ready)
  private _isRestarting = $state<boolean>(false)

  get info() {
    return this._info
  }

  get isDownloading() {
    return this._isDownloading
  }

  get isRestarting() {
    return this._isRestarting
  }

  async check_update() {
    this._info = await toastRun(check_update).finally()
    if (this._info) {
      if (this._info.ready) return
      setTimeout(() => {
        this.check_update()
      }, 1000)
    }
  }

  private async checkUpdateInternal() {
    try {
      this._info = await check_update()
    } catch (_) {} // ignore error

    if (this._info) {
      if (this._info.ready) return
      setTimeout(() => {
        this.checkUpdateInternal()
      }, 1000)
    } else {
      setTimeout(() => {
        this.checkUpdateInternal()
      }, 3600 * 1000)
    }
  }

  async restartApp() {
    this._isRestarting = true
    await toastRun(restart).finally()
    this._isRestarting = false
  }
}

export const updaterStore = new UpdaterStore()

UpdaterStore.init()
