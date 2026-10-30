<script setup>
import { ref, computed, onMounted, onBeforeUnmount, watch, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { open as choose } from '@tauri-apps/api/dialog'
import { open as openLink } from '@tauri-apps/api/shell'
import { updateRustSession } from '@/minecraft-rust/state.js'
import { language } from '@/i18n/index.js'
import MinecraftRustIcon from '@/components/brand/MinecraftRustIcon.vue'
const t = (ru, en) => (language.value === 'ru' ? ru : en)
const state = ref({
  supported: true,
  runtime: false,
  prepared: false,
  running: false,
  stage: 'idle',
  accounts: {},
  instances: [],
  versions: {},
  logs: [],
})
const loading = ref(true),
  sending = ref(false),
  error = ref(''),
  profile = ref('horizon'),
  account = ref(''),
  nickname = ref('U3_Player'),
  name = ref(''),
  mode = ref('offline'),
  confirm = ref(false)
let poll,
  disposed = false,
  polling = false
const busy = computed(() => sending.value || state.value.running || loading.value)
const accounts = computed(() => state.value.accounts?.accounts ?? [])
const profiles = computed(() => (state.value.instances ?? []).filter((p) => !p.java))
const installed = computed(() => state.value.versions?.installed?.length > 0)
const canPlay = computed(
  () =>
    !busy.value &&
    state.value.prepared &&
    !!profile.value &&
    state.value.accounts?.owner &&
    (mode.value === 'offline' ? /^[A-Za-z0-9_]{3,16}$/.test(nickname.value) : !!account.value),
)
const progress = computed(() =>
  state.value.total
    ? Math.min(100, Math.round((state.value.completed / state.value.total) * 100))
    : null,
)
const labels = computed(() => ({
  refresh: t('Проверка', 'Checking'),
  install: t('Установка', 'Installing'),
  download_runtime: t('Загрузка Horizon', 'Downloading Horizon'),
  download_game: t('Загрузка игры', 'Downloading game'),
  prepare_local: t('Подготовка файлов', 'Preparing files'),
  prepare_download: t('Загрузка и подготовка Minecraft', 'Downloading and preparing Minecraft'),
  sign_in: t('Вход Microsoft', 'Microsoft sign-in'),
  create: t('Создание профиля', 'Creating profile'),
  play_online: t('Запуск', 'Starting'),
  play_offline: t('Запуск', 'Starting'),
  playing: t('Игровая сессия', 'Game session'),
  done: t('Готово', 'Done'),
  error: t('Ошибка', 'Error'),
  idle: t('Не установлено', 'Not installed'),
}))
const confirmPanel = ref()
let previousFocus
watch(confirm, async (value) => {
  if (value) {
    previousFocus = document.activeElement
    await nextTick()
    confirmPanel.value?.focus()
  } else previousFocus?.focus?.()
})
function dialogKey(event) {
  if (event.key === 'Escape') {
    event.preventDefault()
    confirm.value = false
  }
  if (event.key !== 'Tab') return
  const buttons = [...confirmPanel.value.querySelectorAll('button:not([disabled])')]
  const first = buttons[0],
    last = buttons.at(-1)
  if (
    event.shiftKey &&
    (document.activeElement === first || document.activeElement === confirmPanel.value)
  ) {
    event.preventDefault()
    last?.focus()
  } else if (!event.shiftKey && document.activeElement === last) {
    event.preventDefault()
    first?.focus()
  }
}
async function createProfile() {
  await action('create', { name: name.value })
  if (!error.value) name.value = ''
}
function downloadConfirmed() {
  confirm.value = false
  return action('prepare_download')
}
function adopt(next) {
  state.value = next
  updateRustSession(next)
  if (!profiles.value.some((p) => p.id === profile.value))
    profile.value = profiles.value[0]?.id ?? ''
  if (!accounts.value.some((a) => a.uuid === account.value))
    account.value = accounts.value.find((a) => a.selected)?.uuid ?? accounts.value[0]?.uuid ?? ''
}
async function getStatus() {
  if (polling || disposed) return
  polling = true
  try {
    const next = await invoke('plugin:minecraft-rust|status')
    if (!disposed) adopt(next)
  } catch (e) {
    if (!disposed) error.value = String(e?.message ?? e)
  } finally {
    polling = false
  }
}
async function action(action, extra = {}) {
  if (busy.value || !state.value.supported) return
  sending.value = true
  error.value = ''
  try {
    adopt(await invoke('plugin:minecraft-rust|start', { request: { action, ...extra } }))
  } catch (e) {
    error.value = String(e?.message ?? e)
  } finally {
    sending.value = false
  }
}
async function localPrepare() {
  const folder = await choose({
    directory: true,
    multiple: false,
    title: t(
      'Папка Minecraft с установленной игрой',
      'Minecraft folder containing the installed game',
    ),
  }).catch((e) => {
    error.value = String(e)
    return null
  })
  if (typeof folder === 'string') await action('prepare_local', { minecraft: folder })
}
function play() {
  return action(mode.value === 'offline' ? 'play_offline' : 'play_online', {
    profile: profile.value,
    name: nickname.value,
    account: account.value,
  })
}
async function cancelSignIn() {
  try {
    await invoke('plugin:minecraft-rust|cancel_sign_in')
  } catch (e) {
    error.value = String(e?.message ?? e)
  }
}
async function browserSignIn() {
  const url = state.value.auth_url
  if (!url) return
  try {
    const u = new URL(url)
    if (
      u.protocol === 'https:' &&
      ['microsoft.com', 'www.microsoft.com', 'login.microsoftonline.com'].includes(u.hostname) &&
      !u.username &&
      !u.password &&
      !u.port
    )
      await openLink(url)
  } catch (e) {
    error.value = String(e)
  }
}
onMounted(async () => {
  await getStatus()
  loading.value = false
  if (!state.value.running && state.value.supported) await action('refresh')
  poll = setInterval(getStatus, 1200)
})
onBeforeUnmount(() => {
  disposed = true
  clearInterval(poll)
})
</script>

<template>
  <main class="minecraft-rust">
    <header class="rust-header">
      <div class="rust-title">
        <MinecraftRustIcon />
        <div>
          <h1>Minecraft Rust</h1>
          <p>Horizon · {{ t('Отдельные профили', 'Separate profiles') }}</p>
        </div>
      </div>
      <span class="rust-status" role="status">{{
        state.running
          ? labels[state.stage] ?? state.stage
          : state.prepared
            ? t('Готов к запуску', 'Ready to play')
            : state.runtime
              ? t('Нужна подготовка', 'Preparation needed')
              : t('Не установлен', 'Not installed')
      }}</span>
    </header>
    <p class="rust-intro">
      {{
        t(
          'Неофициальный Minecraft на Rust. Миры и настройки хранятся отдельно от обычных сборок.',
          'Unofficial Minecraft on Rust. Worlds and settings stay separate from your Java modpacks.',
        )
      }}
    </p>
    <div v-if="!state.supported" class="rust-message" role="alert">
      {{
        t(
          'Эта версия интеграции работает только на Windows x64.',
          'This integration currently supports Windows x64 only.',
        )
      }}
    </div>
    <div v-if="error || state.error" class="rust-message rust-error" role="alert">
      {{ error || state.error }}
    </div>
    <section v-if="state.running" class="rust-card rust-operation" aria-live="polite">
      <div>
        <strong>{{ labels[state.stage] ?? state.stage }}</strong
        ><span v-if="progress !== null">{{ progress }}%</span>
      </div>
      <progress
        v-if="progress !== null"
        :value="progress"
        max="100"
        :aria-label="labels[state.stage]"
      />
      <p v-else>
        {{
          state.stage === 'playing'
            ? t('Закрой игру, чтобы завершить сессию.', 'Close the game to end this session.')
            : t(
                'Дождись завершения. Можно перейти на другую вкладку.',
                'Please wait. You can switch to another tab.',
              )
        }}
      </p>
      <div v-if="state.auth_code" class="rust-device">
        <p>
          {{
            t(
              'Войди в Microsoft в браузере и введи этот код:',
              'Sign in to Microsoft in your browser and enter this code:',
            )
          }}
        </p>
        <code>{{ state.auth_code }}</code
        ><button class="btn btn-primary" @click="browserSignIn">
          {{ t('Открыть Microsoft', 'Open Microsoft') }}
        </button>
        <p>{{ state.auth_url }}</p>
      </div>
    </section>
    <button
      v-if="state.running && state.action === 'sign_in'"
      class="btn rust-cancel"
      @click="cancelSignIn"
    >
      {{ t('Отменить вход', 'Cancel sign-in') }}
    </button>
    <div class="rust-grid">
      <section class="rust-card rust-setup">
        <h2>{{ t('Установка', 'Installation') }}</h2>
        <ol class="rust-steps">
          <li>
            <span class="step-number">1</span>
            <div>
              <h3>Horizon</h3>
              <p>
                {{
                  state.runtime
                    ? t(
                        'Компонент установлен. Можно загрузить свежую сборку игры.',
                        'Runtime installed. You can download the latest game build.',
                      )
                    : t(
                        'Загрузи официальный компонент и игру. Нужен интернет.',
                        'Download the official runtime and game. Internet required.',
                      )
                }}
              </p>
              <button class="btn" :disabled="busy || !state.supported" @click="action('install')">
                {{
                  state.runtime
                    ? t('Обновить игру', 'Update game')
                    : t('Установить Horizon', 'Install Horizon')
                }}
              </button>
            </div>
          </li>
          <li>
            <span class="step-number">2</span>
            <div>
              <h3>{{ t('Файлы Minecraft', 'Minecraft files') }}</h3>
              <p>
                {{
                  t(
                    'Нужна законно полученная Minecraft Java 26.3. Выбери установленную игру или загрузи файлы после входа Microsoft. Java для подготовки подберёт Horizon.',
                    'Requires a legally obtained Minecraft Java 26.3. Choose an installed game or download its files after Microsoft sign-in. Horizon handles Java for preparation.',
                  )
                }}
              </p>
              <div class="rust-buttons">
                <button class="btn" :disabled="busy || !installed" @click="localPrepare">
                  {{ t('Выбрать папку', 'Choose folder') }}</button
                ><button
                  class="btn"
                  :disabled="busy || !installed || !state.accounts?.owner"
                  @click="confirm = true"
                >
                  {{ t('Скачать и подготовить', 'Download and prepare') }}
                </button>
              </div>
              <span v-if="state.prepared" class="rust-ok">{{
                t('Файлы подготовлены', 'Files prepared')
              }}</span>
            </div>
          </li>
        </ol>
        <p class="rust-small">
          {{
            t(
              'Fabric, Forge и NeoForge здесь не работают. Нужен совместимый с Horizon контент.',
              'Fabric, Forge and NeoForge mods do not run here. Use Horizon-compatible content.',
            )
          }}
        </p>
      </section>
      <section class="rust-card rust-launch">
        <h2>{{ t('Запуск', 'Play') }}</h2>
        <label for="rust-profile">{{ t('Профиль Horizon', 'Horizon profile') }}</label>
        <select id="rust-profile" v-model="profile" :disabled="busy || !profiles.length">
          <option v-if="!profiles.length" value="">
            {{ t('Сначала установи Horizon', 'Install Horizon first') }}
          </option>
          <option v-for="p in profiles" :key="p.id" :value="p.id">{{ p.name }}</option>
        </select>
        <form class="rust-create" @submit.prevent="createProfile">
          <label class="sr-only" for="rust-name">{{
            t('Название нового профиля', 'New profile name')
          }}</label
          ><input
            id="rust-name"
            v-model="name"
            maxlength="48"
            :disabled="busy || !state.runtime"
            :placeholder="t('Новый профиль', 'New profile')"
          /><button class="btn" :disabled="busy || !state.runtime || !name.trim()">
            {{ t('Создать', 'Create') }}
          </button>
        </form>
        <fieldset class="rust-mode">
          <legend>{{ t('Режим', 'Mode') }}</legend>
          <label
            ><input v-model="mode" type="radio" value="offline" :disabled="busy" />{{
              t('Офлайн', 'Offline')
            }}</label
          ><label
            ><input v-model="mode" type="radio" value="online" :disabled="busy" />{{
              t('С лицензией', 'Licensed')
            }}</label
          >
        </fieldset>
        <template v-if="mode === 'offline'"
          ><label for="rust-nickname">{{ t('Ник', 'Player name') }}</label
          ><input
            id="rust-nickname"
            v-model="nickname"
            maxlength="16"
            :disabled="busy"
            autocomplete="off"
          />
          <p class="rust-small">
            {{
              t(
                'Одиночная игра, LAN и офлайн-серверы. Horizon требует добавленный аккаунт с лицензией и для этого режима.',
                'Single player, LAN and offline-mode servers. Horizon requires an added licensed account for this mode too.',
              )
            }}
          </p></template
        >
        <template v-else
          ><label for="rust-account">{{ t('Аккаунт Microsoft', 'Microsoft account') }}</label
          ><select id="rust-account" v-model="account" :disabled="busy || !accounts.length">
            <option v-if="!accounts.length" value="">
              {{ t('Добавь аккаунт', 'Add an account') }}
            </option>
            <option v-for="a in accounts" :key="a.uuid" :value="a.uuid">{{ a.name }}</option>
          </select>
          <p class="rust-small">
            {{
              t(
                'Проверка лицензии и доступ к серверам выполняются Horizon.',
                'Horizon handles ownership checks and server authentication.',
              )
            }}
          </p></template
        >
        <button
          class="btn rust-signin"
          :disabled="busy || !state.runtime || state.accounts?.sign_in_configured === false"
          @click="action('sign_in')"
        >
          {{ t('Добавить аккаунт Microsoft', 'Add Microsoft account') }}
        </button>
        <p
          v-if="state.runtime && state.accounts?.sign_in_configured === false"
          class="rust-small rust-error-text"
        >
          {{
            t(
              'В этой сборке Horizon не настроен вход Microsoft. Запуск недоступен.',
              'Microsoft sign-in is not configured in this Horizon build. Play is unavailable.',
            )
          }}
        </p>
        <button class="btn btn-primary rust-play" :disabled="!canPlay" @click="play">
          {{ t('Играть', 'Play') }}
        </button>
        <p v-if="installed" class="rust-small rust-version">{{ state.versions.selected }}</p>
      </section>
    </div>
    <footer class="rust-footer">
      <span>{{
        t(
          'Аккаунты Horizon добавляются отдельно. Пароли не вводятся в U3drinth.',
          'Add Horizon accounts separately. Passwords are never entered in U3drinth.',
        )
      }}</span
      ><button class="btn" :disabled="busy || !state.supported" @click="action('refresh')">
        {{ t('Проверить', 'Refresh') }}
      </button>
    </footer>
    <details v-if="state.logs?.length" class="rust-card rust-log">
      <summary>{{ t('Журнал операции', 'Operation log') }}</summary>
      <pre>{{ state.logs.join('\n') }}</pre>
    </details>
    <div v-if="confirm" class="rust-overlay" @click.self="confirm = false">
      <section
        ref="confirmPanel"
        class="rust-card rust-confirm"
        tabindex="-1"
        role="dialog"
        aria-modal="true"
        aria-labelledby="rust-confirm-title"
        @keydown="dialogKey"
      >
        <h2 id="rust-confirm-title">
          {{ t('Скачать файлы Minecraft?', 'Download Minecraft files?') }}
        </h2>
        <p>
          {{
            t(
              'Horizon проверит лицензию выбранного аккаунта и загрузит файлы с серверов Mojang. Нужен интернет и свободное место. Старые сборки не изменятся.',
              'Horizon will check game ownership and download files from Mojang. Internet and free disk space are required. Existing Java modpacks stay unchanged.',
            )
          }}
        </p>
        <div class="rust-buttons">
          <button class="btn" autofocus @click="confirm = false">{{ t('Отмена', 'Cancel') }}</button
          ><button class="btn btn-primary" @click="downloadConfirmed">
            {{ t('Скачать', 'Download') }}
          </button>
        </div>
      </section>
    </div>
  </main>
</template>

<style scoped>
.minecraft-rust {
  max-width: 1180px;
  margin: 0 auto;
  padding: 32px 24px;
  color: var(--color-base);
}
.rust-header,
.rust-title,
.rust-buttons,
.rust-footer {
  display: flex;
  align-items: center;
  gap: 12px;
}
.rust-header {
  justify-content: space-between;
  flex-wrap: wrap;
}
.rust-title > svg {
  width: 44px;
  height: 44px;
  color: var(--color-brand);
  flex-shrink: 0;
}
h1 {
  font-size: 32px;
  line-height: 1.2;
  margin: 0 0 8px;
}
h2 {
  font-size: 22px;
  margin: 0 0 24px;
}
h3 {
  font-size: 17px;
  margin: 0 0 8px;
}
p {
  color: var(--color-secondary);
  line-height: 1.5;
  margin: 0 0 16px;
}
.rust-title p {
  margin: 0;
}
.rust-intro {
  margin: 24px 0;
  max-width: 760px;
}
.rust-status {
  border: 1px solid var(--color-divider);
  background: var(--color-raised-bg);
  padding: 8px 12px;
  border-radius: 8px;
  font-size: 14px;
}
.rust-cancel {
  margin-bottom: 16px;
}
.rust-grid {
  display: grid;
  grid-template-columns: 1.15fr 1fr;
  gap: 16px;
  align-items: start;
}
.rust-card {
  background: var(--color-raised-bg);
  border: 1px solid var(--color-divider);
  padding: 24px;
  border-radius: 12px;
  min-width: 0;
}
.rust-steps {
  padding: 0;
  margin: 0;
  list-style: none;
}
.rust-steps li {
  display: flex;
  gap: 16px;
  padding-bottom: 24px;
}
.rust-steps li + li {
  padding-top: 24px;
  border-top: 1px solid var(--color-divider);
}
.step-number {
  display: flex;
  justify-content: center;
  align-items: center;
  width: 28px;
  height: 28px;
  flex: 0 0 28px;
  border-radius: 8px;
  background: var(--color-button-bg);
  color: var(--color-base);
  font-weight: 700;
}
.rust-small {
  font-size: 14px;
  margin-top: 12px;
  margin-bottom: 0;
}
.rust-ok {
  display: block;
  color: var(--color-brand);
  font-size: 14px;
  margin-top: 12px;
}
.rust-launch > label {
  display: block;
  font-size: 14px;
  color: var(--color-secondary);
  margin: 0 0 8px;
}
select,
input:not([type='radio']) {
  width: 100%;
  min-height: 44px;
  background: var(--color-bg);
  color: var(--color-base);
  border: 1px solid var(--color-divider);
  border-radius: 8px;
  padding: 10px 12px;
  box-sizing: border-box;
  font: inherit;
  min-width: 0;
}
.rust-create {
  display: flex;
  gap: 8px;
  margin: 12px 0 24px;
}
.rust-create input {
  flex: 1;
}
.rust-mode {
  border: 0;
  padding: 0;
  margin: 0 0 20px;
  display: flex;
  gap: 20px;
  flex-wrap: wrap;
}
.rust-mode legend {
  font-size: 14px;
  color: var(--color-secondary);
  margin-bottom: 12px;
}
.rust-mode label {
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: 44px;
  cursor: pointer;
}
input[type='radio'] {
  width: 18px;
  height: 18px;
  accent-color: var(--color-brand);
}
.btn {
  min-height: 44px;
  white-space: normal;
  text-align: center;
}
.rust-buttons {
  flex-wrap: wrap;
}
.rust-play {
  width: 100%;
  margin-top: 20px;
}
.rust-signin {
  margin-top: 16px;
  width: 100%;
}
.rust-version {
  text-align: center;
  overflow-wrap: anywhere;
}
.rust-footer {
  margin: 24px 0;
  justify-content: space-between;
  color: var(--color-secondary);
  font-size: 14px;
}
.rust-footer > span {
  max-width: 650px;
}
.rust-footer button {
  flex-shrink: 0;
}
.rust-message {
  padding: 16px;
  border: 1px solid var(--color-divider);
  background: var(--color-raised-bg);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow-wrap: anywhere;
}
.rust-error {
  border-left: 3px solid var(--color-red);
}
.rust-error-text {
  color: var(--color-red);
}
.rust-operation {
  margin-bottom: 16px;
}
.rust-operation > div:first-child {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}
.rust-operation p {
  margin-top: 12px;
  margin-bottom: 0;
}
progress {
  width: 100%;
  height: 8px;
  accent-color: var(--color-brand);
  margin-top: 16px;
}
.rust-device {
  padding-top: 16px;
}
.rust-device code {
  display: inline-block;
  font-size: 24px;
  letter-spacing: 3px;
  padding: 12px;
  background: var(--color-bg);
  border-radius: 8px;
  margin: 12px 16px 12px 0;
}
.rust-device p {
  overflow-wrap: anywhere;
}
.rust-log summary {
  cursor: pointer;
  min-height: 24px;
}
.rust-log pre {
  max-height: 280px;
  overflow: auto;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  font-size: 13px;
  line-height: 1.5;
  padding: 16px;
  background: var(--color-bg);
  border-radius: 8px;
}
.rust-overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
  background: #0009;
  display: grid;
  place-items: center;
  padding: 24px;
}
.rust-confirm {
  width: min(480px, 100%);
  box-sizing: border-box;
}
.rust-confirm .rust-buttons {
  justify-content: flex-end;
}
.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}
:focus-visible {
  outline: 2px solid var(--color-brand);
  outline-offset: 3px;
}
@media (max-width: 850px) {
  .rust-grid {
    grid-template-columns: 1fr;
  }
}
@media (max-width: 520px) {
  .minecraft-rust {
    padding: 24px 16px;
  }
  h1 {
    font-size: 28px;
  }
  .rust-title > svg {
    width: 32px;
    height: 32px;
  }
  .rust-card {
    padding: 16px;
  }
  .rust-steps li {
    gap: 12px;
  }
  .rust-buttons {
    align-items: stretch;
  }
  .rust-buttons > button {
    width: 100%;
  }
  .rust-footer {
    align-items: stretch;
    flex-direction: column;
  }
  .rust-create {
    flex-wrap: wrap;
  }
  .rust-create input {
    flex-basis: 100%;
  }
  .rust-create button {
    width: 100%;
  }
  .rust-mode {
    gap: 12px;
  }
}
</style>
