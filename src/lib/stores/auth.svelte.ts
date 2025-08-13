import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

const IDENTITY_EVENT = 'IdentityChanged'

export interface IdentityInfo {
  id: string
  expiration: number | null // in milliseconds
}

export interface UserInfo {
  id: string
  name: string
  image: string
  profile_canister: string
  cose_canister: string | null
  username: string | null
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

export const authStore = $state({
  auth: new AuthInfo()
})

export async function signIn() {
  await invoke('sign_in')
}

export async function signInByUrl(url: string) {
  await invoke('sign_in_by_url', { url })
}

export async function logout() {
  await invoke('logout')
}

export async function get_user() {
  let user: UserInfo = await invoke('get_user')
  console.log('get_user', user)
  return user
}

async function init() {
  if (typeof window !== 'undefined') {
    ;(window as any).signInByUrl = signInByUrl
  }

  listen<IdentityInfo>(IDENTITY_EVENT, (event) => {
    console.log(`${IDENTITY_EVENT}: ${event}`)
    authStore.auth = new AuthInfo(event.payload)
  })

  const res: IdentityInfo = await invoke('identity')
  authStore.auth = new AuthInfo(res)
}

init().catch((err) => {
  console.error('Failed to initialize auth store', err)
})
