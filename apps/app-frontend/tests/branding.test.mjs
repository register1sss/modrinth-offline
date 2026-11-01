import test from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'

const read = (path) => readFileSync(new URL(path, import.meta.url), 'utf8')
const config = JSON.parse(read('../../app/tauri.conf.json'))

test('U3drinth is the desktop product and window title', () => {
  assert.equal(config.package.productName, 'U3drinth')
  assert.equal(config.package.version, '1.0.0')
  assert.ok(config.tauri.windows.every((window) => window.title === 'U3drinth'))
  const mac = JSON.parse(read('../../app/tauri.macos.conf.json'))
  assert.ok(mac.tauri.windows.every((window) => window.title === 'U3drinth'))
  assert.match(read('../index.html'), /<title>U3drinth<\/title>/)
})

test('launcher-facing text and onboarding wordmark are renamed', () => {
  for (const path of [
    '../src/App.vue',
    '../src/pages/Settings.vue',
    '../src/components/ui/ErrorModal.vue',
    '../src/pages/instance/Logs.vue',
  ]) {
    const text = read(path)
    assert.ok(text.includes('U3drinth'), path)
    assert.ok(!text.includes('Modrinth App'), path)
  }
  const gallery = read('../src/components/ui/tutorial/GalleryImage.vue')
  assert.match(gallery, /aria-label="U3drinth">U3drinth/)
  assert.ok(!gallery.includes('modrinth-app-logo'))
})

test('branding does not break legacy data, endpoints or account identity', () => {
  assert.equal(config.tauri.bundle.identifier, 'ModrinthApp')
  assert.match(read('../../../packages/app-lib/src/state/dirs.rs'), /ModrinthApp_Offline/)
  assert.match(read('../src/themes/preferences.js'), /modrinth-offline:atmosphere:v1/)
  assert.match(config.tauri.security.csp, /https:\/\/modrinth\.com/)
  assert.match(read('../src/pages/Settings.vue'), /Sign in to your Modrinth account/)
  assert.match(read('../../app/src/api/mr_auth.rs'), /\.title\("Sign into Modrinth"\)/)
})
