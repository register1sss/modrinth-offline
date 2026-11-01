<script setup>
import { computed, ref, onMounted, onBeforeUnmount } from 'vue'
import { language, setLanguage, tr, translate } from '@/i18n/index.js'
import { invoke as rustInvoke } from '@tauri-apps/api/tauri'
import { open as openExternal } from '@tauri-apps/api/shell'
import { externalLink } from '@/helpers/links.js'
import { updateRustSession } from '@/minecraft-rust/state.js'
import { session as aiSession } from '@/assistant/state.js'
import { RouterView, RouterLink, useRouter, useRoute } from 'vue-router'
import {
  HomeIcon,
  SearchIcon,
  LibraryIcon,
  PlusIcon,
  SettingsIcon,
  PaintBrushIcon,
  FileIcon,
  XIcon,
} from '@modrinth/assets'
import { Button, Notifications, Card } from '@modrinth/ui'
import { useLoading, useTheming } from '@/store/state'
import MinecraftRustIcon from '@/components/brand/MinecraftRustIcon.vue'
import AssistantIcon from '@/components/brand/AssistantIcon.vue'
import ThemeChrome from '@/components/themes/ThemeChrome.vue'
import NavigationBurst from '@/components/themes/NavigationBurst.vue'
import ParticleCanvas from '@/components/themes/ParticleCanvas.vue'
import AccountsCard from '@/components/ui/AccountsCard.vue'
import InstanceCrashPrompt from '@/components/ui/InstanceCrashPrompt.vue'
import InstanceCreationModal from '@/components/ui/InstanceCreationModal.vue'
import { get } from '@/helpers/settings'
import Breadcrumbs from '@/components/ui/Breadcrumbs.vue'
import RunningAppBar from '@/components/ui/RunningAppBar.vue'
import SplashScreen from '@/components/ui/SplashScreen.vue'
import ErrorModal from '@/components/ui/ErrorModal.vue'
import ModrinthLoadingIndicator from '@/components/modrinth-loading-indicator'
import { handleError, useNotifications } from '@/store/notifications.js'
import { command_listener, warning_listener } from '@/helpers/events.js'
import { MinimizeIcon, MaximizeIcon } from '@/assets/icons'
import { type } from '@tauri-apps/api/os'
import { appWindow } from '@tauri-apps/api/window'
import { isDev, getOS, showLauncherLogsFolder } from '@/helpers/utils.js'
import {
  mixpanel_track,
  mixpanel_init,
  mixpanel_opt_out_tracking,
  mixpanel_is_loaded,
} from '@/helpers/mixpanel'
import { saveWindowState, StateFlags } from 'tauri-plugin-window-state-api'
import { getVersion } from '@tauri-apps/api/app'
import { window as TauriWindow } from '@tauri-apps/api'
import { TauriEvent } from '@tauri-apps/api/event'
import URLConfirmModal from '@/components/ui/URLConfirmModal.vue'
import OnboardingScreen from '@/components/ui/tutorial/OnboardingScreen.vue'
import { install_from_file } from './helpers/pack'
import { useError } from '@/store/error.js'
import ModInstallModal from '@/components/ui/install_flow/ModInstallModal.vue'
import IncompatibilityWarningModal from '@/components/ui/install_flow/IncompatibilityWarningModal.vue'
import InstallConfirmModal from '@/components/ui/install_flow/InstallConfirmModal.vue'
import { useInstall } from '@/store/install.js'

const themeStore = useTheming()
const motionPreference = window.matchMedia('(prefers-reduced-motion: reduce)')
const syncMotionPreference = () => {
  themeStore.setReducedMotion(motionPreference.matches)
}
syncMotionPreference()
onMounted(() => motionPreference.addEventListener('change', syncMotionPreference))
onBeforeUnmount(() => motionPreference.removeEventListener('change', syncMotionPreference))
const urlModal = ref(null)
const isLoading = ref(true)

const offline = ref(!navigator.onLine)
const syncOnline = () => {
  offline.value = !navigator.onLine
}
window.addEventListener('offline', syncOnline)
window.addEventListener('online', syncOnline)
onBeforeUnmount(() => {
  window.removeEventListener('offline', syncOnline)
  window.removeEventListener('online', syncOnline)
})

