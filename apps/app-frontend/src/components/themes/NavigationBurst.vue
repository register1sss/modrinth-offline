<script setup>
import { onMounted, onBeforeUnmount, ref } from 'vue'
import { useTheming } from '@/store/theme.js'
import { BURST_DURATION, createBurst, drawBurst } from '@/themes/particles.js'
const canvas = ref(null),
  store = useTheming()
let ctx,
  frame = 0,
  started = 0,
  kind = null,
  particles = []
function stop() {
  cancelAnimationFrame(frame)
  frame = 0
  particles = []
  if (ctx) ctx.clearRect(0, 0, innerWidth, innerHeight)
}
function tick(now) {
  const t = (now - started) / 1000
  ctx.clearRect(0, 0, innerWidth, innerHeight)
  if (t > BURST_DURATION || store.reducedMotion || !store.effectsEnabled || document.hidden) {
    stop()
    return
  }
  // Same art, motion style and orientation rules as the theme's background particles.
  drawBurst(ctx, particles, kind, t)
  frame = requestAnimationFrame(tick)
}
function click(e) {
  if (!(e.target instanceof Element) || !ctx) return
  const control = e.target.closest(
    '.nav-container .btn,.nav-container .avatar-button,.navigation-controls button',
  )
  if (
    !control ||
    e.button !== 0 ||
    e.ctrlKey ||
    e.metaKey ||
    e.shiftKey ||
    e.altKey ||
    control.disabled ||
    control.getAttribute('aria-disabled') === 'true'
  )
    return
  const theme = store.activeTheme
  if (!theme.particle || !store.effectsEnabled || store.reducedMotion) return
  const rect = control.getBoundingClientRect(),
    x = rect.left + rect.width / 2,
    y = rect.top + rect.height / 2
  stop()
  const d = Math.min(devicePixelRatio || 1, 1.5)
  canvas.value.width = innerWidth * d
  canvas.value.height = innerHeight * d
  ctx.setTransform(d, 0, 0, d, 0, 0)
  const count = store.effectsQuality === 'low' ? 14 : store.effectsQuality === 'high' ? 34 : 24
  kind = theme.particle
  particles = createBurst(theme, x, y, count)
  started = performance.now()
  frame = requestAnimationFrame(tick)
  // Decoration must never intercept navigation or delay an action.
}
function hidden() {
  if (document.hidden) stop()
}
onMounted(() => {
  ctx = canvas.value.getContext('2d')
  if (ctx) document.addEventListener('click', click, true)
  document.addEventListener('visibilitychange', hidden)
})
onBeforeUnmount(() => {
  stop()
  document.removeEventListener('click', click, true)
  document.removeEventListener('visibilitychange', hidden)
})
</script>
<template><canvas ref="canvas" class="navigation-burst" aria-hidden="true" /></template>
<style scoped>
.navigation-burst {
  position: fixed;
  inset: 0;
  width: 100vw;
  height: 100vh;
  pointer-events: none;
  z-index: 10000;
}
</style>
