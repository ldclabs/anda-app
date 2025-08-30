<script lang="ts">
  import { page } from '$app/state'
  import DiagnosisModal from '$lib/components/diagnosis/DiagnosisModal.svelte'
  import { authStore, signIn, signInByUrl } from '$lib/stores/auth.svelte'
  import { t } from '$lib/stores/i18n'
  import { toastRun } from '$lib/stores/toast.svelte'
  import { open_update_window, updaterStore } from '$lib/stores/updater.svelte'
  import { osType } from '$lib/utils/tauri.mock'
  import {
    Avatar,
    BottomNav,
    BottomNavItem,
    Button,
    Input,
    Label,
    Modal,
    Sidebar,
    SidebarGroup,
    SidebarItem,
    Spinner
  } from 'flowbite-svelte'
  import {
    BellRingOutline,
    QuestionCircleOutline,
    RefreshOutline,
    UserCircleOutline,
    UserHeadsetOutline
  } from 'flowbite-svelte-icons'
  import { setContext } from 'svelte'

  // Safe OS type detection
  const ot = osType()

  let { children } = $props()

  let signInModal = $state(false)
  let isMobile = $state(ot === 'ios' || ot === 'android')
  let activeUrl = $state(page.url.pathname)
  let signInUrl = $state('')

  let diagnosisModalRef = $state<DiagnosisModal>()

  function onSignInClick() {
    if (authStore.signInFallback) {
      signInModal = true
      return
    }
    onSignIn()
  }

  function onSignIn() {
    signIn((success) => {
      if (!success) {
        signInModal = true
      }
    })
  }

  function onSignInAction() {
    toastRun(async () => {
      await signInByUrl(signInUrl)
      signInModal = false
    }).finally(() => {
      signInUrl = ''
    })
  }

  $effect(() => {
    activeUrl = page.url.pathname
  })

  setContext('diagnosisModalState', () => diagnosisModalRef)
  setContext('signInHandler', onSignInClick)
</script>

