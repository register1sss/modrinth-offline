import { ref } from 'vue'
import { ru, en } from './messages.js'
const KEY = 'u3drinth:language:v1'
let saved
try {
  saved = localStorage.getItem(KEY)
} catch {
  /* private storage */
}
export const language = ref(['ru', 'en'].includes(saved) ? saved : 'ru')
export function setLanguage(value) {
  if (!['ru', 'en'].includes(value)) return
  language.value = value
  document.documentElement.lang = value
  try {
    localStorage.setItem(KEY, value)
  } catch {
    /* current session still works */
  }
}
const simple = (value) => String(value).replace(/[\u2013\u2014]/g, '-')
export const tr = (russian, english) => simple(language.value === 'ru' ? russian : english)
export function translate(text) {
  const key = String(text).replace(/\s+/g, ' ').trim()
  return simple((language.value === 'ru' ? ru[key] : en[key]) ?? text)
}
export const localization = {
  install(app) {
    app.config.globalProperties.$u3t = translate
  },
}
setLanguage(language.value)
