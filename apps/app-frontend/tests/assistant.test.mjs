import test from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { diagnose } from '../src/assistant/diagnostics.js'
import { ru, en } from '../src/i18n/messages.js'
const read = (path) => readFileSync(new URL(path, import.meta.url), 'utf8')
test('offline diagnostics are labelled hypotheses in both languages', () => {
  assert.match(diagnose('OutOfMemoryError', 'ru')[0], /памяти/)
  assert.match(diagnose('OutOfMemoryError', 'en')[0], /memory/)
  assert.deepEqual(diagnose('Starting Minecraft successfully'), [])
  assert.match(diagnose('MixinApplyError', 'en')[0], /may conflict/)
})
test('RU and EN cover core navigation, settings and all custom themes', () => {
  for (const key of [
    'Home',
    'Library',
    'Settings',
    'About',
    'App version',
    'Create profile',
    'No instances running',
  ])
    assert.ok(ru[key], key)
  for (const key of ['Темы', 'Северное сияние', 'Живая лагуна', 'Насыщенные'])
    assert.ok(en[key], key)
})
test('assistant is local-only, fixed model and unloads after the answer', () => {
  const native = read('../../../packages/app-lib/src/assistant.rs')
  assert.match(native, /127\.0\.0\.1:11434\/api\/chat/)
  assert.match(native, /qwen2\.5:1\.5b/)
  assert.match(native, /"keep_alive":\s*0/)
  assert.match(native, /"num_gpu":\s*0/)
  assert.match(native, /"num_ctx":\s*4096/)
  assert.ok(!native.includes('process::Command'))
})
test('ads and support button are removed from launcher routes', () => {
  for (const p of [
    '../src/App.vue',
    '../src/components/ui/RunningAppBar.vue',
    '../src/components/ui/ErrorModal.vue',
  ])
    assert.ok(!read(p).includes('Get support'), p)
  for (const p of [
    '../src/pages/Browse.vue',
    '../src/pages/project/Index.vue',
    '../src/pages/instance/Index.vue',
  ])
    assert.ok(!read(p).includes('<Promotion'), p)
})
test('custom badge, publisher and displayed v1.0 are configured', () => {
  const c = JSON.parse(read('../../app/tauri.conf.json'))
  assert.equal(c.tauri.bundle.publisher, 'U3')
  assert.equal(c.package.version, '1.0.0')
  assert.match(read('../src/pages/Settings.vue'), /U3drinth \(custom modrinth\) v1\.0/)
  assert.match(read('../public/u3drinth.svg'), /#c73562/)
  assert.match(read('../src/assets/stylesheets/motion.scss'), /prefers-reduced-motion:\s*reduce/)
})

test('AI install is an explicit command with fixed signed official runtime', () => {
  const setup = read('../../../packages/app-lib/src/assistant_setup.rs')
  assert.match(setup, /Get-AuthenticodeSignature/)
  assert.match(setup, /https:\/\/ollama.com\/download\/OllamaSetup.exe/)
  assert.match(setup, /trusted_download_url/)
  assert.match(setup, /create_new\(true\)/)
  const page = read('../src/pages/Assistant.vue')
  assert.match(page, /Установить ИИ/)
  assert.ok(!page.includes('ollama pull'))
  assert.ok(!page.includes('ollama.com/download/windows'))
})
test('account migration preserves rows and permits several inactive accounts', () => {
  const migration = read(
    '../../../packages/app-lib/migrations/20261030140000_minecraft_accounts.sql',
  )
  assert.match(migration, /WHERE active = 1/)
  assert.ok(!migration.includes('DELETE'))
  const card = read('../src/components/ui/AccountsCard.vue')
  assert.ok(card.indexOf('await set_default_user(account.id)') < card.indexOf("emit('change')"))
})
test('theme page uses short labels without descriptive subtitles', () => {
  const p = read('../src/pages/Themes.vue')
  for (const label of [
    'Настроение начинается',
    'Найди своё настроение',
    'theme.subtitle',
    'active.subtitle',
    'Атмосфера в движении',
  ])
    assert.ok(!p.includes(label), label)
  assert.ok(!/[\u2013\u2014]/.test(p))
})
