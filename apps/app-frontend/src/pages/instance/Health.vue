<script setup>
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import DependencyTree from '@/components/ui/DependencyTree.vue'
import { Button, Card } from '@modrinth/ui'
import { health } from '@/instance-health/api.js'
import { sampleHistory } from '@/instance-health/history.js'
import { compareSamples } from '@/instance-health/model.js'
const props = defineProps({ instance: { type: Object, required: true }, playing: Boolean })
const inventory = ref([]),
  busy = ref(false),
  error = ref(''),
  result = ref(''),
  scanResults = ref({})
const samples = ref(sampleHistory.get(props.instance.path) || []),
  baseline = ref(0),
  live = ref(null)
const comparison = computed(() =>
  compareSamples(samples.value[Number(baseline.value)], samples.value.at(-1)),
)
const memory = (v) => `${(v / 1024 / 1024).toFixed(0)} МБ`
async function run(task) {
  if (busy.value) return
  busy.value = true
  error.value = ''
  try {
    await task()
  } catch (e) {
    error.value = String(e)
  } finally {
    busy.value = false
  }
}
function inspect() {
  return run(async () => {
    inventory.value = await health('inventory', props.instance.path)
    result.value =
      'Локальная проверка завершена. Признаки ниже являются эвристиками, а не заключением антивируса.'
  })
}
function scanMod(mod) {
  return run(async () => {
    scanResults.value[mod.path] = await health('antivirus', props.instance.path, { path: mod.path })
  })
}
function scanAll() {
  return run(async () => {
    inventory.value = await health('inventory', props.instance.path)
    for (const mod of inventory.value)
      scanResults.value[mod.path] = await health('antivirus', props.instance.path, {
        path: mod.path,
      })
  })
}
function repair() {
  return run(async () => {
    if (
      !window.confirm(
        'Восстановить скачанные файлы игры и загрузчика? Метаданные и options будут сохранены. Миры, моды и config не изменяются. Это не исправляет все причины вылетов.',
      )
    )
      return
    const r = await health('repair', props.instance.path)
    result.value = `Файлы восстановлены. Резервная копия: ${r.backup}`
  })
}
const timer = setInterval(async () => {
  if (!props.playing || polling) return
  polling = true
  try {
    live.value = await health('resources', props.instance.path)
  } catch (e) {
    error.value = String(e)
  } finally {
    polling = false
  }
}, 2500)
let polling = false
onBeforeUnmount(() => clearInterval(timer))
watch(
  () => props.instance.path,
  () => {
    inventory.value = []
    samples.value = sampleHistory.get(props.instance.path) || []
    baseline.value = 0
    scanResults.value = {}
    live.value = null
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
        throw new Error('Для замера запусти ровно один процесс этого экземпляра.')
      if (readings.length && readings[0].pid !== r.processes[0].pid)
        throw new Error('Процесс изменился во время замера. Повтори замер.')
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
</script>
<template>
  <div class="health-page">
    <h2>Диагностика экземпляра</h2>
    <p v-if="error" role="alert" class="error">{{ error }}</p>
    <p v-if="result" role="status">{{ result }}</p>
    <Card>
      <h3>Восстановление после вылета</h3>
      <p>
        При признаках вылета в свежем журнале лаунчер сам предложит диагностику. Восстановление
        скачивает файлы игры и загрузчика заново, но не меняет моды, config или миры. Ошибки
        зависимостей, Java и памяти требуют отдельного решения.
      </p>
      <Button :disabled="busy || playing" @click="repair">{{
        busy ? 'Подожди...' : 'Восстановить файлы игры'
      }}</Button>
    </Card>
    <Card>
      <h3>Потребление ресурсов</h3>
      <p>
        Измеряется процесс Minecraft, не отдельные моды. CPU 100% соответствует одному логическому
        ядру. Память включает JVM.
      </p>
      <div v-for="p in live?.processes || []" :key="p.pid">
        RAM: {{ memory(p.memory_bytes) }} · CPU: {{ p.cpu_percent.toFixed(1) }}%
      </div>
      <p>
        Сделай замер, останови игру, измени один мод и повтори тот же сценарий в том же мире.
        Сравнение показывает изменение нагрузки, но не доказывает, что его вызвал именно мод. Замеры
        сохраняются при переходах между вкладками до закрытия лаунчера.
      </p>
      <Button :disabled="busy || !playing" @click="snapshot">Записать замер (3 пробы)</Button>
      <label v-if="samples.length"
        >Базовый замер
        <select v-model="baseline">
          <option v-for="(s, i) in samples" :key="i" :value="i">
            {{ s.time }} · {{ memory(s.memory_bytes) }}
          </option>
        </select></label
      >
      <ul>
        <li v-for="(s, i) in samples" :key="i">
          {{ s.time }}: {{ memory(s.memory_bytes) }}, CPU {{ s.cpu_percent.toFixed(1) }}%
        </li>
      </ul>
      <div v-if="samples.length > 1 && comparison">
        <p>
          Последний относительно базового: RAM {{ memory(comparison.memory_delta) }}, CPU
          {{ comparison.cpu_delta.toFixed(1) }} п.п.
        </p>
        <p>
          Добавлены, удалены, включены, отключены или изменены файлы:
          {{ comparison.changed.join(', ') || 'нет изменений' }}.
        </p>
        <strong>Это сравнение запусков, а не рейтинг расхода ресурсов по модам.</strong>
      </div>
    </Card>
    <Card>
      <h3>Зависимости и проверка модов</h3>
      <p>
        Fabric, Quilt, Forge и NeoForge: читаются локальные манифесты. Ограничения версий
        показываются, но совместимость версий проверяет загрузчик. Вложенные JAR и неизвестные
        форматы требуют отдельной проверки. Нативные моды Horizon здесь не анализируются.
      </p>
      <div class="button-group">
        <Button :disabled="busy" @click="inspect">Обновить дерево и признаки риска</Button
        ><Button :disabled="busy" @click="scanAll">Проверить все моды антивирусом</Button>
      </div>
      <p>
        Файлы не запускаются и не отправляются лаунчером сторонним сервисам. Антивирусная проверка
        доступна на Windows с Microsoft Defender; действуют его системные настройки, включая
        облачную защиту. Автоматическое удаление отключено. Отсутствие находок не гарантирует
        безопасность.
      </p>
      <details v-for="mod in inventory" :key="mod.path">
        <summary>
          {{ mod.name }} {{ mod.disabled ? '(отключён)' : '' }} ·
          {{ mod.warnings.length ? 'есть признаки для проверки' : 'нет статических находок' }}
        </summary>
        <p>{{ mod.path }}</p>
        <code class="hash">SHA-256: {{ mod.sha256 || 'не вычислен' }}</code>
        <p v-if="!mod.metadata_complete">
          Неполные сведения о зависимостях. Безопасность удаления не гарантируется.
        </p>
        <DependencyTree :mod="mod" :graph="inventory" />
        <ul>
          <li v-for="(w, i) in mod.warnings" :key="i">{{ w }}</li>
        </ul>
        <Button :disabled="busy" @click="scanMod(mod)"
          >Проверить этот файл Microsoft Defender</Button
        >
        <p v-if="scanResults[mod.path]" role="status">{{ scanResults[mod.path].message }}</p>
        <p v-if="mod.warnings.length || scanResults[mod.path]?.status === 'review'">
          Если не доверяешь файлу, отключи его во вкладке содержимого или удали после проверки
          зависимостей. Не отключай антивирус.
        </p>
      </details>
    </Card>
  </div>
</template>
<style scoped>
.health-page {
  display: grid;
  gap: 1rem;
}
.button-group {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
}
details {
  border-top: 1px solid var(--color-divider);
  margin-top: 1rem;
  padding-top: 0.75rem;
}
.hash {
  overflow-wrap: anywhere;
}
.error {
  color: var(--color-red);
}
select {
  margin: 0.5rem;
  max-width: 100%;
}
</style>
