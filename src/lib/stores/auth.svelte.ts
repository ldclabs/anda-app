import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { assistantStore } from './assistant.svelte'

const IDENTITY_EVENT = 'IdentityChanged'

export interface IdentityInfo {
  id: string
  expiration: number | null // in milliseconds
}

export interface UserInfo {
  id: string
  name: string
  image?: string
  profile_canister?: string
  cose_canister?: string
  username?: string
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
  auth: new AuthInfo(),
  user: null as UserInfo | null,
  isSigningIn: false,
  signInFallback: true
})

let prevTimer: number | null = null
export async function signIn() {
  authStore.isSigningIn = true
  await invoke('sign_in')

  prevTimer && clearTimeout(prevTimer)
  prevTimer = setTimeout(() => {
    authStore.signInFallback = authStore.isSigningIn
    authStore.isSigningIn = false
  }, 10000)
}

export async function signInByUrl(url: string) {
  await invoke('sign_in_by_url', { url })
}

export async function logout() {
  await invoke('logout')
}

export async function get_user() {
  let user: UserInfo = await invoke('get_user')
  authStore.user = user
  assistantStore.userName = user.name
  localStorage.setItem(`UserInfo:${user.id}`, JSON.stringify(user))
  return user
}

async function onAuthChanged(auth: AuthInfo) {
  authStore.auth = auth
  authStore.user = null
  assistantStore.userID = authStore.auth.id
  if (authStore.auth.isAuthenticated()) {
    authStore.isSigningIn = false
    authStore.signInFallback = false
    authStore.user = loadUserInfo(authStore.auth.id)

    get_user()
  }
}

async function init() {
  onAuthChanged(new AuthInfo(await invoke('identity')))
  listen<IdentityInfo>(IDENTITY_EVENT, (event) => {
    onAuthChanged(new AuthInfo(event.payload))
  })

  const res: IdentityInfo = await invoke('identity')
  authStore.auth = new AuthInfo(res)
}

init().catch((err) => {
  console.error('Failed to initialize auth store', err)
})

function loadUserInfo(id: string) {
  try {
    let user = localStorage.getItem(`UserInfo:${id}`)
    if (user) {
      return JSON.parse(user) as UserInfo
    }
  } catch (_) {}
  return {
    id,
    name: id.replace(/-.*-/, '*')
  }
}
