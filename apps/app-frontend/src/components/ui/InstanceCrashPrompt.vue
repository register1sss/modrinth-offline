<script setup>
import { nextTick, ref, watch } from 'vue'
import { Button } from '@modrinth/ui'
import { crashAlerts } from '@/instance-health/monitor.js'
import { health } from '@/instance-health/api.js'
import { repairAdvice } from '@/instance-health/model.js'
const dialog = ref(null)
let previousFocus
watch(
  () => crashAlerts[0],
  async (alert, previous) => {
    if (alert) {
      if (!previous) previousFocus = document.activeElement
      await nextTick()
      dialog.value?.focus()
    } else previousFocus?.focus?.()
  },
)
function trapFocus(event) {
  if (event.key !== 'Tab') return
  const elements = [
    ...dialog.value.querySelectorAll('button:not(:disabled), a[href], [tabindex="0"]'),
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
  const alert = crashAlerts[0]
  if (!alert || busy.value) return
  busy.value = true
  error.value = ''
  result.value = ''
  try {
    const r = await health('repair', alert.profileId)
    result.value = `Файлы игры восстановлены. Резервная копия: ${r.backup}. Попробуй запустить игру снова. Причины, связанные с модами, Java и памятью, могут остаться.`
  } catch (e) {
    error.value = String(e)
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
  <div v-if="crashAlerts.length" class="u3-crash-overlay">
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
      <h2 id="u3-crash-title">Возможный вылет игры</h2>
      <p>
        Экземпляр: {{ crashAlerts[0].profileId }}. В свежем журнале найдены признаки ошибки, но это
        не гарантированная причина завершения.
      </p>
      <ul>
        <li v-for="line in repairAdvice(crashAlerts[0].report.findings)" :key="line">{{ line }}</li>
      </ul>
      <p>
        Можно восстановить файлы игры и загрузчика. Метаданные экземпляра и настройки options будут
        сохранены в резервную копию. Миры, моды и config не изменяются. Нужен интернет.
      </p>
      <p v-if="error" role="alert">{{ error }}</p>
      <p v-if="result" role="status">{{ result }}</p>
      <div class="button-group">
        <Button :disabled="busy" @click="close">Позже</Button>
        <RouterLink
          v-if="!busy"
          :to="`/instance/${encodeURIComponent(crashAlerts[0].profileId)}/health`"
          @click="close"
          >Диагностика</RouterLink
        >
        <Button color="primary" :disabled="busy || !!result" @click="repair">{{
          busy ? 'Восстановление...' : 'Восстановить файлы'
        }}</Button>
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
}
.button-group {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
  align-items: center;
}
</style>
