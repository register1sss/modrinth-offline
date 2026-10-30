import { invoke } from '@tauri-apps/api/tauri'
export const ai = (command, args = {}) => invoke(`plugin:assistant|${command}`, args)
