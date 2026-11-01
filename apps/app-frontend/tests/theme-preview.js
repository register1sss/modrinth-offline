// Development-only harness. Production index.html never loads this native API mock.
import { mockIPC, mockWindows, mockConvertFileSrc } from '@tauri-apps/api/mocks'
import { THEME_IDS } from '../src/themes/catalog.js'
const requestedTheme = new URLSearchParams(location.search).get('theme')
mockWindows('main')
mockConvertFileSrc('windows')
const saved = JSON.parse(localStorage.getItem('theme-test:native-settings') || 'null')
window.__nativeSettings = saved ?? {
  theme: 'dark',
  default_page: 'Themes',
  advanced_rendering: true,
  onboarded: true,
  native_decorations: false,
  collapsed_navigation: true,
  telemetry: true,
  extra_launch_args: [],
  custom_env_vars: [],
  custom_dir: null,
  loaded_config_dir: '/test',
  memory: { maximum: 4096 },
  game_resolution: [1280, 720],
  hooks: {},
  max_concurrent_downloads: 10,
  max_concurrent_writes: 10,
  discord_rpc: false,
}
if (THEME_IDS.includes(requestedTheme)) window.__nativeSettings.theme = requestedTheme
window.__mockCalls = []
window.__failThemeSave = false
window.__aiAccess = localStorage.getItem('u3-test-access') !== 'false'
window.__aiReady = false
window.__aiActions = []
window.__aiCalls = []
window.__aiDelays = {}
window.__aiEvents = {}

