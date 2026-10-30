<script setup>
import { ref, watch, onBeforeUnmount } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { open as chooseFile } from '@tauri-apps/api/dialog'
import { open } from '@tauri-apps/api/shell'
import { language } from '@/i18n/index.js'
const props = defineProps({
  profile: { type: String, default: '' },
  busy: Boolean,
  supported: Boolean,
  revision: { type: String, default: '' },
})
const emit = defineEmits(['action'])
const t = (ru, en) => (language.value === 'ru' ? ru : en)
const tab = ref('pvp'),
  search = ref(''),
  offset = ref(0),
  data = ref(null)
const loading = ref(false),
  error = ref(''),
  protect = ref(true)
let serial = 0,
  timer
async function load() {
  const sequence = ++serial
  if (props.busy) {
    loading.value = false
    return
  }
  loading.value = true
  error.value = ''
  data.value = null
  try {
    const result = await invoke('plugin:minecraft-rust|resource_catalog', {
      profile: props.profile || 'horizon',
      tab: tab.value,
      search: search.value,
      offset: offset.value,
    })
    if (sequence === serial) data.value = result
  } catch (e) {
    if (sequence === serial) error.value = String(e?.message ?? e)
  } finally {
    if (sequence === serial) loading.value = false
  }
}
watch(() => [props.profile, props.busy, props.revision, tab.value, offset.value], load, {
  immediate: true,
})
watch(search, () => {
  ++serial
  data.value = null
  clearTimeout(timer)
  timer = setTimeout(() => {
    if (offset.value) offset.value = 0
    else load()
  }, 350)
})
onBeforeUnmount(() => {
  ++serial
  clearTimeout(timer)
})
function select(value) {
  if (tab.value === value) return
  tab.value = value
  offset.value = 0
  search.value = ''
}
function action(action, extra = {}) {
  if (props.busy || !props.supported) return
  emit('action', { action, profile: props.profile || 'horizon', ...extra })
}
async function importArchive() {
  try {
    const path = await chooseFile({
      title: t('Выбери PvP.zip или Default.zip', 'Choose PvP.zip or Default.zip'),
      multiple: false,
      filters: [{ name: 'ZIP', extensions: ['zip'] }],
    })
    if (typeof path === 'string') action('pack_import', { pack_file: path })
  } catch (e) {
    error.value = String(e)
  }
}
async function source(entry) {
  if (!/^[a-zA-Z0-9_-]{1,100}$/.test(entry.project || '')) return
  try {
    await open(`https://modrinth.com/resourcepack/${entry.project}`)
  } catch (e) {
    error.value = String(e)
  }
}
const shortFile = (file) =>
  /^u3-(safe|original)-/.test(file) ? `${file.slice(0, 24)}...zip` : file
