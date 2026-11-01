import test from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import {
  parseLaunchArgs,
  formatLaunchArgs,
  parseEnvVars,
  nativeSettings,
  settingsChanges,
  createSettingsUpdater,
} from '../src/helpers/settings-model.js'
import { readEffects, readFavorites, matchesTheme } from '../src/themes/preferences.js'
import { CUSTOM_THEMES } from '../src/themes/catalog.js'
import { createParticles, advanceParticles } from '../src/themes/particles.js'

test('Java argument editor round-trips quotes, spaces, empty arguments and Windows paths', () => {
  const args = [
    '-Xmx4G',
    '-Dpath=C:\\Program Files\\Java\\',
    '',
    'a"b\'c',
    'two\twords',
    'C:\\java\\bin',
  ]
  assert.deepEqual(parseLaunchArgs(formatLaunchArgs(args)), args)
  assert.deepEqual(parseLaunchArgs('-Dname="My pack" -Xmx4G'), ['-Dname=My pack', '-Xmx4G'])
  assert.throws(() => parseLaunchArgs('-Dname="unfinished'))
})
test('environment variables preserve equals, empty values and quoted spaces', () => {
  const vars = [
    ['TOKEN', 'a=b=='],
    ['EMPTY', ''],
    ['PATH', 'C:\\Program Files\\Java'],
  ]
  assert.deepEqual(parseEnvVars(formatLaunchArgs(vars.map((v) => v.join('=')))), vars)
  assert.throws(() => parseEnvVars('BROKEN'))
  assert.throws(() => parseEnvVars('=value'))
})
test('editor-only fields never reach native settings; hooks and directory normalize', () => {
  const settings = {
    launchArgs: '-Xmx4G',
    envVars: 'A=b=c',
    hooks: { pre_launch: '' },
    custom_dir: '',
  }
  const result = nativeSettings(settings)
  assert.deepEqual(result.extra_launch_args, ['-Xmx4G'])
  assert.deepEqual(result.custom_env_vars, [['A', 'b=c']])
  assert.equal(result.custom_dir, null)
  assert.equal(result.hooks.pre_launch, null)
  assert.ok(!('envVars' in result) && !('launchArgs' in result))
  assert.equal(settings.custom_dir, '')
})
test('settings patches contain only edited properties', () => {
  assert.deepEqual(
    settingsChanges(
      { theme: 'dark', memory: { maximum: 4096 } },
      { theme: 'dark', memory: { maximum: 6144 } },
    ),
    { memory: { maximum: 6144 } },
  )
})
test('concurrent settings edits merge in order and recover after a failed write', async () => {
  let current = { theme: 'dark', memory: 4096 },
    fail = true
  const update = createSettingsUpdater(
    async () => ({ ...current }),
    async (value) => {
      await new Promise((r) => setTimeout(r, 5))
      if (fail) {
        fail = false
        throw new Error('disk unavailable')
      }
      current = value
    },
  )
  const results = await Promise.allSettled([
    update({ memory: 5000 }),
    update({ theme: 'aurora' }),
    update({ memory: 6144 }),
  ])
  assert.deepEqual(
    results.map((r) => r.status),
    ['rejected', 'fulfilled', 'fulfilled'],
  )
  assert.deepEqual(current, { theme: 'aurora', memory: 6144 })
})
test('patch queue captures values before the caller mutates them', async () => {
  let value
  const update = createSettingsUpdater(
    async () => ({}),
    async (v) => {
      value = v
    },
  )
  const patch = { memory: { maximum: 4096 } }
  const operation = update(patch)
  patch.memory.maximum = 0
  await operation
  assert.equal(value.memory.maximum, 4096)
})
test('atmosphere preferences tolerate null, arrays, corrupt JSON and denied storage', () => {
  for (const input of ['null', '[]', '"hello"', '{', '{"quality":"unknown"}']) {
    const value = readEffects({ getItem: () => input })
    assert.equal(value.effectsQuality, 'balanced')
    assert.equal(value.effectsEnabled, true)
  }
  assert.equal(
    readEffects({
      getItem: () => {
        throw Error('blocked')
      },
    }).effectsEnabled,
    true,
  )
  assert.equal(
    readEffects({ getItem: () => '{"enabled":false,"jelly":false,"jellyLevel":"soft"}' })
      .jellyEnabled,
    false,
  )
})
test('favorites reject unknown IDs and duplicates; search handles ё and multiple words', () => {
  assert.deepEqual(readFavorites({ getItem: () => '["aurora","missing","aurora",null]' }), [
    'aurora',
  ])
  assert.deepEqual(readFavorites({ getItem: () => '{}' }), [])
  assert.ok(matchesTheme(CUSTOM_THEMES[0], 'северное сияние'))
  assert.ok(
    matchesTheme(
      CUSTOM_THEMES.find((t) => t.id === 'forest'),
      'зачарованный лес',
    ),
  )
  assert.ok(matchesTheme({ id: 'test', name: 'Тёмный лес' }, 'темный'))
  assert.ok(!matchesTheme(CUSTOM_THEMES[0], 'ocean'))
})
test('invalid canvas dimensions and non-finite frame deltas do not poison particles', () => {
  for (const width of [0, -10, NaN, Infinity])
    assert.deepEqual(createParticles(CUSTOM_THEMES[0], width, 400), [])
  const particles = createParticles(CUSTOM_THEMES[0], 600, 400)
  advanceParticles(particles, 'firefly', 600, 400, NaN, Infinity)
  assert.ok(particles.every((p) => Number.isFinite(p.x) && Number.isFinite(p.y)))
})
test('navigation decoration does not intercept route clicks', () => {
  const source = readFileSync(
    new URL('../src/components/themes/NavigationBurst.vue', import.meta.url),
    'utf8',
  )
  assert.ok(!source.includes('e.preventDefault()'))
  assert.ok(!source.includes('router.push('))
})

test('external-link routing recognizes the actual origin and rejects dangerous schemes', async () => {
  const { externalLink } = await import('../src/helpers/links.js')
  assert.equal(externalLink('http://127.0.0.1:1420/settings', 'http://127.0.0.1:1420'), null)
  assert.equal(externalLink('https://tauri.localhost/themes', 'https://tauri.localhost'), null)
  assert.equal(externalLink('https://modrinth.com/', 'tauri://localhost'), 'https://modrinth.com/')
  assert.equal(externalLink('mailto:help@example.com', 'null'), 'mailto:help@example.com')
  for (const value of [
    'javascript:alert(1)',
    'file:///etc/passwd',
    'https://user:secret@example.com',
    'not a URL',
  ])
    assert.equal(externalLink(value, 'https://tauri.localhost'), null)
})
