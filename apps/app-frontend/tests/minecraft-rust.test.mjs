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
