<script lang="ts">
  import AndaPlaceholder from '$lib/components/ui/AndaPlaceholder.svelte'
  import { authStore, get_user, logout } from '$lib/stores/auth.svelte'
  import { t } from '$lib/stores/i18n'
  import { toastRun } from '$lib/stores/toast.svelte'
  import { shortId } from '$lib/utils/helper'
  import { renderMarkdown } from '$lib/utils/markdown'
  import { Avatar, Button, Clipboard, Heading, Spinner } from 'flowbite-svelte'
  import {
    ArrowRightToBracketOutline,
    CheckOutline,
    FileCopyOutline
  } from 'flowbite-svelte-icons'
  import { onMount } from 'svelte'

  const userInfo = $derived(authStore.user)
  const [profilePlaceholder] = renderMarkdown(t('profile.placeholder'))
  const [profileUpdateInfo] = renderMarkdown(t('profile.update_info'))
  let isLoading = $state(false)
  let isLoggingOut = $state(false)

  onMount(() => {
    if (authStore.auth.isAuthenticated()) {
      isLoading = !userInfo
      toastRun(get_user).finally(() => {
        isLoading = false
      })
    }
  })

  function handleLogout() {
    isLoggingOut = true
    toastRun(logout).finally(() => {
      isLoggingOut = false
    })
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
            <Avatar alt={userInfo.name} size="xl" class="mx-auto mb-4" />
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
          <div class="md-content mx-auto mb-4 text-center dark:text-white"
            >{@html profileUpdateInfo}</div
          >
          <div
            class="mx-auto flex flex-row items-center justify-center text-neutral-500"
          >
            <span>ID: {shortId(userInfo.id, true)}</span>
            <Clipboard
              color="alternative"
              class="border-0 p-2"
              bind:value={userInfo.id}
            >
              {#snippet children(success)}
                {#if success}<CheckOutline
                    color="primary"
                  />{:else}<FileCopyOutline />{/if}
              {/snippet}
            </Clipboard>
          </div>
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
      <AndaPlaceholder><span></span></AndaPlaceholder>
      <div class="md-content mx-auto text-center text-lg dark:text-white"
        >{@html profilePlaceholder}</div
      >
    {/if}
  </div>
</div>
