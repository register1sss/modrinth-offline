import { CUSTOM_THEMES, EFFECT_LEVELS } from './catalog.js'

export const EFFECTS_KEY = 'modrinth-offline:atmosphere:v1'
export const FAVORITES_KEY = 'u3:favorite-themes:v1'
const defaults = () => ({
  effectsEnabled: true,
  effectsQuality: 'balanced',
  jellyEnabled: true,
  jellyLevel: 'spring',
})
export function readEffects(storage) {
  try {
    const value = JSON.parse(storage.getItem(EFFECTS_KEY) || '{}')
    if (!value || typeof value !== 'object' || Array.isArray(value)) return defaults()
    return {
      effectsEnabled: typeof value.enabled === 'boolean' ? value.enabled : true,
      effectsQuality: EFFECT_LEVELS.some((l) => l.id === value.quality)
        ? value.quality
        : 'balanced',
      jellyEnabled: typeof value.jelly === 'boolean' ? value.jelly : true,
      jellyLevel: value.jellyLevel === 'soft' ? 'soft' : 'spring',
    }
  } catch {
    return defaults()
  }
}
export function readFavorites(storage) {
  try {
    const value = JSON.parse(storage.getItem(FAVORITES_KEY) || '[]')
    return Array.isArray(value)
      ? [...new Set(value.filter((id) => CUSTOM_THEMES.some((t) => t.id === id)))]
      : []
  } catch {
    return []
  }
}
export const normalizeSearch = (value) =>
  String(value).normalize('NFKC').toLocaleLowerCase().replaceAll('ё', 'е').trim()

export function matchesTheme(theme, query, translated = '') {
  const text = normalizeSearch(
    [theme.id, theme.name, theme.effect, theme.subtitle, translated].join(' '),
  )
  return normalizeSearch(query)
    .split(/\s+/)
    .every((word) => text.includes(word))
}