const showOnboarding = ref(false)
const nativeDecorations = ref(false)

const onboardingVideo = ref()

const failureText = ref(null)
const os = ref('')

defineExpose({
  initialize: async () => {
    isLoading.value = false
    const {
      native_decorations,
      theme,
      telemetry,
      collapsed_navigation,
      advanced_rendering,
      onboarded,
    } = await get()
    // video should play if the user is not on linux, and has not onboarded
    os.value = await getOS()
    const dev = await isDev()
    const version = await getVersion()
    showOnboarding.value = !onboarded

    nativeDecorations.value = native_decorations
    if (os.value !== 'MacOS') appWindow.setDecorations(native_decorations)

    themeStore.setThemeState(theme)
    themeStore.collapsedNavigation = collapsed_navigation
    themeStore.advancedRendering = advanced_rendering

    mixpanel_init('014c7d6a336d0efaefe3aca91063748d', { debug: dev, persistence: 'localStorage' })
    if (telemetry) {
      mixpanel_opt_out_tracking()
    }
    mixpanel_track('Launched', { version, dev, onboarded })

    if (!dev) document.addEventListener('contextmenu', (event) => event.preventDefault())

    if ((await type()) === 'Darwin') {
      document.getElementsByTagName('html')[0].classList.add('mac')
    } else {
      document.getElementsByTagName('html')[0].classList.add('windows')
    }

    await warning_listener((e) =>
      notificationsWrapper.value.addNotification({
        title: 'Warning',
        text: e.message,
        type: 'warn',
      }),
    )

    if (showOnboarding.value) {
      await onboardingVideo.value?.play()?.catch(() => {})
    }
  },
  failure: async (e) => {
    isLoading.value = false
    failureText.value = e
    os.value = await getOS()
  },
})

const handleClose = async () => {
  await TauriWindow.getCurrent().close()
}

TauriWindow.getCurrent().listen(TauriEvent.WINDOW_CLOSE_REQUESTED, async () => {
  await handleClose()
})

let rustPoll,
  rustPolling = false
onMounted(() => {
  rustPoll = setInterval(async () => {
    if (rustPolling) return
    rustPolling = true
    try {
      updateRustSession(await rustInvoke('plugin:minecraft-rust|status'))
    } catch {
      /* Older native builds have no Rust plugin. */
    } finally {
      rustPolling = false
    }
  }, 2500)
})
onBeforeUnmount(() => clearInterval(rustPoll))
const router = useRouter()
router.afterEach((to, from, failure) => {
  if (mixpanel_is_loaded()) {
    mixpanel_track('PageView', { path: to.path, fromPath: from.path, failed: failure })
  }
})
const route = useRoute()
const isOnBrowse = computed(() => route.path.startsWith('/browse'))

const loading = useLoading()

const notifications = useNotifications()
const notificationsWrapper = ref()

const error = useError()
const errorModal = ref()

const install = useInstall()
const modInstallModal = ref()
const installConfirmModal = ref()
const incompatibilityWarningModal = ref()

onMounted(() => {
  notifications.setNotifs(notificationsWrapper.value)

  error.setErrorModal(errorModal.value)

  install.setIncompatibilityWarningModal(incompatibilityWarningModal)
  install.setInstallConfirmModal(installConfirmModal)
  install.setModInstallModal(modInstallModal)
})

function handleLinkClick(event) {
  if (event.defaultPrevented || !(event.target instanceof Element)) return
  const anchor = event.target.closest('a[href]')
  if (!anchor) return
  const url = externalLink(anchor.href, location.origin)
  if (!url) {
    // Do not hand unsupported schemes or credential-bearing external URLs to the WebView.
    try {
      const destination = new URL(anchor.href)
      if (
        destination.protocol !== location.protocol ||
        destination.host !== location.host ||
        destination.origin !== location.origin
      )
        event.preventDefault()
    } catch {
      event.preventDefault()
    }
    return
  }
  event.preventDefault()
  openExternal(url).catch(handleError)
}
function handleAuxClick(event) {
  if (event.button !== 1 || !(event.target instanceof Element)) return
  const anchor = event.target.closest('a[href]')
  if (!anchor) return
  event.preventDefault()
  anchor.click()
}
onMounted(() => {
  document.body.addEventListener('click', handleLinkClick)
  document.body.addEventListener('auxclick', handleAuxClick)
})
onBeforeUnmount(() => {
  document.body.removeEventListener('click', handleLinkClick)
  document.body.removeEventListener('auxclick', handleAuxClick)
})

