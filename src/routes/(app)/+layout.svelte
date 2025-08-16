<script lang="ts">
  import { page } from '$app/state'
  import DiagnosisModal from '$lib/components/diagnosis/DiagnosisModal.svelte'
  import { authStore, get_user, signIn } from '$lib/stores/auth.svelte'
  import { t } from '$lib/stores/i18n'
  import { isTauriEnvironment, safeOsType } from '$lib/utils/tauri.mock'
  import { type as osType } from '@tauri-apps/plugin-os'
  import {
    BottomNav,
    BottomNavItem,
    Sidebar,
    SidebarGroup,
    SidebarItem,
    Spinner
  } from 'flowbite-svelte'
  import { UserCircleOutline, UserHeadsetOutline } from 'flowbite-svelte-icons'
  import { setContext } from 'svelte'

  // Safe OS type detection
  const ot = isTauriEnvironment() ? osType() : safeOsType()
  const diagnosisModalState = $state({
    open: (view: 'kip' | 'conversation') => {}
  })

  setContext('diagnosisModalState', diagnosisModalState)

  let { children } = $props()
  let isMobile = $state(ot === 'ios' || ot === 'android')
  let activeUrl = $state(page.url.pathname)

  $effect(() => {
    activeUrl = page.url.pathname
  })
</script>

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
        <BottomNavItem btnName={t('assistant.title')} href="/assistant">
          <UserHeadsetOutline size="lg" />
        </BottomNavItem>
        <!-- <BottomNavItem btnName="Discover" href="/discover">
          <SearchOutline size="lg" />
        </BottomNavItem> -->
        <!-- <BottomNavItem btnName="Messages" href="/messages">
          <MessagesOutline size="lg" />
        </BottomNavItem> -->
        {#if authStore.auth.isAuthenticated()}
          {#await get_user()}
            <BottomNavItem btnName={t('app.sign_in')} href="#">
              <UserCircleOutline
                size="lg"
                class={authStore.isSigningIn ? 'animate-bounce' : ''}
              />
            </BottomNavItem>
          {:then userInfo}
            <BottomNavItem btnName={userInfo.name} href="#">
              {#if userInfo.image}
                <img
                  src={userInfo.image}
                  alt={userInfo.name + 'image'}
                  class="size-6 rounded-full"
                />
              {:else}
                <UserCircleOutline size="lg" />
              {/if}
            </BottomNavItem>
          {/await}
        {:else}
          <BottomNavItem btnName={t('app.sign_in')} onclick={signIn}>
            <UserCircleOutline
              size="lg"
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
        divClass="grid grid-rows-[1fr_auto] h-full"
        activeClass="p-2"
        nonActiveClass="p-2"
      >
        <SidebarGroup>
          <SidebarItem label={t('assistant.title')} href="/assistant">
            {#snippet icon()}
              <UserHeadsetOutline size="lg" />
            {/snippet}
          </SidebarItem>
          <!-- <SidebarItem label="Discover" href="/discover">
            {#snippet icon()}
              <SearchOutline size="lg" />
            {/snippet}
          </SidebarItem> -->
          <!-- <SidebarItem label="Messages" href="/messages">
            {#snippet icon()}
              <MessagesOutline size="lg" />
            {/snippet}
          </SidebarItem> -->
        </SidebarGroup>
        <SidebarGroup border>
          {#if authStore.auth.isAuthenticated()}
            {#await get_user()}
              <SidebarItem label={t('app.sign_in')} href="#">
                {#snippet icon()}
                  <UserCircleOutline size="lg" />
                {/snippet}
                {#snippet subtext()}
                  <Spinner class="ms-3 inline-flex" size="4" />
                {/snippet}
              </SidebarItem>
            {:then userInfo}
              <SidebarItem label={userInfo.name} href="/profile">
                {#snippet icon()}
                  {#if userInfo.image}
                    <img
                      src={userInfo.image}
                      alt={userInfo.name + 'image'}
                      class="size-6 rounded-full"
                    />
                  {:else}
                    <UserCircleOutline size="lg" />
                  {/if}
                {/snippet}
              </SidebarItem>
            {/await}
          {:else}
            <li>
              <button
                class="flex w-full items-center rounded-sm p-2 text-base font-normal text-gray-900 hover:bg-gray-100 dark:text-white dark:hover:bg-gray-700"
                onclick={signIn}
                disabled={authStore.isSigningIn}
                ><UserCircleOutline size="lg" />
                <span class="ms-3">{t('app.sign_in')}</span>
                {#if authStore.isSigningIn}
                  <Spinner class="ms-3 inline-flex" size="4" />
                {/if}
              </button>
            </li>
          {/if}
        </SidebarGroup>
      </Sidebar>
      <div class="relative h-full w-full overflow-auto dark:bg-gray-900">
        {@render children()}
      </div>
      <DiagnosisModal callback={diagnosisModalState} />
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
