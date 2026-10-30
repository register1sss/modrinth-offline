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