const accounts = ref(null)

command_listener(async (e) => {
  if (e.event === 'RunMRPack') {
    // RunMRPack should directly install a local mrpack given a path
    if (e.path.endsWith('.mrpack')) {
      await install_from_file(e.path).catch(handleError)
      mixpanel_track('InstanceCreate', {
        source: 'CreationModalFileDrop',
      })
    }
  } else {
    // Other commands are URL-based (deep linking)
    urlModal.value.show(e)
  }
})
</script>

<template>
  <div v-if="failureText" class="failure dark-mode">
    <div class="appbar-failure dark-mode">
      <Button v-if="os != 'MacOS'" icon-only @click="TauriWindow.getCurrent().close()">
        <XIcon />
      </Button>
    </div>
    <div class="error-view dark-mode">
      <Card class="error-text">
        <div class="label">
          <h3>
            <span class="label__title size-card-header">{{ $u3t('Failed to initialize') }}</span>
          </h3>
        </div>
        <div class="error-div">
          {{
            $u3t(
              'U3drinth failed to load correctly. This may be because of a corrupted file, or because the app is missing crucial files.',
            )
          }}
        </div>
        <div class="error-div">
          {{ $u3t('You may be able to fix it in one of the following ways:') }}
        </div>
        <ul class="error-div">
          <li>{{ $u3t('Make sure you are connected to the internet, then restart the app.') }}</li>
          <li>{{ $u3t('Download and reinstall the app.') }}</li>
        </ul>
        <div class="error-div">
          {{
            $u3t(
              'Review the error and recent launcher logs below. U3 AI can help explain crash reports.',
            )
          }}
        </div>
        <div class="error-div">{{ $u3t('The following error was provided:') }}</div>

        <Card class="error-message">
          {{ failureText.message }}
        </Card>

        <div class="button-row push-right">
          <Button @click="showLauncherLogsFolder"
            ><FileIcon /> {{ $u3t('Open launcher logs') }}
          </Button>
        </div>
      </Card>
    </div>
  </div>
  <SplashScreen v-else-if="isLoading" app-loading />
  <OnboardingScreen v-else-if="showOnboarding" :finish="() => (showOnboarding = false)" />
  <div v-else class="container u3-studio" :class="{ 'has-depth': themeStore.advancedRendering }">
    <ParticleCanvas
      v-if="themeStore.activeTheme.particle && themeStore.effectsEnabled"
      class="app-atmosphere"
      :theme="themeStore.activeTheme"
      :quality="themeStore.effectsQuality"
      :reduced-motion="themeStore.reducedMotion"
      subdued
    />
    <NavigationBurst />
    <div class="nav-container">
      <ThemeChrome v-if="themeStore.activeTheme.particle && themeStore.effectsEnabled" vertical />
      <div class="nav-section">
        <suspense>
          <AccountsCard ref="accounts" mode="small" />
        </suspense>
        <div class="pages-list">
          <RouterLink v-tooltip="translate('Home')" to="/" class="btn icon-only collapsed-button">
            <HomeIcon />
          </RouterLink>
          <RouterLink
            v-tooltip="translate('Browse')"
            to="/browse/modpack"
            class="btn icon-only collapsed-button"
            :class="{
              'router-link-active': isOnBrowse,
            }"
          >
            <SearchIcon />
          </RouterLink>
          <RouterLink
            v-tooltip="translate('Library')"
            to="/library"
            class="btn icon-only collapsed-button"
          >
            <LibraryIcon />
          </RouterLink>
          <RouterLink
            v-tooltip="translate('Темы')"
            to="/themes"
            :aria-label="translate('Темы')"
            class="btn icon-only collapsed-button"
          >
            <PaintBrushIcon />
          </RouterLink>
          <RouterLink
            v-tooltip="'U3 AI'"
            to="/assistant"
            class="btn icon-only collapsed-button ai-nav-link"
            aria-label="U3 AI"
            ><AssistantIcon /><span
              v-if="aiSession.alerts.length"
              class="ai-nav-alert"
              :aria-label="$u3t('Errors')"
              >!</span
            ></RouterLink
          >
          <RouterLink
            v-tooltip="'Minecraft Rust'"
            to="/minecraft-rust"
            class="btn icon-only collapsed-button"
            aria-label="Minecraft Rust"
            ><MinecraftRustIcon
          /></RouterLink>
          <Suspense>
            <InstanceCreationModal ref="installationModal" />
          </Suspense>
        </div>
      </div>
      <div class="settings pages-list">
        <Button
          v-tooltip="$u3t('Create profile')"
          class="sleek-primary collapsed-button"
          icon-only
          :disabled="offline"
          @click="() => $refs.installationModal.show()"
        >
          <PlusIcon />
        </Button>
        <RouterLink
          v-tooltip="translate('Settings')"
          to="/settings"
          class="btn icon-only collapsed-button"
        >
          <SettingsIcon />
        </RouterLink>
      </div>
    </div>
    <div class="view">
      <div class="appbar-row">
        <ThemeChrome v-if="themeStore.activeTheme.particle && themeStore.effectsEnabled" />
        <div data-tauri-drag-region class="appbar">
          <section class="navigation-controls">
            <Breadcrumbs data-tauri-drag-region />
          </section>
          <section class="mod-stats">
            <select
              class="language-switch"
              :value="language"
              :aria-label="tr('Язык интерфейса', 'Interface language')"
              @change="setLanguage($event.target.value)"
            >
              <option value="ru">RU</option>
              <option value="en">EN</option>
            </select>
            <Suspense>
              <RunningAppBar />
            </Suspense>
          </section>
        </div>
        <section v-if="!nativeDecorations" class="window-controls">
          <Button class="titlebar-button" icon-only @click="() => appWindow.minimize()">
            <MinimizeIcon />
          </Button>
          <Button class="titlebar-button" icon-only @click="() => appWindow.toggleMaximize()">
            <MaximizeIcon />
          </Button>
          <Button
            class="titlebar-button close"
            icon-only
            @click="
              () => {
                saveWindowState(StateFlags.ALL)
                handleClose()
              }
            "
          >
            <XIcon />
          </Button>
        </section>
      </div>
      <div class="router-view">
        <ModrinthLoadingIndicator
          offset-height="var(--appbar-height)"
          offset-width="var(--sidebar-width)"
        />
        <RouterView v-slot="{ Component }">
          <template v-if="Component">
            <Transition name="route-motion" mode="out-in"
              ><div :key="route.path" class="route-stage">
                <Suspense @pending="loading.startLoading()" @resolve="loading.stopLoading()"
                  ><component :is="Component"
                /></Suspense></div
            ></Transition>
          </template>
        </RouterView>
      </div>
    </div>
  </div>
  <URLConfirmModal ref="urlModal" />
  <InstanceCrashPrompt />
  <Notifications ref="notificationsWrapper" />
  <ErrorModal ref="errorModal" />
  <ModInstallModal ref="modInstallModal" />
  <IncompatibilityWarningModal ref="incompatibilityWarningModal" />
  <InstallConfirmModal ref="installConfirmModal" />
