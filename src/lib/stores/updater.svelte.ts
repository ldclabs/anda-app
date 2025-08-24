import { isTauriEnvironment, safeOsType } from '$lib/utils/tauri.mock'
import { invoke } from '@tauri-apps/api/core'
import { type as osType } from '@tauri-apps/plugin-os'
import { toastRun } from './toast.svelte'

const ot = isTauriEnvironment() ? osType() : safeOsType()

export interface UpdateInfo {
  current_version: string
  version: string
  ready: boolean
}

export async function check_update(): Promise<UpdateInfo | null> {
  return await invoke('check_update')
}

export async function restart(): Promise<void> {
  await invoke('restart')
}

class UpdaterStore {
  static init() {
    if (ot == 'windows' || ot == 'linux') {
      updaterStore.checkUpdateInternal()
    }
  }

  private _info = $state<UpdateInfo | null>(null)
  private _isDownloading = $derived<boolean>(!!this._info && !this._info.ready)
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

  private async checkUpdateInternal() {
    this._info = await toastRun(check_update).finally()

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
