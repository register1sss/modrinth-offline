// Stable IDs are shared with the native Settings::Theme enum. Never persist translated labels.
const palette = (bg, surface, raised, text, muted, accent, secondary, light = false) => ({
  bg,
  surface,
  raised,
  text,
  muted,
  accent,
  secondary,
  light,
})

export const CUSTOM_THEMES = Object.freeze(
  [
    {
      id: 'aurora',
      name: 'Северное сияние',
      subtitle: 'Изумрудный свет в полярной ночи',
      particle: 'firefly',
      effect: 'Светлячки',
      palette: palette('#081b21', '#102b32', '#1c3c43', '#e5f6ef', '#aac9c5', '#73e6bb', '#82adff'),
    },
    {
      id: 'neon',
      name: 'Неоновый Токио',
      subtitle: 'Фуксия, электрический циан и ночной город',
      particle: 'rain',
      effect: 'Неоновый дождь',
      palette: palette('#161222', '#251c36', '#3b2c50', '#f7ecff', '#c4b3d9', '#f3a2ff', '#62e1ef'),
    },
    {
      id: 'ember',
      name: 'Пепел и пламя',
      subtitle: 'Угольные оттенки и тёплая медь',
      particle: 'ember',
      effect: 'Искры огня',
      palette: palette('#1c1210', '#2e1e19', '#493024', '#ffecdf', '#d9b7a2', '#ffb775', '#ffe09b'),
    },
    {
      id: 'sakura',
      name: 'Цветение сакуры',
      subtitle: 'Пыльная роза и мягкий сливовый сумрак',
      particle: 'petal',
      effect: 'Лепестки сакуры',
      palette: palette('#231521', '#352331', '#503449', '#ffeafa', '#d8b5cc', '#ffb4d5', '#c8b9ff'),
    },
    {
      id: 'ocean',
      name: 'Глубины океана',
      subtitle: 'Тёмный сапфир с бирюзовыми бликами',
      particle: 'bubble',
      effect: 'Пузырьки воздуха',
      palette: palette('#091a2b', '#102b43', '#1a405a', '#e6f4ff', '#acc7dc', '#71d6f7', '#9eaaff'),
    },
    {
      id: 'orbit',
      name: 'Полночная орбита',
      subtitle: 'Космический индиго и свет далёких звёзд',
      particle: 'star',
      effect: 'Мерцающие звёзды',
      palette: palette('#111429', '#1d2341', '#303b60', '#edf0ff', '#b9c3e2', '#acbcff', '#dcadff'),
    },
    {
      id: 'forest',
      name: 'Зачарованный лес',
      subtitle: 'Хвойная зелень, мох и солнечные просветы',
      particle: 'leaf',
      effect: 'Парящие листья',
      palette: palette('#111d16', '#1c3023', '#304735', '#eef5df', '#b5c9ae', '#b3df89', '#e2ca88'),
    },
    {
      id: 'frost',
      name: 'Хрустальный иней',
      subtitle: 'Светлый лёд и чистые голубые акценты',
      particle: 'snow',
      effect: 'Снежинки',
      palette: palette(
        '#edf5fa',
        '#ffffff',
        '#dceaf3',
        '#17384b',
        '#496575',
        '#176b95',
        '#446db3',
        true,
      ),
    },
    {
      id: 'dune',
      name: 'Пески времени',
      subtitle: 'Молочный песок, терракота и бронза',
      particle: 'sand',
      effect: 'Золотая пыль',
      palette: palette(
        '#f5eddf',
        '#fffbf4',
        '#e9d9c1',
        '#44301f',
        '#755a40',
        '#994b26',
        '#6f7142',
        true,
      ),
    },
    {
      id: 'amethyst',
      name: 'Аметистовая пещера',
      subtitle: 'Глубокий фиолетовый и кристальный свет',
      particle: 'crystal',
      effect: 'Грани кристаллов',
      palette: palette('#1b1429', '#2b203e', '#44335b', '#f5eaff', '#cbb6dd', '#d6afff', '#89dbe1'),
    },
    {
      id: 'eclipse',
      name: 'Багровое затмение',
      subtitle: 'Обсидиан, рубин и красное свечение',
      particle: 'comet',
      effect: 'Рубиновые кометы',
      palette: palette('#211116', '#361e26', '#522e38', '#ffedf1', '#d9b3bd', '#ffa6b8', '#ffcc9e'),
    },
    {
      id: 'noir',
      name: 'Золото и бархат',
      subtitle: 'Чёрный графит и сдержанное шампанское',
      particle: 'ribbon',
      effect: 'Шёлковые нити',
      palette: palette('#171613', '#292720', '#403c30', '#faf1db', '#ccc1a6', '#e7c983', '#c3c8d2'),
    },
    {
      id: 'lagoon',
      name: 'Живая лагуна',
      subtitle: 'Глубокая бирюза и биолюминесцентный свет',
      particle: 'plankton',
      effect: 'Светящийся планктон',
      palette: palette('#081f23', '#103238', '#1a4850', '#e1fbf7', '#a8d1cd', '#7cead7', '#b7c9ff'),
    },
  ].map((theme) => Object.freeze({ ...theme, palette: Object.freeze(theme.palette) })),
)

