<script lang="ts">
  import { authStore, get_user, logout, signIn } from '$lib/stores/auth.svelte'
  import { Button } from 'flowbite-svelte'

  async function onSignIn(event: Event) {
    event.preventDefault()
    await signIn()
  }

  async function onLogout(event: Event) {
    event.preventDefault()
    await logout()
  }
</script>

<main
  class="m-auto flex h-screen w-full max-w-xs flex-col items-center justify-center gap-4 p-4"
>
  <h1>Welcome to Anda</h1>

  <p>{authStore.auth.id}</p>

  {#key authStore.auth.id}
    {#if authStore.auth.isAuthenticated()}
      {#await get_user()}
        <p>waiting for the promise to resolve...</p>
      {:then userInfo}
        <p>{JSON.stringify(userInfo)}</p>
      {/await}
    {/if}
  {/key}

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
