import { invoke } from '@tauri-apps/api/tauri'
export const health = (command, profileId, args = {}) =>
  invoke(`plugin:instance_health|${command}`, { profileId, ...args })
