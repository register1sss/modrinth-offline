<script setup>
import { computed, ref } from 'vue'
import { tr, translate } from '@/i18n/index.js'
import { useTheming } from '@/store/theme.js'
import { BASE_THEMES, CUSTOM_THEMES, EFFECT_LEVELS } from '@/themes/catalog.js'
import ThemePreview from '@/components/themes/ThemePreview.vue'
import { get, set } from '@/helpers/settings.js'
import { CheckIcon } from '@modrinth/assets'

const themeStore = useTheming()
const active = computed(() => themeStore.activeTheme)
const saving = ref(false)
const notice = ref('')
const saveError = ref(false)
const activeNumber = computed(() => {
  const index = CUSTOM_THEMES.findIndex((theme) => theme.id === active.value.id)
  return index < 0
    ? tr('БАЗОВАЯ', 'BASE')
    : `${String(index + 1).padStart(2, '0')} / ${CUSTOM_THEMES.length}`
})
const query = ref('')
const filter = ref('all')
const filters = computed(() => [
  { id: 'all', name: tr('Все', 'All') },
  { id: 'studio', name: tr('Новые · 19', 'New · 19') },
  { id: 'dark', name: tr('Тёмные', 'Dark') },
  { id: 'light', name: tr('Светлые', 'Light') },
])
const filteredThemes = computed(() =>
  CUSTOM_THEMES.filter((theme) => {
    const matches =
      filter.value === 'all' ||
      (filter.value === 'studio'
        ? theme.collection === 'studio'
        : filter.value === 'light'
          ? theme.palette.light
          : !theme.palette.light)
    const text = [theme.id, theme.name, translate(theme.name)].join(' ').toLocaleLowerCase()
    return matches && text.includes(query.value.trim().toLocaleLowerCase())
  }),
)
async function selectTheme(theme) {
  if (saving.value || theme.id === active.value.id) return
  saving.value = true
  const previous = themeStore.selectedTheme
  themeStore.setThemeState(theme.id)
  notice.value = tr('Сохраняем оформление…', 'Saving appearance…')
  saveError.value = false
  try {
    // Merge into fresh native settings, not a stale page snapshot.
    const settings = await get()
    await set({ ...settings, theme: theme.id })
    notice.value = tr('Сохранено.', 'Saved.')
  } catch {
    themeStore.setThemeState(previous)
    saveError.value = true
    notice.value = tr(
      'Не удалось сохранить тему. Вернули предыдущее оформление. Попробуй ещё раз.',
      'Could not save theme. Previous appearance restored. Try again.',
    )
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <main class="themes-page">
    <header class="themes-heading">
      <div>
        <p class="studio-eyebrow">U3 STUDIO / APPEARANCE</p>
        <div class="title-line">
          <h1>{{ $u3t('Темы') }}</h1>
          <span class="collection-count">
            {{ CUSTOM_THEMES.length + BASE_THEMES.length }} {{ tr('тем', 'themes') }}
          </span>
        </div>
        <p class="heading-copy">
          {{
            tr(
              'Цвет, фактура и движение. Создай свою атмосферу.',
              'Color, texture, and motion. Make it yours.',
            )
          }}
        </p>
      </div>
      <span class="saved-indicator"
        ><CheckIcon aria-hidden="true" /> {{ $u3t('Выбор сохраняется') }}
      </span>
    </header>

    <section class="active-theme-panel" aria-labelledby="current-theme-heading">
      <div class="active-theme-copy">
        <div class="section-kicker">
          <span class="live-dot" /> {{ tr('Текущая тема', 'Current theme') }}
          <span>{{ activeNumber }}</span>
        </div>
        <h2 id="current-theme-heading">{{ $u3t(active.name) }}</h2>
        <div class="active-swatches" aria-hidden="true">
          <span
            v-for="color in [
              active.palette.bg,
              active.palette.surface,
              active.palette.accent,
              active.palette.secondary,
            ]"
            :key="color"
            :style="{ background: color }"
          />
          <span class="effect-caption">{{ $u3t(active.effect) }}</span>
        </div>
        <div class="active-label">
          <CheckIcon aria-hidden="true" />
          {{ saving ? tr('Сохраняем…', 'Saving…') : $u3t('Применена') }}
        </div>
      </div>
      <ThemePreview :theme="active" large />
    </section>

    <section class="effects-panel" aria-labelledby="effects-heading">
      <div class="effect-toggle-row">
        <div class="effect-intro">
          <h2 id="effects-heading">{{ tr('Эффекты', 'Effects') }}</h2>
        </div>
        <label class="motion-switch">
          <input
            type="checkbox"
            :checked="themeStore.effectsEnabled"
            @change="themeStore.setEffectsEnabled($event.target.checked)"
          />
          <span class="switch-track" aria-hidden="true"><span /></span>
          <span> {{ $u3t('Летающие частицы') }} </span>
        </label>
      </div>
      <div class="quality-row">
        <span id="quality-label"> {{ $u3t('Насыщенность эффектов') }} </span>
        <div class="quality-options" role="group" aria-labelledby="quality-label">
          <button
            v-for="level in EFFECT_LEVELS"
            :key="level.id"
            type="button"
            :aria-pressed="themeStore.effectsQuality === level.id"
            :disabled="!themeStore.effectsEnabled"
            @click="themeStore.setEffectsQuality(level.id)"
          >
            {{ $u3t(level.name) }}
          </button>
        </div>
      </div>
      <div class="quality-row jelly-settings">
        <label class="motion-switch">
          <input
            type="checkbox"
            :checked="themeStore.jellyEnabled"
            @change="themeStore.setJellyEnabled($event.target.checked)"
          />
          <span class="switch-track" aria-hidden="true"><span /></span>
          <span>{{ tr('Желейные анимации', 'Jelly animations') }}</span>
        </label>
        <div
          class="quality-options"
          role="group"
          :aria-label="tr('Характер анимаций', 'Motion style')"
        >
          <button
            v-for="level in ['soft', 'spring']"
            :key="level"
            type="button"
            :disabled="!themeStore.jellyEnabled"
            :aria-pressed="themeStore.jellyLevel === level"
            @click="themeStore.setJellyLevel(level)"
          >
            {{ level === 'soft' ? tr('Мягкие', 'Soft') : tr('Упругие', 'Springy') }}
          </button>
        </div>
      </div>
      <p v-if="themeStore.reducedMotion" class="motion-note" role="status">
        {{
          $u3t(
            'Движение приостановлено: в системе включено уменьшение анимации. Превью остаются статичными.',
          )
        }}
      </p>
      <p v-else-if="!active.particle" class="motion-note">
        {{
          tr(
            'Выбери авторскую тему, чтобы добавить частицы.',
            'Choose a custom theme to add particles.',
          )
        }}
      </p>
      <p v-else class="motion-note">
        {{ tr('30 кадров/с · пауза в фоне', '30 fps · paused in background') }}
      </p>
      <p v-if="themeStore.effectsStorageError" class="save-notice error" role="alert">
        {{
          tr(
            'Эффекты применены, но их настройки не удалось сохранить для следующего запуска.',
            'Effects applied, but preferences could not be saved for the next launch.',
          )
        }}
      </p>
    </section>

    <section class="collection" aria-labelledby="collection-heading">
      <div class="collection-heading">
        <h2 id="collection-heading">{{ tr('Все темы', 'All themes') }}</h2>
        <span> {{ tr('19 новых атмосфер', '19 new atmospheres') }} </span>
      </div>
      <div class="collection-toolbar">
        <div class="filter-options" role="group" :aria-label="tr('Фильтр тем', 'Theme filter')">
          <button
            v-for="item in filters"
            :key="item.id"
            type="button"
            :aria-pressed="filter === item.id"
            @click="filter = item.id"
          >
            {{ item.name }}
          </button>
        </div>
        <input
          v-model="query"
          type="search"
          :placeholder="tr('Найти свою тему…', 'Find your theme…')"
          :aria-label="tr('Поиск тем', 'Search themes')"
        />
      </div>
      <p v-if="!filteredThemes.length" class="empty-themes" role="status">
        {{
          tr(
            'Такой темы пока нет. Попробуй другой запрос.',
            'No matching themes. Try another search.',
          )
        }}
      </p>
      <div class="theme-grid" :aria-busy="saving">
        <button
          v-for="theme in filteredThemes"
          :key="theme.id"
          type="button"
          class="theme-choice"
          :class="{ selected: active.id === theme.id }"
          :aria-pressed="active.id === theme.id"
          :aria-label="tr(`Выбрать тему «${theme.name}»`, `Select ${translate(theme.name)} theme`)"
          :disabled="saving"
          :data-theme-id="theme.id"
          @click="selectTheme(theme)"
        >
          <div class="card-preview-wrap">
            <ThemePreview :theme="theme" />
            <span v-if="theme.collection === 'studio'" class="new-theme-badge">STUDIO / 19</span>
            <span class="theme-index">{{
              String(CUSTOM_THEMES.indexOf(theme) + 1).padStart(2, '0')
            }}</span>
            <span v-if="active.id === theme.id" class="selected-mark"
              ><CheckIcon aria-hidden="true" /><span>{{ tr('Выбрана', 'Selected') }}</span></span
            >
          </div>
          <div class="theme-card-copy">
            <div class="card-title-line">
              <h3>{{ $u3t(theme.name) }}</h3>
              <span class="card-swatches" aria-hidden="true"
                ><i :style="{ background: theme.palette.accent }" /><i
                  :style="{ background: theme.palette.secondary }"
              /></span>
            </div>
            <span class="particle-label">{{ $u3t(theme.effect) }}</span>
          </div>
        </button>
      </div>
    </section>

    <section class="base-themes" aria-labelledby="base-themes-heading">
      <div>
        <h2 id="base-themes-heading">{{ tr('Базовые темы', 'Basic themes') }}</h2>
      </div>
      <div class="base-theme-options">
        <button
          v-for="theme in BASE_THEMES"
          :key="theme.id"
          type="button"
          :disabled="saving"
          :aria-pressed="active.id === theme.id"
          :data-theme-id="theme.id"
          @click="selectTheme(theme)"
        >
          <span class="base-dot" :style="{ background: theme.palette.bg }" aria-hidden="true" />{{
            $u3t(theme.name)
          }}<CheckIcon v-if="active.id === theme.id" aria-hidden="true" />
        </button>
      </div>
    </section>
    <p
      class="save-notice"
      :class="{ error: saveError }"
      :role="saveError ? 'alert' : 'status'"
      aria-live="polite"
    >
      {{ notice }}
    </p>
  </main>
</template>

<style scoped>
.studio-eyebrow {
  margin: 0 0 12px;
  color: var(--color-brand);
  font-size: 10px;
  letter-spacing: 0.18em;
  font-weight: 700;
}
.collection-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 14px;
  margin-bottom: 20px;
}
.filter-options {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
}
.filter-options button {
  border: 1px solid transparent;
  padding: 9px 13px;
  background: transparent;
  color: var(--color-secondary);
  border-radius: 14px;
  cursor: pointer;
}
.filter-options button[aria-pressed='true'] {
  background: var(--color-brand-highlight);
  color: var(--color-contrast);
  border-color: var(--theme-border);
}
.collection-toolbar input {
  min-width: 0;
  width: 240px;
  padding: 11px 14px;
  background: var(--color-raised-bg);
  color: var(--color-base);
  border: 1px solid var(--theme-border);
  border-radius: 18px;
}
.collection-toolbar input:focus-visible,
.filter-options button:focus-visible {
  outline: 2px solid var(--color-brand);
  outline-offset: 3px;
}
.new-theme-badge {
  position: absolute;
  top: 12px;
  right: 12px;
  font-size: 9px;
  font-weight: 750;
  letter-spacing: 0.08em;
  background: var(--color-bg);
  color: var(--color-base);
  border: 1px solid var(--theme-border);
  border-radius: 6px;
  padding: 5px 7px;
}
.card-description {
  font-size: 12px;
  line-height: 1.5;
  color: var(--color-secondary);
  margin: 7px 0 12px;
  min-height: 36px;
}
.empty-themes {
  border: 1px dashed var(--theme-border);
  border-radius: 16px;
  padding: 36px 20px;
  color: var(--color-secondary);
}

