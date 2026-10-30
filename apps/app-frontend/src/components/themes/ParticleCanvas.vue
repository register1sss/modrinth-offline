<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { advanceParticles, createParticles, drawParticles } from '@/themes/particles.js'

const props = defineProps({
  theme: { type: Object, required: true },
  animated: { type: Boolean, default: true },
  quality: { type: String, default: 'balanced' },
  reducedMotion: { type: Boolean, default: false },
  subdued: { type: Boolean, default: false },
})
const canvas = ref(null)
const running = ref(false)
const kind = computed(() => props.theme.particle)
let ctx,
  observer,
  particles = [],
  width = 0,
  height = 0,
  frame = 0,
  lastDraw = 0
let focused = true

function paint(time = 0) {
  if (ctx) drawParticles(ctx, particles, kind.value, width, height, time, props.subdued)
}
function stop() {
  cancelAnimationFrame(frame)
  frame = 0
  lastDraw = 0
  running.value = false
}
function tick(time) {
  frame = 0
  if (!running.value) return
  if (!lastDraw || time - lastDraw >= 1000 / 30) {
    const delta = lastDraw ? (time - lastDraw) / 1000 : 0
    advanceParticles(particles, kind.value, width, height, delta, time / 1000)
    paint(time / 1000)
    lastDraw = time
  }
  frame = requestAnimationFrame(tick)
}
function syncAnimation() {
  stop()
  if (
    props.animated &&
    !props.reducedMotion &&
    !document.hidden &&
    focused &&
    kind.value &&
    ctx &&
    width &&
    height
  ) {
    running.value = true
    frame = requestAnimationFrame(tick)
  } else {
    paint()
  }
}
function resize() {
  if (!canvas.value || !ctx) return
  const rect = canvas.value.getBoundingClientRect()
  width = Math.max(1, rect.width)
  height = Math.max(1, rect.height)
  const scale = Math.min(window.devicePixelRatio || 1, 1.5)
  canvas.value.width = Math.round(width * scale)
  canvas.value.height = Math.round(height * scale)
  ctx.setTransform(scale, 0, 0, scale, 0, 0)
  particles = createParticles(props.theme, width, height, props.quality, !props.animated)
  paint()
  syncAnimation()
}
function focus() {
  focused = true
  syncAnimation()
}
function blur() {
  focused = false
  syncAnimation()
}
onMounted(() => {
  ctx = canvas.value.getContext('2d')
  if (!ctx) return
  focused = document.hasFocus()
  observer = new ResizeObserver(resize)
  observer.observe(canvas.value)
  if (props.animated) {
    document.addEventListener('visibilitychange', syncAnimation)
    window.addEventListener('focus', focus)
    window.addEventListener('blur', blur)
  }
  resize()
})
watch(() => [props.theme.id, props.quality, props.reducedMotion, props.animated], resize)
onBeforeUnmount(() => {
  stop()
  observer?.disconnect()
  document.removeEventListener('visibilitychange', syncAnimation)
  window.removeEventListener('focus', focus)
  window.removeEventListener('blur', blur)
})
</script>

<template>
  <canvas
    ref="canvas"
    class="theme-particle-canvas"
    aria-hidden="true"
    :data-particle-kind="kind"
    :data-running="running"
  />
</template>

<style scoped>
.theme-particle-canvas {
  display: block;
  width: 100%;
  height: 100%;
  pointer-events: none;
}
</style>
