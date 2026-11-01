import { createApp } from 'vue'
import { startJellyMotion } from '@/visual/spring.js'
import { localization } from '@/i18n/index.js'
import { startHealthMonitor } from '@/instance-health/monitor.js'
import { startAssistantMonitor } from '@/assistant/monitor.js'
import 'vue-multiselect/dist/vue-multiselect.css'
import '@/assets/stylesheets/motion.scss'
import router from '@/routes'
import App from '@/App.vue'
import { createPinia } from 'pinia'
import '@modrinth/assets/omorphia.scss'
import '@/assets/stylesheets/global.scss'
import '@/assets/stylesheets/themes.scss'
import '@/assets/stylesheets/studio.scss'
import FloatingVue from 'floating-vue'
import 'floating-vue/dist/style.css'
import { get_opening_command, initialize_state } from '@/helpers/state'
import loadCssMixin from './mixins/macCssFix.js'
import { get } from '@/helpers/settings'
import { invoke } from '@tauri-apps/api'
import { isDev } from './helpers/utils.js'
import { createPlugin } from '@vintl/vintl/plugin'

const VIntlPlugin = createPlugin({
  controllerOpts: {
    defaultLocale: 'en-US',
    locale: 'en-US',
    locales: [
      {
        tag: 'en-US',
        meta: {
          displayName: 'American English',
        },
      },
    ],
  },
  globalMixin: true,
  injectInto: [],
})

const pinia = createPinia()

let app = createApp(App)
app.use(router)
app.use(pinia)
app.use(FloatingVue)
app.mixin(loadCssMixin)
app.use(VIntlPlugin)
app.use(localization)

const mountedApp = app.mount('#app')
const stopJellyMotion = startJellyMotion()
if (import.meta.hot) import.meta.hot.dispose(stopJellyMotion)

const raw_invoke = async (plugin, fn, args) => {
  if (plugin === '') {
    await invoke(fn, args)
  } else {
    await invoke('plugin:' + plugin + '|' + fn, args)
  }
}
isDev()
  .then((dev) => {
    if (dev) {
      window.raw_invoke = raw_invoke
    }
  })
  .catch((err) => {
    console.error(err)
  })

initialize_state()
  .then(() => {
    // First, redirect to other landing page if we have that setting
    get()
      .then((fetchSettings) => {
        if (fetchSettings?.default_page && fetchSettings?.default_page !== 'Home') {
          router.push({ name: fetchSettings?.default_page })
        }
      })
      .catch((err) => {
        console.error(err)
      })
      .finally(async () => {
        try {
          await mountedApp.initialize()
        } catch (error) {
          await mountedApp.failure(error)
          return
        }
        startAssistantMonitor().catch(console.error)
        startHealthMonitor().catch(console.error)
        get_opening_command().catch(console.error)
      })
  })
  .catch((err) => {
    console.error('Failed to initialize app', err)
    mountedApp.failure(err)
  })
