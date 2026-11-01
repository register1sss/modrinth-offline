// Shared between modal instances, including nested dialogs.
const dialogs = []
let originalOverflow = ''
export function registerDialog(id) {
  if (dialogs.includes(id)) return
  if (!dialogs.length) {
    originalOverflow = document.body.style.overflow
    document.body.style.overflow = 'hidden'
  }
  dialogs.push(id)
}
export function unregisterDialog(id) {
  const index = dialogs.indexOf(id)
  if (index !== -1) dialogs.splice(index, 1)
  if (!dialogs.length) document.body.style.overflow = originalOverflow
}
export const isTopDialog = (id) => dialogs.at(-1) === id
