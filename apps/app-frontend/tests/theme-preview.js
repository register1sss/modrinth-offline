// Development-only harness. Production index.html never loads this native API mock.
import { mockIPC, mockWindows, mockConvertFileSrc } from '@tauri-apps/api/mocks'
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
window.__mockCalls = []
window.__failThemeSave = false
window.__aiAccess = localStorage.getItem('u3-test-access') !== 'false'
window.__aiReady = false
window.__aiActions = []
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
        files: window.__rustModInstalled ? { 'u3_edit.hnmod': 'a'.repeat(64) } : {},
        ready: !!window.__rustModInstalled,
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
        window.__rustModInstalled = true
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
  if (cmd === 'plugin:settings|settings_get') return structuredClone(window.__nativeSettings)
  if (cmd === 'plugin:settings|settings_set') {
    if (window.__failThemeSave) throw 'Intentional test failure'
    window.__nativeSettings = structuredClone(args.settings)
    localStorage.setItem('theme-test:native-settings', JSON.stringify(args.settings))
    return null
  }
  if (cmd === 'plugin:metadata|metadata_get_loader_versions') return { gameVersions: [] }
  if (cmd === 'plugin:metadata|metadata_get_game_versions') return { versions: [] }
  if (cmd === 'plugin:tags|tags_get_loaders') return []
  if (cmd === 'plugin:profile|profile_get_many' || cmd === 'plugin:profile|profile_list') return []
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
history.replaceState(null, '', '/themes')
await import('../src/main.js')
