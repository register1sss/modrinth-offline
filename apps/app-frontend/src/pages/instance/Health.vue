<script setup>
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { Button, Card } from '@modrinth/ui'
import {
  CheckCircleIcon,
  ChevronRightIcon,
  HammerIcon,
  InfoIcon,
  IssuesIcon,
  SearchIcon,
  ShieldIcon,
  UpdatedIcon,
  XCircleIcon,
} from '@modrinth/assets'
import DependencyTree from '@/components/ui/DependencyTree.vue'
import { health } from '@/instance-health/api.js'
import { sampleHistory } from '@/instance-health/history.js'
import { compareSamples } from '@/instance-health/model.js'
import { analyzeMods } from '@/instance-health/dependencies.js'
import { describeHealthError, describeScan } from '@/instance-health/messages.js'
import { language, tr } from '@/i18n/index.js'

defineOptions({ inheritAttrs: false })
const props = defineProps({ instance: { type: Object, required: true }, playing: Boolean })

const plural = (n, one, few, many) => {
  const a = Math.abs(n) % 100
  const b = a % 10
  if (a > 10 && a < 20) return many
  if (b > 1 && b < 5) return few
  return b === 1 ? one : many
}
const count = (n, one, few, many, en1, enN) =>
  tr(`${n} ${plural(n, one, few, many)}`, `${n} ${n === 1 ? en1 : enN}`)

const inventory = ref([])
const checked = ref(false)
const checkedAt = ref(null)
const busy = ref(false)
const error = ref('')
const result = ref('')
const query = ref('')
const filter = ref('all')
const openMods = ref({})
const flash = ref('')
const report = computed(() => analyzeMods(inventory.value, props.instance, language.value))
const stats = computed(() => report.value.stats)
const urgent = computed(() => report.value.problems.filter((p) => p.severity !== 'info'))
const notes = computed(() => report.value.problems.filter((p) => p.severity === 'info'))
const fail = (e) => (error.value = describeHealthError(e, language.value))

async function run(task) {
  if (busy.value) return
  busy.value = true
  error.value = ''
  try {
    await task()
  } catch (e) {
    fail(e)
  } finally {
    busy.value = false
  }
}
function inspect() {
  return run(async () => {
    inventory.value = await health('inventory', props.instance.path)
    checked.value = true
    checkedAt.value = new Date()
  })
}
onMounted(inspect)

const verdict = computed(() => {
  const s = stats.value
  if (!checked.value)
    return { tone: 'muted', title: tr('Проверяю моды...', 'Checking mods...'), text: '' }
  if (!s.total)
    return {
      tone: 'good',
      title: tr('В профиле нет модов', 'This profile has no mods'),
      text: tr(
        'Папка mods пуста. Моды можно добавить во вкладке «Содержимое».',
        'The mods folder is empty. Add mods on the Content tab.',
      ),
    }
  const time = checkedAt.value?.toLocaleTimeString(language.value === 'en' ? 'en-GB' : 'ru-RU', {
    hour: '2-digit',
    minute: '2-digit',
  })
  const text = tr(
    `Проверено файлов: ${s.total} (включено ${s.active}, отключено ${s.disabled}). Проверка в ${time}.`,
    `Checked ${s.total} files (${s.active} enabled, ${s.disabled} disabled) at ${time}.`,
  )
  if (s.errors)
    return {
      tone: 'bad',
      title: tr(
        `${plural(s.errors, 'Найдена', 'Найдены', 'Найдено')} ${s.errors} ${plural(s.errors, 'проблема', 'проблемы', 'проблем')}: игра может не запуститься`,
        `${s.errors} ${s.errors === 1 ? 'problem' : 'problems'} found: the game may not start`,
      ),
      text,
    }
  if (s.warnings)
    return {
      tone: 'warn',
      title: tr(
        `Серьёзных проблем нет, но стоит проверить: ${s.warnings}`,
        `No blocking problems, ${s.warnings} worth checking`,
      ),
      text,
    }
  return { tone: 'good', title: tr('Проблем с модами не найдено', 'No mod problems found'), text }
})

const FILTERS = [
  ['all', 'Все', 'All'],
  ['problems', 'С проблемами', 'With problems'],
  ['libraries', 'Библиотеки', 'Libraries'],
  ['disabled', 'Отключённые', 'Disabled'],
]
const matches = (entry, kind) =>
  kind === 'problems'
    ? entry.active && (entry.status === 'error' || entry.status === 'warning')
    : kind === 'libraries'
      ? entry.active && entry.dependents.some((d) => d.required)
      : kind === 'disabled'
        ? !entry.active
        : true