</script>
<template>
  <section class="packs-section" aria-labelledby="rust-packs-title">
    <div class="packs-heading">
      <h2 id="rust-packs-title">{{ t('Ресурс-паки Horizon', 'Horizon resource packs') }}</h2>
      <button class="btn" :disabled="busy || !supported" @click="importArchive">
        {{ t('Добавить архив', 'Add archive') }}
      </button>
    </div>
    <p>
      {{
        t(
          'Установка и включение в resourcepacks выбранного профиля. Перезапусти игру после изменения паков.',
          'Install and enable in the selected profile’s resourcepacks folder. Restart the game after changing packs.',
        )
      }}
    </p>
    <p class="packs-muted">
      {{
        t(
          'PvP и Обычные - паки из твоих архивов. Для трёх локальных паков один раз добавь PvP.zip. Остальные скачиваются с Modrinth. Миры и моды не изменяются.',
          'PvP and Regular contain packs from your archives. Import PvP.zip once for the three local packs. Others download from Modrinth. Worlds and mods are not changed.',
        )
      }}
    </p>
    <label class="packs-protect"
      ><input v-model="protect" type="checkbox" :disabled="busy" />{{
        t('Сохранить стандартный интерфейс', 'Keep the standard interface')
      }}</label
    >
    <p class="packs-muted">
      {{
        protect
          ? t(
              'Убирает из устанавливаемой копии замену GUI, шрифтов и шейдеров. Оригинал не меняется. GUI-паки в этом режиме не меняют меню.',
              'Removes GUI, font and shader replacements from the installed copy. The original stays intact. GUI packs will not restyle menus in this mode.',
            )
          : t(
              'Внимание: оригинальный GUI может быть несовместим с Horizon и ломать меню.',
              'Warning: original GUI assets may be incompatible with Horizon and break menus.',
            )
      }}
    </p>
    <div
      class="packs-tabs"
      role="group"
      :aria-label="t('Каталог ресурс-паков', 'Resource pack catalog')"
    >
      <button
        v-for="item in [
          ['pvp', 'PvP'],
          ['default', t('Обычные', 'Regular')],
          ['modrinth', 'Modrinth'],
        ]"
        :key="item[0]"
        class="btn"
        :aria-pressed="tab === item[0]"
        :disabled="busy"
        @click="select(item[0])"
      >
        {{ item[1] }}
      </button>
    </div>
    <div class="packs-heading">
      <input
        v-model="search"
        type="search"
        :disabled="busy"
        :placeholder="t('Поиск ресурс-паков', 'Search resource packs')"
        :aria-label="t('Поиск ресурс-паков', 'Search resource packs')"
      />
      <button class="btn" :disabled="busy || loading" @click="load">
        {{ t('Обновить', 'Refresh') }}
      </button>
    </div>
    <p v-if="tab === 'modrinth'" class="packs-muted">
      {{
        t(
          'Весь каталог Modrinth с отметкой Minecraft 26.3, с поиском и страницами. Метка версии не гарантирует поддержку всех функций Horizon: OptiFine, CIT и CEM могут не работать.',
          'The full Modrinth catalog tagged Minecraft 26.3, with search and pagination. A version tag does not guarantee Horizon feature support: OptiFine, CIT and CEM may not work.',
        )
      }}
    </p>
    <p v-if="loading" role="status">
      {{ t('Загрузка ресурс-паков...', 'Loading resource packs...') }}
    </p>
    <div v-if="error" class="packs-error" role="alert">
      <p>
        {{
          t(
            'Не удалось загрузить список. Проверь соединение и нажми «Обновить».',
            'Could not load packs. Check your connection and refresh.',
          )
        }}
      </p>
      <details>
        <summary>{{ t('Подробности', 'Details') }}</summary>
        {{ error }}
      </details>
    </div>
    <template v-if="data">
      <p v-if="!data.entries.length">
        {{ t('Ресурс-паки не найдены.', 'No resource packs found.') }}
      </p>
      <div class="packs-grid">
        <article v-for="entry in data.entries" :key="entry.id" class="pack-card">
          <h3>{{ entry.name }}</h3>
          <p v-if="entry.author || entry.version" class="packs-muted">
            {{ entry.author || entry.version }}
          </p>
          <p v-if="entry.summary">{{ entry.summary }}</p>
          <p v-if="entry.gui && protect" class="packs-muted">
            {{
              t(
                'Содержит замену интерфейса. Защита оставит стандартный GUI.',
                'Includes interface replacements. Protection keeps the standard GUI.',
              )
            }}
          </p>
          <p v-if="entry.optifine" class="packs-warning">
            {{
              t(
                'Есть функции OptiFine. Полная работа в Horizon не гарантируется.',
                'Includes OptiFine features. Full Horizon support is not guaranteed.',
              )
            }}
          </p>
          <p v-if="!entry.available" class="packs-warning">
            {{
              t(
                'Формат этого архива несовместим с 26.3. Установка заблокирована.',
                'This archive’s format is incompatible with 26.3. Installation blocked.',
              )
            }}
          </p>
          <p v-else-if="entry.local" class="packs-muted">
            {{
              t(
                'Локальный пак: сначала добавь PvP.zip кнопкой выше.',
                'Local pack: first add PvP.zip using the button above.',
              )
            }}
          </p>
          <div class="packs-actions">
            <button
              class="btn btn-primary"
              :disabled="busy || !supported || !entry.available || entry.local"
              @click="action('pack_install', { project: entry.id, protect_gui: protect })"
            >
              {{ t('Установить', 'Install') }}
            </button>
            <button v-if="entry.project" class="btn" :disabled="busy" @click="source(entry)">
              Modrinth
            </button>
          </div>
        </article>
      </div>
      <div v-if="tab === 'modrinth'" class="packs-pages">
        <button
          class="btn"
          :disabled="busy || offset === 0"
          @click="offset = Math.max(0, offset - data.limit)"
        >
          {{ t('Назад', 'Previous') }}
        </button>
        <span
          >{{ Math.floor(offset / data.limit) + 1 }} /
          {{ Math.max(1, Math.ceil(data.total / data.limit)) }}</span
        >
        <button
          class="btn"
          :disabled="busy || offset + data.limit >= data.total"
          @click="offset += data.limit"
        >
          {{ t('Далее', 'Next') }}
        </button>
      </div>
      <details class="packs-installed">
        <summary>
          {{ t('Установленные ресурс-паки', 'Installed resource packs') }} ({{
            data.installed.length
          }})
        </summary>
        <p v-if="!data.installed.length">
          {{ t('В выбранном профиле пока нет паков.', 'No packs in the selected profile yet.') }}
        </p>
        <article v-for="pack in data.installed" :key="pack.file" class="pack-row">
          <div>
            <strong>{{ pack.name || shortFile(pack.file) }}</strong>
            <p class="packs-muted">{{ pack.description }}</p>
          </div>
          <button
            class="btn"
            :disabled="busy || !supported"
            @click="action('pack_toggle', { project: pack.file })"
          >
            {{ pack.enabled ? t('Выключить', 'Disable') : t('Включить', 'Enable') }}
          </button>
        </article>
        <button
          class="btn"
          :disabled="busy || !supported || !data.installed.some((p) => p.enabled)"
          @click="action('pack_repair')"
        >
          {{ t('Защитить интерфейс у включённых ZIP-паков', 'Protect GUI in enabled ZIP packs') }}
        </button>
        <p class="packs-muted">
          {{
            t(
              'Создаёт защищённые копии, выключает оригиналы. Перед изменением сохраняется options.txt в u3-pack-backups профиля. Папки-паки сначала выключи в игре.',
              'Creates protected copies and disables originals. Backs up options.txt to the profile’s u3-pack-backups folder. Disable folder packs in the game first.',
            )
          }}
        </p>
      </details>
    </template>
  </section>
