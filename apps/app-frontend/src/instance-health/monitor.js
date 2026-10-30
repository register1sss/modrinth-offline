import { reactive } from 'vue'
import { process_listener } from '@/helpers/events.js'
import { health } from './api.js'
import { shouldAnalyzeExit } from './model.js'
export const crashAlerts = reactive([])
const starts = new Map()
let unlisten
export async function startHealthMonitor() {
  if (unlisten) return
  unlisten = await process_listener(async (event) => {
    const id = event.profile_path_id
    if (!id) return
    if (event.event === 'launched') {
      starts.set(id, Date.now())
      return
    }
    if (event.event !== 'finished') return
    const sinceMs = starts.get(id)
    starts.delete(id)
    // No startup history: don't show an old crash as a new one.
    if (!shouldAnalyzeExit(event, sinceMs)) return
    try {
      const report = await health('crash_report', id, { sinceMs })
      if (
        (report.crashed || (Number.isInteger(event.exit_code) && event.exit_code !== 0)) &&
        crashAlerts.length < 5 &&
        !crashAlerts.some((a) => a.profileId === id)
      ) {
        crashAlerts.push({
          profileId: id,
          report: {
            ...report,
            exit_code: event.exit_code,
            findings: report.findings?.length ? report.findings : ['crash'],
          },
        })
      }
    } catch (e) {
      console.warn('Local crash diagnostics unavailable', e)
      if (
        Number.isInteger(event.exit_code) &&
        event.exit_code !== 0 &&
        crashAlerts.length < 5 &&
        !crashAlerts.some((a) => a.profileId === id)
      ) {
        crashAlerts.push({
          profileId: id,
          report: { findings: ['crash'], exit_code: event.exit_code },
        })
      }
    }
  })
}