<Modal bind:open={signInModal} size="xs">
  <div class="flex flex-col space-y-6">
    <h3 class="mb-4 text-xl font-medium text-gray-900 dark:text-white"
      >{t('app.sign_in_fallback.title')}</h3
    >
    <Label class="space-y-2">
      <Input
        type="text"
        name="signInUrl"
        bind:value={signInUrl}
        placeholder="https://anda.ai/deeplink/?os=xx&action=SignIn..."
        required
      />
    </Label>
    <Button disabled={authStore.isSigningIn} onclick={onSignInAction}
      >{t('app.sign_in_fallback.by_url')}</Button
    >
    <Button
      color="alternative"
      disabled={authStore.isSigningIn}
      onclick={onSignIn}
    >
      <span>{t('app.sign_in_fallback.again')}</span>
      {#if authStore.isSigningIn}
        <Spinner class="ms-3 inline-flex" size="4" />
      {/if}
    </Button>
  </div>
</Modal>
{#key authStore.auth.id}
  {#if isMobile}
    <main class="relative grid h-dvh w-dvw grid-rows-[1fr_auto]">
      <div class="relative w-full overflow-auto">
        {@render children()}
      </div>

      <BottomNav
        {activeUrl}
        innerClass="grid-cols-5"
        outerClass="relative anda-nav"
        activeClass="font-bold text-green-500 hover:text-green-900 dark:hover:text-green-700 dark:text-green-300"
      >
        <BottomNavItem btnName={t('assistant.title')} href="/app/assistant">
          <UserHeadsetOutline size="lg" />
        </BottomNavItem>
        <!-- <BottomNavItem btnName="Discover" href="/app/discover">
          <SearchOutline size="lg" />
        </BottomNavItem> -->
        <!-- <BottomNavItem btnName="Messages" href="/app/messages">
          <MessagesOutline size="lg" />
        </BottomNavItem> -->
        {#if authStore.user}
          <BottomNavItem btnName={authStore.user.name} href="#">
            {#if authStore.user.image}
              <img
                src={authStore.user.image}
                alt={authStore.user.name + 'image'}
                class="size-6 rounded-full"
              />
            {:else}
              <UserCircleOutline size="xl" />
            {/if}
          </BottomNavItem>
        {:else}
          <BottomNavItem btnName={t('app.sign_in')} onclick={onSignIn}>
            <UserCircleOutline
              size="xl"
              class={authStore.isSigningIn ? 'animate-bounce' : ''}
            />
          </BottomNavItem>
        {/if}
      </BottomNav>
    </main>
  {:else}
    <main class="relative grid h-dvh w-dvw grid-cols-[auto_1fr]">
      <Sidebar
        {activeUrl}
        backdrop={false}
        alwaysOpen={true}
        class="anda-nav relative h-full"
        classes={{
          div: 'grid grid-rows-[1fr_auto] h-full',
          nonactive: 'p-2',
          active: 'p-2'
        }}
      >
        <SidebarGroup>
          <SidebarItem label={t('assistant.title')} href="/app/assistant">
            {#snippet icon()}
              <UserHeadsetOutline size="lg" />
            {/snippet}
          </SidebarItem>
          <!-- <SidebarItem label="Discover" href="/app/discover">
            {#snippet icon()}
              <SearchOutline size="lg" />
            {/snippet}
          </SidebarItem> -->
          <!-- <SidebarItem label="Messages" href="/app/messages">
            {#snippet icon()}
              <MessagesOutline size="lg" />
            {/snippet}
          </SidebarItem> -->
        </SidebarGroup>
        <SidebarGroup border>
          {#if authStore.user}
            <SidebarItem label={authStore.user.name} href="/app/profile">
              {#snippet icon()}
                {#if authStore.user!.image}
                  <Avatar
                    data-name={authStore.user!.name}
                    src={authStore.user!.image}
                    size="sm"
                  />
                {:else}
                  <Avatar size="sm" />
                {/if}
              {/snippet}
            </SidebarItem>
          {:else}
            <li>
              <button
                class="flex w-full items-center rounded-sm p-2 text-base font-normal text-gray-900 hover:bg-gray-100 disabled:cursor-not-allowed dark:text-white dark:hover:bg-gray-700"
                onclick={onSignInClick}
                disabled={authStore.isSigningIn}
                ><UserCircleOutline size="xl" />
                <span class="ms-3">{t('app.sign_in')}</span>
                {#if authStore.isSigningIn}
                  <Spinner class="ms-3 inline-flex" size="4" />
                {:else if authStore.signInFallback}
                  <QuestionCircleOutline size="md" class="ms-1" />
                {/if}
              </button>
            </li>
          {/if}
          {#if updaterStore.info && ot == 'macos'}
            <li>
              <button
                class="flex w-full items-center rounded-sm p-2 text-base font-normal hover:bg-gray-100 dark:text-white dark:hover:bg-gray-700"
                onclick={() => open_update_window()}
              >
                <BellRingOutline
                  class="text-primary-500 inline-flex"
                  size="md"
                />
                <span class="ms-3">
                  {t('app.new_version', {
                    version: updaterStore.info?.version || 'v1.0.0'
                  })}
                </span>
              </button>
            </li>
          {:else if updaterStore.info}
            <li>
              <button
                class="flex w-full items-center rounded-sm p-2 text-base font-normal hover:bg-gray-100 dark:text-white dark:hover:bg-gray-700 {updaterStore.isDownloading
                  ? 'cursor-not-allowed text-gray-500'
                  : ''}"
                onclick={() => updaterStore.restartApp()}
                disabled={updaterStore.isDownloading ||
                  updaterStore.isRestarting}
              >
                {#if updaterStore.isDownloading}
                  <Spinner class="ms-3 inline-flex" size="4" />
                  <span class="ms-3">
                    {t('app.download_update', {
                      version: updaterStore.info?.version
                    })}
                  </span>
                {:else}
                  <RefreshOutline
                    size="lg"
                    class="text-primary-500 {updaterStore.isDownloading
                      ? 'animate-spin'
                      : ''}"
                  />
                  <span class="ms-3">{t('app.update_restart')}</span>
                {/if}
              </button>
            </li>
          {/if}
        </SidebarGroup>
      </Sidebar>
      <div class="relative h-full w-full overflow-auto dark:bg-gray-900">
        {@render children()}
      </div>
      <DiagnosisModal bind:this={diagnosisModalRef} />
    </main>
  {/if}
{/key}

<style>
  :global(.anda-nav [aria-current='page'] svg) {
    color: #11c291 !important;
  }

  :global(.dark .anda-nav [aria-current='page'] svg) {
    color: #11c291 !important;
  }
</style>