</template>
<style scoped>
.packs-section {
  margin: 2rem 0;
  color: var(--color-base);
}
.packs-heading,
.packs-actions,
.packs-tabs,
.packs-pages,
.pack-row {
  display: flex;
  gap: 0.75rem;
  align-items: center;
  flex-wrap: wrap;
}
.packs-heading {
  justify-content: space-between;
  margin: 1rem 0;
}
.packs-heading h2 {
  margin: 0;
}
.packs-heading input {
  border: 1px solid var(--color-secondary) !important;
  border-radius: 0.5rem;
  background: var(--color-bg);
  color: var(--color-base);
  padding: 0.75rem;
  min-width: 0;
  width: min(100%, 22rem);
}
.packs-protect {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  margin-top: 1rem;
}
.packs-protect input {
  flex-shrink: 0;
}
.packs-muted {
  color: var(--color-secondary);
  font-size: 0.9rem;
}
.packs-warning {
  color: var(--color-base);
  border-left: 3px solid #d9a441;
  padding-left: 0.75rem;
  font-size: 0.9rem;
}
.packs-tabs button[aria-pressed='true'] {
  outline: 2px solid var(--color-brand);
  outline-offset: 2px;
}
.packs-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 1rem;
}
.pack-card {
  background: var(--color-raised-bg);
  border: 1px solid var(--color-button-bg);
  border-radius: 0.8rem;
  padding: 1.25rem;
  display: flex;
  flex-direction: column;
  overflow-wrap: anywhere;
}
.pack-card h3 {
  margin: 0;
}
.packs-actions {
  margin-top: auto;
  padding-top: 0.75rem;
}
.packs-error {
  border-left: 3px solid #ff416c;
  background: var(--color-raised-bg);
  padding: 1rem;
  overflow-wrap: anywhere;
}
.packs-pages {
  justify-content: center;
  margin-top: 1rem;
}
.packs-installed {
  background: var(--color-raised-bg);
  border-radius: 0.8rem;
  padding: 1rem;
  margin-top: 1.5rem;
}
.packs-installed summary {
  cursor: pointer;
}
.pack-row {
  justify-content: space-between;
  padding: 0.75rem 0;
  border-bottom: 1px solid var(--color-button-bg);
}
.pack-row > div {
  min-width: 0;
  flex: 1;
  overflow-wrap: anywhere;
}
.packs-installed > button {
  margin-top: 1rem;
  white-space: normal;
  height: auto;
  min-height: 2.75rem;
}
@media (max-width: 700px) {
  .packs-grid {
    grid-template-columns: minmax(0, 1fr);
  }
  .packs-heading {
    align-items: stretch;
  }
  .packs-heading input {
    width: 100%;
  }
  .packs-tabs {
    gap: 0.5rem;
  }
  .pack-card {
    padding: 1rem;
  }
}
</style>
