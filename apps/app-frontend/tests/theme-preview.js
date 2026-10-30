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
mockIPC(async (cmd, args) => {
  window.__mockCalls.push(cmd === 'tauri' ? JSON.stringify(args) : cmd)
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
  if (cmd === 'plugin:jre|jre_get_max_memory') return 16384
  if (cmd === 'is_dev') return true
  if (cmd === 'plugin:utils|get_os') return 'Windows'
  if (cmd === 'plugin:auth|auth_users' || cmd === 'plugin:process|process_get_all') return []
  if (cmd === 'plugin:utils|progress_bars_list') return {}
  if (cmd === 'tauri') {
    if (args.__tauriModule === 'Os') return 'Windows_NT'
    if (args.__tauriModule === 'App') return '0.8.0-3'
    if (args.__tauriModule === 'Event') return 1
  }
  return null
})
history.replaceState(null, '', '/themes')
await import('../src/main.js')
