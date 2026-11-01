import { particleArt } from './particle-art.js'
import { EFFECT_LEVELS } from './catalog.js'

// The same deterministic renderer powers gallery previews and the real atmosphere.
export function seededRandom(seed) {
  let value = [...seed].reduce(
    (sum, char) => Math.imul(sum ^ char.charCodeAt(0), 16777619),
    2166136261,
  )
  return () => {
    value += 0x6d2b79f5
    let n = Math.imul(value ^ (value >>> 15), 1 | value)
    n ^= n + Math.imul(n ^ (n >>> 7), 61 | n)
    return ((n ^ (n >>> 14)) >>> 0) / 4294967296
  }
}

export function createParticles(theme, width, height, quality = 'balanced', preview = false) {
  if (
    !theme.particle ||
    !Number.isFinite(width) ||
    !Number.isFinite(height) ||
    width <= 0 ||
    height <= 0
  )
    return []
  const random = seededRandom(`${theme.id}:${quality}:${preview}`)
  const budget = EFFECT_LEVELS.find((level) => level.id === quality)?.count ?? 52
  const count = preview
    ? 16
    : Math.max(12, Math.round(budget * Math.min(1, (width * height) / 900000)))
  return Array.from({ length: count }, () => ({
    x: random() * width,
    y: random() * height,
    size: (preview ? 1.2 : 1.7) + random() * (preview ? 2 : 3.4),
    speed: 0.45 + random() * 0.7,
    phase: random() * Math.PI * 2,
    rotation: random() * Math.PI * 2,
    color: random() > 0.3 ? theme.palette.accent : theme.palette.secondary,
    alpha: 0.3 + random() * 0.4,
  }))
}

export function advanceParticles(particles, kind, width, height, delta, time) {
  const dt = Number.isFinite(delta) ? Math.min(Math.max(delta, 0), 0.08) : 0
  time = Number.isFinite(time) ? time : 0
  for (const p of particles) {
    const drift = Math.sin(time * 0.45 + p.phase)
    let vx = drift * 7
    let vy = -7 * p.speed
    switch (kind) {
      case 'rain':
        vx = -22 * p.speed
        vy = 92 * p.speed
        break
      case 'ember':
        vx = drift * 13
        vy = -26 * p.speed
        break
      case 'petal':
        vx = 13 + drift * 12
        vy = 17 * p.speed
        break
      case 'bubble':
        vx = drift * 6
        vy = -13 * p.speed
        break
      case 'star':
        vx = 2 * p.speed
        vy = -1
        break
      case 'leaf':
        vx = 9 + drift * 16
        vy = 12 * p.speed
        break
      case 'snow':
        vx = drift * 9
        vy = 16 * p.speed
        break
      case 'sand':
        vx = 26 * p.speed
        vy = -3 + drift * 2
        break
      case 'crystal':
        vx = drift * 4
        vy = -6 * p.speed
        break
      case 'comet':
        vx = 34 * p.speed
        vy = 17 * p.speed
        break
      case 'ribbon':
        vx = 5 + drift * 5
        vy = -5 * p.speed
        break
      case 'plankton':
        vx = drift * 9
        vy = Math.cos(time * 0.3 + p.phase) * 8 - 2
        break
      default:
        vx = drift * 10
        vy = Math.cos(time * 0.5 + p.phase) * 8 - 3
    }
    p.x += vx * dt
    p.y += vy * dt
    p.rotation += dt * (kind === 'petal' || kind === 'leaf' ? 0.45 : 0.08)
    const margin = 28
    if (p.x < -margin) p.x = width + margin
    if (p.x > width + margin) p.x = -margin
    if (p.y < -margin) p.y = height + margin
    if (p.y > height + margin) p.y = -margin
  }
}

export function drawParticles(ctx, particles, kind, width, height, time = 0, subdued = false) {
  ctx.clearRect(0, 0, width, height)
  for (const p of particles) {
    ctx.save()
    ctx.translate(p.x, p.y)
    ctx.rotate(p.rotation)
    ctx.scale(p.size, p.size)
    ctx.globalAlpha = p.alpha * (subdued ? 0.65 : 1) * (0.82 + Math.sin(time + p.phase) * 0.15)
    particleArt(ctx, kind, p.color, p.phase, time)
    ctx.restore()
  }
}