const filterCount = (kind) => report.value.mods.filter((e) => matches(e, kind)).length
const visibleMods = computed(() => {
  const q = query.value.trim().toLowerCase()
  return report.value.mods.filter(
    (e) =>
      matches(e, filter.value) &&
      (!q ||
        e.title.toLowerCase().includes(q) ||
        e.file.toLowerCase().includes(q) ||
        (e.mod.ids || []).some((id) => id.toLowerCase().includes(q))),
  )
})
const anchor = (path) => `u3-mod-${encodeURIComponent(path).replace(/[^a-zA-Z0-9_-]/g, '_')}`
const titleOf = (path) => report.value.byPath.get(path)?.title ?? path
// Two files of the same mod need their versions (or file names) to be told apart.
function chipLabel(path, problem) {
  const entry = report.value.byPath.get(path)
  if (!entry) return path
  const twin = problem.mods.some(
    (other) => other !== path && report.value.byPath.get(other)?.title === entry.title,
  )
  return twin ? `${entry.title} ${entry.version || entry.file}` : entry.title
}
const worst = (entry) => (entry.problems.some((p) => p.severity === 'error') ? 'bad' : 'warn')
async function focusMod(path) {
  if (!report.value.byPath.has(path)) return
  if (!visibleMods.value.some((e) => e.path === path)) {
    filter.value = 'all'
    query.value = ''
  }
  openMods.value[path] = true
  flash.value = path
  await nextTick()
  document.getElementById(anchor(path))?.scrollIntoView({ behavior: 'smooth', block: 'center' })
  setTimeout(() => {
    if (flash.value === path) flash.value = ''
  }, 1600)
}
const requiredMods = (entry) => entry.required.filter((d) => d.kind === 'mod').length
const size = (bytes) =>
  bytes >= 1024 * 1024
    ? tr(`${(bytes / 1024 / 1024).toFixed(1)} МБ`, `${(bytes / 1024 / 1024).toFixed(1)} MB`)
    : tr(
        `${Math.max(1, Math.round(bytes / 1024))} КБ`,
        `${Math.max(1, Math.round(bytes / 1024))} KB`,
      )

// Microsoft Defender, one file at a time, with progress and cancel.
const scanResults = ref({})
const scan = ref({ running: false, done: 0, total: 0, cancel: false })
const scanText = (path) => describeScan(scanResults.value[path], language.value)
function scanMod(mod) {
  return run(async () => {
    scanResults.value[mod.path] = await health('antivirus', props.instance.path, { path: mod.path })
  })
}
async function scanAll() {
  if (busy.value) return
  busy.value = true
  error.value = ''
  const files = inventory.value.filter((m) => m.sha256)
  scan.value = { running: true, done: 0, total: files.length, cancel: false }
  try {
    for (const mod of files) {
      if (scan.value.cancel) break
      scanResults.value[mod.path] = await health('antivirus', props.instance.path, {
        path: mod.path,
      })
      scan.value.done++
      if (scanResults.value[mod.path]?.status === 'unavailable') break
    }
  } catch (e) {
    fail(e)
  } finally {
    scan.value.running = false
    busy.value = false
  }
}
const flagged = computed(() =>
  Object.entries(scanResults.value)
    .filter(([, r]) => r?.status === 'review')
    .map(([path]) => path),
)
const scanUnavailable = computed(() =>
  Object.values(scanResults.value).some((r) => r?.status === 'unavailable'),
)

function repair() {
  return run(async () => {
    if (
      !window.confirm(
        tr(
          'Скачать заново файлы Minecraft и загрузчика? Моды, миры и config не изменятся, options.txt попадёт в резервную копию. Это помогает при повреждённых файлах, но не при конфликтах модов.',
          'Download Minecraft and loader files again? Mods, worlds and config stay untouched, options.txt is backed up. This helps with damaged files, not with mod conflicts.',
        ),
      )
    )
      return
    const r = await health('repair', props.instance.path)
    result.value = tr(
      `Файлы игры восстановлены. Резервная копия: ${r.backup}`,
      `Game files restored. Backup: ${r.backup}`,
    )
  })
}

