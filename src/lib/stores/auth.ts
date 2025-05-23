import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { writable, type Readable } from 'svelte/store'

const IDENTITY_EVENT = 'IdentityChanged'

export interface IdentityInfo {
  id: string
  expiration: number | null // in milliseconds
}

export class AuthInfo {
  static AnonymousId = '2vxsx-fae'

  id: string
  expiration: number | null

  constructor(
    info: IdentityInfo = { id: AuthInfo.AnonymousId, expiration: null }
  ) {
    this.id = info.id
    this.expiration = info.expiration
  }

  isAnonymous(): boolean {
    return this.id === AuthInfo.AnonymousId
  }

  isAuthenticated(): boolean {
    return (
      this.id !== AuthInfo.AnonymousId &&
      (this.expiration === null || this.expiration > Date.now())
    )
  }
}

export interface AuthStore extends Readable<AuthInfo> {
  get identity(): IdentityInfo
  sync: () => Promise<void>
  signIn: () => Promise<void>
  logout: () => Promise<void>
}

function initAuthStore(): AuthStore {
  let info: IdentityInfo = { id: AuthInfo.AnonymousId, expiration: null }

  const { subscribe, set } = writable<AuthInfo>(new AuthInfo(info))

  listen<IdentityInfo>(IDENTITY_EVENT, (event) => {
    console.log(`${IDENTITY_EVENT}: ${event}`)
    info = event.payload
    set(new AuthInfo(info))
  })

  const store = {
    subscribe,

    get identity() {
      return { id: info.id, expiration: info.expiration }
    },

    sync: async () => {
      const res: IdentityInfo = await invoke('identity')
      set(new AuthInfo(res))
    },

    signIn: async () => {
      await invoke('sign_in')
    },
    logout: async () => {
      await invoke('logout')
    }
  }

  return store
}

export const authStore = initAuthStore()
