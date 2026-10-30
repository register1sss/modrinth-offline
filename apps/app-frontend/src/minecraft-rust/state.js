import { ref } from 'vue'
export const rustSession = ref({ running: false, action: '' })
export function updateRustSession(snapshot) {
  rustSession.value = { running: snapshot.running, action: snapshot.action }
}
