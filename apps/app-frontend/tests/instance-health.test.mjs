import test from 'node:test'
import assert from 'node:assert/strict'
import {
  removalImpact,
  compareSamples,
  repairAdvice,
  shouldAnalyzeExit,
} from '../src/instance-health/model.js'
const mod = (id, dependencies = [], extra = {}) => ({
  path: `mods/${id}.jar`,
  name: id,
  ids: [id],
  dependencies,
  disabled: false,
  metadata_complete: true,
  ...extra,
})
const required = (id) => ({ id, required: true, version: '*' })
test('removal resolves transitive mandatory dependency chains', () => {
  const graph = [mod('a'), mod('b', [required('a')]), mod('c', [required('b')])]
  assert.deepEqual(
    removalImpact(graph, ['mods/a.jar']).affected.map((m) => m.name),
    ['b', 'c'],
  )
})
test('optional dependencies do not break launch', () => {
  const result = removalImpact([mod('a'), mod('b', [{ id: 'a', required: false }])], ['mods/a.jar'])
  assert.equal(result.affected.length, 0)
  assert.equal(result.optional[0].name, 'b')
})
test('alternative providers and provided ids are respected', () => {
  const graph = [
    mod('a', [], { ids: ['a', 'api'] }),
    mod('other', [], { ids: ['api'] }),
    mod('b', [required('api')]),
  ]
  assert.equal(removalImpact(graph, ['mods/a.jar']).affected.length, 0)
  assert.equal(removalImpact(graph, ['mods/a.jar', 'mods/other.jar']).affected[0].name, 'b')
})
test('disabled mods do not provide runtime dependencies', () => {
  const graph = [
    mod('a'),
    mod('other', [], { ids: ['a'], disabled: true }),
    mod('b', [required('a')]),
    mod('disabled', [required('a')], { disabled: true }),
  ]
  assert.deepEqual(
    removalImpact(graph, ['mods/a.jar']).affected.map((m) => m.name),
    ['b'],
  )
})
test('cycles terminate and selected mods are not counted twice', () => {
  const graph = [mod('a', [required('c')]), mod('b', [required('a')]), mod('c', [required('b')])]
  assert.equal(removalImpact(graph, ['mods/a.jar']).affected.length, 2)
  assert.equal(
    removalImpact(
      graph,
      graph.map((m) => m.path),
    ).affected.length,
    0,
  )
})
test('unknown metadata remains unknown, not safe', () => {
  assert.deepEqual(
    removalImpact([mod('unknown', [], { metadata_complete: false })], []).incomplete,
    ['mods/unknown.jar'],
  )
})
test('Windows project paths normalize for removal preview', () => {
  assert.equal(
    removalImpact([mod('a'), mod('b', [required('a')])], ['mods\\a.jar']).affected.length,
    1,
  )
})
test('resource comparison detects changes without pretending per-mod attribution', () => {
  const before = {
    mods: [{ path: 'a', sha256: '1' }],
    memory_bytes: 100,
    cpu_percent: 4,
    processes: 1,
  }
  const after = {
    mods: [
      { path: 'a', sha256: '2' },
      { path: 'b', sha256: '3' },
    ],
    memory_bytes: 200,
    cpu_percent: 2,
    processes: 1,
  }
  const c = compareSamples(before, after)
  assert.deepEqual(c.changed, ['a', 'b'])
  assert.equal(c.memory_delta, 100)
  assert.equal(c.cpu_delta, -2)
  assert.equal(c.attribution, 'correlation_only')
})
test('repair advice distinguishes file repair from memory and Java issues', () => {
  assert.match(repairAdvice(['java'])[0], /не исправляет/)
  assert.match(repairAdvice(['memory'])[0], /не исправляет/)
  assert.match(repairAdvice(['files'])[0], /восстановить/)
})

test('normal exit, manual stop and stale launch state never trigger crash repair', () => {
  assert.equal(shouldAnalyzeExit({ event: 'finished', exit_code: 0 }, 100), false)
  assert.equal(
    shouldAnalyzeExit({ event: 'finished', exit_code: -1, message: 'Stopped by user' }, 100),
    false,
  )
  assert.equal(shouldAnalyzeExit({ event: 'finished', exit_code: 1 }, undefined), false)
  assert.equal(shouldAnalyzeExit({ event: 'finished', exit_code: 1 }, 100), true)
  assert.equal(shouldAnalyzeExit({ event: 'finished', exit_code: null }, 100), true)
})
test('removal follows libraries bundled inside another JAR and names them', () => {
  const graph = [
    mod('fabric-api', [], { name: 'Fabric API', bundled: [{ id: 'fabric-key-binding-api-v1' }] }),
    mod('zoom', [required('fabric-key-binding-api-v1')], { name: 'Zoomify' }),
    mod('menu', [{ id: 'fabric-api', required: false }], { name: 'mods/menu.jar' }),
  ]
  const result = removalImpact(graph, ['mods/fabric-api.jar'])
  assert.deepEqual(
    result.affected.map((m) => [m.name, m.dependencyName]),
    [['Zoomify', 'Fabric API']],
  )
  assert.equal(result.optional[0].name, 'menu')
})
