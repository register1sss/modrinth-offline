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
  if (!theme.particle) return []
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
  const dt = Math.min(Math.max(delta, 0), 0.08)
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
    const s = p.size
    ctx.save()
    ctx.translate(p.x, p.y)
    ctx.globalAlpha = p.alpha * (subdued ? 0.42 : 0.85) * (0.72 + Math.sin(time + p.phase) * 0.2)
    ctx.strokeStyle = p.color
    ctx.fillStyle = p.color
    ctx.lineWidth = 0.9
    ctx.beginPath()
    switch (kind) {
      case 'rain':
        ctx.moveTo(0, 0)
        ctx.lineTo(-s * 1.6, s * 6)
        ctx.stroke()
        break
      case 'ember':
        ctx.rotate(p.rotation)
        ctx.moveTo(0, -s * 1.8)
        ctx.lineTo(s * 0.5, 0)
        ctx.lineTo(0, s)
        ctx.lineTo(-s * 0.5, 0)
        ctx.closePath()
        ctx.fill()
        break
      case 'petal':
        ctx.rotate(p.rotation)
        ctx.moveTo(0, -s)
        ctx.bezierCurveTo(s * 2, -s * 0.6, s * 1.4, s * 1.8, 0, s * 1.9)
        ctx.bezierCurveTo(-s * 1.4, s * 1.8, -s * 2, -s * 0.6, 0, -s)
        ctx.fill()
        break
      case 'bubble':
        ctx.arc(0, 0, s * 1.6, 0, Math.PI * 2)
        ctx.stroke()
        ctx.beginPath()
        ctx.arc(-s * 0.45, -s * 0.45, s * 0.8, Math.PI, Math.PI * 1.5)
        ctx.stroke()
        break
      case 'star':
        ctx.rotate(p.rotation)
        ctx.moveTo(0, -s * 1.8)
        ctx.lineTo(s * 0.35, -s * 0.35)
        ctx.lineTo(s * 1.8, 0)
        ctx.lineTo(s * 0.35, s * 0.35)
        ctx.lineTo(0, s * 1.8)
        ctx.lineTo(-s * 0.35, s * 0.35)
        ctx.lineTo(-s * 1.8, 0)
        ctx.lineTo(-s * 0.35, -s * 0.35)
        ctx.closePath()
        ctx.fill()
        break
      case 'leaf':
        ctx.rotate(p.rotation)
        ctx.moveTo(0, -s * 1.5)
        ctx.quadraticCurveTo(s * 2.2, 0, 0, s * 1.5)
        ctx.quadraticCurveTo(-s * 2.2, 0, 0, -s * 1.5)
        ctx.fill()
        ctx.globalAlpha *= 0.6
        ctx.strokeStyle = '#102117'
        ctx.moveTo(0, -s)
        ctx.lineTo(0, s)
        ctx.stroke()
        break
      case 'snow':
        ctx.rotate(p.rotation)
        for (let arm = 0; arm < 6; arm++) {
          ctx.moveTo(0, 0)
          ctx.lineTo(0, -s * 1.8)
          ctx.moveTo(0, -s)
          ctx.lineTo(-s * 0.5, -s * 1.45)
          ctx.moveTo(0, -s)
          ctx.lineTo(s * 0.5, -s * 1.45)
          ctx.rotate(Math.PI / 3)
        }
        ctx.stroke()
        break
      case 'sand':
        ctx.rotate(-0.15)
        ctx.ellipse(0, 0, s * 1.5, s * 0.35, 0, 0, Math.PI * 2)
        ctx.fill()
        break
      case 'crystal':
        ctx.rotate(p.rotation)
        ctx.moveTo(0, -s * 2)
        ctx.lineTo(s, 0)
        ctx.lineTo(0, s * 2)
        ctx.lineTo(-s, 0)
        ctx.closePath()
        ctx.stroke()
        ctx.moveTo(0, -s * 2)
        ctx.lineTo(0, s * 2)
        ctx.moveTo(-s, 0)
        ctx.lineTo(s, 0)
        ctx.stroke()
        break
      case 'comet':
        ctx.moveTo(-s * 7, -s * 3.5)
        ctx.lineTo(0, 0)
        ctx.stroke()
        ctx.beginPath()
        ctx.arc(0, 0, s * 0.7, 0, Math.PI * 2)
        ctx.fill()
        break
      case 'ribbon':
        ctx.rotate(p.rotation)
        ctx.moveTo(-s * 5, 0)
        ctx.bezierCurveTo(-s * 2, -s * 3, s * 2, s * 3, s * 5, 0)
        ctx.stroke()
        break
      case 'plankton':
        ctx.arc(0, 0, s, 0, Math.PI * 2)
        ctx.fill()
        ctx.globalAlpha *= 0.3
        ctx.beginPath()
        ctx.arc(0, 0, s * 2.4, 0, Math.PI * 2)
        ctx.stroke()
        break
      default: // fireflies: a distinct softly glowing core, without costly shadowBlur.
        ctx.globalAlpha *= 0.22
        ctx.arc(0, 0, s * 2.7, 0, Math.PI * 2)
        ctx.fill()
        ctx.globalAlpha *= 4
        ctx.beginPath()
        ctx.arc(0, 0, s * 0.65, 0, Math.PI * 2)
        ctx.fill()
    }
    ctx.restore()
  }
}