export const BASE_THEMES = Object.freeze([
  {
    id: 'dark',
    name: 'Тёмная',
    effect: 'Без частиц',
    particle: null,
    palette: palette('#16181c', '#26292f', '#40434a', '#ecf9fb', '#b0bac5', '#1bd96a', '#4f9cff'),
  },
  {
    id: 'light',
    name: 'Светлая',
    effect: 'Без частиц',
    particle: null,
    palette: palette(
      '#e5e7eb',
      '#ffffff',
      '#dce0e7',
      '#1a202c',
      '#596472',
      '#087a46',
      '#1f68c0',
      true,
    ),
  },
  {
    id: 'oled',
    name: 'OLED',
    effect: 'Без частиц',
    particle: null,
    palette: palette('#000000', '#101013', '#222329', '#ecf9fb', '#b0bac5', '#1bd96a', '#4f9cff'),
  },
])
export const ALL_THEMES = [...BASE_THEMES, ...CUSTOM_THEMES]
export const THEME_IDS = ALL_THEMES.map((theme) => theme.id)
export const EFFECT_LEVELS = Object.freeze([
  { id: 'low', name: 'Лёгкие', count: 28 },
  { id: 'balanced', name: 'Баланс', count: 52 },
  { id: 'high', name: 'Насыщенные', count: 84 },
])
export const getTheme = (id) => ALL_THEMES.find((theme) => theme.id === id) ?? BASE_THEMES[0]
export const rgba = (hex, alpha) => {
  const value = Number.parseInt(hex.slice(1), 16)
  return `rgba(${(value >> 16) & 255}, ${(value >> 8) & 255}, ${value & 255}, ${alpha})`
}

export function themeVariables(theme) {
  const p = theme.palette
  return {
    '--app-view-bg': 'transparent',
    '--app-router-bg': 'transparent',
    '--color-bg': p.bg,
    '--color-raised-bg': p.surface,
    '--color-super-raised-bg': p.raised,
    '--color-button-bg': p.raised,
    '--color-scrollbar': p.muted,
    '--color-base': p.text,
    '--color-contrast': p.text,
    '--color-secondary': p.muted,
    '--color-accent-contrast': p.light ? '#ffffff' : '#101719',
    '--color-brand': p.accent,
    '--color-link': p.accent,
    '--color-brand-highlight': rgba(p.accent, p.light ? 0.12 : 0.16),
    '--color-brand-shadow': rgba(p.accent, 0.3),
    '--theme-secondary': p.secondary,
    '--theme-border': rgba(p.muted, p.light ? 0.7 : 0.66),
    '--theme-glow': rgba(p.accent, p.light ? 0.06 : 0.08),
    '--theme-glow-secondary': rgba(p.secondary, p.light ? 0.05 : 0.07),
  }
}
