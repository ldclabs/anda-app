<script lang="ts">
  import AndaPlaceholder from '$lib/components/ui/AndaPlaceholder.svelte'
  import { authStore, get_user, logout } from '$lib/stores/auth.svelte'
  import { t } from '$lib/stores/i18n'
  import { Avatar, Button, Heading, Spinner } from 'flowbite-svelte'
  import {
    ArrowRightToBracketOutline,
    UserCircleOutline
  } from 'flowbite-svelte-icons'
  import { onMount } from 'svelte'

  const userInfo = $derived(authStore.user)
  let isLoading = $state(false)
  let isLoggingOut = $state(false)

  onMount(async () => {
    if (authStore.auth.isAuthenticated()) {
      isLoading = !userInfo
      try {
        await get_user()
      } catch (error) {
        console.error('Failed to get user info:', error)
      } finally {
        isLoading = false
      }
    } else {
      isLoading = false
    }
  })

  async function handleLogout() {
    isLoggingOut = true
    try {
      await logout()
    } catch (error) {
      console.error('Logout failed:', error)
    } finally {
      isLoggingOut = false
    }
  }
</script>

<div class="flex h-full w-full items-center justify-center p-4">
  <div class="w-full max-w-md">
    {#if isLoading}
      <div class="flex justify-center">
        <Spinner size="8" />
      </div>
    {:else if userInfo}
      <div class="text-center">
        <div class="mb-6">
          {#if userInfo.image}
            <Avatar
              src={userInfo.image}
              alt={userInfo.name}
              size="xl"
              class="mx-auto mb-4"
            />
          {:else}
            <UserCircleOutline size="xl" class="mx-auto mb-4 text-gray-400" />
          {/if}

          <Heading tag="h2" class="mb-2">{userInfo.name}</Heading>

          {#if userInfo.username}
            <p class="text-primary-500 mb-2"
              ><a
                class="underline underline-offset-4"
                href="https://dmsg.net/{userInfo.username}"
                target="_blank">@{userInfo.username}</a
              ></p
            >
          {/if}

          <p class="mb-1 text-sm text-gray-400">ID: {userInfo.id}</p>

          {#if userInfo.profile_canister}
            <p class="mb-1 text-sm text-gray-400">
              Profile Canister: {userInfo.profile_canister}
            </p>
          {/if}

          {#if userInfo.cose_canister}
            <p class="text-sm text-gray-400">
              COSE Canister: {userInfo.cose_canister}
            </p>
          {/if}
        </div>

        <Button
          color="secondary"
          class="w-full"
          disabled={isLoggingOut}
          onclick={handleLogout}
        >
          <ArrowRightToBracketOutline class="mr-2" />
          {t('app.log_out')}
        </Button>
      </div>
    {:else}
      <AndaPlaceholder>
        <Heading tag="h2">Profile</Heading>
      </AndaPlaceholder>
    {/if}
  </div>
</div>
