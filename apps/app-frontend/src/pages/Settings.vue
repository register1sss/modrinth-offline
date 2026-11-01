<script setup>
import { ref, watch, onBeforeUnmount } from 'vue'
import { language, setLanguage, tr, translate } from '@/i18n/index.js'
import { LogOutIcon, LogInIcon, BoxIcon, FolderSearchIcon } from '@modrinth/assets'
import { Card, Slider, DropdownSelect, Toggle, Button } from '@modrinth/ui'
import { handleError, useTheming } from '@/store/state'
import { get, update } from '@/helpers/settings'
import { get_java_versions, get_max_memory, set_java_version } from '@/helpers/jre'
import { get as getCreds, logout } from '@/helpers/mr_auth.js'
import JavaSelector from '@/components/ui/JavaSelector.vue'
import ModrinthLoginScreen from '@/components/ui/tutorial/ModrinthLoginScreen.vue'
import { mixpanel_opt_out_tracking, mixpanel_opt_in_tracking } from '@/helpers/mixpanel'
import { open } from '@tauri-apps/api/dialog'
import { getOS } from '@/helpers/utils.js'
import { get_user } from '@/helpers/cache.js'

import { getTheme } from '@/themes/catalog.js'

import { formatLaunchArgs, nativeSettings, settingsChanges } from '@/helpers/settings-model.js'

const pageOptions = ['Home', 'Library']

const themeStore = useTheming()

const accessSettings = async () => {
  const settings = await get()

  settings.launchArgs = formatLaunchArgs(settings.extra_launch_args)
  settings.envVars = formatLaunchArgs(settings.custom_env_vars.map((x) => x.join('=')))

  return settings
}

const fetchSettings = await accessSettings().catch(handleError)

const settings = ref(fetchSettings)
// const settingsDir = ref(settings.value.loaded_config_dir)

const maxMemory = ref(Math.floor((await get_max_memory().catch(handleError)) / 1024))

let lastSaved = nativeSettings(settings.value)
let saveTimer,
  pendingSave = Promise.resolve()
function saveSettings() {
  clearTimeout(saveTimer)
  saveTimer = null
  // Capture the current editor snapshot; serialize saves and don't overwrite unrelated changes.
  let snapshot
  try {
    snapshot = nativeSettings(settings.value)
  } catch (error) {
    handleError(error)
    return
  }
  pendingSave = pendingSave
    .catch(() => {})
    .then(async () => {
      const patch = settingsChanges(lastSaved, snapshot)
      if (!Object.keys(patch).length) return
      try {
        await update(patch)
      } catch (error) {
        if ('theme' in patch && settings.value.theme === snapshot.theme) {
          settings.value.theme = lastSaved.theme
          themeStore.setThemeState(lastSaved.theme)
        }
        throw error
      }
      lastSaved = snapshot
      if (snapshot.telemetry) mixpanel_opt_out_tracking()
      else mixpanel_opt_in_tracking()
    })
    .catch(handleError)
}
watch(
  settings,
  () => {
    clearTimeout(saveTimer)
    saveTimer = setTimeout(saveSettings, 350)
  },
  { deep: true },
)
onBeforeUnmount(() => {
  if (saveTimer) saveSettings()
})

const javaVersions = ref(await get_java_versions().catch(handleError))
async function updateJavaVersion(version) {
  if (version?.path === '') {
    version.path = undefined
  }

  if (version?.path) {
    version.path = version.path.replace('java.exe', 'javaw.exe')
  }

  await set_java_version(version).catch(handleError)
}

async function fetchCredentials() {
  const creds = await getCreds().catch(handleError)
  if (creds && creds.user_id) {
    creds.user = await get_user(creds.user_id).catch(handleError)
  }
  credentials.value = creds
}

const credentials = ref()
await fetchCredentials()

const loginScreenModal = ref()

async function logOut() {
  await logout().catch(handleError)
  await fetchCredentials()
}

async function signInAfter() {
  await fetchCredentials()
}

async function findLauncherDir() {
  const newDir = await open({
    multiple: false,
    directory: true,
    title: translate('Select a new app directory'),
  })

  if (newDir) {
    settings.value.custom_dir = newDir
  }
}
</script>

