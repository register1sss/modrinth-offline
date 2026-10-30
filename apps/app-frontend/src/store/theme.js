import { defineStore } from 'pinia'
import { getTheme, THEME_IDS, themeVariables, EFFECT_LEVELS } from '@/themes/catalog.js'

const EFFECTS_KEY = 'modrinth-offline:atmosphere:v1'
function loadEffects() {
  try {
    const data = JSON.parse(localStorage.getItem(EFFECTS_KEY) ?? '{}')
    return {
      jellyEnabled: typeof data.jelly === 'boolean' ? data.jelly : true,
      jellyLevel: data.jellyLevel === 'soft' ? 'soft' : 'spring',
      effectsEnabled: typeof data.enabled === 'boolean' ? data.enabled : true,
      effectsQuality: EFFECT_LEVELS.some((level) => level.id === data.quality)
        ? data.quality
        : 'balanced',
    }
  } catch {
    return {
      effectsEnabled: true,
      effectsQuality: 'balanced',
      jellyEnabled: true,
      jellyLevel: 'spring',
    }
  }
}
const managedProperties = Object.keys(themeVariables(getTheme('aurora')))

export const useTheming = defineStore('themeStore', {
  state: () => ({
    themeOptions: [...THEME_IDS],
    advancedRendering: true,
    selectedTheme: 'dark',
    reducedMotion: false,
    effectsStorageError: false,
    ...loadEffects(),
  }),
  getters: {
    activeTheme: (state) => getTheme(state.selectedTheme),
  },
  actions: {
    setThemeState(newTheme) {
      if (!this.themeOptions.includes(newTheme)) return false
      this.selectedTheme = newTheme
      this.setThemeClass()
      return true
    },
    setThemeClass() {
      const root = document.documentElement
      for (const id of this.themeOptions) root.classList.remove(`${id}-mode`)
      for (const property of managedProperties) root.style.removeProperty(property)
      const theme = this.activeTheme
      root.dataset.material = theme.material || (theme.palette.light ? 'paper' : 'glass')
      this.syncMotionState()
      const base = theme.id === 'oled' ? 'oled' : theme.palette.light ? 'light' : 'dark'
      root.classList.add(`${base}-mode`)
      root.style.colorScheme = theme.palette.light ? 'light' : 'dark'
      if (theme.particle) {
        root.dataset.customTheme = theme.id
        for (const [name, value] of Object.entries(themeVariables(theme)))
          root.style.setProperty(name, value)
      } else {
        delete root.dataset.customTheme
      }
    },
    saveEffects() {
      try {
        localStorage.setItem(
          EFFECTS_KEY,
          JSON.stringify({
            enabled: this.effectsEnabled,
            quality: this.effectsQuality,
            jelly: this.jellyEnabled,
            jellyLevel: this.jellyLevel,
          }),
        )
        this.effectsStorageError = false
      } catch {
        // Effects still apply for this session; the UI explains that persistence failed.
        this.effectsStorageError = true
      }
    },
    syncMotionState() {
      document.documentElement.dataset.jelly =
        this.jellyEnabled && !this.reducedMotion ? 'on' : 'off'
      document.documentElement.dataset.jellyLevel = this.jellyLevel
    },
    setReducedMotion(value) {
      this.reducedMotion = Boolean(value)
      this.syncMotionState()
    },
    setJellyEnabled(value) {
      this.jellyEnabled = Boolean(value)
      this.syncMotionState()
      this.saveEffects()
    },
    setJellyLevel(value) {
      if (!['soft', 'spring'].includes(value)) return
      this.jellyLevel = value
      this.syncMotionState()
      this.saveEffects()
    },
    setEffectsEnabled(enabled) {
      this.effectsEnabled = Boolean(enabled)
      this.saveEffects()
    },
    setEffectsQuality(quality) {
      if (!EFFECT_LEVELS.some((level) => level.id === quality)) return
      this.effectsQuality = quality
      this.saveEffects()
    },
  },
})
