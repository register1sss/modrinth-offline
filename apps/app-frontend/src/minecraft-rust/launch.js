// Offline launch never depends on Microsoft account or ownership state.
export function canLaunch(state, { mode, nickname, account, busy }) {
  if (busy || !state.supported) return false
  if (mode === 'offline') return /^[A-Za-z0-9_]{3,16}$/.test(nickname)
  return (
    mode === 'online' &&
    !!state.accounts?.owner &&
    !!state.accounts?.accounts?.some((a) => a.uuid === account)
  )
}
