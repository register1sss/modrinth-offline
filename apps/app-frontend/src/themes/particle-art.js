// Vector particle art shared by the background, theme panels, previews and click bursts.
// Every custom theme owns a unique kind. No images, network requests or DOM nodes per particle.
const TAU = Math.PI * 2
const cache = new Map()
function remember(key, make) {
  let value = cache.get(key)
  if (value === undefined) {
    if (cache.size > 768) cache.clear()
    value = make()
    cache.set(key, value)
  }
  return value
}
const channels = (hex) => {
  const n = Number.parseInt(String(hex).slice(1, 7), 16) || 0
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255]
}
const toHex = (rgb) =>
  '#' +
  rgb
    .map((v) =>
      Math.round(Math.max(0, Math.min(255, v)))
        .toString(16)
        .padStart(2, '0'),
    )
    .join('')
export const mix = (a, b, t) =>
  remember(`${a}${b}${t}`, () => {
    const x = channels(a),
      y = channels(b)
    return toHex(x.map((v, i) => v + (y[i] - v) * t))
  })
export const shade = (hex, amount) =>
  amount >= 0 ? mix(hex, '#ffffff', amount) : mix(hex, '#000000', -amount)
export const alpha = (hex, a) =>
  String(hex).slice(0, 7) +
  Math.round(Math.max(0, Math.min(1, a)) * 255)
    .toString(16)
    .padStart(2, '0')
export function hash(n) {
  const s = Math.sin(n * 127.1 + 311.7) * 43758.5453123
  return s - Math.floor(s)
}

function dot(c, x, y, r) {
  c.beginPath()
  c.arc(x, y, Math.max(0, r), 0, TAU)
  c.fill()
}
function seg(c, x, y, xx, yy) {
  c.beginPath()
  c.moveTo(x, y)
  c.lineTo(xx, yy)
  c.stroke()
}
function halo(c, color, r, strength = 0.55, x = 0, y = 0) {
  if (r <= 0) return
  const g = c.createRadialGradient(x, y, 0, x, y, r)
  g.addColorStop(0, alpha(color, strength))
  g.addColorStop(1, alpha(color, 0))
  c.fillStyle = g
  dot(c, x, y, r)
}
function polygon(c, sides, r, rotation = 0, x = 0, y = 0) {
  c.beginPath()
  for (let i = 0; i < sides; i++) {
    const a = rotation + (i / sides) * TAU
    const px = x + Math.cos(a) * r,
      py = y + Math.sin(a) * r
    if (i) c.lineTo(px, py)
    else c.moveTo(px, py)
  }
  c.closePath()
}
// Four-point flare used for glints, stars and sparkles.
function sparkle(c, x, y, size, color = '#ffffff') {
  if (!(size > 0.03)) return
  const s = size * 0.18
  c.fillStyle = color
  c.beginPath()
  c.moveTo(x, y - size)
  c.quadraticCurveTo(x + s, y - s, x + size, y)
  c.quadraticCurveTo(x + s, y + s, x, y + size)
  c.quadraticCurveTo(x - s, y + s, x - size, y)
  c.quadraticCurveTo(x - s, y - s, x, y - size)
  c.fill()
}
// Quadratic spline through the midpoints of a polyline; gives organic outlines cheaply.
function spline(c, pts, close = false) {
  const n = pts.length
  if (close) {
    c.moveTo((pts[0][0] + pts[n - 1][0]) / 2, (pts[0][1] + pts[n - 1][1]) / 2)
    for (let i = 0; i < n; i++) {
      const a = pts[i],
        b = pts[(i + 1) % n]
      c.quadraticCurveTo(a[0], a[1], (a[0] + b[0]) / 2, (a[1] + b[1]) / 2)
    }
    c.closePath()
    return
  }
  c.moveTo(pts[0][0], pts[0][1])
  for (let i = 1; i < n - 1; i++)
    c.quadraticCurveTo(
      pts[i][0],
      pts[i][1],
      (pts[i][0] + pts[i + 1][0]) / 2,
      (pts[i][1] + pts[i + 1][1]) / 2,
    )
  c.lineTo(pts[n - 1][0], pts[n - 1][1])
}
function polyline(c, pts) {
  c.beginPath()
  pts.forEach(([x, y], i) => (i ? c.lineTo(x, y) : c.moveTo(x, y)))
  c.stroke()
}
// Almond leaf from (0, -length) to (0, length); optional serrated edge.
function leafPath(c, width, length, serrate = 0) {
  const steps = 10
  c.beginPath()
  c.moveTo(0, -length)
  for (let i = 1; i <= steps; i++) {
    const t = i / steps
    const w = Math.sin(t * Math.PI) ** 0.8 * width + (i % 2 ? serrate : 0) * Math.sin(t * Math.PI)
    c.lineTo(w, -length + t * length * 2)
  }
  for (let i = steps - 1; i >= 1; i--) {
    const t = i / steps
    const w = Math.sin(t * Math.PI) ** 0.8 * width + (i % 2 ? serrate : 0) * Math.sin(t * Math.PI)
    c.lineTo(-w, -length + t * length * 2)
  }
  c.closePath()
}
function heartPath(c) {
  c.beginPath()
  c.moveTo(0, 2.3)
  c.bezierCurveTo(-1.2, 1.45, -2.6, 0.35, -2.4, -0.9)
  c.bezierCurveTo(-2.2, -2.35, -0.55, -2.5, 0, -1.25)
  c.bezierCurveTo(0.55, -2.5, 2.2, -2.35, 2.4, -0.9)
  c.bezierCurveTo(2.6, 0.35, 1.2, 1.45, 0, 2.3)
  c.closePath()
}
const GLYPHS = '0123456789ABCDEF<>/{}[]=+*#$%&'