</template>

<style lang="scss" scoped>
.sleek-primary {
  background-color: var(--color-brand-highlight);
  transition: all ease-in-out 0.1s;
}

.navigation-controls {
  flex-grow: 1;
  width: min-content;
}

.appbar-row {
  display: flex;
  flex-direction: row;
}

.window-controls {
  z-index: 20;
  display: none;
  flex-direction: row;
  align-items: center;

  .titlebar-button {
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all ease-in-out 0.1s;
    background-color: var(--color-raised-bg);
    color: var(--color-base);
    border-radius: 0;
    height: 3.25rem;

    &.close {
      &:hover,
      &:active {
        background-color: var(--color-red);
        color: var(--color-accent-contrast);
      }
    }

    &:hover,
    &:active {
      background-color: var(--color-button-bg);
      color: var(--color-contrast);
    }
  }
}

.app-atmosphere {
  position: fixed;
  inset: 0;
  width: 100vw;
  height: 100vh;
  z-index: -1;
}

.container {
  isolation: isolate;
  --appbar-height: 3.25rem;
  --sidebar-width: 4.5rem;

  height: 100vh;
  display: flex;
  flex-direction: row;
  overflow: hidden;

  .view {
    width: calc(100% - var(--sidebar-width));
    background-color: var(--app-view-bg, var(--color-raised-bg));

    .appbar {
      display: flex;
      align-items: center;
      flex-grow: 1;
      background: var(--color-raised-bg);
      text-align: center;
      padding: var(--gap-md);
      height: 3.25rem;
      gap: var(--gap-sm);
      //no select
      user-select: none;
      -webkit-user-select: none;
    }

    .router-view {
      width: 100%;
      height: calc(100% - 3.125rem);
      overflow: auto;
      overflow-x: hidden;
      background-color: var(--app-router-bg, var(--color-bg));
      border-top-left-radius: var(--radius-xl);
    }
  }
}

