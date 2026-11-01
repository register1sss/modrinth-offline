<script setup>
import { computed, nextTick, ref, watch } from 'vue'
import { Button } from '@modrinth/ui'
import { crashAlerts } from '@/instance-health/monitor.js'
import { health } from '@/instance-health/api.js'
import { crashAdvice, describeHealthError, describeLoaderLine } from '@/instance-health/messages.js'
import { get as getProfile } from '@/helpers/profile'
import { language, tr } from '@/i18n/index.js'

const dialog = ref(null)
const instanceName = ref('')
let previousFocus
const alert = computed(() => crashAlerts[0])
const report = computed(() => alert.value?.report ?? {})
const advice = computed(() => crashAdvice(report.value.findings || [], language.value))
const details = computed(() =>
  (report.value.details || []).map((line) => describeLoaderLine(line, language.value)),
)
const evidence = computed(() => (report.value.evidence || []).map((e) => e.line).filter(Boolean))
const canRepair = computed(() => (report.value.findings || []).some((k) => k === 'files'))

watch(
  alert,
  async (current, previous) => {
    if (current) {
      if (!previous) previousFocus = document.activeElement
      instanceName.value = current.profileId
      getProfile(current.profileId)
        .then((profile) => {
          if (crashAlerts[0] === current && profile?.name) instanceName.value = profile.name
        })
        .catch(() => {})
      await nextTick()
      dialog.value?.focus()
    } else previousFocus?.focus?.()
  },
  { immediate: true },
)
function trapFocus(event) {
  if (event.key !== 'Tab') return
  const elements = [
    ...dialog.value.querySelectorAll('button:not(:disabled), a[href], summary, [tabindex="0"]'),
  ]
  if (!elements.length) {
    event.preventDefault()
    return
  }
  const first = elements[0],
    last = elements.at(-1)
  if (
    event.shiftKey &&
    (document.activeElement === first || document.activeElement === dialog.value)
  ) {
    event.preventDefault()
    last.focus()
  } else if (
    !event.shiftKey &&
    (document.activeElement === last || document.activeElement === dialog.value)
  ) {
    event.preventDefault()
    first.focus()
  }
}
const busy = ref(false)
const result = ref('')
const error = ref('')
async function repair() {
  const current = alert.value
  if (!current || busy.value) return
  busy.value = true
  error.value = ''
  result.value = ''
  try {
    const r = await health('repair', current.profileId)
    result.value = tr(
      `Файлы игры восстановлены, резервная копия: ${r.backup}. Запусти игру снова. Если вылет повторится, дело не в файлах: открой диагностику.`,
      `Game files repaired, backup: ${r.backup}. Launch the game again. If it still crashes, the files are not the cause: open Diagnostics.`,
    )
  } catch (e) {
    error.value = describeHealthError(e, language.value)
  } finally {
    busy.value = false
  }
}
function close() {
  if (!busy.value) {
    crashAlerts.shift()
    error.value = ''
    result.value = ''
  }
}
</script>
<template>
  <div v-if="alert" class="u3-crash-overlay">
    <section
      ref="dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby="u3-crash-title"
      class="u3-crash-card"
      tabindex="-1"
      @keydown.esc="close"
      @keydown="trapFocus"
    >
      <h2 id="u3-crash-title">{{ tr('Игра завершилась с ошибкой', 'The game crashed') }}</h2>
      <p class="u3-crash-lead">
        <template v-if="Number.isInteger(report.exit_code)">
          {{
            tr(
              `Профиль «${instanceName}» закрылся с кодом ${report.exit_code}.`,
              `Profile "${instanceName}" exited with code ${report.exit_code}.`,
            )
          }}
        </template>
        <template v-else>
          {{
            tr(
              `Профиль «${instanceName}» закрылся с ошибкой.`,
              `Profile "${instanceName}" exited with an error.`,
            )
          }}
        </template>
        {{
          tr('Ниже то, что удалось понять по журналу игры.', 'Here is what the game log suggests.')
        }}
      </p>
      <h3>{{ tr('Вероятная причина', 'Likely cause') }}</h3>
      <ul class="u3-crash-list">
        <li v-for="line in advice" :key="line">{{ line }}</li>
      </ul>
      <template v-if="details.length">
        <h3>{{ tr('Что пишет загрузчик модов', 'What the mod loader says') }}</h3>
        <ul class="u3-crash-list u3-crash-list--loader">
          <li v-for="(line, i) in details" :key="i">{{ line }}</li>
        </ul>
      </template>
      <details v-if="evidence.length" class="u3-crash-evidence">
        <summary>{{ tr('Строки из журнала', 'Log lines') }}</summary>
        <pre v-for="(line, i) in evidence" :key="i">{{ line }}</pre>
      </details>
      <p class="u3-crash-note">
        {{
          tr(
            '«Восстановить файлы» заново скачает файлы игры и загрузчика (нужен интернет). Миры, моды и настройки не меняются, параметры профиля сохраняются в резервную копию.',
            '"Repair files" downloads the game and loader files again (internet required). Worlds, mods and settings stay as they are; profile options are backed up.',
          )
        }}
      </p>
      <p v-if="error" role="alert" class="u3-crash-error">{{ error }}</p>
      <p v-if="result" role="status" class="u3-crash-ok">{{ result }}</p>
      <div class="button-group">
        <Button :disabled="busy" @click="close">{{ tr('Закрыть', 'Close') }}</Button>
        <Button
          v-if="!busy"
          :color="canRepair ? 'default' : 'primary'"
          :link="`/instance/${encodeURIComponent(alert.profileId)}/health`"
          @click="close"
          >{{ tr('Открыть диагностику', 'Open Diagnostics') }}</Button
        >
        <Button
          :color="canRepair ? 'primary' : 'default'"
          :disabled="busy || !!result"
          @click="repair"
          >{{
            busy
              ? tr('Восстановление...', 'Repairing...')
              : tr('Восстановить файлы', 'Repair files')
          }}</Button
        >
      </div>
    </section>
  </div>
</template>
<style scoped>
.u3-crash-overlay {
  position: fixed;
  inset: 0;
  background: #000a;
  z-index: 10000;
  display: grid;
  place-items: center;
  padding: 1rem;
}
.u3-crash-card {
  width: min(100%, 42rem);
  max-height: 90vh;
  overflow: auto;
  background: var(--color-raised-bg);
  border: 1px solid var(--color-divider);
  border-radius: 1rem;
  padding: 1.5rem;
  display: grid;
  gap: 0.6rem;
}
.u3-crash-card:focus {
  outline: none;
}
.u3-crash-card h2 {
  margin: 0;
}
.u3-crash-card h3 {
  margin: 0.4rem 0 0;
  font-size: 1rem;
}
.u3-crash-lead,
.u3-crash-note {
  margin: 0;
  color: var(--color-secondary);
}
.u3-crash-list {
  margin: 0;
  padding-left: 1.2rem;
  display: grid;
  gap: 0.3rem;
}
.u3-crash-list--loader li {
  overflow-wrap: anywhere;
}
.u3-crash-evidence summary {
  cursor: pointer;
  color: var(--color-secondary);
}
.u3-crash-evidence pre {
  margin: 0.4rem 0 0;
  padding: 0.5rem 0.7rem;
  border-radius: 0.5rem;
  background: var(--color-bg);
  font-size: 0.8rem;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}
.u3-crash-error {
  margin: 0;
  color: var(--color-red);
}
.u3-crash-ok {
  margin: 0;
  color: var(--color-green);
}
.button-group {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
  align-items: center;
  margin-top: 0.4rem;
}
</style>
