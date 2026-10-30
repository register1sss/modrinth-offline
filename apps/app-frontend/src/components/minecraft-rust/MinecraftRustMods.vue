<script setup>
import { ref, watch, computed, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { open } from '@tauri-apps/api/shell'
import { language } from '@/i18n/index.js'
import { installRequest, modInstalled, modSummary } from '@/minecraft-rust/mods.js'
const props = defineProps({ profile: String, busy: Boolean, supported: Boolean, revision: String })
const emit = defineEmits(['install'])
const t = (ru, en) => (language.value === 'ru' ? ru : en)
const catalog = ref(null),
  loading = ref(false),
  error = ref(''),
  chosen = ref(null),
  consent = ref(false),
  dialog = ref(null),
  search = ref('')
let serial = 0,
  previousFocus
async function load() {
  if (props.busy) return
  const request = ++serial
  loading.value = true
  error.value = ''
  try {
    const result = await invoke('plugin:minecraft-rust|catalog', {
      profile: props.profile || 'horizon',
    })
    if (request === serial) catalog.value = result
  } catch (e) {
    if (request === serial) {
      catalog.value = null
      error.value = String(e?.message ?? e)
    }
  } finally {
    if (request === serial) loading.value = false
  }
}
watch(() => [props.profile, props.busy, props.revision], load, { immediate: true })
const groups = computed(() => [
  { title: t('Моды U3', 'U3 mods'), entries: catalog.value?.own ?? [] },
  {
    title: t('Моды других авторов', 'Community mods'),
    entries: (catalog.value?.community ?? []).filter((e) =>
      `${e.name} ${(e.authors ?? []).join(' ')}`.toLowerCase().includes(search.value.toLowerCase()),
    ),
  },
])
watch(chosen, async (value) => {
  consent.value = false
  if (value) {
    previousFocus = document.activeElement
    await nextTick()
    dialog.value?.focus()
  } else previousFocus?.focus?.()
})
watch(
  () => props.profile,
  () => {
    chosen.value = null
  },
)
function key(event) {
  if (event.key === 'Escape') {
    event.preventDefault()
    chosen.value = null
  }
  if (event.key !== 'Tab') return
  const nodes = [...dialog.value.querySelectorAll('button:not([disabled]), input, a[href]')]
  const first = nodes[0],
    last = nodes.at(-1)
  if (event.shiftKey && [first, dialog.value].includes(document.activeElement)) {
    event.preventDefault()
    last?.focus()
  } else if (!event.shiftKey && [last, dialog.value].includes(document.activeElement)) {
    event.preventDefault()
    first?.focus()
  }
}
function install() {
  const request = installRequest(chosen.value, catalog.value, props.profile, consent.value)
  if (!request || props.busy) return
  chosen.value = null
  emit('install', request)
}
async function source(entry) {
  try {
    const url = new URL(entry.source)
    if (
      url.protocol !== 'https:' ||
      url.hostname !== 'github.com' ||
      url.username ||
      url.password ||
      url.port
    )
      return
    await open(url.href)
  } catch (e) {
    error.value = String(e)
  }
}
</script>
<template>
  <section class="mods-section" aria-labelledby="rust-mods-title">
    <div class="mods-heading">
      <h2 id="rust-mods-title">{{ t('Моды Horizon', 'Horizon mods') }}</h2>
      <button class="btn" :disabled="busy || loading" @click="load">
        {{ t('Обновить список', 'Refresh list') }}
      </button>
    </div>
    <p class="mods-note">
      {{
        t(
          'Java-моды здесь не работают. Нативные моды собираются вместе с игрой.',
          'Java mods do not work here. Native mods are compiled into the game.',
        )
      }}
    </p>
    <p v-if="catalog" class="mods-note">
      {{ t('Профиль', 'Profile') }}: <strong>{{ catalog.target_name }}</strong
      ><span v-if="!profile || profile === 'horizon'"
        >. {{ t('Отдельные миры для модов.', 'Separate worlds for mods.') }}</span
      >
    </p>
    <p v-if="loading" role="status">{{ t('Загрузка списка...', 'Loading mods...') }}</p>
    <p v-if="error" class="mods-error" role="alert">
      {{
        t(
          'Не удалось загрузить каталог. Попробуй ещё раз.',
          'Could not load the catalog. Please retry.',
        )
      }}
      <details>
        <summary>{{ t('Подробности', 'Details') }}</summary>
        {{ error }}
      </details>
    </p>
    <template v-if="catalog">
      <div v-for="(group, i) in groups" :key="i" class="mod-group">
        <div class="mods-heading">
          <h3>{{ group.title }}</h3>
          <input
            v-if="i === 1"
            v-model="search"
            type="search"
            :aria-label="t('Поиск модов', 'Search mods')"
            :placeholder="t('Поиск модов', 'Search mods')"
          />
        </div>
        <div class="mod-grid">
          <article v-for="entry in group.entries" :key="entry.id" class="mod-card">
            <div class="mod-card-heading">
              <h4>{{ entry.name }}</h4>
              <span>{{ entry.version }}</span>
            </div>
            <p class="mod-author">{{ (entry.authors ?? []).join(', ') }}</p>
            <p v-if="entry.id === 'u3_edit'">
              {{
                t(
                  'Заливка, замена блоков и стены. До 32768 блоков. Без отмены.',
                  'Fill, replace blocks and build walls. Up to 32768 blocks. No undo.',
                )
              }}
            </p>
            <p v-else class="mod-summary">{{ modSummary(entry, language) }}</p>
            <details v-if="entry.id === 'u3_edit'">
              <summary>{{ t('Как пользоваться', 'How to use') }}</summary>
              <p>
                {{
                  t(
                    'Сделай копию мира. Нужны читы или права оператора.',
                    'Back up your world. Requires cheats or operator rights.',
                  )
                }}
              </p>
              <code
                >/u3edit pos1<br />/u3edit pos2<br />/u3edit set minecraft:stone<br />/u3edit
                apply</code
              >
              <p>
                {{
                  t(
                    'Сначала выбери два угла. set показывает план, apply выполняет его. Все команды: /u3edit help.',
                    'Select two corners. set previews the plan, apply executes it. All commands: /u3edit help.',
                  )
                }}
              </p>
            </details>
            <p v-if="!entry.available" class="mods-note">
              {{ t('Эта версия пока не поддерживается.', 'This version is not supported yet.') }}
            </p>
            <div class="mod-actions">
              <button
                class="btn btn-primary"
                :disabled="
                  busy || loading || !supported || !entry.available || modInstalled(entry, catalog)
                "
                @click="chosen = entry"
              >
                {{
                  modInstalled(entry, catalog)
                    ? t('Установлен', 'Installed')
                    : catalog.files?.[`${entry.id}.hnmod`] === entry.checksum
                      ? t('Собрать заново', 'Rebuild')
                      : t('Установить', 'Install')
                }}</button
              ><button v-if="entry.source" class="btn" @click="source(entry)">
                {{ t('Исходники', 'Source') }}
              </button>
            </div>
          </article>
        </div>
        <p v-if="!group.entries.length" class="mods-note">
          {{ t('Моды не найдены.', 'No mods found.') }}
        </p>
      </div>
    </template>
    <Teleport to="body">
      <div v-if="chosen" class="mods-backdrop" @click.self="chosen = null">
        <section
          ref="dialog"
          class="mods-dialog"
          role="dialog"
          aria-modal="true"
          aria-labelledby="mod-consent-title"
          tabindex="-1"
          @keydown="key"
        >
          <h2 id="mod-consent-title">
            {{ t('Установить нативный мод?', 'Install a native mod?') }}
          </h2>
          <p>
            <strong>{{ chosen.name }} {{ chosen.version }}</strong
            ><br />{{ (chosen.authors ?? []).join(', ') }} · {{ catalog.target_name }}
          </p>
          <p>
            {{
              t(
                'Нативный код получает доступ к файлам и сети с правами твоего пользователя. Устанавливай только моды, которым доверяешь.',
                'Native code can access files and the network with your user permissions. Only install mods you trust.',
              )
            }}
          </p>
          <p>
            {{
              t(
                'Первая сборка может занять 20-30 минут. Нужны интернет и несколько ГБ свободного места. Лаунчер сам загрузит инструменты и соберёт игру.',
                'The first build can take 20-30 minutes. Internet and several GB of free space are required. The launcher downloads the tools and builds the game.',
              )
            }}
          </p>
          <details>
            <summary>{{ t('Код для сборки и SHA-256', 'Build inputs and SHA-256') }}</summary>
            <ul>
              <li>
                {{ chosen.id }}<code>{{ chosen.checksum }}</code>
              </li>
              <li v-for="(checksum, file) in catalog.files" :key="file">
                {{ file }}<code>{{ checksum }}</code>
              </li>
            </ul>
          </details>
          <label class="mod-consent"
            ><input v-model="consent" type="checkbox" />{{
              t('Доверяю этому коду и разрешаю сборку.', 'I trust this code and allow the build.')
            }}</label
          >
          <div class="mod-actions">
            <button class="btn" @click="chosen = null">{{ t('Отмена', 'Cancel') }}</button
            ><button class="btn btn-primary" :disabled="!consent || busy" @click="install">
              {{ t('Установить и собрать', 'Install and build') }}
            </button>
          </div>
        </section>
      </div>
    </Teleport>
  </section>
</template>
<style scoped>
.mods-section {
  line-height: 1.5;
  margin: 2rem 0;
}
.mods-heading,
.mod-card-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}
.mods-heading {
  flex-wrap: wrap;
}
h2,
h3,
h4,
p {
  margin: 0;
}
h2 {
  font-size: 1.5rem;
}
h3 {
  font-size: 1.2rem;
}
h4 {
  font-size: 1.1rem;
}
.mods-note,
.mod-author {
  color: var(--color-secondary);
}
.mods-note {
  margin: 0.75rem 0;
}
.mod-group {
  margin: 1.5rem 0;
}
.mod-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 1rem;
  margin-top: 1rem;
}
.mod-card {
  padding: 1.5rem;
  border: 1px solid var(--color-button-bg);
  border-radius: 12px;
  background: var(--color-raised-bg);
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  min-width: 0;
}
.mod-card-heading span,
.mod-author {
  font-size: 0.875rem;
}
.mod-summary {
  line-height: 1.5;
}
.mod-actions {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
  margin-top: auto;
  padding-top: 0.5rem;
}
.btn {
  min-height: 44px;
  white-space: normal;
}
.mods-section input[type='search'] {
  max-width: 100%;
  min-height: 44px;
  padding: 10px 12px;
  border: 1px solid var(--color-secondary);
  border-radius: 8px;
  background: var(--color-bg);
  color: var(--color-base);
}
code {
  overflow-wrap: anywhere;
  font-size: 0.875rem;
}
summary {
  cursor: pointer;
  min-height: 32px;
}
details p {
  margin: 0.5rem 0;
}
.mods-error {
  border-left: 3px solid var(--color-red);
  padding: 1rem;
  background: var(--color-raised-bg);
  overflow-wrap: anywhere;
}
.mods-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10000;
  background: #000a;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
}
.mods-dialog {
  line-height: 1.5;
  width: 100%;
  max-width: 600px;
  max-height: calc(100dvh - 32px);
  overflow: auto;
  background: var(--color-raised-bg);
  color: var(--color-base);
  border: 1px solid var(--color-button-bg);
  border-radius: 12px;
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.mods-dialog li {
  margin-bottom: 8px;
}
.mods-dialog code {
  display: block;
}
.mod-consent {
  display: flex;
  align-items: center;
  gap: 12px;
  min-height: 44px;
}
.mod-consent input {
  flex-shrink: 0;
  width: 20px;
  height: 20px;
}
:focus-visible {
  outline: 2px solid var(--color-brand);
  outline-offset: 3px;
}
@media (max-width: 800px) {
  .mod-grid {
    grid-template-columns: 1fr;
  }
  .mod-card {
    padding: 16px;
  }
  .mods-dialog {
    padding: 20px;
  }
  .mod-actions .btn {
    flex: 1;
  }
}
</style>

<style scoped>
input[type='search']::placeholder {
  color: var(--color-secondary);
  opacity: 1;
}
</style>
