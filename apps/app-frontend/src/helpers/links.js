// Only known external URL schemes may reach the OS shell. Internal links stay in Vue Router.
export function externalLink(value, origin) {
  try {
    const url = new URL(value)
    if (url.username || url.password) return null
    if (['http:', 'https:'].includes(url.protocol)) return url.origin === origin ? null : url.href
    if (['mailto:', 'tel:'].includes(url.protocol)) return url.href
  } catch {
    /* Not an absolute external URL. */
  }
  return null
}
