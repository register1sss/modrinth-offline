import test from 'node:test'
import assert from 'node:assert/strict'
import {
  compareVersions,
  describeRange,
  versionSatisfies,
} from '../src/instance-health/versions.js'
import { analyzeMods, modTitle, prettyId } from '../src/instance-health/dependencies.js'
import {
  crashAdvice,
  describeHealthError,
  describeLoaderLine,
  describeScan,
  describeWarning,
} from '../src/instance-health/messages.js'

const mod = (id, extra = {}) => ({
  path: `mods/${id}.jar`,
  name: id,
  ids: [id],
  dependencies: [],
  disabled: false,
  metadata_complete: true,
  warnings: [],
  loaders: ['fabric'],
  version: '1.0.0',
  conflicts: [],
  bundled: [],
  ...extra,
})
const dep = (id, version = '*', required = true) => ({ id, version, required })
const fabric = { loader: 'fabric', game_version: '1.20.1', loader_version: '0.15.11' }
const kinds = (report) => report.problems.map((p) => p.kind)

test('versions compare like loaders do', () => {
  assert.equal(compareVersions('0.92.1+1.20.1', '0.92.0'), 1)
  assert.equal(compareVersions('1.0-beta', '1.0'), -1)
  assert.equal(compareVersions('1.0', '1.0.0'), 0)
  assert.equal(compareVersions('mc1.20-2.0', '1.0'), null)
  assert.equal(versionSatisfies('>=0.92.0', '0.91.0+1.20.1'), false)
  assert.equal(versionSatisfies('~1.20.1', '1.20.4'), true)
  assert.equal(versionSatisfies('~1.20.1', '1.21'), false)
  assert.equal(versionSatisfies('1.20.x', '1.20.6'), true)
  assert.equal(versionSatisfies(['1.20', '1.20.1'], '1.20.1'), true)
  assert.equal(versionSatisfies('>=1.20- <1.21-', '1.21'), false)
  assert.equal(versionSatisfies('[47,)', '47.2.0', 'maven'), true)
  assert.equal(versionSatisfies('[1.20.1,1.21)', '1.21', 'maven'), false)
  assert.equal(
    versionSatisfies('47.1', '1.0', 'maven'),
    true,
    'bare Forge version is only a recommendation',
  )
  assert.equal(versionSatisfies('>=1.0', 'beta-build'), null, 'unknown formats are never judged')
})
test('ranges read as plain language in both languages', () => {
  assert.equal(describeRange('>=0.92.0'), 'не ниже 0.92.0')
  assert.equal(describeRange('>=0.92.0', 'fabric', 'en'), '0.92.0 or newer')
  assert.equal(describeRange('1.20.1'), 'только 1.20.1')
  assert.equal(describeRange(['1.20', '1.20.1']), '1.20 или 1.20.1')
  assert.equal(describeRange('[1.20.1,1.21)', 'maven'), 'не ниже 1.20.1, ниже 1.21')
  assert.equal(describeRange('*'), 'любая версия')
  assert.equal(describeRange(null, 'maven', 'en'), 'any version')
  assert.equal(describeRange('1.20.x'), 'любая 1.20.x')
})
test('names come from manifests, file names or known libraries', () => {
  assert.equal(
    modTitle({ name: 'mods/cool-mod-1.2.jar.disabled', path: 'mods/cool-mod-1.2.jar.disabled' }),
    'cool-mod-1.2',
  )
  assert.equal(modTitle({ name: 'Sodium', path: 'mods/sodium.jar' }), 'Sodium')
  assert.equal(prettyId('fabric-resource-loader-v0'), 'Fabric Resource Loader (v0)')
  const report = analyzeMods([mod('a', { dependencies: [dep('cloth-config')] })], fabric)
  assert.equal(report.problems[0].title, 'Не установлен «Cloth Config API»')
})
test('missing dependencies are grouped and explain who needs them', () => {
  const report = analyzeMods(
    [
      mod('a', { name: 'Alpha', dependencies: [dep('lib', '>=2.0')] }),
      mod('b', { name: 'Beta', dependencies: [dep('lib')] }),
      mod('c', { dependencies: [dep('extra', '*', false)] }),
    ],
    fabric,
  )
  assert.deepEqual(kinds(report), ['missing'])
  const [p] = report.problems
  assert.equal(p.severity, 'error')
  assert.match(p.text, /«Alpha», «Beta»/)
  assert.match(p.text, /не ниже 2\.0/)
  assert.match(p.fix, /Fabric 1\.20\.1/)
  const c = report.byPath.get('mods/c.jar')
  assert.equal(c.optional[0].status, 'optional-missing')
  assert.equal(c.status, 'ok', 'a missing optional dependency is not a problem')
})
test('providers, versions, reverse dependents and libraries', () => {
  const report = analyzeMods(
    [
      mod('api', { name: 'Fabric API', ids: ['fabric-api'], version: '0.92.0+1.20.1' }),
      mod('a', {
        name: 'Alpha',
        dependencies: [
          dep('fabric-api', '>=0.90'),
          dep('minecraft', '1.20.1'),
          dep('fabricloader', '>=0.14'),
        ],
      }),
      mod('b', { name: 'Beta', dependencies: [dep('fabric-api', '>=0.95')] }),
    ],
    fabric,
  )
  const alpha = report.byPath.get('mods/a.jar')
  assert.deepEqual(
    alpha.required.map((r) => [r.id, r.status]),
    [
      ['fabric-api', 'ok'],
      ['fabricloader', 'platform'],
      ['minecraft', 'platform'],
    ],
  )
  assert.equal(alpha.required[0].installed, '0.92.0+1.20.1')
  const api = report.byPath.get('mods/api.jar')
  assert.deepEqual(api.dependents.map((d) => d.title).sort(), ['Alpha', 'Beta'])
  assert.equal(report.stats.libraries, 1)
  assert.deepEqual(kinds(report), ['version'])
  assert.match(report.problems[0].text, /не ниже 0\.95, установлена 0\.92\.0\+1\.20\.1/)
})
test('bundled Jar-in-Jar modules satisfy dependencies', () => {
  const api = mod('api', {
    name: 'Fabric API',
    ids: ['fabric-api'],
    bundled: [
      { id: 'fabric-resource-loader-v0', name: 'Fabric Resource Loader (v0)', version: '0.11.10' },
    ],
  })
  const self = mod('self', {
    dependencies: [dep('mixinextras', '>=0.2')],
    bundled: [{ id: 'mixinextras', version: '0.3.5' }],
  })
  const user = mod('user', {
    dependencies: [dep('fabric-resource-loader-v0'), dep('fabric-key-binding-api-v1')],
  })
  const report = analyzeMods([api, self, user], fabric)
  const u = report.byPath.get('mods/user.jar')
  assert.equal(u.required.find((r) => r.id === 'fabric-resource-loader-v0').status, 'bundled')
  assert.equal(
    u.required.find((r) => r.id === 'fabric-resource-loader-v0').host.path,
    'mods/api.jar',
  )
  assert.equal(
    u.required.find((r) => r.id === 'fabric-key-binding-api-v1').status,
    'missing',
    'real JiJ data wins over guesses',
  )
  assert.equal(report.byPath.get('mods/self.jar').required[0].status, 'bundled')
  // Older native data without bundled lists: Fabric API modules are assumed present.
  const legacy = analyzeMods([{ ...api, bundled: undefined }, user], fabric)
  assert.equal(legacy.problems.length, 0)
})
test('disabled providers, duplicates and declared conflicts are explained', () => {
  const report = analyzeMods(
    [
      mod('lib', { name: 'Lib', disabled: true, path: 'mods/lib.jar.disabled' }),
      mod('a', { name: 'Alpha', dependencies: [dep('lib')] }),
      mod('sodium', { name: 'Sodium', path: 'mods/sodium-0.5.jar' }),
      mod('sodium', { name: 'Sodium', path: 'mods/sodium-0.6.jar' }),
      mod('opti', { name: 'OptiFabric', ids: ['optifabric'] }),
      mod('fast', {
        name: 'Fast',
        conflicts: [{ id: 'optifabric', version: '*', required: true }],
      }),
    ],
    fabric,
  )
  assert.deepEqual(kinds(report).sort(), ['conflict', 'disabled', 'duplicate'])
  const disabled = report.problems.find((p) => p.kind === 'disabled')
  assert.match(disabled.title, /«Lib» отключён/)
  assert.match(disabled.fix, /Включи «Lib»/)
  assert.match(
    report.problems.find((p) => p.kind === 'duplicate').text,
    /sodium-0\.5\.jar, sodium-0\.6\.jar/,
  )
  assert.match(
    report.problems.find((p) => p.kind === 'conflict').title,
    /«Fast» несовместим с «OptiFabric»/,
  )
})
test('wrong loader, Minecraft and loader versions are detected', () => {
  const forgeMod = mod('forgey', {
    name: 'Forgey',
    loaders: ['forge'],
    dependencies: [dep('forge', '[47,)')],
  })
  const report = analyzeMods(
    [
      forgeMod,
      mod('old', { name: 'Old', dependencies: [dep('minecraft', '1.19.2')] }),
      mod('new', { name: 'New', dependencies: [dep('fabricloader', '>=0.16')] }),
    ],
    fabric,
  )
  assert.deepEqual(kinds(report).sort(), ['loader', 'loader-version', 'minecraft'])
  assert.match(report.problems.find((p) => p.kind === 'loader').title, /сделан для Forge/)
  assert.match(
    report.problems.find((p) => p.kind === 'minecraft').text,
    /поддерживает Minecraft: 1\.19\.2\./,
  )
  assert.match(report.problems.find((p) => p.kind === 'loader-version').text, /в профиле 0\.15\.11/)
  // Quilt loads Fabric mods; NeoForge 1.20.1 loads Forge mods; Connector loads Fabric mods.
  assert.equal(
    analyzeMods([mod('f', { dependencies: [dep('fabricloader')] })], {
      loader: 'quilt',
      game_version: '1.20.1',
    }).problems.length,
    0,
  )
  assert.equal(
    analyzeMods([forgeMod], { loader: 'neoforge', game_version: '1.20.1' }).problems.length,
    0,
  )
  assert.equal(
    analyzeMods([forgeMod], { loader: 'neoforge', game_version: '1.21.1' }).problems[0].kind,
    'loader',
  )
  const viaConnector = analyzeMods(
    [
      mod('connector', { loaders: ['neoforge'] }),
      mod('f', { dependencies: [dep('fabricloader')] }),
    ],
    { loader: 'neoforge', game_version: '1.21.1' },
  )
  assert.deepEqual(
    viaConnector.problems.map((p) => p.severity),
    ['info'],
  )
  assert.equal(
    analyzeMods([forgeMod], {
      loader: 'forge',
      game_version: '1.20.1',
      loader_version: '1.20.1-46.0.1',
    }).problems[0].kind,
    'loader-version',
  )
  const vanilla = analyzeMods([forgeMod], { loader: 'vanilla', game_version: '1.20.1' })
  assert.deepEqual(kinds(vanilla), ['no-loader'])
})
test('unreadable files and risk findings become readable problems', () => {
  const report = analyzeMods(
    [
      mod('notes', {
        ids: [],
        loaders: [],
        metadata_complete: false,
        warnings: ['Not a readable JAR/ZIP; format was not inspected'],
        path: 'mods/notes.txt',
        name: 'notes.txt',
      }),
      mod('sus', {
        name: 'Sus',
        warnings: ['Contains executable/script: run.ps1 (not proof of malware)'],
      }),
      mod('big', {
        name: 'Big',
        metadata_complete: false,
        warnings: ['Nested JAR dependencies require loader validation'],
      }),
    ],
    fabric,
  )
  assert.deepEqual(kinds(report), ['unrecognized', 'risk', 'partial'])
  assert.match(report.problems[0].title, /«notes\.txt» не распознан как мод/)
  assert.match(report.problems[1].text, /run\.ps1/)
  assert.equal(report.problems[2].severity, 'info')
  assert.equal(report.byPath.get('mods/big.jar').status, 'ok')
})
test('English report and native messages are translated', () => {
  const report = analyzeMods(
    [mod('a', { name: 'Alpha', dependencies: [dep('lib')] })],
    fabric,
    'en',
  )
  assert.equal(report.problems[0].title, '"lib" is not installed')
  assert.match(report.problems[0].text, /Required by: "Alpha"/)
  assert.equal(describeWarning('Unsafe archive path: ../x').kind, 'risk')
  assert.match(describeWarning('expected `=` at line 3 column 5').text, /повреждено/)
  assert.equal(
    describeHealthError('Stop the instance before changing files'),
    'Сначала закрой игру.',
  )
  assert.equal(describeHealthError(new Error('weird')), 'weird')
  assert.equal(describeScan({ status: 'review' }).tone, 'warn')
  assert.match(crashAdvice(['memory'])[0], /не исправит/)
  assert.equal(
    describeLoaderLine(
      "- Mod 'Sodium' (sodium) 0.5.8 requires any version of fabric-api, which is missing!",
    ),
    '«Sodium» требует «Fabric API», но его нет.',
  )
  assert.equal(describeLoaderLine('- Install fabric-api, any version.'), 'Установи «Fabric API».')
  assert.equal(
    describeLoaderLine(
      "- Mod 'Iris' (iris) 1.6.4 requires version 0.5.0 or later of mod 'Sodium' (sodium), but only the wrong version is present: 0.4.10!",
    ),
    '«Iris» требует «Sodium» (не ниже 0.5.0), а установлена версия 0.4.10.',
  )
  assert.match(
    describeLoaderLine(
      "Mod ID: 'cloth_config', Requested by: 'xaero', Expected range: '[11,)', Actual version: '[MISSING]'",
    ),
    /«xaero» требует «Cloth Config API» \(не ниже 11\), но его нет/,
  )
})
