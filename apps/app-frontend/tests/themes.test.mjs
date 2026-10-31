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
import { createParticles, advanceParticles, drawParticles } from '../src/themes/particles.js'

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
  assert.equal(new Set(CUSTOM_THEMES.map((t) => t.particle)).size, 13)
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
test('preview particles are deterministic and static scenes stay small', () => {
  for (const theme of CUSTOM_THEMES) {
    const first = createParticles(theme, 300, 140, 'low', true)
    assert.deepEqual(first, createParticles(theme, 300, 140, 'low', true))
    assert.equal(first.length, 16)
  }
})
test('particle budgets are bounded, scale down with viewport, and respect quality', () => {
  const t = CUSTOM_THEMES[0]
  const count = (q, w = 1280, h = 800) => createParticles(t, w, h, q).length
  assert.equal(count('low'), 28)
  assert.equal(count('balanced'), 52)
  assert.equal(count('high'), 84)
  assert.ok(count('high', 390, 800) < count('high'))
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
      assert.ok(p.x >= -28 && p.x <= 628 && p.y >= -28 && p.y <= 428)
    }
  }
})
test('all thirteen drawing paths execute independently without leaking canvas state', () => {
  const signatures = new Set()
  for (const theme of CUSTOM_THEMES) {
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
    const p = { x: 10, y: 10, size: 3, rotation: 0.5, phase: 1, alpha: 0.5, color: '#ffffff' }
    drawParticles(ctx, [p], theme.particle, 600, 400)
    assert.equal(calls.filter((c) => c[0] === 'save').length, 1)
    assert.equal(calls.filter((c) => c[0] === 'restore').length, 1)
    signatures.add(JSON.stringify(calls))
  }
  assert.equal(signatures.size, 13)
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
