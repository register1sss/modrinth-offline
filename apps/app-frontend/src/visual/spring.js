// Bounded damped spring. Keyframes are generated once, not simulated every frame.
export function springFrames(level = 'spring') {
  const amplitude = level === 'soft' ? 0.023 : 0.045
  const frames = []
  for (let i = 0; i <= 40; i++) {
    const t = i / 40
    const wave = Math.exp(-t * 7) * Math.cos(t * 17)
    const x = 1 + amplitude * wave
    const y = 1 - amplitude * wave * 0.65
    frames.push({ offset: t, scale: `${x.toFixed(5)} ${y.toFixed(5)}` })
  }
  frames[frames.length - 1] = { offset: 1, scale: '1 1' }
  return frames
}
export function motionAllowed({ enabled, reduced, hidden }) {
  return enabled && !reduced && !hidden
}
export function startJellyMotion() {
  const media = window.matchMedia('(prefers-reduced-motion: reduce)')
  const root = document.documentElement
  const active = new Map()
  const allowed = () =>
    motionAllowed({
      enabled: root.dataset.jelly !== 'off',
      reduced: media.matches,
      hidden: document.hidden,
    })
  const target = (event) => {
    if (!(event.target instanceof Element)) return null
    const el = event.target.closest('button, a.btn, .btn[role="button"], .studio-action')
    if (
      !el ||
      el.matches(':disabled,[aria-disabled="true"]') ||
      el.closest('.window-controls,[data-no-jelly]')
    )
      return null
    return el
  }
  function animate(el, pressed) {
    if (!allowed() || typeof el.animate !== 'function' || !CSS.supports('scale', '1')) return
    active.get(el)?.cancel()
    if (active.size > 16) {
      const [old, animation] = active.entries().next().value
      animation.cancel()
      active.delete(old)
    }
    const animation = el.animate(
      pressed
        ? [
            { scale: '1 1' },
            { scale: root.dataset.jellyLevel === 'soft' ? '0.985 0.975' : '0.98 0.95' },
          ]
        : springFrames(root.dataset.jellyLevel),
      {
        duration: pressed ? 100 : 560,
        easing: pressed ? 'ease-out' : 'linear',
        fill: pressed ? 'forwards' : 'none',
      },
    )
    active.set(el, animation)
    if (!pressed)
      animation.onfinish = () => {
        if (active.get(el) === animation) active.delete(el)
      }
  }
  let pressed = null
  const down = (e) => {
    if (e.button !== 0) return
    const el = target(e)
    if (el) {
      pressed = el
      animate(el, true)
    }
  }
  const release = () => {
    if (pressed) animate(pressed, false)
    pressed = null
  }
  const key = (e) => {
    if (!e.repeat && ['Enter', ' '].includes(e.key)) {
      const el = target(e)
      if (el) animate(el, false)
    }
  }
  const cancel = () => {
    for (const a of active.values()) a.cancel()
    active.clear()
    pressed = null
  }
  const check = () => {
    if (!allowed()) cancel()
  }
  const observer = new MutationObserver(check)
  observer.observe(root, { attributes: true, attributeFilter: ['data-jelly'] })
  document.addEventListener('pointerdown', down, { passive: true })
  document.addEventListener('pointerup', release, { passive: true })
  document.addEventListener('pointercancel', cancel, { passive: true })
  document.addEventListener('keydown', key)
  document.addEventListener('visibilitychange', check)
  window.addEventListener('blur', cancel)
  media.addEventListener('change', check)
  return () => {
    cancel()
    observer.disconnect()
    document.removeEventListener('pointerdown', down)
    document.removeEventListener('pointerup', release)
    document.removeEventListener('pointercancel', cancel)
    document.removeEventListener('keydown', key)
    document.removeEventListener('visibilitychange', check)
    window.removeEventListener('blur', cancel)
    media.removeEventListener('change', check)
  }
}
