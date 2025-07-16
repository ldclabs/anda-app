<script lang="ts">
  import { page } from '$app/state'
  import { authStore, get_user, logout, signIn } from '$lib/stores/auth.svelte'
  import { isTauriEnvironment, safeOsType } from '$lib/utils/tauri-mock'
  import { type as osType } from '@tauri-apps/plugin-os'
  import {
    BottomNav,
    BottomNavItem,
    Dropdown,
    DropdownGroup,
    DropdownHeader,
    DropdownItem,
    Sidebar,
    SidebarGroup,
    SidebarItem,
    Spinner
  } from 'flowbite-svelte'
  import {
    ArrowRightToBracketOutline,
    CogOutline,
    ComputerSpeakerOutline,
    UserCircleOutline,
    UserHeadsetOutline,
    WalletOutline
  } from 'flowbite-svelte-icons'

  // Safe OS type detection
  const ot = isTauriEnvironment() ? osType() : safeOsType()
  const enableMobile = ot != 'ios' && ot != 'android'

  let { children } = $props()
  let isMobile = $state(ot === 'ios' || ot === 'android')
  let activeUrl = $state(page.url.pathname)
  let isSignIn = $state(false)

  $effect(() => {
    activeUrl = page.url.pathname
  })

  async function onSignIn(event: Event) {
    event.preventDefault()

    isSignIn = true
    setTimeout(() => {
      isSignIn = false
    }, 10000)

    await signIn()
  }

  async function onLogout() {
    isSignIn = false
    await logout()
  }
</script>

{#key authStore.auth.id}
  {#if isMobile}
    <main class="relative grid h-dvh w-dvw grid-rows-[1fr_auto]">
      <div class="relative w-full overflow-auto">
        {@render children()}
      </div>

      <BottomNav
        {activeUrl}
        innerClass="grid-cols-4"
        outerClass="relative anda-nav"
        activeClass="font-bold text-green-500 hover:text-green-900 dark:hover:text-green-700 dark:text-green-300"
      >
        <BottomNavItem btnName="Agent" href="/agent">
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
            <BottomNavItem btnName="Sign In" href="#">
              <UserCircleOutline
                size="lg"
                class={isSignIn ? 'animate-bounce' : ''}
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
          <BottomNavItem btnName="Sign In" onclick={onSignIn}>
            <UserCircleOutline
              size="lg"
              class={isSignIn ? 'animate-bounce' : ''}
            />
          </BottomNavItem>
        {/if}
      </BottomNav>
    </main>
  {:else}
    <Dropdown
      simple
      placement="top"
      class="min-w-48"
      offset={12}
      triggeredBy="#dd-management"
    >
      <DropdownGroup>
        <DropdownItem
          class="flex w-full flex-row items-center gap-2 text-left"
          onclick={() => {}}
          disabled={true}><WalletOutline />Wallet</DropdownItem
        >
        {#if enableMobile}
          <DropdownItem
            class="flex w-full flex-row items-center gap-2 text-left"
            onclick={() => (isMobile = !isMobile)}
            ><ComputerSpeakerOutline />Switch UI</DropdownItem
          >
        {/if}
      </DropdownGroup>
      {#if authStore.auth.isAuthenticated()}
        <DropdownHeader
          class="border-t border-gray-200 px-0 dark:border-gray-700"
        >
          <DropdownItem
            onclick={onLogout}
            class="flex w-full flex-row items-center gap-2 text-left text-orange-600"
            ><ArrowRightToBracketOutline />Logout
          </DropdownItem>
        </DropdownHeader>
      {/if}
    </Dropdown>
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
          <SidebarItem label="Agent" href="/agent">
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
              <SidebarItem label="Sign In" href="#">
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
                onclick={onSignIn}
                ><UserCircleOutline size="lg" />
                <span class="ms-3">Sign In</span>
                {#if isSignIn}
                  <Spinner class="ms-3 inline-flex" size="4" />
                {/if}
              </button>
            </li>
          {/if}

          <li>
            <button
              id="dd-management"
              class="flex w-full items-center rounded-sm p-2 text-base font-normal text-gray-900 hover:bg-gray-100 dark:text-white dark:hover:bg-gray-700"
              ><CogOutline size="lg" /><span class="ms-3">Management</span
              ></button
            >
          </li>
        </SidebarGroup>
      </Sidebar>
      <div class="relative h-full w-full overflow-auto">
        {@render children()}
      </div>
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
