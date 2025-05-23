<script lang="ts">
  import { authStore, logout, signIn } from '$lib/stores/auth.svelte'
  import { Button } from 'flowbite-svelte'

  async function onSignIn(event: Event) {
    event.preventDefault()
    console.log('sign in', event)
    await signIn()
    console.log('sign in after', authStore.auth)
  }

  async function onLogout(event: Event) {
    event.preventDefault()
    console.log('logout', event)
    await logout()
    console.log('logout after', authStore.auth)
  }
</script>

<main
  class="m-auto flex h-screen w-full max-w-xs flex-col items-center justify-center gap-4 p-4"
>
  <h1>Welcome to Anda</h1>

  <p>{authStore.auth.id}</p>

  {#if authStore.auth.isAuthenticated()}
    <Button
      color="yellow"
      class="m-auto w-full max-w-xs cursor-pointer"
      onclick={onLogout}>Logout</Button
    >
  {:else}
    <Button
      color="dark"
      class="m-auto w-full max-w-xs cursor-pointer"
      onclick={onSignIn}>Sign In</Button
    >
  {/if}
</main>

<style>
</style>
