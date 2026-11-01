import test from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import {
  CUSTOM_THEMES,
  BASE_THEMES,
  THEME_IDS,
  getTheme,
  themeVariables,
} from '../src/themes/catalog.js'
import {
  createParticles,
  advanceParticles,
  drawParticles,
  createBurst,
  drawBurst,
  PARTICLE_PROFILES,
  particleProfile,
} from '../src/themes/particles.js'
import { PARTICLE_KINDS } from '../src/themes/particle-art.js'
import { PANEL_KINDS, drawPanel, panelScene } from '../src/themes/panel-art.js'
import { en } from '../src/i18n/messages.js'

// A canvas stand-in that records every call; gradients are recorded too.
function recorder() {
  const calls = []
  const ctx = new Proxy(
    {},
    {
      get:
        (_, method) =>
        (...args) => {
          calls.push([method, ...args])
          if (String(method).startsWith('create'))
            return { addColorStop: (...a) => calls.push(['stop', ...a]) }
        },
      set: () => true,
    },
  )
  return { ctx, calls }
}
const balanced = (calls) => {
  let depth = 0
  for (const [method] of calls) {
    if (method === 'save') depth++
    if (method === 'restore') depth--
    if (depth < 0) return false
  }
  return depth === 0
}

const luminance = (hex) => {
  const c = hex
    .slice(1)
    .match(/../g)
    .map((v) => Number.parseInt(v, 16) / 255)
    .map((v) => (v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4))
  return c[0] * 0.2126 + c[1] * 0.7152 + c[2] * 0.0722
}
const contrast = (a, b) => {
  const x = luminance(a),
    y = luminance(b)
  return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05)
}

test('32 custom palettes include 19 Studio additions and preserve three legacy themes', () => {
  assert.equal(CUSTOM_THEMES.length, 32)
  assert.equal(CUSTOM_THEMES.filter((t) => t.collection === 'studio').length, 19)
  assert.equal(new Set(CUSTOM_THEMES.map((t) => t.particle)).size, 32)
  assert.equal(new Set(THEME_IDS).size, 35)
  assert.equal(new Set(CUSTOM_THEMES.map((t) => t.palette.bg)).size, 32)
  assert.deepEqual(
    BASE_THEMES.map((t) => t.id),
    ['dark', 'light', 'oled'],
  )
})
test('every custom theme uses AA text and accent-label contrast', () => {
  for (const t of CUSTOM_THEMES) {
    for (const bg of [t.palette.bg, t.palette.surface, t.palette.raised]) {
      for (const fg of [t.palette.text, t.palette.muted])
        assert.ok(contrast(fg, bg) >= 4.5, `${t.id} ${fg} on ${bg}`)
    }
    assert.ok(contrast(t.palette.accent, themeVariables(t)['--color-accent-contrast']) >= 4.5, t.id)
    assert.ok(contrast(t.palette.accent, t.palette.surface) >= 3, t.id)
  }
})
test('stable frontend IDs are accepted by the native theme schema', () => {
  const native = readFileSync(
    new URL('../../../packages/app-lib/src/state/settings.rs', import.meta.url),
    'utf8',
  )
  for (const id of THEME_IDS) assert.ok(native.includes(`"${id}" => Theme::`), id)
  assert.equal(getTheme('missing').id, 'dark')
})
test('every custom theme owns a unique particle kind with art, motion, frame and a label', () => {
  const kinds = CUSTOM_THEMES.map((t) => t.particle)
  assert.equal(new Set(kinds).size, CUSTOM_THEMES.length)
  assert.equal(new Set(CUSTOM_THEMES.map((t) => t.effect)).size, CUSTOM_THEMES.length)
  for (const theme of CUSTOM_THEMES) {
    assert.ok(PARTICLE_KINDS.includes(theme.particle), theme.particle)
    assert.ok(PARTICLE_PROFILES[theme.particle], theme.particle)
    assert.ok(PANEL_KINDS.includes(theme.particle), theme.particle)
    assert.ok(en[theme.effect], `English label for ${theme.effect}`)
  }
})
test('preview particles are deterministic and static scenes stay small', () => {
  for (const theme of CUSTOM_THEMES) {
    const first = createParticles(theme, 300, 140, 'low', true)
    assert.deepEqual(first, createParticles(theme, 300, 140, 'low', true))
    const density = PARTICLE_PROFILES[theme.particle].density
    assert.equal(first.length, Math.min(24, Math.max(5, Math.round(16 * density))))
  }
})
test('particle budgets are bounded, scale down with viewport, and respect quality', () => {
  for (const t of CUSTOM_THEMES) {
    const density = PARTICLE_PROFILES[t.particle].density
    const count = (q, w = 1280, h = 800) => createParticles(t, w, h, q).length
    assert.equal(count('balanced'), Math.round(52 * density), t.id)
    assert.ok(count('low') <= count('balanced') && count('balanced') < count('high'), t.id)
    assert.ok(count('high') <= 84 * 1.3, t.id)
    assert.ok(count('high', 390, 800) < count('high'), t.id)
  }
  assert.equal(createParticles(BASE_THEMES[0], 1280, 800).length, 0)
})
test('all motion models stay finite, clamp long frame deltas, and wrap within bounds', () => {
  for (const theme of CUSTOM_THEMES) {
    const particles = createParticles(theme, 600, 400)
    for (let frame = 0; frame < 800; frame++)
      advanceParticles(
        particles,
        theme.particle,
        600,
        400,
        frame % 80 === 0 ? 15 : 1 / 30,
        frame / 30,
      )
    for (const p of particles) {
      assert.ok(Number.isFinite(p.x) && Number.isFinite(p.y) && Number.isFinite(p.rotation))
      const m = p.size * particleProfile(theme.particle, p.variant).extent + 8
      assert.ok(p.x >= -m && p.x <= 600 + m && p.y >= -m && p.y <= 400 + m, theme.id)
    }
  }
})
test('directional particles point where they travel and orbits stay around the centre', () => {
  const neon = CUSTOM_THEMES.find((t) => t.particle === 'rain')
  const rain = createParticles(neon, 800, 600)
  advanceParticles(rain, 'rain', 800, 600, 1 / 30, 1)
  for (const p of rain) {
    // Rain art is drawn along +y, so after rotation +y must follow the velocity.
    const dir = [-Math.sin(p.rotation), Math.cos(p.rotation)]
    const speed = Math.hypot(p.vx, p.vy)
    assert.ok(Math.abs(dir[0] - p.vx / speed) < 1e-9 && Math.abs(dir[1] - p.vy / speed) < 1e-9)
  }
  const fish = createParticles(
    CUSTOM_THEMES.find((t) => t.particle === 'fish'),
    800,
    600,
  )
  assert.ok(fish.some((p) => p.dir === 1) && fish.some((p) => p.dir === -1))
  for (const p of fish) assert.equal(Math.sign(p.vx), p.dir)
  const orbit = createParticles(
    CUSTOM_THEMES.find((t) => t.particle === 'orbit'),
    800,
    600,
  )
  for (let i = 0; i < 300; i++) advanceParticles(orbit, 'orbit', 800, 600, 1 / 30, i)
  for (const p of orbit) assert.ok(p.x >= 0 && p.x <= 800 && p.y >= 0 && p.y <= 600)
  // Petals tumble (3D flip) while code columns stay upright.
  const petals = createParticles(
    CUSTOM_THEMES.find((t) => t.particle === 'petal'),
    800,
    600,
  )
  advanceParticles(petals, 'petal', 800, 600, 1 / 30, 3)
  assert.ok(petals.some((p) => Math.abs(p.flip) < 0.99))
  const code = createParticles(
    CUSTOM_THEMES.find((t) => t.particle === 'code'),
    800,
    600,
  )
  advanceParticles(code, 'code', 800, 600, 1 / 30, 3)
  assert.ok(code.every((p) => p.rotation === 0 && p.vy > 0))
})
test('all 32 drawing paths execute independently without leaking canvas state', () => {
  const signatures = new Set()
  for (const theme of CUSTOM_THEMES) {
    const { ctx, calls } = recorder()
    const p = { x: 10, y: 10, size: 3, rotation: 0.5, phase: 1, alpha: 0.5, color: '#ffffff' }
    drawParticles(ctx, [p], theme.particle, 600, 400)
    assert.ok(calls.some((c) => c[0] === 'save'))
    assert.ok(balanced(calls), theme.particle)
    signatures.add(JSON.stringify(calls))
  }
  assert.equal(signatures.size, 32)
})
test('click bursts and bar ornaments draw every kind with balanced canvas state', () => {
  for (const theme of CUSTOM_THEMES) {
    const burst = createBurst(theme, 100, 100, 12, () => 0.42)
    assert.equal(burst.length, 12)
    const { ctx, calls } = recorder()
    for (const t of [0, 0.4, 1]) drawBurst(ctx, burst, theme.particle, t)
    assert.ok(balanced(calls), theme.particle)
    for (const vertical of [false, true]) {
      const panel = recorder()
      const w = vertical ? 72 : 1200,
        h = vertical ? 800 : 48
      drawPanel(panel.ctx, theme, w, h, vertical, 2.5, panelScene(theme, vertical))
      assert.ok(panel.calls.length > 20, theme.id)
      assert.ok(balanced(panel.calls), `${theme.id} ${vertical}`)
    }
  }
  assert.deepEqual(createBurst(BASE_THEMES[0], 0, 0, 5), [])
})

