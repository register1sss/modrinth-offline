import { ref } from 'vue'
export const rustSession = ref({ running: false, action: '' })
export function updateRustSession(snapshot) {
  rustSession.value = {
    running: !!snapshot.running && snapshot.stage === 'playing',
    action: snapshot.action,
  }
}