mockIPC(async (cmd, args) => {
  window.__mockCalls.push(cmd === 'tauri' ? JSON.stringify(args) : cmd)
  if (cmd.startsWith('plugin:minecraft-rust|')) {
    const tool = cmd.split('|')[1]
    window.__rustState ??= {
      supported: true,
      runtime: false,
      prepared: false,
      offline_ready: false,
      release: 'launcher@0.2.28',
      running: false,
      stage: 'idle',
      completed: 0,
      total: 0,
      error: null,
      logs: [],
      accounts: {},
      instances: [],
      versions: {},
    }
    if (tool === 'resource_catalog') {
      if (window.__rustPackFail) throw 'Resource catalog test failure'
      const all =
        args.tab === 'modrinth'
          ? Array.from({ length: 20 }, (_, i) => ({
              id: `mr:pack${args.offset + i}`,
              project: `pack${args.offset + i}`,
              name: `Modrinth pack ${args.offset + i}`,
              summary: 'Minecraft 26.3 resource pack',
              available: true,
            }))
          : [
              {
                id: 'safe',
                name: args.tab === 'pvp' ? 'PvP test pack' : 'Regular test pack',
                available: true,
                gui: true,
                project: 'abcd1234',
              },
              { id: 'old', name: 'Old incompatible pack', available: false },
              { id: 'local', name: 'Local pack', local: true, available: true, optifine: true },
            ]
      return {
        entries: all.filter((p) => p.name.toLowerCase().includes(args.search.toLowerCase())),
        total: 45,
        limit: 20,
        offset: args.offset,
        installed: window.__rustPackInstalled
          ? [
              {
                file: 'u3-safe-test.zip',
                name: 'PvP test pack',
                enabled: true,
                description: 'Test protected pack',
              },
            ]
          : [],
      }
    }
    if (tool === 'catalog') {
      if (window.__rustCatalogFail) throw 'Catalog test failure'
      return {
        own: [
          {
            id: 'u3_edit',
            name: 'U3 Edit',
            authors: ['U3'],
            version: '0.1.0',
            checksum: 'a'.repeat(64),
            available: true,
            native: true,
          },
          {
            id: 'u3_fixes',
            automatic: true,
            installed: !!window.__rustFixInstalled,
            name: 'U3 Fixes',
            authors: ['U3'],
            version: '0.1.0',
            checksum: 'f'.repeat(64),
            summary: 'Client trident state fixes. Not all video issues are fixed.',
            available: true,
            native: true,
          },
        ],
        community: [
          {
            id: 'zoom',
            name: 'Zoom',
            authors: ['Horizon'],
            version: '2.0.0',
            checksum: 'b'.repeat(64),
            summary: 'Hold a key to zoom in. Adjustable magnification.',
            available: true,
            native: true,
            source: 'https://github.com/horizon-rust/horizon-mods',
          },
          {
            id: 'coords_hud',
            name: 'Coordinates HUD',
            authors: ['Horizon'],
            version: '2.0.0',
            checksum: 'c'.repeat(64),
            summary: 'Coordinates and direction on your screen.',
            available: true,
            native: true,
          },
          {
            id: 'blockscope',
            name: 'Blockscope',
            authors: ['Domanffe'],
            version: '0.2.0',
            checksum: 'd'.repeat(64),
            summary: 'The block under your crosshair.',
            available: false,
            native: false,
          },
        ],
        trust_token: 'e'.repeat(64),
        target: args.profile === 'horizon' ? 'u3-mods' : args.profile,
        target_name: args.profile === 'horizon' ? 'U3 Mods' : 'My profile',
        files: {
          ...(window.__rustModInstalled ? { 'u3_edit.hnmod': 'a'.repeat(64) } : {}),
          ...(window.__rustFixInstalled ? { 'u3_fixes.hnmod': 'f'.repeat(64) } : {}),
        },
        ready: !!(window.__rustModInstalled || window.__rustFixInstalled),
      }
    }
    if (tool === 'cancel_sign_in') {
      window.__rustState.running = false
      window.__rustState.auth_code = null
      window.__rustState.auth_url = null
    }
    if (tool === 'start') {
      window.__rustActions ??= []
      window.__rustActions.push(args.request)
      if (window.__rustFail) throw 'Horizon test failure'
      const r = args.request,
        s = window.__rustState
      s.action = r.action
      s.error = null
      if (r.action === 'install') {
        s.runtime = true
        s.offline_ready = true
        s.versions = { installed: ['nightly-2026-10-30'], selected: 'nightly-2026-10-30' }
        s.instances = [{ id: 'horizon', name: 'Horizon', java: null }]
        s.accounts.sign_in_configured = true
      }
      if (r.action.startsWith('pack_')) {
        window.__rustPackInstalled = true
        s.running = false
        s.stage = 'done'
        s.result_profile = r.profile
      }
      if (r.action === 'mod_install') {
        if (!r.trust_native_code) throw 'Native consent required'
        if (r.project === 'u3_fixes') window.__rustFixInstalled = true
        else window.__rustModInstalled = true
        s.runtime = true
        s.prepared = true
        s.result_profile = 'u3-mods'
        s.instances = [
          { id: 'horizon', name: 'Horizon', java: null },
          { id: 'u3-mods', name: 'U3 Mods', kind: 'own', java: null },
        ]
        s.running = false
        s.stage = 'done'
      }
      if (r.action === 'sign_in') {
        s.running = true
        s.stage = 'sign_in'
        s.auth_url = 'https://www.microsoft.com/link'
        s.auth_code = 'TEST12345'
      }
      if (r.action === 'create') s.instances.push({ id: 'test-profile', name: r.name, java: null })
      if (r.action.startsWith('prepare_')) s.prepared = true
      if (r.action.startsWith('play_')) {
        if (!window.__rustFixInstalled) {
          window.__rustFixInstalled = true
          window.__rustFixBuilds = (window.__rustFixBuilds ?? 0) + 1
        }
        s.result_profile = r.profile || 'horizon'
        s.runtime = true
        s.prepared = true
        s.offline_ready = true
        s.instances = [{ id: 'horizon', name: 'Horizon', kind: 'default', java: null }]
        s.versions = { installed: ['nightly-2026-10-30'], selected: 'nightly-2026-10-30' }
        s.running = true
        s.stage = 'playing'
      }
    }
    return JSON.parse(JSON.stringify(window.__rustState))
  }
  if (cmd.startsWith('plugin:assistant|')) {
    const tool = cmd.split('|')[1]
    window.__aiCalls.push({ tool, args: structuredClone(args) })
    if (window.__aiDelays[tool]) await new Promise((r) => setTimeout(r, window.__aiDelays[tool]))
    if (tool === 'install') {
      window.__aiInstallAt = Date.now()
      window.__aiInstallCancelled = false
      return {
        running: true,
        stage: 'download_runtime',
        completed: 0,
        total: 100,
        cancellable: true,
      }
    }
    if (tool === 'install_cancel') {
      window.__aiInstallCancelled = true
      return { running: false, stage: 'cancelled', completed: 20, total: 100, cancellable: false }
    }
    if (tool === 'install_status') {
      if (window.__aiInstallCancelled)
        return { running: false, stage: 'cancelled', cancellable: false }
      if (!window.__aiInstallAt) return { running: false, stage: 'idle', cancellable: false }
      const elapsed = Date.now() - window.__aiInstallAt
      if (elapsed > 4000) {
        window.__aiReady = true
        return { running: false, stage: 'ready', completed: 100, total: 100, cancellable: false }
      }
      return {
        running: true,
        stage: elapsed > 1800 ? 'download_model' : 'download_runtime',
        completed: Math.round(elapsed / 40),
        total: 100,
        cancellable: true,
      }
    }
    if (tool === 'preferences') return { file_access: window.__aiAccess }
    if (tool === 'set_access') {
      window.__aiAccess = args.enabled
      localStorage.setItem('u3-test-access', String(args.enabled))
      return { file_access: args.enabled }
    }
    if (tool === 'status')
      return { connected: true, ready: window.__aiReady, model: 'qwen2.5:1.5b' }
    if (tool === 'chat')
      return {
        reply:
          args.language === 'ru'
            ? 'Предлагаю Sodium и Fabric API. Сначала проверь зависимости.'
            : 'I suggest Sodium and Fabric API. Verify dependencies first.',
        projects: ['sodium', 'fabric-api'],
      }
    if (!window.__aiAccess) throw 'Assistant file access is disabled'
    if (tool === 'profiles')
      return [
        { path: 'Test Profile', name: 'Test Profile', game_version: '1.21.11', loader: 'fabric' },
        { path: 'Other Profile', name: 'Other Profile', game_version: '1.21.11', loader: 'fabric' },
      ]
    if (tool === 'search')
      return [
        {
          project_id: 'sodium',
          title: 'Sodium',
          description: 'Rendering optimization for Fabric',
          project_type: 'mod',
        },
      ]
    if (tool === 'prepare')
      return {
        token: 'test-plan',
        name: args.request.name,
        game: args.request.game,
        loader: args.request.loader,
        files: [{ id: 'test-sodium', name: 'Sodium compatible version' }],
      }
    if (tool === 'build') {
      window.__aiActions.push('build')
      return { profile: 'U3 Collection', installed: ['mods/sodium.jar'], errors: [], ready: true }
    }
    if (tool === 'files')
      return args.folder
        ? args.folder === 'config'
          ? [{ path: 'config/test.json', directory: false, bytes: 20 }]
          : [{ path: 'logs/latest.log', directory: false, bytes: 180 }]
        : [
            { path: 'config', directory: true, bytes: 0 },
            { path: 'logs', directory: true, bytes: 0 },
          ]
    if (tool === 'read')
      return args.path.startsWith('config/')
        ? '{"renderDistance": 8}'
        : '[ERROR] OutOfMemoryError: Java heap space\nCaused by: allocation failure'
    if (tool === 'write_config') {
      window.__aiActions.push('write_config')
      return '.u3-backups/test.bak'
    }
    if (tool === 'collection') {
      window.__aiActions.push('collection')
      return 'u3-collections/test.json'
    }
    if (tool === 'toggle') {
      window.__aiActions.push('toggle')
      return args.path + '.disabled'
    }
  }
  if (cmd === 'initialize_state') {
    await new Promise((resolve) => setTimeout(resolve, 100))
    return null
  }
  if (cmd === 'plugin:profile|profile_get') return structuredClone(window.__mockProfile)
  if (cmd === 'plugin:process|process_get_by_profile_path') return []
  if (cmd === 'plugin:logs|logs_get_logs')
    return [
      { filename: 'latest.log', log_type: 'InfoLog', stdout: 'cleared' },
      {
        filename: 'crash-2025-01-01_12.00.00-client.txt',
        log_type: 'CrashReport',
        stdout: 'cleared',
      },
    ]
  if (cmd === 'plugin:logs|logs_get_output_by_filename')
    return [
      '[12:00:00] [main/INFO]: Loading Minecraft 1.20.1 with Fabric Loader 0.15.11',
      '[12:00:01] [main/WARN]: Mod sodium-extra requires reeses-sodium-options (optional)',
      '[12:00:02] [main/ERROR]: Incompatible mods found!',
      '# This is a comment line',
    ].join('\n')
  if (cmd === 'plugin:profile|profile_get_projects')
    return Object.fromEntries(
      window.__mockInventory.map((m) => [
        m.path,
        { file_name: m.path.split('/').pop(), project_type: 'mod', metadata: null },
      ]),
    )
  if (cmd === 'plugin:profile|profile_remove_project') {
    window.__mockInventory = window.__mockInventory.filter((m) => m.path !== args.projectPath)
    return null
  }
  if (/^plugin:cache\|get_(project|version|team|organization)_many$/.test(cmd)) return []
  if (cmd === 'plugin:profile|profile_get_optimal_jre_key')
    return { path: 'C:/Program Files/Java/jdk-17/bin/javaw.exe', major_version: 17 }
  if (cmd === 'plugin:profile|profile_get_pack_export_candidates')
    return ['mods', 'config', 'resourcepacks', 'options.txt']
  if (cmd === 'plugin:instance_health|inventory') return structuredClone(window.__mockInventory)
  if (cmd === 'plugin:instance_health|antivirus')
    return { status: args.path.includes('free-op') ? 'review' : 'no_detection' }
  if (cmd === 'plugin:settings|settings_get') return structuredClone(window.__nativeSettings)
  if (cmd === 'plugin:settings|settings_set') {
    if (window.__failThemeSave) throw 'Intentional test failure'
    window.__nativeSettings = structuredClone(args.settings)
    localStorage.setItem('theme-test:native-settings', JSON.stringify(args.settings))
    return null
  }
  if (cmd === 'plugin:metadata|metadata_get_loader_versions')
    return {
      gameVersions: [
        { id: '1.20.1', stable: true, loaders: [{ id: '0.15.11', stable: true }] },
        { id: '1.19.2', stable: true, loaders: [{ id: '0.15.11', stable: true }] },
      ],
    }
  if (cmd === 'plugin:metadata|metadata_get_game_versions') return { versions: [] }
  if (cmd === 'plugin:tags|tags_get_loaders') return []
  if (cmd === 'plugin:profile|profile_get_many' || cmd === 'plugin:profile|profile_list') {
    if (window.__profileListError) throw 'Test profile list error'
    return []
  }
  if (cmd === 'plugin:jre|get_java_versions') return {}
  if (cmd === 'plugin:jre|jre_get_max_memory') return 16777216
  if (cmd === 'is_dev') return true
  if (cmd === 'plugin:utils|get_os') return 'Windows'
  if (cmd === 'plugin:auth|auth_users' || cmd === 'plugin:process|process_get_all') return []
  if (cmd === 'plugin:utils|progress_bars_list') return {}
  if (cmd === 'tauri') {
    if (args.__tauriModule === 'Os') return 'Windows_NT'
    if (args.__tauriModule === 'App') return '1.0.0'
    if (args.__tauriModule === 'Event') return 1
  }
  return null
})
// Diagnostics fixture: one profile with typical dependency problems.
const jar = (file, name, version, extra = {}) => ({
  path: `mods/${file}`,
  name,
  version,
  ids: [extra.id ?? name.toLowerCase().replace(/[^a-z0-9]+/g, '-')],
  dependencies: [],
  conflicts: [],
  bundled: [],
  loaders: ['fabric'],
  disabled: file.endsWith('.disabled'),
  sha256: 'a3f1'.repeat(16),
  bytes: 1_250_000,
  warnings: [],
  metadata_complete: true,
  ...extra,
})
const need = (id, version = '*', required = true) => ({ id, version, required })
const base = [need('fabricloader', '>=0.14.21'), need('minecraft', '1.20.1')]
const modules = ['fabric-api-base', 'fabric-resource-loader-v0', 'fabric-key-binding-api-v1']
  .concat(['fabric-rendering-v1', 'fabric-screen-api-v1', 'fabric-lifecycle-events-v1'])
  .concat(['fabric-rendering-data-attachment-v1', 'fabric-block-view-api-v2'])