test('button springs settle exactly, stay bounded, and offer gentler motion', async () => {
  const { springFrames, motionAllowed } = await import('../src/visual/spring.js')
  for (const level of ['soft', 'spring']) {
    const frames = springFrames(level)
    assert.equal(frames.length, 41)
    assert.deepEqual(frames.at(-1), { offset: 1, scale: '1 1' })
    for (const frame of frames) {
      const values = frame.scale.split(' ').map(Number)
      assert.ok(values.every((v) => Number.isFinite(v) && v >= 0.9 && v <= 1.09))
    }
  }
  assert.ok(
    Number(springFrames('soft')[0].scale.split(' ')[0]) <
      Number(springFrames()[0].scale.split(' ')[0]),
  )
  for (const enabled of [true, false])
    for (const reduced of [true, false])
      for (const hidden of [true, false])
        assert.equal(motionAllowed({ enabled, reduced, hidden }), enabled && !reduced && !hidden)
})

test('welcome art stays uncluttered and modal dialogs escape filtered sidebars', () => {
  const welcome = readFileSync(
    new URL('../src/components/ui/StudioWelcome.vue', import.meta.url),
    'utf8',
  )
  for (const removed of [
    'art-label',
    'welcome-footer',
    'в твоей библиотеке',
    'translate(theme.activeTheme.name)',
  ])
    assert.ok(!welcome.includes(removed), removed)
  const modal = readFileSync(
    new URL('../../../packages/ui/src/components/modal/Modal.vue', import.meta.url),
    'utf8',
  )
  assert.ok(modal.includes('<Teleport to="body">'))
  assert.ok(modal.includes('role="dialog"'))
  assert.ok(modal.includes('width: min(600px, calc(100vw'))
  const spring = readFileSync(new URL('../src/visual/spring.js', import.meta.url), 'utf8')
  assert.ok(spring.includes("'pointerover', hover"))
  assert.ok(spring.includes("if (!CSS.supports('scale', '1'))"))
})
