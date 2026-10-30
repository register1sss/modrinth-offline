import { test } from 'node:test'
import assert from 'node:assert/strict'
import { canLaunch } from '../src/minecraft-rust/launch.js'
import { rustSession, updateRustSession } from '../src/minecraft-rust/state.js'
const ready = {
  supported: true,
  runtime: true,
  prepared: true,
  offline_ready: true,
  accounts: { owner: false, accounts: [], sign_in_configured: false },
}
const options = { mode: 'offline', profile: 'horizon', nickname: 'U3_Player', busy: false }
test('offline launch works without any Microsoft account or ownership', () =>
  assert.equal(canLaunch(ready, options), true))
test('one click starts installation and preparation without a profile or files', () => {
  assert.equal(
    canLaunch(
      { supported: true, runtime: false, prepared: false, offline_ready: false },
      { ...options, profile: '' },
    ),
    true,
  )
  assert.equal(canLaunch({ ...ready, supported: false }, options), false)
  assert.equal(canLaunch(ready, { ...options, busy: true }), false)
})
test('offline names are validated', () => {
  for (const nickname of ['ab', '../test', 'two words', '--help', 'Пользователь'])
    assert.equal(canLaunch(ready, { ...options, nickname }), false)
})
test('licensed mode still requires the owned selected Microsoft account', () => {
  const mode = { ...options, mode: 'online', account: 'uuid' }
  assert.equal(canLaunch(ready, mode), false)
  assert.equal(
    canLaunch({ ...ready, accounts: { owner: true, accounts: [{ uuid: 'uuid' }] } }, mode),
    true,
  )
  assert.equal(canLaunch({ ...ready, accounts: { owner: true, accounts: [] } }, mode), false)
})

test('installation and preparation are not shown as a running game', () => {
  for (const stage of ['play_offline', 'download_runtime', 'download_game', 'prepare_auto']) {
    updateRustSession({ running: true, action: 'play_offline', stage })
    assert.equal(rustSession.value.running, false)
  }
  updateRustSession({ running: true, action: 'play_offline', stage: 'playing' })
  assert.equal(rustSession.value.running, true)
  updateRustSession({ running: false, action: 'play_offline', stage: 'done' })
  assert.equal(rustSession.value.running, false)
})

import { installRequest, modInstalled } from '../src/minecraft-rust/mods.js'
test('native install requires explicit consent and a supported entry', () => {
  const mod = { id: 'u3_edit', checksum: 'a'.repeat(64), available: true }
  const catalog = {
    trust_token: 'b'.repeat(64),
    files: { 'u3_edit.hnmod': mod.checksum },
    ready: false,
  }
  assert.equal(installRequest(mod, catalog, 'horizon', false), null)
  assert.equal(installRequest({ ...mod, available: false }, catalog, 'horizon', true), null)
  assert.equal(installRequest(mod, {}, 'horizon', true), null)
  assert.equal(installRequest(mod, catalog, '', true).profile, 'horizon')
  assert.equal(installRequest(mod, catalog, 'own', true).trust_native_code, true)
  assert.equal(modInstalled(mod, catalog), false)
  assert.equal(modInstalled(mod, { ...catalog, ready: true }), true)
  assert.equal(
    modInstalled({ ...mod, checksum: 'c'.repeat(64) }, { ...catalog, ready: true }),
    false,
  )
})

test('U3 Fixes keeps explicit native consent and checksum-bound installed status', () => {
  const mod = { id: 'u3_fixes', checksum: 'f'.repeat(64), available: true }
  const catalog = {
    trust_token: 'e'.repeat(64),
    files: { 'u3_edit.hnmod': 'a'.repeat(64) },
    ready: true,
  }
  assert.equal(installRequest(mod, catalog, 'own', false), null)
  const request = installRequest(mod, catalog, 'own', true)
  assert.equal(request.project, 'u3_fixes')
  assert.equal(request.checksum, mod.checksum)
  assert.equal(modInstalled(mod, catalog), false)
  assert.equal(modInstalled(mod, { ...catalog, files: { 'u3_fixes.hnmod': mod.checksum } }), true)
})

test('automatic patch status belongs to the current profile, not the U3 Mods editor target', () => {
  const entry = { id: 'u3_fixes', automatic: true, installed: true, checksum: 'a'.repeat(64) }
  assert.equal(modInstalled(entry, { files: {}, ready: false }), true)
  assert.equal(
    installRequest({ ...entry, available: true }, { trust_token: 'hash' }, 'horizon', true),
    null,
  )
  assert.equal(
    modInstalled(
      { ...entry, installed: false },
      { files: { 'u3_fixes.hnmod': entry.checksum }, ready: true },
    ),
    false,
  )
})
