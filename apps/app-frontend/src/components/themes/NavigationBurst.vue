<script setup>
import { onMounted, onBeforeUnmount, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useTheming } from '@/store/theme.js'
import { particleArt } from '@/themes/particle-art.js'
const canvas = ref(null),
  store = useTheming(),
  router = useRouter()
let ctx,
  frame = 0,
  started = 0,
  particles = [],
  timer = 0,
  pending = ''
function stop() {
  cancelAnimationFrame(frame)
  frame = 0
  particles = []
  if (ctx) ctx.clearRect(0, 0, innerWidth, innerHeight)
}
function tick(now) {
  const t = (now - started) / 1000
  ctx.clearRect(0, 0, innerWidth, innerHeight)
  if (t > 1.05 || store.reducedMotion || !store.effectsEnabled || document.hidden) {
    stop()
    return
  }
  for (const p of particles) {
    const fall = ['petal', 'snow', 'leaf', 'sand'].includes(p.kind)
    const x = p.x + p.vx * t + (fall ? Math.sin(t * 5 + p.phase) * 18 : 0)
    const y = p.y + p.vy * t + (fall ? 95 : 30) * t * t
    ctx.save()
    ctx.translate(x, y)
    ctx.rotate(p.phase + t * p.spin)
    ctx.scale(p.size, p.size)
    ctx.globalAlpha = Math.min(1, (1.05 - t) * 2) * 0.92
    particleArt(ctx, p.kind, p.color, p.phase, t)
    ctx.restore()
  }
  frame = requestAnimationFrame(tick)
}
function click(e) {
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
  const count = store.effectsQuality === 'low' ? 22 : store.effectsQuality === 'high' ? 56 : 38
  particles = Array.from({ length: count }, () => {
    const angle = Math.random() * Math.PI * 2,
      speed = 35 + Math.random() * 125
    return {
      x,
      y,
      kind: theme.particle,
      vx: Math.cos(angle) * speed,
      vy: Math.sin(angle) * speed - (theme.particle === 'ember' ? 80 : 15),
      size: 1.5 + Math.random() * 2.2,
      phase: Math.random() * 6.28,
      spin: (Math.random() - 0.5) * 4,
      color: Math.random() > 0.25 ? theme.palette.accent : theme.palette.secondary,
    }
  })
  started = performance.now()
  frame = requestAnimationFrame(tick)
  const link = control.closest('a[href]')
  if (link && link.origin === location.origin && link.pathname !== router.currentRoute.value.path) {
    e.preventDefault()
    e.stopPropagation()
    clearTimeout(timer)
    pending = link.pathname + link.search + link.hash
    timer = setTimeout(() => {
      const to = pending
      pending = ''
      timer = 0
      router.push(to)
    }, 260)
  }
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
  clearTimeout(timer)
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