.themes-page {
  max-width: 1248px;
  margin: 0 auto;
  padding: 32px 36px 24px;
  color: var(--color-base);
  --theme-border: var(--color-button-bg);
}
:global(html[data-custom-theme]) .themes-page {
  --theme-border: inherit;
}
.themes-heading {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 24px;
  margin-bottom: 28px;
}
.eyebrow {
  margin: 0 0 10px;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 700;
  letter-spacing: 0.16em;
  color: var(--color-secondary);
}
.eyebrow svg {
  width: 16px;
  height: 16px;
  color: var(--color-brand);
}
.title-line {
  display: flex;
  align-items: center;
  gap: 14px;
}
h1 {
  font-size: 36px;
  line-height: 1.15;
  margin: 0;
  font-weight: 700;
  letter-spacing: -0.04em;
}
.collection-count {
  background: var(--color-brand-highlight);
  color: var(--color-contrast);
  border: 1px solid var(--theme-border);
  border-radius: 6px;
  padding: 5px 9px;
  font-size: 14px;
  font-weight: 600;
}
.heading-copy {
  margin: 12px 0 0;
  color: var(--color-secondary);
  font-size: 16px;
  line-height: 1.5;
}
.saved-indicator {
  display: flex;
  gap: 7px;
  align-items: center;
  white-space: nowrap;
  font-size: 14px;
  color: var(--color-secondary);
}
.saved-indicator svg {
  width: 16px;
  height: 16px;
  color: var(--color-brand);
}
.active-theme-panel {
  position: relative;
  display: grid;
  grid-template-columns: 1fr 42%;
  gap: 32px;
  align-items: center;
  padding: 28px;
  border: 1px solid var(--theme-border);
  border-radius: 18px;
  background: radial-gradient(ellipse at top left, var(--color-brand-highlight), transparent 70%),
    var(--color-raised-bg);
  overflow: hidden;
}
.active-theme-copy {
  min-width: 0;
}
.section-kicker {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
  color: var(--color-secondary);
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 0.09em;
}
.section-kicker > span:last-child {
  margin-left: 8px;
  font-variant-numeric: tabular-nums;
}
.live-dot {
  display: inline-block;
  width: 6px;
  height: 6px;
  background: var(--color-brand);
  border-radius: 50%;
}
.active-theme-copy h2 {
  margin: 18px 0 10px;
  font-size: 30px;
  font-weight: 600;
  line-height: 1.2;
  letter-spacing: -0.035em;
  color: var(--color-contrast);
}
.active-theme-copy > p {
  max-width: 370px;
  margin: 0;
  color: var(--color-secondary);
  font-size: 15px;
  line-height: 1.55;
}
.active-swatches {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 22px;
  flex-wrap: wrap;
}
.active-swatches > span:not(.effect-caption) {
  width: 21px;
  height: 21px;
  border: 1px solid var(--theme-border);
  border-radius: 50%;
}
.effect-caption {
  font-size: 14px;
  margin-left: 6px;
  color: var(--color-secondary);
}
.active-label {
  margin-top: 18px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: var(--color-brand);
  font-size: 14px;
  font-weight: 600;
}
.active-label svg {
  width: 16px;
  height: 16px;
}
.effects-panel {
  border: 1px solid var(--theme-border);
  border-radius: 18px;
  padding: 22px 24px 18px;
  background: var(--color-raised-bg);
  margin: 16px 0 32px;
}
.effect-toggle-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
}
.effect-intro h2,
.collection-heading h2,
.base-themes h2 {
  margin: 0;
  color: var(--color-contrast);
  font-size: 18px;
  line-height: 1.4;
  font-weight: 600;
  letter-spacing: -0.02em;
}
.effect-intro p,
.base-themes p {
  margin: 5px 0 0;
  font-size: 14px;
  line-height: 1.5;
  color: var(--color-secondary);
}
.motion-switch {
  position: relative;
  display: flex;
  align-items: center;
  gap: 10px;
  min-height: 44px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  flex-shrink: 0;
}
.motion-switch input {
  position: absolute;
  width: 44px;
  height: 28px;
  opacity: 0;
  margin: 0;
}
.switch-track {
  pointer-events: none;
  width: 44px;
  height: 26px;
  border: 1px solid var(--color-secondary);
  border-radius: 20px;
  display: flex;
  align-items: center;
  padding: 3px;
  background: var(--color-bg);
  transition: background 0.16s;
}
.switch-track > span {
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: var(--color-secondary);
  transition: transform 0.16s;
}
.motion-switch input:checked + .switch-track {
  pointer-events: none;
  background: var(--color-brand);
  border-color: var(--color-brand);
}
.motion-switch input:checked + .switch-track > span {
  background: var(--color-accent-contrast);
  transform: translateX(17px);
}
.motion-switch input:focus-visible + .switch-track {
  pointer-events: none;
  outline: 2px solid var(--color-brand);
  outline-offset: 4px;
}
.quality-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  margin-top: 18px;
  padding-top: 16px;
  border-top: 1px solid var(--theme-border);
  font-size: 14px;
}
.quality-options {
  display: flex;
  gap: 4px;
  padding: 4px;
  border-radius: 8px;
  background: var(--color-bg);
}
.quality-options button {
  min-height: 44px;
  border-radius: 5px;
  padding: 8px 14px;
  color: var(--color-secondary);
  background: transparent;
  border: 1px solid transparent;
  font-size: 14px;
  cursor: pointer;
}
.quality-options button[aria-pressed='true'] {
  color: var(--color-contrast);
  background: var(--color-raised-bg);
  border-color: var(--theme-border);
}
.quality-options button:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
.motion-note {
  margin: 12px 0 0;
  font-size: 14px;
  line-height: 1.5;
  color: var(--color-secondary);
}
.collection-heading {
  display: flex;
  align-items: baseline;
  gap: 16px;
  justify-content: space-between;
  margin-bottom: 16px;
}
.collection-heading > span {
  font-size: 14px;
  color: var(--color-secondary);
  white-space: nowrap;
}
.theme-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 16px;
}
.theme-choice {
  display: block;
  padding: 0;
  margin: 0;
  min-width: 0;
  border: 1px solid var(--theme-border);
  border-radius: 14px;
  background: var(--color-raised-bg);
  color: var(--color-base);
  text-align: left;
  font: inherit;
  cursor: pointer;
  overflow: hidden;
  transition:
    border-color 0.16s,
    transform 0.16s;
}
.theme-choice:not(:disabled):hover {
  transform: translateY(-3px);
  border-color: var(--color-brand);
}
.theme-choice.selected {
  border-color: var(--color-brand);
  box-shadow: 0 0 0 1px var(--color-brand);
}
.theme-choice:disabled {
  cursor: progress;
}
.theme-choice:focus-visible,
.quality-options button:focus-visible,
.base-theme-options button:focus-visible {
  outline: 2px solid var(--color-brand);
  outline-offset: 4px;
}
.card-preview-wrap {
  position: relative;
  border-bottom: 1px solid var(--theme-border);
}
.theme-index {
  position: absolute;
  top: 12px;
  left: 12px;
  color: #edf4ff;
  background: #10171de6;
  border: 1px solid #ffffff35;
  border-radius: 4px;
  padding: 3px 5px;
  font-size: 14px;
  line-height: 1.25;
  font-variant-numeric: tabular-nums;
}
.selected-mark {
  position: absolute;
  top: 10px;
  right: 10px;
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 5px 7px;
  border-radius: 5px;
  color: var(--color-accent-contrast);
  background: var(--color-brand);
  font-size: 14px;
  font-weight: 600;
}
.selected-mark svg {
  width: 14px;
  height: 14px;
}
.theme-card-copy {
  padding: 16px;
}
.card-title-line {
  display: flex;
  gap: 12px;
  align-items: center;
  justify-content: space-between;
}
.card-title-line h3 {
  font-size: 16px;
  line-height: 1.4;
  font-weight: 600;
  margin: 0;
  color: var(--color-contrast);
  letter-spacing: -0.02em;
}
.card-swatches {
  display: flex;
  flex-shrink: 0;
  padding-right: 3px;
}
.card-swatches i {
  display: block;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: 1px solid #ffffff55;
}
.card-swatches i + i {
  margin-left: -3px;
}
.theme-card-copy p {
  margin: 7px 0 12px;
  min-height: 42px;
  font-size: 14px;
  line-height: 1.5;
  color: var(--color-secondary);
}
.particle-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
  color: var(--color-base);
}
.particle-label > span {
  color: var(--color-brand);
  font-size: 18px;
  line-height: 1;
}
.base-themes {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  align-items: center;
  gap: 20px;
  margin-top: 28px;
  padding: 24px;
  background: var(--color-raised-bg);
  border: 1px solid var(--theme-border);
  border-radius: 18px;
}
.base-theme-options {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.base-theme-options button {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  min-height: 44px;
  padding: 10px 12px;
  border-radius: 6px;
  border: 1px solid var(--theme-border);
  background: var(--color-bg);
  font-size: 14px;
  color: var(--color-base);
  cursor: pointer;
}
.base-theme-options button[aria-pressed='true'] {
  border-color: var(--color-brand);
}
.base-theme-options svg {
  width: 14px;
  height: 14px;
  color: var(--color-brand);
}
.base-dot {
  width: 13px;
  height: 13px;
  border: 1px solid var(--color-secondary);
  border-radius: 50%;
}
.save-notice {
  min-height: 22px;
  font-size: 14px;
  line-height: 1.5;
  margin: 16px 0 0;
  color: var(--color-secondary);
}
.save-notice.error {
  color: var(--color-red);
}
@media (max-width: 1150px) {
  .themes-page {
    padding: 28px 24px 24px;
  }
  .active-theme-panel {
    gap: 20px;
    padding: 24px;
  }
  .saved-indicator {
    display: none;
  }
}
@media (max-width: 900px) {
  .theme-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
  .active-theme-panel {
    grid-template-columns: 1fr;
  }
  .active-theme-panel > .theme-preview {
    display: none;
  }
  .effect-toggle-row {
    flex-wrap: wrap;
    gap: 12px;
  }
}
@media (max-width: 600px) {
  .collection-toolbar input {
    width: 100%;
  }
  .title-line {
    flex-wrap: wrap;
  }
  .themes-page {
    padding: 24px 16px;
  }
  h1 {
    font-size: 30px;
  }
  .theme-grid {
    grid-template-columns: minmax(0, 1fr);
  }
  .active-theme-panel {
    padding: 20px;
  }
  .active-theme-copy h2 {
    font-size: 27px;
  }
  .effects-panel {
    padding: 20px 16px;
  }
  .quality-row {
    align-items: flex-start;
    flex-direction: column;
  }
  .quality-options {
    width: 100%;
  }
  .quality-options button {
    padding: 8px;
    flex: 1;
  }
  .collection-heading {
    align-items: flex-start;
    flex-direction: column;
    gap: 6px;
  }
  .base-themes {
    padding: 20px 16px;
  }
  .theme-card-copy p {
    min-height: auto;
  }
}
@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    transition: none !important;
  }
  .theme-choice:hover {
    transform: none !important;
  }
}
</style>