.failure {
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background-color: var(--color-bg);

  .appbar-failure {
    display: flex; /* Change to flex to align items horizontally */
    justify-content: flex-end; /* Align items to the right */
    height: 3.25rem;
    //no select
    user-select: none;
    -webkit-user-select: none;
  }

  .error-view {
    display: flex; /* Change to flex to align items horizontally */
    justify-content: center;
    width: 100%;
    background-color: var(--color-bg);

    color: var(--color-base);

    .card {
      background-color: var(--color-raised-bg);
    }

    .error-text {
      display: flex;
      max-width: 60%;
      gap: 0.25rem;
      flex-direction: column;

      .error-div {
        // spaced out
        margin: 0.5rem;
      }

      .error-message {
        margin: 0.5rem;
        background-color: var(--color-button-bg);
      }
    }
  }
}

.nav-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: space-between;
  height: 100%;
  background-color: var(--color-raised-bg);
  box-shadow: var(--shadow-inset-sm), var(--shadow-floating);
  padding: var(--gap-md);
}

.pages-list {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  justify-content: flex-start;
  width: 100%;
  gap: 0.5rem;

  a {
    display: flex;
    align-items: center;
    word-spacing: 3px;
    background: inherit;
    transition: all ease-in-out 0.1s;
    color: var(--color-base);
    box-shadow: none;

    &.router-link-active {
      color: var(--color-contrast);
      background: var(--color-button-bg);
      box-shadow: var(--shadow-floating);
    }

    &:hover {
      background-color: var(--color-button-bg);
      color: var(--color-contrast);
      box-shadow: 0px 4px 4px rgba(0, 0, 0, 0.25);
      text-decoration: none;
    }
  }

  &.primary {
    color: var(--color-accent-contrast);
    background-color: var(--color-brand);
  }
}

.collapsed-button {
  height: 3rem !important;
  width: 3rem !important;
  padding: 0.75rem;
  border-radius: var(--radius-md);
  box-shadow: none;

  svg {
    width: 1.5rem !important;
    height: 1.5rem !important;
    max-width: 1.5rem !important;
    max-height: 1.5rem !important;
  }
}

.nav-section {
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
  align-items: center;
  width: 100%;
  height: 100%;
  gap: 1rem;
}

.button-row {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  gap: var(--gap-md);

  .transparent {
    padding: var(--gap-sm) 0;
  }
}
</style>

<style scoped>
.nav-container,
.appbar-row {
  position: relative;
  isolation: isolate;
}
.nav-container > .nav-section,
.nav-container > .settings,
.appbar-row > .appbar,
.appbar-row > .window-controls {
  position: relative;
  z-index: 1;
}
.container .view .appbar-row > .appbar {
  background: transparent;
}
.appbar-row {
  background: var(--color-raised-bg);
}
</style>

<style scoped>
/* Keep account popovers above the page without moving decoration above controls. */
.nav-container {
  z-index: 20;
}
.appbar-row {
  z-index: 10;
}
</style>
