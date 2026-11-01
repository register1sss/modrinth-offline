<script setup>
import { onMounted, onBeforeUnmount, ref, watch } from 'vue'
import { useTheming } from '@/store/theme.js'
import { createParticles, advanceParticles, drawParticles } from '@/themes/particles.js'
import { panelScene, drawPanel } from '@/themes/panel-art.js'
const props = defineProps({ vertical: Boolean })
const store = useTheming(),
  canvas = ref(null)
let ctx,
  observer,
  frame = 0,
  last = 0,
  w = 0,
  h = 0,
  particles = [],
  nodes = [],
  focused = true
function paint(t = 0, dt = 0) {
  if (!ctx || !panelCtx) return
  advanceParticles(particles, store.activeTheme.particle, w, h, dt, t)
  drawParticles(ctx, particles, store.activeTheme.particle, w, h, t)
  // drawPanel clears its own surface; use a separate small cached decorative layer.
  drawPanel(panelCtx, store.activeTheme, w, h, props.vertical, t, nodes)
  ctx.drawImage(panel, 0, 0, w, h)
}
let panel, panelCtx
function stop() {
  cancelAnimationFrame(frame)
  frame = 0
  last = 0
}
function tick(t) {
  if (t - last >= 1000 / 30) {
    paint(t / 1000, last ? (t - last) / 1000 : 0)
    last = t
  }
  frame = requestAnimationFrame(tick)
}
function sync() {
  stop()
  paint()
  if (!document.hidden && focused && !store.reducedMotion && store.effectsEnabled)
    frame = requestAnimationFrame(tick)
}
function resize() {
  if (!ctx || !panelCtx) return
  const b = canvas.value.getBoundingClientRect()
  w = Math.max(1, b.width)
  h = Math.max(1, b.height)
  const d = Math.min(devicePixelRatio || 1, 1.25)
  canvas.value.width = Math.round(w * d)
  canvas.value.height = Math.round(h * d)
  ctx.setTransform(d, 0, 0, d, 0, 0)
  panel.width = Math.round(w * d)
  panel.height = Math.round(h * d)
  panelCtx.setTransform(d, 0, 0, d, 0, 0)
  nodes = panelScene(store.activeTheme, props.vertical)
  particles = createParticles(store.activeTheme, w, h, store.effectsQuality).slice(
    0,
    store.effectsQuality === 'low' ? 8 : 16,
  )
  sync()
}
function focus() {
  focused = true
  sync()
}
function blur() {
  focused = false
  sync()
}
onMounted(() => {
  ctx = canvas.value.getContext('2d')
  panel = document.createElement('canvas')
  panelCtx = panel.getContext('2d')
  if (!ctx || !panelCtx) return
  focused = document.hasFocus()
  observer = new ResizeObserver(resize)
  observer.observe(canvas.value)
  window.addEventListener('focus', focus)
  window.addEventListener('blur', blur)
  document.addEventListener('visibilitychange', sync)
  resize()
})
watch(
  () => [
    store.selectedTheme,
    store.reducedMotion,
    store.effectsQuality,
    store.effectsEnabled,
    props.vertical,
  ],
  resize,
)
onBeforeUnmount(() => {
  stop()
  observer?.disconnect()
  window.removeEventListener('focus', focus)
  window.removeEventListener('blur', blur)
  document.removeEventListener('visibilitychange', sync)
})
</script>
<template>
  <canvas
    ref="canvas"
    class="theme-chrome"
    :data-panel-theme="store.selectedTheme"
    :data-panel-direction="vertical ? 'side' : 'top'"
    aria-hidden="true"
  />
</template>
<style scoped>
.theme-chrome {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  z-index: 0;
  opacity: 0.85;
}
</style>