// Live resources and manual before/after samples.
const samples = ref(sampleHistory.get(props.instance.path) || [])
const baseline = ref(0)
const live = ref(null)
const comparison = computed(() =>
  compareSamples(samples.value[Number(baseline.value)], samples.value.at(-1)),
)
const memory = (v) => tr(`${(v / 1024 / 1024).toFixed(0)} МБ`, `${(v / 1024 / 1024).toFixed(0)} MB`)
const signed = (v, digits = 0) => `${v > 0 ? '+' : ''}${v.toFixed(digits)}`
let polling = false
const timer = setInterval(async () => {
  if (!props.playing || polling) return
  polling = true
  try {
    live.value = await health('resources', props.instance.path)
  } catch (e) {
    fail(e)
  } finally {
    polling = false
  }
}, 2500)
onBeforeUnmount(() => clearInterval(timer))
watch(
  () => props.playing,
  (playing) => {
    if (!playing) live.value = null
  },
)
watch(
  () => props.instance.path,
  () => {
    inventory.value = []
    checked.value = false
    openMods.value = {}
    samples.value = sampleHistory.get(props.instance.path) || []
    baseline.value = 0
    scanResults.value = {}
    live.value = null
    inspect()
  },
)
function snapshot() {
  return run(async () => {
    const mods = await health('inventory', props.instance.path)
    const readings = []
    // Three samples reduce single-frame noise; this is still a process-level comparison, not a profiler.
    for (let i = 0; i < 3; i++) {
      const r = await health('resources', props.instance.path)
      if (r.processes.length !== 1)
        throw new Error(
          tr(
            'Для замера должна быть запущена ровно одна копия игры этого профиля.',
            'Exactly one game process of this profile must be running.',
          ),
        )
      if (readings.length && readings[0].pid !== r.processes[0].pid)
        throw new Error(
          tr(
            'Игра перезапустилась во время замера. Повтори замер.',
            'The game restarted during sampling. Try again.',
          ),
        )
      readings.push(r.processes[0])
    }
    inventory.value = mods
    const avg = (key) => readings.reduce((s, r) => s + r[key], 0) / readings.length
    samples.value.push({
      time: new Date().toLocaleTimeString(),
      memory_bytes: avg('memory_bytes'),
      cpu_percent: avg('cpu_percent'),
      processes: 1,
      mods: mods.filter((m) => !m.disabled).map((m) => ({ path: m.path, sha256: m.sha256 })),
    })
    sampleHistory.set(props.instance.path, samples.value)
    if (sampleHistory.size > 50) sampleHistory.delete(sampleHistory.keys().next().value)
    if (samples.value.length > 10) {
      samples.value.shift()
      baseline.value = Math.max(0, baseline.value - 1)
    }
  })
}
const changedFiles = computed(() =>
  (comparison.value?.changed || []).map((p) => titleOf(p) || p.split('/').pop()),
)
</script>
<template>
  <div class="health-page">
    <header class="health-head">
      <div>
        <h2>{{ tr('Диагностика профиля', 'Profile diagnostics') }}</h2>
        <p>
          {{
            tr(
              'Проверяет моды, их зависимости и файлы игры. Файлы читаются только на этом компьютере и никуда не отправляются.',
              'Checks mods, their dependencies and game files. Files are read only on this computer and never uploaded.',
            )
          }}
        </p>
      </div>
      <Button :disabled="busy" @click="inspect">
        <UpdatedIcon :class="{ spinning: busy && !scan.running }" />
        {{ checked ? tr('Проверить снова', 'Check again') : tr('Проверить', 'Check') }}
      </Button>
    </header>

    <p v-if="error" role="alert" class="banner bad">{{ error }}</p>
    <p v-if="result" role="status" class="banner good">{{ result }}</p>

    <section class="verdict" :data-tone="verdict.tone" aria-live="polite">
      <span class="verdict-icon" aria-hidden="true">
        <XCircleIcon v-if="verdict.tone === 'bad'" />
        <IssuesIcon v-else-if="verdict.tone === 'warn'" />
        <CheckCircleIcon v-else-if="verdict.tone === 'good'" />
        <UpdatedIcon v-else class="spinning" />
      </span>
      <div>
        <strong>{{ verdict.title }}</strong>
        <p v-if="verdict.text">{{ verdict.text }}</p>
      </div>
    </section>

    <div v-if="checked && stats.total" class="tiles">
      <div class="tile">
        <span>{{ stats.active }}</span>
        {{ tr('модов включено', 'mods enabled') }}
      </div>
      <div class="tile">
        <span>{{ stats.libraries }}</span>
        {{ tr('нужны другим модам', 'needed by other mods') }}
      </div>
      <div class="tile" :data-tone="stats.errors ? 'bad' : 'good'">
        <span>{{ stats.errors }}</span>
        {{ tr('мешают запуску', 'block launch') }}
      </div>
      <div class="tile" :data-tone="stats.warnings ? 'warn' : 'good'">
        <span>{{ stats.warnings }}</span>
        {{ tr('стоит проверить', 'worth checking') }}
      </div>
    </div>

    <Card v-if="urgent.length" class="section">
      <h3>{{ tr('Что исправить', 'What to fix') }}</h3>
      <ol class="problems">
        <li v-for="p in urgent" :key="p.key" :data-severity="p.severity">
          <span class="problem-icon" aria-hidden="true">
            <XCircleIcon v-if="p.severity === 'error'" />
            <IssuesIcon v-else />
          </span>
          <div class="problem-body">
            <strong>{{ p.title }}</strong>
            <p>{{ p.text }}</p>
            <p class="fix">
              <b>{{ tr('Как исправить:', 'How to fix:') }}</b> {{ p.fix }}
            </p>
            <div v-if="p.mods.length" class="chips">
              <button v-for="path in p.mods" :key="path" class="chip" @click="focusMod(path)">
                {{ chipLabel(path, p) }}
              </button>
            </div>
          </div>
        </li>
      </ol>
    </Card>

    <details v-if="notes.length" class="notes">
      <summary>
        <InfoIcon aria-hidden="true" />
        {{ tr(`Заметки проверки: ${notes.length}`, `Check notes: ${notes.length}`) }}
      </summary>
      <ul>
        <li v-for="p in notes" :key="p.key">
          <strong>{{ p.title }}.</strong> {{ p.text }} <span class="muted">{{ p.fix }}</span>
        </li>
      </ul>
    </details>

    <Card v-if="checked && stats.total" class="section">
      <div class="tree-head">
        <div>
          <h3>{{ tr('Дерево зависимостей', 'Dependency tree') }}</h3>
          <p class="muted">
            {{
              tr(
                'Открой мод, чтобы увидеть, что ему нужно для запуска и каким модам нужен он сам.',
                'Open a mod to see what it needs to start and which mods need it.',
              )
            }}
          </p>
        </div>
        <label class="search">
          <SearchIcon aria-hidden="true" />
          <input
            v-model="query"
            type="search"
            :placeholder="tr('Найти мод', 'Find a mod')"
            :aria-label="tr('Найти мод', 'Find a mod')"
          />
        </label>
      </div>
      <div class="segmented" role="tablist">
        <button
          v-for="[key, ru, en] in FILTERS"
          :key="key"
          role="tab"
          :aria-selected="filter === key"
          :class="{ active: filter === key }"
          @click="filter = key"
        >
          {{ tr(ru, en) }} <span>{{ filterCount(key) }}</span>
        </button>
      </div>
      <ul class="mods">
        <li
          v-for="entry in visibleMods"
          :id="anchor(entry.path)"
          :key="entry.path"
          class="mod"
          :data-status="entry.status"
          :class="{ flash: flash === entry.path, open: openMods[entry.path] }"
        >
          <button
            class="mod-row"
            :aria-expanded="!!openMods[entry.path]"
            @click="openMods[entry.path] = !openMods[entry.path]"
          >
            <ChevronRightIcon class="chevron" aria-hidden="true" />
            <span class="dot" aria-hidden="true"></span>
            <span class="mod-title">{{ entry.title }}</span>
            <span v-if="entry.version" class="mod-version">{{ entry.version }}</span>
            <span class="badges">
              <span v-if="!entry.active" class="badge">{{ tr('отключён', 'disabled') }}</span>
              <span
                v-if="entry.problems.some((p) => p.severity !== 'info')"
                class="badge"
                :class="worst(entry)"
                >{{
                  count(
                    entry.problems.filter((p) => p.severity !== 'info').length,
                    'проблема',
                    'проблемы',
                    'проблем',
                    'problem',
                    'problems',
                  )
                }}</span
              >
              <span v-if="entry.dependents.length" class="badge">{{
                tr(`нужен ${entry.dependents.length}`, `needed by ${entry.dependents.length}`)
              }}</span>
              <span v-if="requiredMods(entry)" class="badge">{{
                tr(`требует ${requiredMods(entry)}`, `needs ${requiredMods(entry)}`)
              }}</span>
            </span>
          </button>
          <div v-if="openMods[entry.path]" class="mod-body">
            <DependencyTree
              :entry="entry"
              :report="report"
              :instance="instance"
              @select="focusMod"
            />
            <ul v-if="entry.notes.length" class="mod-notes">
              <li v-for="(n, i) in entry.notes" :key="i" :data-kind="n.kind">{{ n.text }}</li>
            </ul>
            <details class="tech">
              <summary>{{ tr('Сведения о файле', 'File details') }}</summary>
              <dl>
                <dt>{{ tr('Файл', 'File') }}</dt>
                <dd>{{ entry.path }}</dd>
                <dt>{{ tr('Размер', 'Size') }}</dt>
                <dd>{{ size(entry.mod.bytes || 0) }}</dd>
                <template v-if="entry.family">
                  <dt>{{ tr('Загрузчик', 'Loader') }}</dt>
                  <dd>{{ entry.family }}</dd>
                </template>
                <template v-if="(entry.mod.ids || []).length">
                  <dt>ID</dt>
                  <dd>{{ entry.mod.ids.join(', ') }}</dd>
                </template>
                <dt>SHA-256</dt>
                <dd class="hash">{{ entry.mod.sha256 || tr('не вычислен', 'not computed') }}</dd>
              </dl>
              <Button :disabled="busy || !entry.mod.sha256" @click="scanMod(entry.mod)">
                <ShieldIcon />
                {{ tr('Проверить Microsoft Defender', 'Scan with Microsoft Defender') }}
              </Button>
              <p
                v-if="scanText(entry.path)"
                role="status"
                class="scan"
                :data-tone="scanText(entry.path).tone"
              >
                {{ scanText(entry.path).text }}
              </p>
            </details>
          </div>
        </li>
      </ul>
      <p v-if="!visibleMods.length" class="muted empty">
        {{ tr('Ничего не найдено.', 'Nothing found.') }}
      </p>
      <p class="muted footnote">
        {{
          tr(
            'Сведения берутся из описаний модов (fabric.mod.json, quilt.mod.json, mods.toml), включая встроенные JAR. Окончательно совместимость проверяет загрузчик при запуске. Нативные моды Horizon здесь не анализируются.',
            'Data comes from mod descriptions (fabric.mod.json, quilt.mod.json, mods.toml), including nested JARs. The loader makes the final compatibility check at launch. Horizon native mods are not analyzed here.',
          )
        }}
      </p>
    </Card>

    <Card class="section">
      <h3>
        <HammerIcon aria-hidden="true" />
        {{ tr('Восстановление файлов игры', 'Repair game files') }}
      </h3>
      <p>
        {{
          tr(
            'Если игра вылетает сразу при запуске или пишет о повреждённых файлах, лаунчер может заново скачать файлы Minecraft и загрузчика. Моды, миры и config не меняются, options.txt сохраняется в резервную копию. Нужен интернет.',
            'If the game crashes right at launch or reports damaged files, the launcher can download Minecraft and loader files again. Mods, worlds and config stay unchanged, options.txt is backed up. Requires internet.',
          )
        }}
      </p>
      <p class="muted">
        {{
          tr(
            'При вылете лаунчер сам предложит диагностику. Проблемы с зависимостями, Java и памятью восстановление не исправит: смотри список выше и параметры профиля.',
            'After a crash the launcher offers diagnostics. Repair does not fix dependency, Java or memory problems: see the list above and profile options.',
          )
        }}
      </p>
      <Button :disabled="busy || playing" @click="repair">
        {{
          busy
            ? tr('Подожди...', 'Please wait...')
            : tr('Восстановить файлы игры', 'Repair game files')
        }}
      </Button>
      <p v-if="playing" class="muted">{{ tr('Сначала закрой игру.', 'Close the game first.') }}</p>
    </Card>

    <Card class="section">
      <h3>{{ tr('Нагрузка во время игры', 'Load while playing') }}</h3>
      <div v-if="playing && live?.processes?.length" class="tiles">
        <template v-for="p in live.processes" :key="p.pid">
          <div class="tile">
            <span>{{ memory(p.memory_bytes) }}</span>
            {{ tr('оперативной памяти', 'of RAM') }}
          </div>
          <div class="tile">
            <span>{{ p.cpu_percent.toFixed(1) }}%</span>
            {{ tr('процессора (100% = 1 ядро)', 'CPU (100% = 1 core)') }}
          </div>
        </template>
      </div>
      <p v-else class="muted">
        {{
          playing
            ? tr('Получаю данные о процессе игры...', 'Reading the game process...')
            : tr(
                'Запусти игру, чтобы увидеть, сколько памяти и процессора она использует.',
                'Start the game to see how much memory and CPU it uses.',
              )
        }}
      </p>
      <p>
        {{
          tr(
            'Хочешь понять, какой мод нагружает игру? Сделай замер, закрой игру, отключи один мод, зайди в тот же мир и сделай замер снова. Лаунчер покажет разницу и какие файлы изменились. Это сравнение запусков, а не точный профайлер.',
            'Want to find a heavy mod? Take a sample, close the game, disable one mod, join the same world and sample again. The launcher shows the difference and which files changed. This compares runs; it is not a profiler.',
          )
        }}
      </p>
      <div class="row">
        <Button :disabled="busy || !playing" @click="snapshot">
          {{ tr('Сделать замер (3 пробы)', 'Take a sample (3 readings)') }}
        </Button>
        <label v-if="samples.length > 1" class="baseline">
          {{ tr('Сравнивать с', 'Compare with') }}
          <select v-model="baseline">
            <option v-for="(s, i) in samples" :key="i" :value="i">
              {{ s.time }} · {{ memory(s.memory_bytes) }}
            </option>
          </select>
        </label>
      </div>
      <ul v-if="samples.length" class="samples">
        <li v-for="(s, i) in samples" :key="i">
          {{ s.time }}: {{ memory(s.memory_bytes) }}, CPU {{ s.cpu_percent.toFixed(1) }}%
        </li>
      </ul>
      <div v-if="samples.length > 1 && comparison" class="comparison">
        <p>
          {{ tr('Последний замер относительно выбранного:', 'Latest sample vs selected:') }}
          <b>RAM {{ signed(comparison.memory_delta / 1024 / 1024) }} {{ tr('МБ', 'MB') }}</b
          >,
          <b>CPU {{ signed(comparison.cpu_delta, 1) }} {{ tr('п.п.', 'pp') }}</b>
        </p>
        <p>
          {{ tr('Изменённые файлы:', 'Changed files:') }}
          {{ changedFiles.join(', ') || tr('нет', 'none') }}
        </p>
      </div>
    </Card>

    <Card class="section">
      <h3><ShieldIcon aria-hidden="true" /> {{ tr('Проверка безопасности', 'Security check') }}</h3>
      <p>
        {{
          tr(
            'Лаунчер ищет в модах подозрительные признаки (скрипты, запуск командной строки, доступ к токенам Discord) и может проверить файлы через Microsoft Defender. Файлы не запускаются, ничего не удаляется автоматически. Отсутствие находок не гарантирует безопасность.',
            'The launcher looks for suspicious signs in mods (scripts, shell launching, Discord token access) and can scan files with Microsoft Defender. Files are never run and nothing is deleted automatically. No findings is not a safety guarantee.',
          )
        }}
      </p>
      <div class="row">
        <Button :disabled="busy || !inventory.length" @click="scanAll">
          <ShieldIcon />
          {{ tr('Проверить все моды антивирусом', 'Scan all mods with antivirus') }}
        </Button>
        <Button v-if="scan.running" @click="scan.cancel = true">{{
          tr('Остановить', 'Stop')
        }}</Button>
      </div>
      <div v-if="scan.total" class="progress" role="status">
        <div class="bar">
          <span :style="{ width: `${(scan.done / scan.total) * 100}%` }"></span>
        </div>
        {{ tr(`Проверено ${scan.done} из ${scan.total}`, `Scanned ${scan.done} of ${scan.total}`) }}
      </div>
      <p v-if="scanUnavailable" class="muted">
        {{ describeScan({ status: 'unavailable' }, language).text }}
      </p>
      <p
        v-else-if="!scan.running && scan.total && scan.done === scan.total && !flagged.length"
        class="scan"
        data-tone="ok"
      >
        {{
          tr(
            'Defender ничего не нашёл в проверенных модах.',
            'Defender found nothing in the scanned mods.',
          )
        }}
      </p>
      <div v-if="flagged.length" class="scan" data-tone="warn">
        {{ tr('Defender отметил файлы:', 'Defender flagged:') }}
        <button v-for="path in flagged" :key="path" class="chip" @click="focusMod(path)">
          {{ titleOf(path) }}
        </button>
      </div>
    </Card>
  </div>