<template>
  <div class="settings-page">
    <Card
      ><div class="adjacent-input">
        <label for="interface-language"
          ><span class="label__title">{{ tr('Язык интерфейса', 'Interface language') }}</span
          ><span class="label__description">{{
            tr('Сохраняется на этом компьютере.', 'Saved on this computer.')
          }}</span></label
        ><select
          id="interface-language"
          :value="language"
          @change="setLanguage($event.target.value)"
        >
          <option value="ru">Русский</option>
          <option value="en">{{ $u3t('English') }}</option>
        </select>
      </div></Card
    >
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header"> {{ $u3t('General settings') }} </span>
        </h3>
      </div>
      <ModrinthLoginScreen ref="loginScreenModal" :callback="signInAfter" />
      <div class="adjacent-input">
        <label for="theme">
          <span class="label__title"> {{ $u3t('Manage account') }} </span>
          <span v-if="credentials" class="label__description">
            {{ $u3t('You are currently logged in as') }} {{ credentials.user.username }}.
          </span>
          <span v-else> {{ $u3t('Sign in to your Modrinth account.') }} </span>
        </label>
        <button v-if="credentials" class="btn" @click="logOut">
          <LogOutIcon /> {{ $u3t('Sign out') }}
        </button>
        <button v-else class="btn" @click="$refs.loginScreenModal.show()">
          <LogInIcon /> {{ $u3t('Sign in') }}
        </button>
      </div>
      <label for="theme">
        <span class="label__title"> {{ $u3t('App directory') }} </span>
        <span class="label__description">
          {{ $u3t('The directory where the launcher stores all of its files.') }}
        </span>
      </label>
      <div class="app-directory">
        <div class="iconified-input">
          <BoxIcon />
          <input id="appDir" v-model="settings.custom_dir" type="text" class="input" />
          <Button class="r-btn" @click="findLauncherDir">
            <FolderSearchIcon />
          </Button>
        </div>
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header"> {{ $u3t('Display') }} </span>
        </h3>
      </div>
      <div class="adjacent-input">
        <label for="theme">
          <span class="label__title"> {{ $u3t('Color theme') }} </span>
          <span class="label__description">
            {{ $u3t('Change the global launcher color theme.') }}
          </span>
        </label>
        <DropdownSelect
          id="theme"
          name="Theme dropdown"
          :options="themeStore.themeOptions"
          :display-name="(id) => translate(getTheme(id).name)"
          :default-value="settings.theme"
          :model-value="settings.theme"
          class="theme-dropdown"
          @change="
            (e) => {
              themeStore.setThemeState(e.option.toLowerCase())
              settings.theme = themeStore.selectedTheme
            }
          "
        />
      </div>
      <RouterLink to="/themes" class="btn theme-gallery-link">
        {{ tr('Открыть галерею тем', 'Open theme gallery') }} · {{ themeStore.themeOptions.length }}
      </RouterLink>
      <div class="adjacent-input">
        <label for="advanced-rendering">
          <span class="label__title"> {{ $u3t('Advanced rendering') }} </span>
          <span class="label__description">
            {{
              $u3t(
                'Enables advanced rendering such as blur effects that may cause performance issues without hardware-accelerated rendering.',
              )
            }}
          </span>
        </label>
        <Toggle
          id="advanced-rendering"
          :model-value="themeStore.advancedRendering"
          :checked="themeStore.advancedRendering"
          @update:model-value="
            (e) => {
              themeStore.advancedRendering = e
              settings.advanced_rendering = themeStore.advancedRendering
            }
          "
        />
      </div>
      <div class="adjacent-input">
        <label for="minimize-launcher">
          <span class="label__title"> {{ $u3t('Minimize launcher') }} </span>
          <span class="label__description">
            {{ $u3t('Minimize the launcher when a Minecraft process starts.') }}
          </span>
        </label>
        <Toggle
          id="minimize-launcher"
          :model-value="settings.hide_on_process_start"
          :checked="settings.hide_on_process_start"
          @update:model-value="
            (e) => {
              settings.hide_on_process_start = e
            }
          "
        />
      </div>
      <div v-if="getOS() != 'MacOS'" class="adjacent-input">
        <label for="native-decorations">
          <span class="label__title"> {{ $u3t('Native decorations') }} </span>
          <span class="label__description">
            {{ $u3t('Use system window frame (app restart required).') }}
          </span>
        </label>
        <Toggle
          id="native-decorations"
          :model-value="settings.native_decorations"
          :checked="settings.native_decorations"
          @update:model-value="
            (e) => {
              settings.native_decorations = e
            }
          "
        />
      </div>
      <div class="adjacent-input">
        <label for="opening-page">
          <span class="label__title"> {{ $u3t('Default landing page') }} </span>
          <span class="label__description">
            {{ $u3t('Change the page to which the launcher opens on.') }}
          </span>
        </label>
        <DropdownSelect
          id="opening-page"
          name="Opening page dropdown"
          :options="pageOptions"
          :display-name="translate"
          :default-value="settings.default_page"
          :model-value="settings.default_page"
          class="opening-page"
          @change="
            (e) => {
              settings.default_page = e.option
            }
          "
        />
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header"> {{ $u3t('Resource management') }} </span>
        </h3>
      </div>

      <div class="adjacent-input">
        <label for="max-downloads">
          <span class="label__title"> {{ $u3t('Maximum concurrent downloads') }} </span>
          <span class="label__description">
            {{
              $u3t(
                'The maximum amount of files the launcher can download at the same time. Set this to a lower value if you have a poor internet connection. (app restart required to take effect)',
              )
            }}
          </span>
        </label>
        <Slider
          id="max-downloads"
          v-model="settings.max_concurrent_downloads"
          :min="1"
          :max="10"
          :step="1"
        />
      </div>

      <div class="adjacent-input">
        <label for="max-writes">
          <span class="label__title"> {{ $u3t('Maximum concurrent writes') }} </span>
          <span class="label__description">
            {{
              $u3t(
                'The maximum amount of files the launcher can write to the disk at once. Set this to a lower value if you are frequently getting I/O errors. (app restart required to take effect)',
              )
            }}
          </span>
        </label>
        <Slider
          id="max-writes"
          v-model="settings.max_concurrent_writes"
          :min="1"
          :max="50"
          :step="1"
        />
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header"> {{ $u3t('Privacy') }} </span>
        </h3>
      </div>
      <div class="adjacent-input">
        <label for="opt-out-analytics">
          <span class="label__title">
            {{ tr('Не отправлять аналитику', 'Do not send analytics') }}
          </span>
          <span class="label__description">
            {{
              tr(
                'Если включено, аналитика лаунчера не отправляется. Каталоги и вход в аккаунт всё равно используют сеть.',
                'When enabled, launcher analytics are not sent. Catalogs and sign-in still use the network.',
              )
            }}
          </span>
        </label>
        <Toggle
          id="opt-out-analytics"
          :model-value="settings.telemetry"
          :checked="settings.telemetry"
          @update:model-value="
            (e) => {
              settings.telemetry = e
            }
          "
        />
      </div>
      <div class="adjacent-input">
        <label for="disable-discord-rpc">
          <span class="label__title">Discord RPC</span>
          <span class="label__description">
            {{
              $u3t(
                'Manages the legacy Discord Rich Presence integration, which uses the Modrinth Discord application. Disable it to hide launcher activity on your Discord profile. This does not disable any instance-specific Discord Rich Presence integrations, such as those added by mods. (app restart required to take effect)',
              )
            }}
          </span>
        </label>
        <Toggle
          id="disable-discord-rpc"
          v-model="settings.discord_rpc"
          :checked="settings.discord_rpc"
        />
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header"> {{ $u3t('Java settings') }} </span>
        </h3>
      </div>
      <template v-for="version in [21, 17, 8]" :key="version">
        <label :for="'java-selector-' + version">
          <span class="label__title"> {{ $u3t('Path to') }} {{ $u3t('Java') }} {{ version }}</span>
        </label>
        <JavaSelector
          :id="'java-selector-' + version"
          v-model="javaVersions[version]"
          :version="version"
          @update:model-value="updateJavaVersion"
        />
      </template>
      <hr class="card-divider" />
      <label for="java-args">
        <span class="label__title"> {{ $u3t('Java arguments') }} </span>
      </label>
      <input
        id="java-args"
        v-model="settings.launchArgs"
        autocomplete="off"
        type="text"
        class="installation-input"
        :placeholder="$u3t('Enter java arguments...')"
      />
      <label for="env-vars">
        <span class="label__title"> {{ $u3t('Environmental variables') }} </span>
      </label>
      <input
        id="env-vars"
        v-model="settings.envVars"
        autocomplete="off"
        type="text"
        class="installation-input"
        :placeholder="$u3t('Enter environmental variables...')"
      />
      <hr class="card-divider" />
      <div class="adjacent-input">
        <label for="max-memory">
          <span class="label__title"> {{ $u3t('Java memory') }} </span>
          <span class="label__description">
            {{ $u3t('The memory allocated to each instance when it is ran.') }}
          </span>
        </label>
        <Slider
          id="max-memory"
          v-model="settings.memory.maximum"
          :min="8"
          :max="maxMemory"
          :step="64"
          :unit="$u3t('mb')"
        />
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header"> {{ $u3t('Hooks') }} </span>
        </h3>
      </div>
      <div class="adjacent-input">
        <label for="pre-launch">
          <span class="label__title"> {{ $u3t('Pre launch') }} </span>
          <span class="label__description">
            {{ $u3t('Ran before the instance is launched.') }}
          </span>
        </label>
        <input
          id="pre-launch"
          v-model="settings.hooks.pre_launch"
          autocomplete="off"
          type="text"
          :placeholder="$u3t('Enter pre-launch command...')"
        />
      </div>
      <div class="adjacent-input">
        <label for="wrapper">
          <span class="label__title"> {{ $u3t('Wrapper') }} </span>
          <span class="label__description">
            {{ $u3t('Wrapper command for launching Minecraft.') }}
          </span>
        </label>
        <input
          id="wrapper"
          v-model="settings.hooks.wrapper"
          autocomplete="off"
          type="text"
          :placeholder="$u3t('Enter wrapper command...')"
        />
      </div>
      <div class="adjacent-input">
        <label for="post-exit">
          <span class="label__title"> {{ $u3t('Post exit') }} </span>
          <span class="label__description"> {{ $u3t('Ran after the game closes.') }} </span>
        </label>
        <input
          id="post-exit"
          v-model="settings.hooks.post_exit"
          autocomplete="off"
          type="text"
          :placeholder="$u3t('Enter post-exit command...')"
        />
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header"> {{ $u3t('Window size') }} </span>
        </h3>
      </div>
      <div class="adjacent-input">
        <label for="fullscreen">
          <span class="label__title"> {{ $u3t('Fullscreen') }} </span>
          <span class="label__description">
            {{ $u3t('Overwrites the options.txt file to start in full screen when launched.') }}
          </span>
        </label>
        <Toggle
          id="fullscreen"
          :model-value="settings.force_fullscreen"
          :checked="settings.force_fullscreen"
          @update:model-value="
            (e) => {
              settings.force_fullscreen = e
            }
          "
        />
      </div>
      <div class="adjacent-input">
        <label for="width">
          <span class="label__title"> {{ $u3t('Width') }} </span>
          <span class="label__description">
            {{ $u3t('The width of the game window when launched.') }}
          </span>
        </label>
        <input
          id="width"
          v-model="settings.game_resolution[0]"
          :disabled="settings.force_fullscreen"
          autocomplete="off"
          type="number"
          :placeholder="$u3t('Enter width...')"
        />
      </div>
      <div class="adjacent-input">
        <label for="height">
          <span class="label__title"> {{ $u3t('Height') }} </span>
          <span class="label__description">
            {{ $u3t('The height of the game window when launched.') }}
          </span>
        </label>
        <input
          id="height"
          v-model="settings.game_resolution[1]"
          :disabled="settings.force_fullscreen"
          autocomplete="off"
          type="number"
          class="input"
          :placeholder="$u3t('Enter height...')"
        />
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header"> {{ $u3t('About') }} </span>
        </h3>
      </div>
      <div>
        <label>
          <span class="label__title"> {{ $u3t('App version') }} </span>
          <span class="label__description">U3drinth (custom modrinth) v1.0</span>
        </label>
      </div>
    </Card>
  </div>
</template>

<style lang="scss" scoped>
.settings-page select {
  min-height: 44px;
  min-width: 140px;
  padding: 8px 12px;
  background: var(--color-button-bg);
  color: var(--color-base);
  border: 1px solid var(--theme-border, var(--color-secondary));
  border-radius: 8px;
  font: inherit;
}
.settings-page select:focus-visible {
  outline: 2px solid var(--color-brand);
  outline-offset: 3px;
}

.settings-page {
  margin: 1rem;
}

.installation-input {
  width: 100% !important;
  flex-grow: 1;
}

.theme-gallery-link {
  display: inline-flex;
  width: fit-content;
  margin: 0.15rem 0 1.1rem;
}
.theme-dropdown {
  text-transform: capitalize;
}

.card-divider {
  margin: 1rem 0;
}

.app-directory {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: var(--gap-sm);

  .iconified-input {
    flex-grow: 1;

    input {
      flex-basis: auto;
    }
  }
}
</style>