window.__mockProfile = {
  path: 'survival-plus',
  name: 'Survival+',
  loader: 'fabric',
  game_version: '1.20.1',
  loader_version: '0.15.11',
  install_stage: 'installed',
  groups: [],
  linked_data: null,
  icon_path: null,
  java_path: null,
  memory: null,
  game_resolution: null,
  force_fullscreen: null,
  extra_launch_args: null,
  custom_env_vars: null,
  hooks: { pre_launch: null, wrapper: null, post_exit: null },
}
window.__mockInventory = [
  jar('fabric-api-0.92.2+1.20.1.jar', 'Fabric API', '0.92.2+1.20.1', {
    id: 'fabric-api',
    bundled: modules.map((id) => ({ id, name: null, version: '1.0.0' })),
    dependencies: [need('fabricloader', '>=0.14.22')],
  }),
  jar('sodium-fabric-0.5.11+mc1.20.1.jar', 'Sodium', '0.5.11+mc1.20.1', {
    dependencies: [
      ...base,
      need('fabric-rendering-data-attachment-v1'),
      need('fabric-block-view-api-v2'),
    ],
  }),
  jar('iris-1.7.2+mc1.20.1.jar', 'Iris', '1.7.2+mc1.20.1', {
    dependencies: [...base, need('sodium', '0.5.11+mc1.20.1')],
  }),
  jar('sodium-extra-0.5.4+mc1.20.1.jar', 'Sodium Extra', '0.5.4+mc1.20.1', {
    dependencies: [
      ...base,
      need('sodium', '>=0.5.4'),
      need('fabric-api'),
      need('reeses-sodium-options', '*', false),
    ],
  }),
  jar('modmenu-7.2.2.jar', 'Mod Menu', '7.2.2', {
    id: 'modmenu',
    dependencies: [...base, need('fabric-screen-api-v1'), need('fabric-key-binding-api-v1')],
  }),
  jar('BetterThirdPerson-Fabric-1.9.0.jar', 'Better Third Person', '1.9.0', {
    dependencies: [...base, need('cloth-config', '>=11.0.0'), need('modmenu', '*', false)],
  }),
  jar('zoomify-2.11.2.jar', 'Zoomify', '2.11.2', {
    dependencies: [...base, need('yet_another_config_lib_v3', '>=3.2.0'), need('fabric-api')],
  }),
  jar('yet-another-config-lib-3.2.1.jar.disabled', 'YetAnotherConfigLib', '3.2.1+1.20', {
    id: 'yet_another_config_lib_v3',
  }),
  jar('lithium-fabric-mc1.20.1-0.11.2.jar', 'Lithium', '0.11.2', { dependencies: base }),
  jar('lithium-fabric-mc1.20.1-0.11.1.jar', 'Lithium', '0.11.1', { dependencies: base }),
  jar('jei-1.20.1-forge-15.2.0.27.jar', 'Just Enough Items', '15.2.0.27', {
    id: 'jei',
    loaders: ['forge'],
    dependencies: [need('forge', '[47.1.3,)'), need('minecraft', '[1.20.1,1.21)')],
  }),
  jar('appleskin-fabric-mc1.19-2.4.1.jar', 'AppleSkin', '2.4.1+mc1.19', {
    dependencies: [need('fabricloader', '>=0.14'), need('minecraft', '1.19.x')],
  }),
  jar('continuity-3.0.0+1.20.1.jar', 'Continuity', '3.0.0+1.20.1', {
    dependencies: [...base, need('fabric-api', '>=0.95.0')],
  }),
  jar('free-op-mod.jar', 'Free OP', '1.0', {
    warnings: ['Contains executable/script: payload/run.bat (not proof of malware)'],
  }),
  jar('readme.txt', 'mods/readme.txt', null, {
    ids: [],
    loaders: [],
    sha256: 'b7c2'.repeat(16),
    bytes: 2048,
    metadata_complete: false,
    warnings: ['Not a readable JAR/ZIP; format was not inspected'],
  }),
]
const previewParams = new URLSearchParams(location.search)
const requestedRoute = previewParams.get('route')
// A deep link must not be replaced by the landing page from settings.
if (requestedRoute) window.__nativeSettings.default_page = 'Home'
history.replaceState(null, '', requestedRoute || '/themes')
await import('../src/main.js')
// ?crash=1 shows the crash dialog with a typical Fabric dependency failure.
if (previewParams.get('crash')) {
  const { crashAlerts } = await import('../src/instance-health/monitor.js')
  setTimeout(
    () =>
      crashAlerts.push({
        profileId: 'survival-plus',
        report: {
          crashed: true,
          exit_code: 1,
          findings: ['dependencies'],
          evidence: [
            {
              kind: 'dependencies',
              line: '[main/ERROR]: Incompatible mods found!',
            },
          ],
          details: [
            '- Install cloth-config, version 11.0.0 or later.',
            "- Mod 'Better Third Person' (betterthirdperson) 1.9.0 requires version 11.0.0 or later of cloth-config, which is missing!",
            "- Mod 'Iris' (iris) 1.6.4 requires version 0.5.0 or later of mod 'Sodium' (sodium), but only the wrong version is present: 0.4.10!",
          ],
        },
      }),
    1200,
  )
}
