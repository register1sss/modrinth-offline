import { test } from 'node:test'
import assert from 'node:assert/strict'
import { canLaunch } from '../src/minecraft-rust/launch.js'
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
test('offline launch rejects missing files, unchecked engine and busy state', () => {
  for (const key of ['supported', 'runtime', 'prepared', 'offline_ready'])
    assert.equal(canLaunch({ ...ready, [key]: false }, options), false)
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
