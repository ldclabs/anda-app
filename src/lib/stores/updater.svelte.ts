import { isTauriEnvironment, safeOsType } from '$lib/utils/tauri.mock'
import { invoke } from '@tauri-apps/api/core'
import { type as osType } from '@tauri-apps/plugin-os'

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
    try {
      this._info = await check_update()
      if (this._info) {
        if (this._info.ready) return
        setTimeout(async () => {
          await this.checkUpdateInternal()
        }, 1000)
      } else {
        setTimeout(async () => {
          await this.checkUpdateInternal()
        }, 3600 * 1000)
      }
    } catch (error) {
      console.error('Failed to check update', error)
    }
  }

  async restartApp() {
    this._isRestarting = true
    try {
      await restart()
    } catch (error) {
      console.error('Failed to restart', error)
    } finally {
      this._isRestarting = false
    }
  }
}

export const updaterStore = new UpdaterStore()

UpdaterStore.init()
