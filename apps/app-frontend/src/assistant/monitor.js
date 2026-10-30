import { process_listener } from '@/helpers/events.js'
import { ai } from './api.js'
import { session } from './state.js'
import { diagnose } from './diagnostics.js'
import { language } from '@/i18n/index.js'
let unlisten
export async function startAssistantMonitor() {
  if (unlisten) return
  unlisten = await process_listener(async (event) => {
    if (event.event !== 'finished' || !event.profile_path_id) return
    try {
      if (!(await ai('preferences')).file_access) return
      const text = await ai('read', { profileId: event.profile_path_id, path: 'logs/latest.log' })
      const findings = diagnose(text, language.value)
      if (findings.length) {
        session.alerts.push({ name: event.profile_path_id, summary: findings[0] })
        session.alerts = session.alerts.slice(-5)
      }
    } catch {
      /* Missing logs or revoked access are normal. No cloud reports are sent. */
    }
  })
}