</template>
<style scoped>
.health-page {
  display: grid;
  gap: 1rem;
  min-width: 0;
}
.health-head {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  align-items: flex-start;
  gap: 0.75rem 1rem;
}
.health-head h2 {
  margin: 0 0 0.25rem;
}
.health-head p {
  margin: 0;
  color: var(--color-secondary);
  max-width: 46rem;
}
.spinning {
  animation: u3-spin 1s linear infinite;
}
@keyframes u3-spin {
  to {
    transform: rotate(360deg);
  }
}
.banner {
  margin: 0;
  padding: 0.7rem 0.9rem;
  border-radius: var(--radius-md);
  border: 1px solid var(--color-divider);
}
.banner.bad {
  color: var(--color-red);
  border-color: color-mix(in srgb, var(--color-red) 50%, transparent);
  background: color-mix(in srgb, var(--color-red) 10%, transparent);
}
.banner.good {
  border-color: color-mix(in srgb, var(--color-green) 50%, transparent);
  background: color-mix(in srgb, var(--color-green) 10%, transparent);
}
.verdict {
  display: flex;
  gap: 0.9rem;
  align-items: center;
  padding: 1rem 1.1rem;
  border-radius: var(--radius-lg);
  background: var(--color-raised-bg);
  border: 1px solid var(--color-divider);
  border-left-width: 4px;
}
.verdict strong {
  font-size: 1.05rem;
  color: var(--color-contrast);
}
.verdict p {
  margin: 0.2rem 0 0;
  color: var(--color-secondary);
}
.verdict-icon {
  display: inline-flex;
  width: 2rem;
  height: 2rem;
  flex: none;
}
.verdict-icon svg {
  width: 100%;
  height: 100%;
}
.verdict[data-tone='bad'] {
  border-left-color: var(--color-red);
}
.verdict[data-tone='bad'] .verdict-icon {
  color: var(--color-red);
}
.verdict[data-tone='warn'] {
  border-left-color: var(--color-orange);
}
.verdict[data-tone='warn'] .verdict-icon {
  color: var(--color-orange);
}
.verdict[data-tone='good'] {
  border-left-color: var(--color-green);
}
.verdict[data-tone='good'] .verdict-icon {
  color: var(--color-green);
}
.verdict[data-tone='muted'] .verdict-icon {
  color: var(--color-secondary);
}
.tiles {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(9.5rem, 1fr));
  gap: 0.6rem;
}
.tile {
  padding: 0.75rem 0.9rem;
  border-radius: var(--radius-md);
  background: var(--color-raised-bg);
  border: 1px solid var(--color-divider);
  color: var(--color-secondary);
  font-size: 0.85rem;
}
.tile span {
  display: block;
  font-size: 1.45rem;
  font-weight: 700;
  color: var(--color-contrast);
}
.tile[data-tone='bad'] span {
  color: var(--color-red);
}
.tile[data-tone='warn'] span {
  color: var(--color-orange);
}
.section h3 {
  display: flex;
  align-items: center;
  gap: 0.45rem;
  margin: 0 0 0.6rem;
}
.section h3 svg {
  width: 1.1rem;
  height: 1.1rem;
}
.section > p {
  margin: 0 0 0.75rem;
}
.problems {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0.6rem;
}
.problems li {
  display: flex;
  gap: 0.75rem;
  padding: 0.8rem 0.9rem;
  border-radius: var(--radius-md);
  border: 1px solid var(--color-divider);
  background: var(--color-bg);
}
.problems li[data-severity='error'] {
  border-color: color-mix(in srgb, var(--color-red) 45%, transparent);
}
.problems li[data-severity='warning'] {
  border-color: color-mix(in srgb, var(--color-orange) 45%, transparent);
}
.problem-icon {
  display: inline-flex;
  width: 1.3rem;
  height: 1.3rem;
  flex: none;
  margin-top: 0.1rem;
}
.problem-icon svg {
  width: 100%;
  height: 100%;
}
[data-severity='error'] .problem-icon {
  color: var(--color-red);
}
[data-severity='warning'] .problem-icon {
  color: var(--color-orange);
}
.problem-body {
  min-width: 0;
  display: grid;
  gap: 0.35rem;
}
.problem-body strong {
  color: var(--color-contrast);
  overflow-wrap: anywhere;
}
.problem-body p {
  margin: 0;
  overflow-wrap: anywhere;
}
.fix {
  color: var(--color-contrast);
}
.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
}
.chip {
  border: 1px solid color-mix(in srgb, var(--color-brand) 45%, transparent);
  background: color-mix(in srgb, var(--color-brand) 12%, transparent);
  color: var(--color-contrast);
  border-radius: 999px;
  padding: 0.2rem 0.7rem;
  font-size: 0.85rem;
  cursor: pointer;
}
.chip:hover,
.chip:focus-visible {
  background: color-mix(in srgb, var(--color-brand) 24%, transparent);
}
.notes {
  padding: 0.7rem 0.9rem;
  border-radius: var(--radius-md);
  background: var(--color-raised-bg);
  border: 1px solid var(--color-divider);
}
.notes summary {
  display: flex;
  align-items: center;
  gap: 0.45rem;
  cursor: pointer;
  color: var(--color-secondary);
}
.notes summary svg {
  width: 1rem;
  height: 1rem;
}
.notes ul {
  margin: 0.6rem 0 0;
  padding-left: 1.2rem;
  display: grid;
  gap: 0.35rem;
}
.muted {
  color: var(--color-secondary);
}
.tree-head {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  gap: 0.6rem 1rem;
  align-items: flex-end;
}
.tree-head > div {
  flex: 1 1 22rem;
  min-width: 0;
}
.tree-head p {
  margin: 0;
}
.search {
  position: relative;
  display: block;
  flex: 0 1 18rem;
  width: min(100%, 18rem);
}
.search svg {
  position: absolute;
  left: 0.8rem;
  top: 50%;
  width: 1.05rem;
  height: 1.05rem;
  transform: translateY(-50%);
  color: var(--color-secondary);
  pointer-events: none;
}
.search input {
  box-sizing: border-box;
  width: 100%;
  min-height: 2.5rem;
  padding: 0.55rem 0.9rem 0.55rem 2.4rem;
  border: 1px solid var(--color-divider);
  border-radius: var(--radius-md, 0.75rem);
  background: var(--color-button-bg);
  color: var(--color-contrast);
  font-size: 0.95rem;
  line-height: 1.3;
  transition:
    border-color 0.15s ease,
    box-shadow 0.15s ease;
}
.search input::placeholder {
  color: var(--color-secondary);
  opacity: 1;
}
.search input:focus,
.search input:focus-visible {
  outline: none;
  border-color: var(--color-brand);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-brand) 25%, transparent);
}
.segmented {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
  margin: 0.8rem 0 0.6rem;
}
.segmented button {
  border: 1px solid var(--color-divider);
  background: transparent;
  color: var(--color-base);
  border-radius: 999px;
  padding: 0.25rem 0.75rem;
  cursor: pointer;
}
.segmented button span {
  color: var(--color-secondary);
  margin-left: 0.2rem;
}
.segmented button.active {
  background: var(--color-brand);
  border-color: var(--color-brand);
  color: var(--color-accent-contrast);
}
.segmented button.active span {
  color: inherit;
  opacity: 0.8;
}
.mods {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0.4rem;
}
.mod {
  border-radius: var(--radius-md);
  border: 1px solid var(--color-divider);
  background: var(--color-bg);
  transition:
    box-shadow 0.3s,
    border-color 0.3s;
}
.mod.flash {
  border-color: var(--color-brand);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-brand) 35%, transparent);
}
.mod-row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.35rem 0.6rem;
  width: 100%;
  padding: 0.6rem 0.75rem;
  border: 0;
  background: transparent;
  color: inherit;
  text-align: left;
  cursor: pointer;
  border-radius: var(--radius-md);
}
.mod-row:hover,
.mod-row:focus-visible {
  background: color-mix(in srgb, var(--color-button-bg) 60%, transparent);
}
.chevron {
  width: 1rem;
  height: 1rem;
  flex: none;
  color: var(--color-secondary);
  transition: transform 0.15s;
}
.mod.open .chevron {
  transform: rotate(90deg);
}
.dot {
  width: 0.6rem;
  height: 0.6rem;
  border-radius: 50%;
  flex: none;
  background: var(--color-green);
}
[data-status='error'] .dot {
  background: var(--color-red);
}
[data-status='warning'] .dot {
  background: var(--color-orange);
}
[data-status='disabled'] .dot {
  background: var(--color-secondary);
}
[data-status='disabled'] .mod-title {
  color: var(--color-secondary);
}
.mod-title {
  font-weight: 600;
  color: var(--color-contrast);
  overflow-wrap: anywhere;
}
.mod-version {
  font-size: 0.8rem;
  color: var(--color-secondary);
  overflow-wrap: anywhere;
}
.badges {
  display: inline-flex;
  flex-wrap: wrap;
  gap: 0.3rem;
  margin-left: auto;
}
.badge {
  font-size: 0.75rem;
  padding: 0.1rem 0.5rem;
  border-radius: 999px;
  background: var(--color-button-bg);
  color: var(--color-base);
}
.badge.bad {
  background: color-mix(in srgb, var(--color-red) 18%, transparent);
  color: var(--color-red);
  font-weight: 600;
}
.badge.warn {
  background: color-mix(in srgb, var(--color-orange) 18%, transparent);
  color: var(--color-orange);
  font-weight: 600;
}
.mod-body {
  display: grid;
  gap: 0.8rem;
  padding: 0.2rem 0.9rem 0.9rem 2.3rem;
}
.mod-notes {
  margin: 0;
  padding-left: 1.1rem;
  font-size: 0.88rem;
  display: grid;
  gap: 0.2rem;
}
.mod-notes [data-kind='risk'] {
  color: var(--color-orange);
}
.tech summary {
  cursor: pointer;
  color: var(--color-secondary);
  font-size: 0.85rem;
}
.tech dl {
  display: grid;
  grid-template-columns: max-content 1fr;
  gap: 0.25rem 0.8rem;
  margin: 0.6rem 0;
  font-size: 0.85rem;
}
.tech dt {
  color: var(--color-secondary);
}
.tech dd {
  margin: 0;
  overflow-wrap: anywhere;
}
.hash {
  font-family: var(--mono-font, monospace);
  font-size: 0.78rem;
}
.scan {
  margin: 0.5rem 0 0;
  padding: 0.5rem 0.7rem;
  border-radius: var(--radius-md);
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
  align-items: center;
}
.scan[data-tone='ok'] {
  background: color-mix(in srgb, var(--color-green) 12%, transparent);
}
.scan[data-tone='warn'] {
  background: color-mix(in srgb, var(--color-orange) 14%, transparent);
}
.scan[data-tone='muted'] {
  color: var(--color-secondary);
}
.empty {
  text-align: center;
  padding: 1rem 0;
}
.footnote {
  margin: 0;
  padding-top: 0.9rem;
  font-size: 0.8rem;
}
.row {
  display: flex;
  flex-wrap: wrap;
  gap: 0.6rem;
  align-items: center;
}
.baseline select {
  margin-left: 0.4rem;
  max-width: 100%;
}
.samples {
  margin: 0.75rem 0 0;
  padding-left: 1.2rem;
  color: var(--color-secondary);
}
.comparison {
  margin-top: 0.75rem;
  padding: 0.7rem 0.9rem;
  border-radius: var(--radius-md);
  background: var(--color-bg);
}
.comparison p {
  margin: 0.2rem 0;
}
.progress {
  margin-top: 0.75rem;
  display: grid;
  gap: 0.35rem;
  font-size: 0.85rem;
  color: var(--color-secondary);
}
.bar {
  height: 0.45rem;
  border-radius: 999px;
  background: var(--color-button-bg);
  overflow: hidden;
}
.bar span {
  display: block;
  height: 100%;
  background: var(--color-brand);
  transition: width 0.3s;
}
</style>