const ART = {
  // Northern lights: a folded curtain with a bright lower hem and luminous rays.
  aurora(c, color, phase, time, alt) {
    const sway = Math.sin(time * 0.8 + phase)
    const top = -6.4,
      bottom = 2.4,
      rows = 7
    const left = [],
      right = []
    for (let i = 0; i <= rows; i++) {
      const t = i / rows
      const y = bottom + (top - bottom) * t
      const fold = Math.sin(t * 5 + time * 1.3 + phase) * 0.45
      const lean = sway * t * 0.9
      const w = 0.85 + 0.35 * Math.sin(t * 3 + phase * 2)
      left.push([lean + fold - w, y])
      right.push([lean + fold * 0.6 + w, y])
    }
    const g = c.createLinearGradient(0, top, 0, bottom)
    g.addColorStop(0, alpha(alt, 0))
    g.addColorStop(0.42, alpha(alt, 0.3))
    g.addColorStop(0.84, alpha(color, 0.8))
    g.addColorStop(1, alpha(color, 0))
    c.fillStyle = g
    c.beginPath()
    spline(c, [...left, ...right.reverse()], true)
    c.fill()
    c.lineWidth = 0.08
    c.strokeStyle = alpha(mix(color, '#ffffff', 0.5), 0.3)
    for (let i = 0; i < 4; i++) {
      const x = -0.7 + i * 0.46
      seg(c, x + Math.sin(time + i + phase) * 0.15, 2.1, x + sway * 0.8, -4.6)
    }
    c.strokeStyle = alpha(mix(color, '#ffffff', 0.35), 0.75)
    c.lineWidth = 0.14
    c.beginPath()
    c.moveTo(-1.1, bottom - 0.1)
    c.quadraticCurveTo(0, bottom + 0.3 + sway * 0.15, 1.1, bottom - 0.15)
    c.stroke()
  },
  // Neon rain: a glowing streak with a chromatic echo. Drawn along +y, aligned to velocity.
  rain(c, color, phase, time, alt) {
    c.strokeStyle = alpha(color, 0.16)
    c.lineWidth = 0.95
    seg(c, 0, -4.4, 0, 2.4)
    const g = c.createLinearGradient(0, -4.6, 0, 2.8)
    g.addColorStop(0, alpha(color, 0))
    g.addColorStop(0.7, alpha(color, 0.92))
    g.addColorStop(1, '#ffffff')
    c.strokeStyle = g
    c.lineWidth = 0.26
    seg(c, 0, -4.6, 0, 2.8)
    c.strokeStyle = alpha(alt, 0.55)
    c.lineWidth = 0.08
    seg(c, 0.5, -2.6, 0.5, 1.4)
    c.fillStyle = '#ffffff'
    dot(c, 0, 2.8, 0.2)
  },
  // Ember: a flickering flame tongue; variant 1 is a drifting ash flake.
  ember(c, color, phase, time, alt, variant) {
    if (variant === 1) {
      c.fillStyle = '#8f8580d0'
      c.beginPath()
      c.moveTo(-1.2, -0.4)
      c.lineTo(-0.2, -1)
      c.lineTo(1.1, -0.5)
      c.lineTo(0.8, 0.7)
      c.lineTo(-0.6, 0.9)
      c.closePath()
      c.fill()
      c.strokeStyle = alpha(color, 0.65)
      c.lineWidth = 0.1
      c.stroke()
      return
    }
    const flicker = 1 + Math.sin(time * 9 + phase * 5) * 0.12
    halo(c, color, 3.6, 0.5)
    const g = c.createLinearGradient(0, -3 * flicker, 0, 2)
    g.addColorStop(0, alpha(color, 0.08))
    g.addColorStop(0.55, color)
    g.addColorStop(1, '#db4825')
    c.fillStyle = g
    c.beginPath()
    c.moveTo(0, -3 * flicker)
    c.bezierCurveTo(1, -1.9 * flicker, -0.1, -1.1, 1.1, -0.5)
    c.bezierCurveTo(2.1, 1.6, 0.1, 2.2, -0.9, 1.2)
    c.bezierCurveTo(-2, 0.2, -0.6, -0.5 * flicker, 0, -3 * flicker)
    c.fill()
    c.fillStyle = '#fff0bd'
    c.beginPath()
    c.moveTo(0, -0.7 * flicker)
    c.quadraticCurveTo(1.2, 1.4, 0, 1.5)
    c.quadraticCurveTo(-0.8, 0.9, 0, -0.7 * flicker)
    c.fill()
  },
  // Sakura: a notched cherry-blossom petal with a blushing base.
  petal(c, color) {
    const g = c.createLinearGradient(0, -2.5, 0, 2.6)
    g.addColorStop(0, '#fff4f9')
    g.addColorStop(0.5, color)
    g.addColorStop(1, shade(color, -0.22))
    c.fillStyle = g
    c.beginPath()
    c.moveTo(0, 2.6)
    c.bezierCurveTo(-1.9, 1.3, -2.3, -1.3, -1, -2.4)
    c.quadraticCurveTo(-0.4, -2.65, 0, -1.85)
    c.quadraticCurveTo(0.4, -2.65, 1, -2.4)
    c.bezierCurveTo(2.3, -1.3, 1.9, 1.3, 0, 2.6)
    c.fill()
    c.fillStyle = alpha(shade(color, -0.35), 0.3)
    c.beginPath()
    c.ellipse(0, 1.7, 0.5, 0.75, 0, 0, TAU)
    c.fill()
    c.strokeStyle = '#ffffff80'
    c.lineWidth = 0.09
    seg(c, 0, 2.2, 0, -1.4)
    seg(c, 0, 0.9, -0.75, -0.6)
    seg(c, 0, 0.9, 0.75, -0.6)
  },
  // Ocean: a wobbling air bubble with rim light and reflections.
  bubble(c, color, phase, time) {
    const wob = Math.sin(time * 3 + phase) * 0.05
    c.save()
    c.scale(1 + wob, 1 - wob)
    const g = c.createRadialGradient(-0.7, -0.7, 0.1, 0, 0, 2)
    g.addColorStop(0, '#ffffff52')
    g.addColorStop(0.55, alpha(color, 0.05))
    g.addColorStop(1, alpha(color, 0.5))
    c.fillStyle = g
    dot(c, 0, 0, 2)
    c.strokeStyle = alpha(color, 0.9)
    c.lineWidth = 0.13
    c.beginPath()
    c.arc(0, 0, 2, 0, TAU)
    c.stroke()
    c.strokeStyle = '#ffffffc0'
    c.lineWidth = 0.22
    c.beginPath()
    c.arc(-0.05, -0.05, 1.5, 3.5, 4.7)
    c.stroke()
    c.strokeStyle = alpha(color, 0.45)
    c.lineWidth = 0.1
    c.beginPath()
    c.arc(0.1, 0.1, 1.55, 0.2, 1.3)
    c.stroke()
    c.fillStyle = '#ffffff'
    dot(c, -0.85, -1, 0.2)
    c.restore()
  },
  // Midnight orbit: a twinkling star with a short curved orbit trail behind it.
  orbit(c, color, phase, time, alt) {
    const tw = 0.7 + 0.3 * Math.sin(time * 2.2 + phase * 3)
    const g = c.createLinearGradient(-6.5, 0, 0, 0)
    g.addColorStop(0, alpha(alt, 0))
    g.addColorStop(1, alpha(alt, 0.6))
    c.strokeStyle = g
    c.lineWidth = 0.2
    c.beginPath()
    c.moveTo(-6.5, 1.1)
    c.quadraticCurveTo(-3.2, 0.15, 0, 0)
    c.stroke()
    halo(c, color, 3 * tw, 0.5)
    sparkle(c, 0, 0, 2.5 * tw, color)
    c.save()
    c.rotate(Math.PI / 4)
    sparkle(c, 0, 0, 1.25 * tw, '#ffffffb0')
    c.restore()
    c.fillStyle = '#ffffff'
    dot(c, 0, 0, 0.38)
  },
  // Enchanted forest: a firefly that blinks its lantern on and off.
  firefly(c, color, phase, time) {
    const blink = Math.max(0, Math.sin(time * 1.3 + phase * 4)) ** 2
    halo(c, color, 2.4 + blink * 3, 0.22 + blink * 0.5, 0, 0.65)
    const flap = 0.85 + Math.sin(time * 24 + phase) * 0.15
    c.fillStyle = '#dcf5ff80'
    c.beginPath()
    c.ellipse(-0.75, -0.45, 0.6 * flap, 1.3, -0.6, 0, TAU)
    c.fill()
    c.beginPath()
    c.ellipse(0.75, -0.45, 0.6 * flap, 1.3, 0.6, 0, TAU)
    c.fill()
    c.fillStyle = shade(color, -0.55)
    c.beginPath()
    c.ellipse(0, -0.35, 0.36, 0.75, 0, 0, TAU)
    c.fill()
    c.fillStyle = mix(color, '#fff8c0', 0.3 + blink * 0.6)
    dot(c, 0, 0.75, 0.62)
    c.fillStyle = alpha('#fffbe0', 0.35 + blink * 0.65)
    dot(c, 0, 0.8, 0.3)
    c.strokeStyle = shade(color, -0.4)
    c.lineWidth = 0.09
    seg(c, -0.15, -1, -0.6, -1.7)
    seg(c, 0.15, -1, 0.6, -1.7)
  },
  // Frost: a six-armed dendrite snowflake with a hexagonal centre plate.
  snow(c, color, phase) {
    halo(c, color, 2.8, 0.28)
    c.strokeStyle = color
    c.fillStyle = color
    c.lineWidth = 0.17
    for (let a = 0; a < 6; a++) {
      seg(c, 0, 0, 0, -2.4)
      seg(c, 0, -0.85, -0.5, -1.3)
      seg(c, 0, -0.85, 0.5, -1.3)
      seg(c, 0, -1.55, -0.38, -1.9)
      seg(c, 0, -1.55, 0.38, -1.9)
      dot(c, 0, -2.4, 0.14)
      c.rotate(Math.PI / 3)
    }
    c.fillStyle = mix(color, '#ffffff', 0.6)
    polygon(c, 6, 0.45, phase)
    c.fill()
    c.lineWidth = 0.08
    c.stroke()
  },
  // Sands of time: a cluster of grains with wind streaks, aligned to the gust.
  sand(c, color, phase, time, alt) {
    c.strokeStyle = alpha(color, 0.32)
    c.lineWidth = 0.11
    seg(c, -4.4, -0.45, -1.3, -0.25)
    seg(c, -3.6, 0.7, -1, 0.55)
    seg(c, -5, 0.15, -2.6, 0.12)
    const grains = [
      [0, 0, 0.72, color],
      [-1.35, 0.85, 0.42, alt],
      [-0.7, -0.95, 0.34, color],
      [1.15, 0.6, 0.3, mix(color, '#f3d9a4', 0.5)],
    ]
    for (const [x, y, r, col] of grains) {
      c.fillStyle = col
      c.beginPath()
      c.ellipse(x, y, r * 1.25, r * 0.85, 0.3, 0, TAU)
      c.fill()
      c.fillStyle = '#fff6dc90'
      dot(c, x - r * 0.35, y - r * 0.25, r * 0.3)
    }
  },
  // Amethyst cave: a faceted crystal with a companion shard and a passing glint.
  crystal(c, color, phase, time) {
    c.fillStyle = alpha(shade(color, 0.15), 0.8)
    c.beginPath()
    c.moveTo(-1.65, -0.7)
    c.lineTo(-1.05, 0.3)
    c.lineTo(-1.35, 1.5)
    c.lineTo(-2.15, 0.65)
    c.closePath()
    c.fill()
    const g = c.createLinearGradient(-1, -2.8, 1, 2.5)
    g.addColorStop(0, '#f7edff')
    g.addColorStop(0.4, color)
    g.addColorStop(1, shade(color, -0.55))
    c.fillStyle = g
    c.beginPath()
    c.moveTo(0, -2.8)
    c.lineTo(1.2, -0.7)
    c.lineTo(0.8, 1.7)
    c.lineTo(0, 2.5)
    c.lineTo(-1.1, 0.5)
    c.lineTo(-1, -1)
    c.closePath()
    c.fill()
    c.strokeStyle = '#ffffffb0'
    c.lineWidth = 0.12
    c.stroke()
    c.lineWidth = 0.1
    seg(c, 0, -2.8, 0.3, -0.4)
    seg(c, 0.3, -0.4, 0, 2.5)
    seg(c, -1, -1, 0.3, -0.4)
    seg(c, 0.3, -0.4, 1.2, -0.7)
    seg(c, -1.1, 0.5, 0.3, -0.4)
    sparkle(c, 0, -2.8, Math.max(0, Math.sin(time * 0.9 + phase * 3)) ** 10 * 2.2)
  },
  // Crimson eclipse: the "diamond ring" — dark moon, glowing corona and a white-hot bead.
  corona(c, color, phase, time, alt) {
    const pulse = 0.86 + 0.14 * Math.sin(time * 1.4 + phase)
    const outer = 4.3 * pulse
    const g = c.createRadialGradient(0, 0, 1.4, 0, 0, outer)
    g.addColorStop(0, alpha(color, 0.8))
    g.addColorStop(0.35, alpha(color, 0.28))
    g.addColorStop(1, alpha(color, 0))
    c.fillStyle = g
    dot(c, 0, 0, outer)
    c.strokeStyle = alpha(alt, 0.4)
    c.lineWidth = 0.09
    for (let i = 0; i < 12; i++) {
      const a = (i / 12) * TAU + phase
      const r = 2.5 + hash(i + phase * 7) * 1.7 * pulse
      seg(c, Math.cos(a) * 1.75, Math.sin(a) * 1.75, Math.cos(a) * r, Math.sin(a) * r)
    }
    c.fillStyle = '#0e0508eb'
    dot(c, 0, 0, 1.6)
    c.strokeStyle = '#ffffff99'
    c.lineWidth = 0.12
    c.beginPath()
    c.arc(0, 0, 1.62, 0, TAU)
    c.stroke()
    const a = phase + time * 0.15
    const bx = Math.cos(a) * 1.65,
      by = Math.sin(a) * 1.65
    halo(c, '#ffffff', 1.4, 0.85, bx, by)
    sparkle(c, bx, by, 1.5 * pulse, '#fff6f0')
  },
  // Gold and velvet: a crinkled flake of gold leaf whose sheen follows the tumble.
  goldleaf(c, color, phase, time, alt, variant, flip) {
    const light = (flip + 1) / 2
    const g = c.createLinearGradient(-2.2, -1.8, 2.2, 1.8)
    g.addColorStop(0, shade(color, -0.5))
    g.addColorStop(Math.min(0.92, Math.max(0.08, light)), mix(color, '#fff7dc', 0.7))
    g.addColorStop(1, shade(color, -0.35))
    c.fillStyle = g
    c.beginPath()
    for (let i = 0; i < 8; i++) {
      const a = (i / 8) * TAU + phase
      const r = 1.35 + hash(phase * 17 + i) * 0.95
      const x = Math.cos(a) * r * 1.15,
        y = Math.sin(a) * r * 0.85
      if (i) c.lineTo(x, y)
      else c.moveTo(x, y)
    }
    c.closePath()
    c.fill()
    c.strokeStyle = '#fff3c473'
    c.lineWidth = 0.07
    seg(c, -1.1, -0.3, 0.6, 0.4)
    seg(c, -0.2, -0.9, 0.3, 0.9)
    c.strokeStyle = alpha(shade(color, -0.6), 0.35)
    seg(c, -0.6, 0.6, 1, -0.2)
  },
  // Living lagoon: bioluminescent plankton; variant 1 is a cluster of tiny sparks.
  plankton(c, color, phase, time, alt, variant) {
    const pulse = 0.75 + 0.25 * Math.sin(time * 2 + phase * 3)
    if (variant === 1) {
      halo(c, color, 2.4 * pulse, 0.55)
      c.fillStyle = mix(color, '#ffffff', 0.6)
      dot(c, 0, 0, 0.35)
      dot(c, 0.9, 0.5, 0.2)
      dot(c, -0.7, 0.8, 0.16)
      return
    }
    halo(c, color, 3.6 * pulse, 0.45)
    c.strokeStyle = color
    c.lineWidth = 0.12
    c.beginPath()
    c.ellipse(0, 0, 1, 1.7, 0, 0, TAU)
    c.stroke()
    for (let i = 0; i < 5; i++) {
      const y = (i - 2) * 0.5
      const k = Math.sin(time * 3 + i + phase) * 0.25
      seg(c, -0.7, y, -1.7, y + 0.45 + k)
      seg(c, 0.7, y, 1.7, y + 0.45 - k)
    }
    c.fillStyle = mix(color, '#ffffff', 0.75)
    dot(c, 0, 0, 0.5 * pulse)
    c.strokeStyle = alpha(alt, 0.8)
    c.beginPath()
    c.moveTo(0, 1.6)
    c.bezierCurveTo(2, 2, -1, 3, 1, 3.5)
    c.stroke()
  },
  // Graphite: drafting figures — triangle, square, hexagon or protractor — with construction marks.
  geometry(c, color, phase, time, alt, variant) {
    c.strokeStyle = alpha(color, 0.9)
    c.fillStyle = color
    c.lineWidth = 0.1
    const marks = (pts) => pts.forEach(([x, y]) => dot(c, x, y, 0.16))
    const ring = (r) => {
      c.beginPath()
      c.arc(0, 0, r, 0, TAU)
      c.stroke()
    }
    if (variant === 0) {
      const pts = [0, 1, 2].map((i) => [
        Math.cos(-Math.PI / 2 + (i * TAU) / 3) * 2.2,
        Math.sin(-Math.PI / 2 + (i * TAU) / 3) * 2.2,
      ])
      polygon(c, 3, 2.2, -Math.PI / 2)
      c.stroke()
      c.strokeStyle = alpha(alt, 0.45)
      ring(2.2)
      seg(c, 0, -2.2, 0, 1.1)
      marks(pts)
    } else if (variant === 1) {
      c.strokeRect(-1.7, -1.7, 3.4, 3.4)
      c.strokeStyle = alpha(alt, 0.5)
      seg(c, -1.7, -1.7, 1.7, 1.7)
      seg(c, -1.7, 1.7, 1.7, -1.7)
      ring(1.7)
      marks([
        [-1.7, -1.7],
        [1.7, -1.7],
        [1.7, 1.7],
        [-1.7, 1.7],
      ])
    } else if (variant === 2) {
      polygon(c, 6, 2.1, Math.PI / 6)
      c.stroke()
      c.strokeStyle = alpha(alt, 0.45)
      for (let i = 0; i < 3; i++) {
        const a = Math.PI / 6 + (i * Math.PI) / 3
        seg(c, Math.cos(a) * 2.1, Math.sin(a) * 2.1, -Math.cos(a) * 2.1, -Math.sin(a) * 2.1)
      }
      dot(c, 0, 0, 0.18)
    } else {
      ring(2)
      for (let i = 0; i < 12; i++) {
        const a = (i / 12) * TAU
        const r = i % 3 ? 1.75 : 1.45
        seg(c, Math.cos(a) * r, Math.sin(a) * r, Math.cos(a) * 2, Math.sin(a) * 2)
      }
      c.strokeStyle = alpha(alt, 0.65)
      seg(c, 0, 0, Math.cos(phase) * 2, Math.sin(phase) * 2)
      dot(c, 0, 0, 0.16)
    }
  },
  // Rose quartz: a faceted heart gem with an occasional sparkle.
  heart(c, color, phase, time) {
    halo(c, color, 3.4, 0.28)
    const g = c.createLinearGradient(-2, -2, 2, 2)
    g.addColorStop(0, '#fff3fa')
    g.addColorStop(0.45, color)
    g.addColorStop(1, shade(color, -0.35))
    heartPath(c)
    c.fillStyle = g
    c.fill()
    c.save()
    heartPath(c)
    c.clip()
    c.fillStyle = '#ffffff38'
    c.beginPath()
    c.moveTo(-2.4, -0.9)
    c.lineTo(0, 0.2)
    c.lineTo(-1.3, -2.1)
    c.closePath()
    c.fill()
    c.strokeStyle = '#ffffff73'
    c.lineWidth = 0.09
    seg(c, 0, -1.25, 0, 2.3)
    seg(c, -2.4, -0.9, 0, 0.2)
    seg(c, 2.4, -0.9, 0, 0.2)
    seg(c, -1.3, -2.1, 0, 0.2)
    seg(c, 1.3, -2.1, 0, 0.2)
    c.restore()
    c.strokeStyle = alpha(shade(color, -0.3), 0.6)
    c.lineWidth = 0.1
    heartPath(c)
    c.stroke()
    sparkle(c, -1.3, -1.5, Math.max(0, Math.sin(time * 1.1 + phase * 4)) ** 12 * 1.6)
  },
  // Synthwave: an oscilloscope wave packet with a chromatic echo.
  wave(c, color, phase, time, alt) {
    const len = 9
    const trace = (dx, dy, stroke, width) => {
      c.strokeStyle = stroke
      c.lineWidth = width
      c.beginPath()
      for (let i = 0; i <= 26; i++) {
        const t = i / 26
        const x = -len / 2 + t * len
        const y = Math.sin(x * 1.6 + time * 3 + phase) * 0.95 * Math.sin(t * Math.PI)
        if (i) c.lineTo(x + dx, y + dy)
        else c.moveTo(x + dx, y + dy)
      }
      c.stroke()
    }
    trace(0, 0, alpha(color, 0.18), 0.9)
    trace(0.25, 0.18, alpha(alt, 0.8), 0.14)
    trace(0, 0, color, 0.18)
    trace(0, 0, '#ffffffb0', 0.06)
    c.fillStyle = alt
    dot(c, len / 2, 0, 0.16)
    dot(c, -len / 2, 0, 0.16)
  },
  // Matcha: a serrated tea leaf; variant 1 is a two-leaf bud.
  tea(c, color, phase, time, alt, variant) {
    if (variant === 1) {
      for (const side of [-1, 1]) {
        c.save()
        c.rotate(side * 0.55)
        c.translate(0, 0.6)
        leafPath(c, 0.55, 1.7)
        c.fillStyle = shade(color, side > 0 ? 0.12 : -0.12)
        c.fill()
        c.restore()
      }
      c.fillStyle = mix(color, '#f2f7df', 0.6)
      c.beginPath()
      c.ellipse(0, -1.3, 0.42, 1.05, 0, 0, TAU)
      c.fill()
      c.strokeStyle = shade(color, -0.35)
      c.lineWidth = 0.12
      seg(c, 0, 0.4, 0, 2.7)
      return
    }
    const g = c.createLinearGradient(-0.8, -2.8, 0.8, 2.8)
    g.addColorStop(0, shade(color, 0.28))
    g.addColorStop(1, shade(color, -0.28))
    leafPath(c, 1.05, 2.9, 0.12)
    c.fillStyle = g
    c.fill()
    c.strokeStyle = alpha(mix(color, '#f6f9e8', 0.75), 0.75)
    c.lineWidth = 0.09
    seg(c, 0, -2.7, 0, 3.2)
    for (let i = 0; i < 4; i++) {
      const y = -1.6 + i * 1
      seg(c, 0, y + 0.45, 0.72, y - 0.1)
      seg(c, 0, y + 0.45, -0.72, y - 0.1)
    }
  },
  // Blueberry mousse: a dusky berry with a crown; variant 1 is a cream dollop.
  berry(c, color, phase, time, alt, variant) {
    if (variant === 1) {
      c.fillStyle = '#00000022'
      c.beginPath()
      c.ellipse(0.15, 1.15, 1.7, 0.45, 0, 0, TAU)
      c.fill()
      const g = c.createLinearGradient(0, -2, 0, 1.3)
      g.addColorStop(0, '#fffdf7')
      g.addColorStop(1, mix('#fff3dc', alt, 0.25))
      c.fillStyle = g
      c.beginPath()
      c.ellipse(0, 0.65, 1.6, 0.62, 0, 0, TAU)
      c.fill()
      c.beginPath()
      c.ellipse(0.05, -0.05, 1.15, 0.52, 0, 0, TAU)
      c.fill()
      c.beginPath()
      c.moveTo(-0.65, -0.35)
      c.quadraticCurveTo(0.1, -2.3, 0.35, -1.85)
      c.quadraticCurveTo(0.75, -1.1, 0.65, -0.35)
      c.closePath()
      c.fill()
      c.strokeStyle = alpha(alt, 0.35)
      c.lineWidth = 0.08
      seg(c, -1.1, 0.5, 1, 0.45)
      return
    }
    const g = c.createRadialGradient(-0.6, -0.7, 0.2, 0, 0, 2)
    g.addColorStop(0, shade(color, 0.35))
    g.addColorStop(0.6, shade(color, -0.15))
    g.addColorStop(1, shade(color, -0.55))
    c.fillStyle = g
    dot(c, 0, 0, 1.9)
    c.fillStyle = '#ffffff1f'
    dot(c, 0.35, 0.4, 1.2)
    c.fillStyle = shade(color, -0.65)
    polygon(c, 5, 0.55, -Math.PI / 2, 0, -1.15)
    c.fill()
    c.fillStyle = shade(color, -0.35)
    dot(c, 0, -1.15, 0.22)
    c.fillStyle = '#ffffffb0'
    dot(c, -0.8, -0.55, 0.26)
  },
  // Peach cloud: a soft, flat-bottomed cloud with a tinted underside.
  cloud(c, color) {
    const puffs = [
      [-1.7, 0.35, 1.05],
      [-0.5, -0.35, 1.45],
      [0.95, -0.1, 1.2],
      [2.05, 0.45, 0.85],
      [0.25, 0.55, 1.1],
    ]
    c.fillStyle = alpha(color, 0.26)
    for (const [x, y, r] of puffs) dot(c, x + 0.18, y + 0.38, r)
    const g = c.createLinearGradient(0, -2, 0, 1.6)
    g.addColorStop(0, '#ffffff')
    g.addColorStop(1, mix('#ffffff', color, 0.32))
    c.fillStyle = g
    for (const [x, y, r] of puffs) dot(c, x, y, r)
    c.fillRect(-1.7, 0.4, 3.75, 0.9)
  },
  // Espresso: a roasted bean with its crease; variant 1 is a rising curl of steam.
  coffee(c, color, phase, time, alt, variant) {
    if (variant === 1) {
      const g = c.createLinearGradient(0, 3, 0, -3.6)
      g.addColorStop(0, alpha(color, 0))
      g.addColorStop(0.45, alpha(color, 0.55))
      g.addColorStop(1, alpha(color, 0))
      const s = Math.sin(time * 1.4 + phase)
      c.strokeStyle = g
      c.lineWidth = 0.34
      c.beginPath()
      c.moveTo(0, 3)
      c.bezierCurveTo(1.4 + s * 0.45, 1.5, -1.4 - s * 0.45, -0.5, 0.3, -3.6)
      c.stroke()
      c.lineWidth = 0.18
      c.beginPath()
      c.moveTo(0.8, 2.6)
      c.bezierCurveTo(-0.4 - s * 0.3, 1, 1.6 + s * 0.3, -0.8, 0.6, -2.8)
      c.stroke()
      return
    }
    const g = c.createRadialGradient(-0.6, -0.8, 0.2, 0, 0, 2.3)
    g.addColorStop(0, mix(color, '#7a4a2b', 0.55))
    g.addColorStop(0.5, '#6b3f24')
    g.addColorStop(1, '#2a160c')
    c.fillStyle = g
    c.beginPath()
    c.ellipse(0, 0, 1.45, 2.05, 0, 0, TAU)
    c.fill()
    c.strokeStyle = '#1b0d06'
    c.lineWidth = 0.22
    c.beginPath()
    c.moveTo(0.1, -1.8)
    c.bezierCurveTo(-0.7, -0.6, 0.7, 0.6, -0.1, 1.8)
    c.stroke()
    c.strokeStyle = alpha(color, 0.55)
    c.lineWidth = 0.08
    c.beginPath()
    c.moveTo(0.38, -1.6)
    c.bezierCurveTo(-0.35, -0.55, 0.95, 0.55, 0.22, 1.65)
    c.stroke()
  },
  // Lavender milk: a sprig of buds; variant 1 is a single loose bud.
  lavender(c, color, phase, time, alt, variant) {
    if (variant === 1) {
      c.fillStyle = color
      c.beginPath()
      c.ellipse(0, 0, 0.55, 0.95, 0, 0, TAU)
      c.fill()
      c.fillStyle = shade(color, 0.35)
      dot(c, -0.15, -0.35, 0.22)
      c.strokeStyle = '#7d9474'
      c.lineWidth = 0.12
      seg(c, 0, 0.9, 0.1, 1.7)
      return
    }
    c.strokeStyle = '#7d9474'
    c.lineWidth = 0.14
    c.beginPath()
    c.moveTo(0, 3.3)
    c.quadraticCurveTo(0.35, 0.6, 0, -2.9)
    c.stroke()
    for (let i = 0; i < 7; i++) {
      const t = i / 6
      const y = -2.7 + t * 3.5
      const s = 0.62 + t * 0.4
      for (const side of [-1, 1]) {
        c.fillStyle = side < 0 ? color : shade(color, 0.18)
        c.beginPath()
        c.ellipse(side * 0.34 * s, y, 0.28 * s, 0.44 * s, side * -0.5, 0, TAU)
        c.fill()
      }
    }
    c.fillStyle = shade(color, 0.3)
    dot(c, 0, -2.95, 0.22)
  },
  // Jade garden: a fan-shaped ginkgo leaf with fine golden veins.
  ginkgo(c, color, phase, time, alt) {
    const outline = () => {
      c.beginPath()
      c.moveTo(0, 1.5)
      c.lineTo(-2.5, -1.15)
      c.quadraticCurveTo(-1.6, -2.65, -0.25, -2)
      c.lineTo(0, -1.2)
      c.lineTo(0.25, -2)
      c.quadraticCurveTo(1.6, -2.65, 2.5, -1.15)
      c.closePath()
    }
    const g = c.createRadialGradient(0, 1.6, 0.2, 0, 1.6, 3.8)
    g.addColorStop(0, shade(color, -0.3))
    g.addColorStop(0.7, color)
    g.addColorStop(1, shade(color, 0.25))
    outline()
    c.fillStyle = g
    c.fill()
    c.save()
    outline()
    c.clip()
    c.strokeStyle = alpha(mix(alt, '#fff4cd', 0.4), 0.7)
    c.lineWidth = 0.06
    for (let i = -6; i <= 6; i++) {
      const a = -Math.PI / 2 + i * 0.13
      seg(c, 0, 1.5, Math.cos(a) * 3.6, 1.5 + Math.sin(a) * 3.6)
    }
    c.restore()
    c.strokeStyle = shade(color, -0.35)
    c.lineWidth = 0.15
    c.beginPath()
    c.moveTo(0, 1.5)
    c.quadraticCurveTo(0.3, 2.6, -0.2, 3.4)
    c.stroke()
  },
  // Cobalt: a comet with a tapered dust tail and a fainter ion tail.
  comet(c, color, phase, time, alt) {
    const tail = 9
    const g = c.createLinearGradient(-tail, 0, 0, 0)
    g.addColorStop(0, alpha(color, 0))
    g.addColorStop(1, alpha(color, 0.8))
    c.fillStyle = g
    c.beginPath()
    c.moveTo(0, -0.7)
    c.quadraticCurveTo(-tail * 0.5, -0.45, -tail, 0)
    c.quadraticCurveTo(-tail * 0.5, 0.45, 0, 0.7)
    c.closePath()
    c.fill()
    const ion = c.createLinearGradient(-tail * 1.1, -1, 0, 0)
    ion.addColorStop(0, alpha(alt, 0))
    ion.addColorStop(1, alpha(alt, 0.65))
    c.strokeStyle = ion
    c.lineWidth = 0.12
    seg(c, -0.3, -0.25, -tail * 1.1, -1.1)
    c.fillStyle = '#ffffffb0'
    for (let i = 0; i < 4; i++) {
      const x = -1.5 - hash(phase + i) * 6
      dot(c, x, (hash(phase * 3 + i) - 0.5) * 0.7, 0.08 + 0.06 * Math.sin(time * 4 + i))
    }
    halo(c, color, 2.4, 0.7)
    c.fillStyle = '#ffffff'
    dot(c, 0, 0, 0.55)
  },
  // Cherry velvet: soft, out-of-focus bokeh; variant 1 uses a six-blade aperture.
  bokeh(c, color, phase, time, alt, variant) {
    const breathe = 0.8 + 0.2 * Math.sin(time * 0.7 + phase)
    const r = 2.4
    const shape = () => {
      if (variant === 1) polygon(c, 6, r, phase)
      else {
        c.beginPath()
        c.arc(0, 0, r, 0, TAU)
      }
    }
    halo(c, color, r * 1.55, 0.12 * breathe)
    const g = c.createRadialGradient(0, 0, 0, 0, 0, r)
    g.addColorStop(0, alpha(color, 0.16 * breathe))
    g.addColorStop(0.78, alpha(color, 0.28 * breathe))
    g.addColorStop(1, alpha(color, 0.5 * breathe))
    shape()
    c.fillStyle = g
    c.fill()
    c.strokeStyle = alpha(mix(color, '#ffffff', 0.45), 0.4 * breathe)
    c.lineWidth = 0.12
    shape()
    c.stroke()
  },
  // Honey light: a cluster of comb cells, one filled with honey; variant 1 is a honey drop.
  honeycomb(c, color, phase, time, alt, variant) {
    const honey = (y0, y1) => {
      const g = c.createLinearGradient(0, y0, 0, y1)
      g.addColorStop(0, '#ffd975')
      g.addColorStop(0.55, '#e7a52a')
      g.addColorStop(1, '#b8720f')
      return g
    }
    if (variant === 1) {
      c.fillStyle = honey(-2, 1.6)
      c.beginPath()
      c.moveTo(0, -2.1)
      c.bezierCurveTo(0.5, -0.9, 1.3, -0.2, 1.3, 0.55)
      c.arc(0, 0.55, 1.3, 0, Math.PI)
      c.bezierCurveTo(-1.3, -0.2, -0.5, -0.9, 0, -2.1)
      c.fill()
      c.fillStyle = '#ffffffc0'
      dot(c, -0.45, 0.3, 0.25)
      return
    }
    const R = 1
    const cells = [
      [-0.87, -0.5],
      [0.87, -0.5],
      [0, 1],
    ]
    const filled = Math.floor(hash(phase * 5) * 3)
    cells.forEach(([x, y], i) => {
      polygon(c, 6, R, Math.PI / 6, x, y)
      if (i === filled) {
        c.fillStyle = honey(y - R, y + R)
        c.fill()
      } else {
        c.fillStyle = alpha(color, 0.12)
        c.fill()
      }
      c.strokeStyle = color
      c.lineWidth = 0.16
      c.stroke()
    })
    const [fx, fy] = cells[filled]
    c.strokeStyle = '#ffffffb0'
    c.lineWidth = 0.1
    c.beginPath()
    c.arc(fx, fy, 0.55, 3.6, 4.6)
    c.stroke()
  },
  // Arctic night: diamond dust — tiny ice plates that flash as they catch the light.
  diamond(c, color, phase, time) {
    const flash = Math.max(0, Math.sin(time * 1.6 + phase * 5)) ** 14
    c.fillStyle = alpha(mix(color, '#ffffff', 0.5), 0.75)
    polygon(c, 6, 0.9)
    c.fill()
    c.strokeStyle = '#ffffffcc'
    c.lineWidth = 0.08
    c.stroke()
    if (flash > 0.02) {
      halo(c, '#ffffff', 0.6 + flash * 2.4, 0.8 * flash)
      const l = 1 + flash * 3.5
      c.strokeStyle = alpha('#ffffff', flash)
      c.lineWidth = 0.1
      seg(c, -l, 0, l, 0)
      seg(c, 0, -l, 0, l)
      c.lineWidth = 0.06
      const d = l * 0.5
      seg(c, -d, -d, d, d)
      seg(c, -d, d, d, -d)
    }
  },
  // Terminal: a falling column of phosphor glyphs with a bright head.
  code(c, color, phase, time) {
    c.font = '700 1.9px Consolas, "Cascadia Mono", "JetBrains Mono", "Courier New", monospace'
    c.textAlign = 'center'
    c.textBaseline = 'middle'
    const tick = Math.floor(time * 5 + phase * 13)
    halo(c, color, 1.9, 0.45)
    for (let i = 0; i < 6; i++) {
      const slow = i === 0 ? tick : Math.floor((tick + i * 7) / 3)
      const glyph = GLYPHS[Math.floor(hash(phase * 97 + i * 13 + slow) * GLYPHS.length)]
      c.fillStyle = i === 0 ? '#f2fff4' : alpha(color, Math.max(0, 0.95 - i * 0.16))
      c.fillText(glyph, 0, -i * 2.05)
    }
  },
  // Moonstone: a silver moon whose phase slowly waxes and wanes.
  moon(c, color, phase, time) {
    const f = (((phase / TAU + time * 0.015) % 1) + 1) % 1
    const r = 1.9
    halo(c, color, r * 1.9, 0.22)
    c.fillStyle = alpha(color, 0.3)
    dot(c, 0, 0, r)
    c.save()
    if (f > 0.5) c.scale(-1, 1)
    const theta = (f > 0.5 ? 1 - f : f) * TAU
    const rx = Math.abs(Math.cos(theta)) * r
    c.beginPath()
    c.arc(0, 0, r, -Math.PI / 2, Math.PI / 2, false)
    c.ellipse(0, 0, Math.max(0.001, rx), r, 0, Math.PI / 2, -Math.PI / 2, theta < Math.PI / 2)
    c.closePath()
    const g = c.createRadialGradient(0.6, -0.6, 0.2, 0, 0, r)
    g.addColorStop(0, '#ffffff')
    g.addColorStop(1, mix('#ffffff', color, 0.38))
    c.fillStyle = g
    c.fill()
    c.clip()
    c.fillStyle = alpha(color, 0.22)
    dot(c, 0.7, -0.5, 0.35)
    dot(c, 0.2, 0.75, 0.25)
    dot(c, 1.15, 0.45, 0.18)
    c.restore()
    c.strokeStyle = alpha(color, 0.55)
    c.lineWidth = 0.09
    c.beginPath()
    c.arc(0, 0, r, 0, TAU)
    c.stroke()
  },
  // Coral reef: a small reef fish; variant 0 is a clownfish, variant 1 a tang.
  fish(c, color, phase, time, alt, variant) {
    const body = variant === 1 ? alt : color
    const wag = Math.sin(time * 7 + phase) * 0.35
    c.fillStyle = shade(body, -0.15)
    c.beginPath()
    c.moveTo(-1.6, 0)
    c.lineTo(-2.8, -1 + wag)
    c.quadraticCurveTo(-2.35, wag * 0.5, -2.8, 1 + wag)
    c.closePath()
    c.fill()
    c.fillStyle = shade(body, -0.22)
    c.beginPath()
    c.moveTo(-0.6, -0.95)
    c.quadraticCurveTo(0.2, -1.95, 0.95, -0.9)
    c.closePath()
    c.fill()
    const g = c.createLinearGradient(0, -1.2, 0, 1.2)
    g.addColorStop(0, shade(body, 0.22))
    g.addColorStop(1, shade(body, -0.25))
    c.fillStyle = g
    c.beginPath()
    c.ellipse(0, 0, 2, 1.15, 0, 0, TAU)
    c.fill()
    if (variant === 1) {
      c.strokeStyle = '#1d2b4a99'
      c.lineWidth = 0.18
      c.beginPath()
      c.moveTo(-1.2, -0.2)
      c.quadraticCurveTo(0, -0.9, 1.3, -0.1)
      c.stroke()
      c.fillStyle = '#ffd86b'
      c.beginPath()
      c.moveTo(-2.2, -0.35 + wag * 0.4)
      c.lineTo(-2.75, -0.95 + wag)
      c.lineTo(-2.75, 0.95 + wag)
      c.lineTo(-2.2, 0.35 + wag * 0.4)
      c.closePath()
      c.fill()
    } else {
      c.save()
      c.beginPath()
      c.ellipse(0, 0, 2, 1.15, 0, 0, TAU)
      c.clip()
      for (const x of [-0.75, 0.7]) {
        c.fillStyle = '#ffffff'
        c.fillRect(x - 0.22, -1.3, 0.44, 2.6)
        c.strokeStyle = '#2b1612'
        c.lineWidth = 0.08
        c.strokeRect(x - 0.22, -1.3, 0.44, 2.6)
      }
      c.restore()
    }
    c.fillStyle = '#ffffff'
    dot(c, 1.25, -0.25, 0.28)
    c.fillStyle = '#141414'
    dot(c, 1.32, -0.25, 0.15)
  },
  // Ultraviolet: a crackling plasma arc that re-forms several times per second.
  plasma(c, color, phase, time, alt) {
    const step = Math.floor(time * 7 + phase * 10)
    const visible = hash(step * 0.37 + phase)
    halo(c, color, 3.2, visible < 0.12 ? 0.12 : 0.26)
    if (visible < 0.12) return
    const pts = [[-3, 0]]
    for (let i = 1; i <= 5; i++)
      pts.push([-3 + i * 1.2, (hash(step + i * 3.1 + phase) - 0.5) * 1.8])
    const fork = pts[2]
    const branch = [
      fork,
      [fork[0] + 0.7, fork[1] + 0.9 + hash(step + phase * 2) * 0.6],
      [fork[0] + 1.3, fork[1] + 1.8],
    ]
    for (const [stroke, width] of [
      [alpha(color, 0.35), 0.6],
      [alt, 0.16],
      ['#ffffff', 0.06],
    ]) {
      c.strokeStyle = stroke
      c.lineWidth = width
      polyline(c, pts)
      c.lineWidth = width * 0.7
      polyline(c, branch)
    }
  },
  // Titanium: a machined gear with spokes; variant 1 is a hex nut.
  gear(c, color, phase, time, alt, variant) {
    const metal = (r) => {
      const g = c.createLinearGradient(-r, -r, r, r)
      g.addColorStop(0, shade(color, 0.4))
      g.addColorStop(0.5, color)
      g.addColorStop(1, shade(color, -0.45))
      return g
    }
    if (variant === 1) {
      polygon(c, 6, 1.9)
      c.fillStyle = metal(1.9)
      c.fill()
      c.strokeStyle = '#ffffff40'
      c.lineWidth = 0.08
      c.stroke()
      c.strokeStyle = alpha(shade(color, -0.4), 0.7)
      c.beginPath()
      c.arc(0, 0, 1.35, 0, TAU)
      c.stroke()
      c.fillStyle = '#00000080'
      dot(c, 0, 0, 0.8)
      return
    }
    const teeth = 10,
      R = 2.2,
      r = 1.75
    c.beginPath()
    for (let i = 0; i < teeth; i++) {
      const a = (i / teeth) * TAU
      const s = TAU / teeth
      const pts = [
        [r, a],
        [R, a + s * 0.2],
        [R, a + s * 0.5],
        [r, a + s * 0.7],
      ]
      pts.forEach(([rad, ang], j) => {
        const x = Math.cos(ang) * rad,
          y = Math.sin(ang) * rad
        if (i === 0 && j === 0) c.moveTo(x, y)
        else c.lineTo(x, y)
      })
    }
    c.closePath()
    c.fillStyle = metal(R)
    c.fill()
    c.strokeStyle = '#ffffff40'
    c.lineWidth = 0.08
    c.stroke()
    c.fillStyle = '#00000040'
    dot(c, 0, 0, 1.25)
    c.strokeStyle = color
    c.lineWidth = 0.22
    for (let i = 0; i < 4; i++) {
      const a = (i / 4) * TAU + 0.4
      seg(c, Math.cos(a) * 0.45, Math.sin(a) * 0.45, Math.cos(a) * 1.25, Math.sin(a) * 1.25)
    }
    c.fillStyle = color
    dot(c, 0, 0, 0.5)
    c.fillStyle = '#00000099'
    dot(c, 0, 0, 0.22)
  },
  // After sunset: a paper sky lantern with a flickering flame.
  lantern(c, color, phase, time) {
    const flick = 0.85 + 0.15 * Math.sin(time * 11 + phase * 7) * Math.sin(time * 4.3 + phase)
    halo(c, color, 4.2, 0.34 * flick, 0, 0.4)
    const g = c.createLinearGradient(0, -2.4, 0, 2.2)
    g.addColorStop(0, shade(color, -0.25))
    g.addColorStop(0.55, mix(color, '#fff3d6', 0.35 * flick))
    g.addColorStop(1, '#fff1c9')
    c.fillStyle = g
    c.beginPath()
    c.moveTo(-1.5, -2.3)
    c.quadraticCurveTo(0, -2.7, 1.5, -2.3)
    c.lineTo(1.05, 2)
    c.quadraticCurveTo(0, 2.25, -1.05, 2)
    c.closePath()
    c.fill()
    c.strokeStyle = alpha(shade(color, -0.4), 0.4)
    c.lineWidth = 0.07
    seg(c, -0.5, -2.45, -0.35, 2.12)
    seg(c, 0.5, -2.45, 0.35, 2.12)
    c.fillStyle = alpha('#fff6d8', 0.95 * flick)
    c.beginPath()
    c.ellipse(0, 1.72, 0.3, 0.45 * flick, 0, 0, TAU)
    c.fill()
  },
}

export const PARTICLE_KINDS = Object.freeze(Object.keys(ART))

export function particleArt(
  c,
  kind,
  color,
  phase = 0,
  time = 0,
  alt = color,
  variant = 0,
  flip = 1,
) {
  c.fillStyle = color
  c.strokeStyle = color
  c.lineWidth = 0.16
  c.lineCap = 'round'
  c.lineJoin = 'round'
  ;(ART[kind] ?? ART.firefly)(c, color, phase, time, alt || color, variant || 0, flip)
}
