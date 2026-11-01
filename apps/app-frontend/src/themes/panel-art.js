// Ornaments for the top bar and the sidebar. Every theme kind has its own frame pattern
// (drawn along the bar) and its own motif (drawn at a few anchor points). Purely decorative:
// the canvas sits below the controls and never receives pointer events.
import { particleArt, mix, shade, alpha, hash } from './particle-art.js'
import { seededRandom } from './particles.js'

const TAU = Math.PI * 2
const sessionSeed = Date.now().toString(36)

export function panelScene(theme, vertical) {
  const r = seededRandom(`${sessionSeed}:${theme.id}:${vertical}`)
  const count = vertical ? 9 : 12
  return Array.from({ length: count }, (_, i) => ({
    index: i,
    count,
    t: (i + 0.3 + r() * 0.4) / count,
    side: r() > 0.5 ? 1 : -1,
    size: 0.85 + r() * 0.3,
    phase: r() * TAU,
    seed: r(),
  }))
}

// ---------- small drawing helpers ----------
function along(c, P, fn, step = 8, from = 0, to = P.len) {
  c.beginPath()
  let first = true
  for (let u = from; ; u += step) {
    const x = Math.min(u, to)
    const v = fn(x)
    if (first) c.moveTo(x, v)
    else c.lineTo(x, v)
    first = false
    if (x >= to) break
  }
}
function stroke(c, color, width) {
  c.strokeStyle = color
  c.lineWidth = width
  c.stroke()
}
function glow(c, color, width) {
  stroke(c, alpha(color, 0.16), width * 4)
  stroke(c, color, width)
}
function disc(c, x, y, r) {
  c.beginPath()
  c.arc(x, y, Math.max(0, r), 0, TAU)
  c.fill()
}
function line(c, x, y, xx, yy) {
  c.beginPath()
  c.moveTo(x, y)
  c.lineTo(xx, yy)
  c.stroke()
}
function art(c, kind, color, s, phase, time, alt = color, variant = 0, flip = 1) {
  c.save()
  c.scale(s, s)
  particleArt(c, kind, color, phase, time, alt, variant, flip)
  c.restore()
}
function at(c, x, y, rotation, draw) {
  c.save()
  c.translate(x, y)
  if (rotation) c.rotate(rotation)
  draw()
  c.restore()
}
function hex(c, x, y, r) {
  c.beginPath()
  for (let i = 0; i < 6; i++) {
    const a = Math.PI / 6 + (i * TAU) / 6
    if (i) c.lineTo(x + Math.cos(a) * r, y + Math.sin(a) * r)
    else c.moveTo(x + Math.cos(a) * r, y + Math.sin(a) * r)
  }
  c.closePath()
}
function jagged(c, x0, y0, x1, y1, seed, parts = 6, amp = 3) {
  c.beginPath()
  c.moveTo(x0, y0)
  const dx = x1 - x0,
    dy = y1 - y0,
    len = Math.hypot(dx, dy) || 1
  for (let i = 1; i < parts; i++) {
    const t = i / parts,
      off = (hash(seed + i * 1.7) - 0.5) * 2 * amp
    c.lineTo(x0 + dx * t - (dy / len) * off, y0 + dy * t + (dx / len) * off)
  }
  c.lineTo(x1, y1)
}
// A soft colour that stays visible on both light and dark palettes.
const ink = (P, color, lift = 0.35) => (P.light ? shade(color, -0.1) : mix(color, '#ffffff', lift))

// ---------- designs ----------
// mode: where motifs sit. hang = from the top edge, grow = up from the bottom edge,
// wall = into the bar from either edge, float = inside the bar. sideMode overrides it
// for the vertical sidebar. Edge patterns are drawn in bar space: u along, v across.
const DESIGNS = {
  aurora: {
    mode: 'hang',
    edge(c, P) {
      for (let j = 0; j < 3; j++) {
        along(
          c,
          P,
          (u) =>
            P.thick * (0.18 + j * 0.1) +
            Math.sin(u * 0.012 + j * 1.7 + P.time * 0.25) * P.thick * 0.09,
          10,
        )
        stroke(
          c,
          alpha(j === 1 ? P.pal.secondary : P.pal.accent, 0.1 + j * 0.03),
          P.thick * (0.2 - j * 0.05),
        )
      }
    },
    motif(c, P, n) {
      at(c, 0, 15, 0, () => art(c, 'aurora', P.pal.accent, 2.2, n.phase, P.time, P.pal.secondary))
      c.fillStyle = ink(P, P.pal.secondary, 0.6)
      disc(c, 9, 6 + n.seed * 6, 0.9)
      disc(c, -8, 20 - n.seed * 5, 0.7)
    },
  },
  rain: {
    mode: 'grow',
    sideMode: 'float',
    edge(c, P) {
      c.setLineDash([P.len * 0.16, 26, P.len * 0.06, 18])
      const flicker = 0.75 + 0.25 * Math.sin(P.time * 7) * Math.sin(P.time * 2.3)
      along(c, P, () => 3, P.len)
      glow(c, alpha(P.pal.accent, 0.8 * flicker), 1)
      c.lineDashOffset = 40
      along(c, P, () => P.thick - 3, P.len)
      glow(c, alpha(P.pal.secondary, 0.7), 1)
      c.setLineDash([])
      c.lineDashOffset = 0
    },
    motif(c, P, n) {
      const k = (P.time * 0.9 + n.seed) % 1
      c.strokeStyle = alpha(P.pal.secondary, 0.7 * (1 - k))
      c.lineWidth = 0.9
      for (const r of [k, (k + 0.5) % 1]) {
        c.beginPath()
        c.ellipse(0, -3, 3 + r * 11, 1 + r * 3, 0, 0, TAU)
        c.stroke()
      }
      const drop = (P.time * 1.6 + n.phase) % 1
      at(c, 0, -26 + drop * 22, 0, () =>
        art(c, 'rain', P.pal.accent, 1.2, n.phase, P.time, P.pal.secondary),
      )
    },
  },
  ember: {
    mode: 'grow',
    edge(c, P) {
      along(c, P, (u) => P.thick - 1.5 - Math.abs(Math.sin(u * 0.05 + P.time * 0.8)) * 1.5, 6)
      const flicker = 0.7 + 0.3 * Math.sin(P.time * 5)
      glow(c, alpha(P.pal.accent, 0.75 * flicker), 1.4)
      c.fillStyle = mix(P.pal.accent, '#fff2c4', 0.4)
      for (let i = 0; i < P.len / 40; i++) {
        const life = (P.time * 0.35 + hash(i * 3.1)) % 1
        c.globalAlpha = 0.5 * (1 - life)
        disc(c, (hash(i) * P.len + life * 9) % P.len, P.thick - 2 - life * P.thick * 0.8, 0.9)
      }
      c.globalAlpha = 0.5
    },
    motif(c, P, n) {
      for (let a = 0; a < 3; a++) {
        const flick = Math.sin(P.time * 3 + a + n.phase)
        c.save()
        c.translate((a - 1) * 6, -7 - (a === 1 ? 3 : 0))
        c.rotate(Math.sin(P.time * 2 + n.phase + a) * 0.12)
        c.scale(2.4, (a === 1 ? 4.6 : 3.4) + flick * 0.6)
        particleArt(c, 'ember', P.pal.accent, a, P.time)
        c.restore()
      }
      c.strokeStyle = alpha(P.pal.muted, 0.4)
      c.lineWidth = 1.6
      c.beginPath()
      c.moveTo(0, -20)
      c.bezierCurveTo(7, -26, -5, -30, 2 + Math.sin(P.time + n.phase) * 3, -38)
      c.stroke()
    },
  },
  petal: {
    mode: 'hang',
    edge(c, P) {
      along(c, P, (u) => 4 + Math.sin(u * 0.02) * 2.5, 8)
      stroke(c, alpha(shade(P.pal.accent, -0.55), 0.9), 1.6)
      for (let u = 30; u < P.len; u += 70) {
        const v = 4 + Math.sin(u * 0.02) * 2.5
        line(c, u, v, u + 12, v + 6)
      }
    },
    motif(c, P, n) {
      c.strokeStyle = alpha(shade(P.pal.accent, -0.55), 0.9)
      c.lineWidth = 1
      line(c, 0, 4, 2, 9)
      const bloom = (x, y, s, rot) =>
        at(c, x, y, rot, () => {
          for (let a = 0; a < 5; a++) {
            c.rotate(TAU / 5)
            c.save()
            c.translate(0, -s * 2.2)
            art(c, 'petal', P.pal.accent, s, a, P.time)
            c.restore()
          }
          c.fillStyle = '#ffe3a8'
          disc(c, 0, 0, s * 0.8)
        })
      bloom(2, 11, 1.25, n.phase)
      bloom(-6, 7, 0.8, n.phase * 2)
      const fall = (P.time * 0.18 + n.seed) % 1
      c.globalAlpha *= 1 - fall
      at(
        c,
        4 + Math.sin(P.time * 2 + n.phase) * 5,
        16 + fall * (P.thick - 12),
        P.time * 1.4 + n.phase,
        () => art(c, 'petal', P.pal.accent, 1, n.phase, P.time),
      )
    },
  },
  bubble: {
    mode: 'grow',
    sideMode: 'float',
    edge(c, P) {
      along(c, P, (u) => 4 + Math.sin(u * 0.045 + P.time * 1.2) * 1.4, 5)
      stroke(c, alpha(P.pal.accent, 0.75), 1.1)
      along(c, P, (u) => 7.5 + Math.sin(u * 0.03 - P.time * 0.9 + 1) * 1.2, 6)
      stroke(c, alpha(P.pal.secondary, 0.35), 0.8)
    },
    motif(c, P, n) {
      for (let i = 0; i < 4; i++) {
        const life = (P.time * 0.22 + n.seed + i * 0.25) % 1
        c.save()
        c.globalAlpha *= Math.min(1, (1 - life) * 2.5)
        c.translate(Math.sin(life * 9 + n.phase + i) * 2.5, -3 - life * 30)
        art(c, 'bubble', P.pal.accent, 0.7 + (1 - life) * 0.9, n.phase + i, P.time)
        c.restore()
      }
    },
  },
  orbit: {
    mode: 'float',
    edge(c, P) {
      c.setLineDash([1, 7])
      c.lineDashOffset = -P.time * 4
      along(c, P, () => 3, P.len)
      stroke(c, alpha(P.pal.secondary, 0.8), 1.2)
      along(c, P, () => P.thick - 3, P.len)
      stroke(c, alpha(P.pal.accent, 0.6), 1.2)
      c.setLineDash([])
      c.lineDashOffset = 0
    },
    motif(c, P, n) {
      const tilt = -0.35 + n.seed * 0.3
      const a = P.time * (0.6 + n.seed * 0.5) + n.phase
      const mx = Math.cos(a) * 10,
        my = Math.sin(a) * 3.4
      const ringBack = () => {
        c.strokeStyle = alpha(P.pal.secondary, 0.75)
        c.lineWidth = 0.9
        c.beginPath()
        c.ellipse(0, 0, 9, 2.8, tilt, Math.PI, TAU)
        c.stroke()
      }
      const moon = () => {
        c.fillStyle = ink(P, P.pal.secondary, 0.5)
        disc(
          c,
          mx * Math.cos(tilt) - my * Math.sin(tilt),
          mx * Math.sin(tilt) + my * Math.cos(tilt),
          1.1,
        )
      }
      ringBack()
      if (my < 0) moon()
      const g = c.createRadialGradient(-1.5, -1.5, 0.5, 0, 0, 5)
      g.addColorStop(0, mix(P.pal.accent, '#ffffff', 0.45))
      g.addColorStop(1, shade(P.pal.accent, -0.35))
      c.fillStyle = g
      disc(c, 0, 0, 4.4)
      c.strokeStyle = P.pal.secondary
      c.lineWidth = 0.9
      c.beginPath()
      c.ellipse(0, 0, 9, 2.8, tilt, 0, Math.PI)
      c.stroke()
      if (my >= 0) moon()
    },
  },
  firefly: {
    mode: 'float',
    edge(c, P) {
      const vine = (u) => 5 + Math.sin(u * 0.03) * 3
      along(c, P, vine, 6)
      stroke(c, alpha(shade(P.pal.accent, -0.25), 0.8), 1.2)
      c.fillStyle = alpha(shade(P.pal.accent, -0.15), 0.75)
      for (let u = 14, i = 0; u < P.len; u += 23, i++) {
        const side = i % 2 ? 1 : -1
        at(c, u, vine(u), side * 0.9 + Math.sin(P.time * 0.8 + i) * 0.1, () => {
          c.beginPath()
          c.ellipse(0, side * 3.2, 1.5, 3.2, 0, 0, TAU)
          c.fill()
        })
      }
    },
    motif(c, P, n) {
      at(c, Math.sin(P.time * 0.5 + n.phase) * 4, Math.sin(P.time * 0.8 + n.phase) * 3, 0, () =>
        art(c, 'firefly', P.pal.accent, 1.6, n.phase, P.time),
      )
    },
  },
  snow: {
    mode: 'hang',
    edge(c, P) {
      c.beginPath()
      c.moveTo(0, P.thick)
      for (let u = 0, i = 0; u < P.len; i++) {
        const r = 3 + hash(i * 2.3) * 4
        c.arc(u + r, P.thick, r, Math.PI, TAU)
        u += r * 2
      }
      c.fillStyle = alpha(ink(P, P.pal.accent, 0.75), P.light ? 0.16 : 0.22)
      c.fill()
      c.strokeStyle = alpha(ink(P, P.pal.accent, 0.75), 0.5)
      c.lineWidth = 0.8
      c.stroke()
    },
    motif(c, P, n) {
      const len = 7 + n.seed * 9
      c.strokeStyle = alpha(ink(P, P.pal.accent, 0.5), 0.6)
      c.lineWidth = 0.6
      line(c, 0, 0, 0, len)
      at(c, 0, len + 5, Math.sin(P.time * 0.6 + n.phase) * 0.5, () =>
        art(c, 'snow', P.pal.accent, 1.9, n.phase, P.time),
      )
    },
  },
  sand: {
    mode: 'float',
    edge(c, P) {
      for (let j = 0; j < 2; j++) {
        c.beginPath()
        c.moveTo(0, P.thick)
        for (let u = 0; u <= P.len; u += 8) {
          const v = P.thick - 3 - j * 4 - (Math.sin(u * 0.012 + j * 2.2) * 0.5 + 0.5) * (4 + j * 2)
          c.lineTo(u, v)
        }
        c.lineTo(P.len, P.thick)
        c.closePath()
        c.fillStyle = alpha(j ? P.pal.secondary : P.pal.accent, (P.light ? 0.2 : 0.12) + j * 0.06)
        c.fill()
      }
    },
    motif(c, P, n) {
      const spin = P.time * 0.5 + n.phase
      c.strokeStyle = alpha(P.pal.accent, 0.9)
      c.lineWidth = 1.1
      c.beginPath()
      for (let i = 0; i <= 40; i++) {
        const t = i / 40
        const a = spin + t * TAU * 1.4
        const r = 1 + t * 7
        const x = Math.cos(a) * r - t * 10,
          y = Math.sin(a) * r * 0.55
        if (i) c.lineTo(x, y)
        else c.moveTo(x, y)
      }
      c.stroke()
      c.fillStyle = P.pal.secondary
      for (let i = 0; i < 3; i++) disc(c, 6 + i * 4, Math.sin(P.time * 2 + i + n.phase) * 1.5, 0.8)
    },
  },
  crystal: {
    mode: 'wall',
    edge(c, P) {
      along(c, P, (u) => 3 + (Math.abs(((u / 6) % 2) - 1) - 0.5) * 3, 6)
      stroke(c, alpha(P.pal.accent, 0.7), 0.8)
    },
    motif(c, P, n) {
      const shards = [
        [-5, -0.45, 1.3, 2.1, P.pal.secondary],
        [5, 0.5, 1.2, 1.8, P.pal.accent],
        [0, 0.04, 1.6, 2.8, P.pal.accent],
      ]
      for (const [x, rot, sx, sy, color] of shards)
        at(c, x, 0, rot, () => {
          c.translate(0, -2.5 * sy)
          c.scale(sx, sy)
          particleArt(c, 'crystal', color, n.phase + x, P.time)
        })
    },
  },
  corona: {
    mode: 'float',
    edge(c, P) {
      along(c, P, () => 2.5, P.len)
      stroke(c, alpha(P.pal.accent, 0.55), 0.7)
      along(c, P, () => P.thick - 2.5, P.len)
      stroke(c, alpha(P.pal.secondary, 0.45), 0.7)
    },
    // The bar reads as an eclipse sequence: partial, total, partial.
    motif(c, P, n) {
      const t = n.count > 1 ? n.index / (n.count - 1) : 0.5
      const offset = (t - 0.5) * 2 * 9
      const total = Math.abs(offset) < 2.2
      const pulse = 0.85 + 0.15 * Math.sin(P.time * 1.3 + n.phase)
      const g = c.createRadialGradient(0, 0, 3, 0, 0, total ? 11 * pulse : 7)
      g.addColorStop(0, alpha(P.pal.accent, total ? 0.85 : 0.5))
      g.addColorStop(1, alpha(P.pal.accent, 0))
      c.fillStyle = g
      disc(c, 0, 0, total ? 11 : 7)
      c.fillStyle = mix(P.pal.accent, '#fff3e8', 0.55)
      disc(c, 0, 0, 4.6)
      c.fillStyle = '#12070a'
      disc(c, offset, 0, 4.4)
      if (total) {
        c.fillStyle = '#ffffff'
        disc(c, 3.2, -3.1, 0.9)
      }
    },
  },
  goldleaf: {
    mode: 'wall',
    edge(c, P) {
      const gold = P.pal.accent
      along(c, P, () => 2, P.len)
      stroke(c, alpha(gold, 0.85), 0.8)
      along(c, P, () => 4.5, P.len)
      stroke(c, alpha(gold, 0.5), 0.5)
      along(c, P, () => P.thick - 2, P.len)
      stroke(c, alpha(gold, 0.45), 0.6)
      c.fillStyle = gold
      for (let u = 40; u < P.len; u += 80) {
        c.beginPath()
        c.moveTo(u - 3, 3.2)
        c.lineTo(u, 1)
        c.lineTo(u + 3, 3.2)
        c.lineTo(u, 5.4)
        c.closePath()
        c.fill()
      }
    },
    // An Art Deco fan rising from the frame.
    motif(c, P, n) {
      const gold = P.pal.accent
      const sheen = 0.6 + 0.4 * Math.sin(P.time * 0.9 + n.phase)
      c.strokeStyle = alpha(gold, 0.9)
      c.lineWidth = 0.7
      for (let i = 0; i <= 8; i++) {
        const a = Math.PI + (i / 8) * Math.PI
        const r = i % 2 ? 9 : 12
        line(c, Math.cos(a) * 3, Math.sin(a) * 3, Math.cos(a) * r, Math.sin(a) * r)
      }
      c.strokeStyle = alpha(mix(gold, '#fff7dc', sheen * 0.6), 0.9)
      c.lineWidth = 0.9
      c.beginPath()
      c.arc(0, 0, 13, Math.PI, TAU)
      c.stroke()
      c.beginPath()
      c.arc(0, 0, 3, Math.PI, TAU)
      c.stroke()
      c.fillStyle = gold
      disc(c, 0, -13, 1)
    },
  },
  plankton: {
    mode: 'float',
    edge(c, P) {
      for (let j = 0; j < 3; j++) {
        along(c, P, (u) => 6 + Math.sin(u * 0.015 + j + P.time * 0.14) * 4 + j * 4, 6)
        stroke(c, alpha(j === 1 ? P.pal.secondary : P.pal.accent, 0.14 + j * 0.04), 2.6)
      }
    },
    motif(c, P, n) {
      at(c, 0, Math.sin(P.time * 0.7 + n.phase) * 2.5, Math.sin(P.time * 0.4 + n.phase) * 0.3, () =>
        art(c, 'plankton', P.pal.accent, 1.7, n.phase, P.time, P.pal.secondary),
      )
      c.fillStyle = ink(P, P.pal.secondary, 0.6)
      for (let i = 0; i < 3; i++) {
        const tw = 0.5 + 0.5 * Math.sin(P.time * 2.4 + i * 2 + n.phase)
        disc(c, Math.cos(i * 2.1 + n.phase) * 9, Math.sin(i * 2.1 + n.phase) * 6, 0.5 + tw * 0.6)
      }
    },
  },
  geometry: {
    mode: 'float',
    edge(c, P) {
      c.strokeStyle = alpha(P.pal.accent, 0.7)
      c.lineWidth = 0.6
      line(c, 0, 1, P.len, 1)
      for (let u = 0, i = 0; u < P.len; u += 8, i++) line(c, u, 1, u, i % 5 ? 3 : 6)
      c.strokeStyle = alpha(P.pal.secondary, 0.4)
      c.setLineDash([6, 4])
      line(c, 0, P.thick - 2, P.len, P.thick - 2)
      c.setLineDash([])
    },
    motif(c, P, n) {
      c.strokeStyle = alpha(P.pal.secondary, 0.45)
      c.lineWidth = 0.5
      c.setLineDash([2, 2])
      line(c, -13, 0, 13, 0)
      line(c, 0, -10, 0, 10)
      c.setLineDash([])
      at(c, 0, 0, n.phase + P.time * 0.05, () =>
        art(c, 'geometry', P.pal.accent, 2.6, n.phase, P.time, P.pal.secondary, n.index % 4),
      )
    },
  },
  heart: {
    mode: 'hang',
    edge(c, P) {
      c.strokeStyle = alpha(P.pal.accent, 0.6)
      c.lineWidth = 0.7
      for (let u = 2, i = 0; u < P.len; u += 5, i++) {
        c.beginPath()
        if (i % 2) c.ellipse(u, 2.5, 2.6, 1.3, 0, 0, TAU)
        else c.ellipse(u, 2.5, 1.4, 0.7, 0, 0, TAU)
        c.stroke()
      }
    },
    motif(c, P, n) {
      const len = 6 + n.seed * 10
      c.rotate(Math.sin(P.time * 1.2 + n.phase) * 0.14)
      c.strokeStyle = alpha(P.pal.accent, 0.6)
      c.lineWidth = 0.6
      line(c, 0, 0, 0, len)
      at(c, 0, len + 4, 0, () => art(c, 'heart', P.pal.accent, 1.6, n.phase, P.time))
    },
  },
  wave: {
    mode: 'float',
    edge(c, P) {
      const base = P.thick - 1
      c.strokeStyle = alpha(P.pal.accent, 0.55)
      c.lineWidth = 0.7
      for (let k = 0; k < 4; k++) {
        const v = base - k ** 1.5 * 2.4
        line(c, 0, v, P.len, v)
      }
      const horizon = base - 3 ** 1.5 * 2.4
      const mid = P.len / 2,
        shift = (P.time * 10) % 26
      c.strokeStyle = alpha(P.pal.secondary, 0.45)
      for (let u = -26 + shift; u < P.len + 26; u += 26)
        line(c, u, base, mid + (u - mid) * 0.7, horizon)
    },
    motif(c, P, n) {
      if (n.index % 3 === 0) {
        const g = c.createLinearGradient(0, -9, 0, 0)
        g.addColorStop(0, P.pal.secondary)
        g.addColorStop(1, P.pal.accent)
        c.fillStyle = g
        c.beginPath()
        c.arc(0, 2, 9, Math.PI, TAU)
        c.fill()
        c.fillStyle = P.pal.bg
        for (let i = 0; i < 3; i++) c.fillRect(-10, -1 - i * 2.6, 20, 0.9 + i * 0.25)
      } else {
        c.fillStyle = alpha(P.pal.accent, 0.85)
        for (let i = 0; i < 5; i++) {
          const v = 0.35 + 0.65 * Math.abs(Math.sin(P.time * (2 + i * 0.4) + n.phase + i))
          c.fillRect(-7 + i * 3, 3 - v * 10, 1.8, v * 10)
        }
      }
    },
  },
  tea: {
    mode: 'hang',
    edge(c, P) {
      along(c, P, () => 4, P.len)
      stroke(c, alpha(shade(P.pal.accent, 0.1), 0.6), 2.4)
      c.strokeStyle = alpha(shade(P.pal.accent, -0.3), 0.8)
      c.lineWidth = 0.9
      for (let u = 20; u < P.len; u += 36) line(c, u, 2.3, u, 5.7)
    },
    motif(c, P, n) {
      c.rotate(Math.sin(P.time * 0.9 + n.phase) * 0.08)
      for (const side of [-1, 1])
        at(c, side * 1.5, 4, side * 0.75, () => {
          c.translate(0, 9)
          art(c, 'tea', P.pal.accent, 1.15, n.phase + side, P.time)
        })
      if (n.seed > 0.5)
        at(c, 0, 9, 0, () => art(c, 'tea', P.pal.accent, 0.9, n.phase, P.time, P.pal.secondary, 1))
    },
  },
  berry: {
    mode: 'hang',
    edge(c, P) {
      along(c, P, (u) => 3 + Math.sin(u * 0.025) * 1.5, 8)
      stroke(c, alpha(shade(P.pal.secondary, -0.35), 0.8), 1)
      c.fillStyle = alpha(mix(P.pal.secondary, '#7aa36b', 0.6), 0.7)
      for (let u = 25; u < P.len; u += 55)
        at(c, u, 3 + Math.sin(u * 0.025) * 1.5, 0.6, () => {
          c.beginPath()
          c.ellipse(3, 0, 3.2, 1.3, 0, 0, TAU)
          c.fill()
        })
    },
    motif(c, P, n) {
      c.rotate(Math.sin(P.time * 0.8 + n.phase) * 0.1)
      const berries = [
        [-4, 11],
        [4, 13],
        [0, 19],
      ]
      c.strokeStyle = alpha(shade(P.pal.secondary, -0.35), 0.8)
      c.lineWidth = 0.6
      for (const [x, y] of berries) line(c, 0, 3, x, y - 2)
      for (const [x, y] of berries)
        at(c, x, y, 0, () => art(c, 'berry', P.pal.accent, 1.15, n.phase + x, P.time))
    },
  },
  cloud: {
    mode: 'float',
    edge(c, P) {
      c.fillStyle = alpha(
        P.light ? mix(P.pal.accent, '#ffffff', 0.45) : P.pal.accent,
        P.light ? 0.26 : 0.14,
      )
      for (let u = -6, i = 0; u < P.len + 12; i++) {
        const r = 6 + hash(i * 1.9) * 7
        disc(c, u, P.thick + 2 - r * 0.35, r)
        u += r * 1.3
      }
    },
    motif(c, P, n) {
      at(c, Math.sin(P.time * 0.2 + n.phase) * 4, 0, 0, () =>
        art(c, 'cloud', P.pal.accent, 2, n.phase, P.time),
      )
    },
  },
  coffee: {
    mode: 'grow',
    sideMode: 'float',
    edge(c, P) {
      along(c, P, () => 2.5, P.len)
      stroke(c, alpha(P.pal.accent, 0.45), 0.7)
      for (let u = 35, i = 0; u < P.len; u += 70, i++)
        at(c, u, 5, i * 1.3, () => art(c, 'coffee', P.pal.accent, 0.95, i, P.time))
    },
    motif(c, P, n) {
      const y = P.vertical ? 8 : -2
      c.fillStyle = alpha(P.pal.accent, 0.85)
      c.beginPath()
      c.moveTo(-6, y - 9)
      c.lineTo(6, y - 9)
      c.quadraticCurveTo(6, y - 1, 0, y - 1)
      c.quadraticCurveTo(-6, y - 1, -6, y - 9)
      c.fill()
      c.strokeStyle = alpha(P.pal.accent, 0.85)
      c.lineWidth = 1.1
      c.beginPath()
      c.arc(6.5, y - 6.2, 2.2, -1.2, 1.4)
      c.stroke()
      c.fillRect(-8, y, 16, 1)
      for (let i = 0; i < 2; i++) {
        const k = (P.time * 0.4 + n.seed + i * 0.5) % 1
        c.save()
        c.globalAlpha *= Math.sin(k * Math.PI)
        c.translate(-2 + i * 4, y - 13 - k * 8)
        art(c, 'coffee', ink(P, P.pal.secondary, 0.4), 1.2, n.phase + i, P.time, P.pal.accent, 1)
        c.restore()
      }
    },
  },
  lavender: {
    mode: 'grow',
    edge(c, P) {
      c.strokeStyle = alpha(mix(P.pal.secondary, '#7d9474', 0.6), 0.7)
      c.lineWidth = 0.7
      for (let u = 0, i = 0; u < P.len; u += 4, i++) {
        const tall = 2 + hash(i * 0.7) * 5
        const bend = Math.sin(P.time * 1.1 + u * 0.05) * 1.2
        line(c, u, P.thick, u + bend, P.thick - tall)
      }
    },
    motif(c, P, n) {
      for (let i = 0; i < 3; i++) {
        const lean = (i - 1) * 0.28 + Math.sin(P.time * 1.1 + n.phase + i) * 0.08
        at(c, (i - 1) * 3, 0, lean, () => {
          c.translate(0, -9 - (i === 1 ? 3 : 0))
          art(c, 'lavender', P.pal.accent, i === 1 ? 2.8 : 2.3, n.phase + i, P.time)
        })
      }
    },
  },
  ginkgo: {
    mode: 'hang',
    edge(c, P) {
      along(c, P, (u) => 4 + Math.sin(u * 0.018) * 2, 8)
      stroke(c, alpha(shade(P.pal.accent, -0.35), 0.75), 1.3)
      along(c, P, (u) => 6.5 + Math.sin(u * 0.018 + 0.6) * 2, 8)
      stroke(c, alpha(P.pal.secondary, 0.45), 0.5)
    },
    motif(c, P, n) {
      for (const side of [-1, 1]) {
        const sway = Math.sin(P.time * 0.9 + n.phase + side) * 0.12
        at(c, side * 2, 4, side * 0.45 + sway, () => {
          c.translate(0, 9)
          c.rotate(Math.PI)
          art(
            c,
            'ginkgo',
            side > 0 ? P.pal.accent : P.pal.secondary,
            1.45,
            n.phase,
            P.time,
            P.pal.secondary,
          )
        })
      }
    },
  },
  comet: {
    mode: 'float',
    edge(c, P) {
      for (let i = 0; i < P.len / 9; i++) {
        const tw = 0.4 + 0.6 * Math.abs(Math.sin(P.time * (0.6 + hash(i) * 1.4) + i))
        c.fillStyle = alpha(i % 3 ? P.pal.accent : P.pal.secondary, 0.55 * tw)
        disc(c, hash(i * 1.3) * P.len, hash(i * 2.7) * P.thick, 0.5 + hash(i * 4.1) * 0.8)
      }
    },
    motif(c, P, n) {
      const travel = ((P.time * 0.12 + n.seed) % 1) * 10 - 5
      at(c, travel, 0, P.vertical ? 1.25 : 0.28, () =>
        art(c, 'comet', P.pal.accent, 1.3, n.phase, P.time, P.pal.secondary),
      )
    },
  },
  bokeh: {
    mode: 'float',
    edge(c, P) {
      for (let u = 6, i = 0; u < P.len; u += 14, i++) {
        const breathe = 0.6 + 0.4 * Math.sin(P.time * 0.7 + i)
        c.fillStyle = alpha(i % 2 ? P.pal.secondary : P.pal.accent, 0.12 + 0.12 * breathe)
        disc(c, u, 4 + hash(i) * 3, 1.5 + hash(i * 3.3) * 2.5)
      }
    },
    motif(c, P, n) {
      art(c, 'bokeh', P.pal.accent, 2.4, n.phase, P.time, P.pal.secondary, n.index % 2)
      at(c, 8, 5, 0, () => art(c, 'bokeh', P.pal.secondary, 1.2, n.phase + 1, P.time, P.pal.accent))
      at(c, -7, -6, 0, () =>
        art(c, 'bokeh', P.pal.accent, 0.8, n.phase + 2, P.time, P.pal.secondary),
      )
    },
  },
  honeycomb: {
    mode: 'hang',
    edge(c, P) {
      const r = 4.2,
        step = r * Math.sqrt(3)
      for (let u = 0, i = 0; u < P.len + step; u += step, i++) {
        hex(c, u, 3, r)
        if (hash(i * 2.9) > 0.6) {
          c.fillStyle = alpha('#e7a52a', 0.45)
          c.fill()
        }
        stroke(c, alpha(P.pal.accent, 0.65), 0.7)
      }
    },
    motif(c, P, n) {
      const cycle = (P.time * 0.25 + n.seed) % 1
      const stretch = Math.min(1, cycle / 0.7)
      const g = c.createLinearGradient(0, 4, 0, 18)
      g.addColorStop(0, '#ffd975')
      g.addColorStop(1, '#c9831a')
      c.fillStyle = g
      c.beginPath()
      c.moveTo(-3, 5)
      c.quadraticCurveTo(-1.2, 7 + stretch * 6, -1.4, 8 + stretch * 7)
      c.arc(0, 8 + stretch * 7, 1.4, Math.PI, 0, true)
      c.quadraticCurveTo(1.2, 7 + stretch * 6, 3, 5)
      c.closePath()
      c.fill()
      if (cycle > 0.7) {
        const fall = (cycle - 0.7) / 0.3
        c.globalAlpha *= 1 - fall
        at(c, 0, 18 + fall * 20, 0, () =>
          art(c, 'honeycomb', P.pal.accent, 0.9, n.phase, P.time, P.pal.accent, 1),
        )
      }
    },
  },
  diamond: {
    mode: 'wall',
    edge(c, P) {
      const ice = ink(P, P.pal.accent, 0.55)
      for (let u = 0, i = 0; u < P.len; i++) {
        const wid = 3 + hash(i * 1.1) * 4
        const len = 3 + hash(i * 2.3) ** 2 * 11
        const g = c.createLinearGradient(0, 0, 0, len)
        g.addColorStop(0, alpha(ice, 0.55))
        g.addColorStop(1, alpha(ice, 0.05))
        c.fillStyle = g
        c.beginPath()
        c.moveTo(u, 0)
        c.lineTo(u + wid / 2, len)
        c.lineTo(u + wid, 0)
        c.closePath()
        c.fill()
        u += wid
      }
    },
    // Frost ferns grow from the frame into the bar.
    motif(c, P, n) {
      const ice = ink(P, P.pal.accent, 0.5)
      c.strokeStyle = alpha(ice, 0.8)
      const branch = (len, depth, width) => {
        c.lineWidth = width
        c.beginPath()
        c.moveTo(0, 0)
        c.quadraticCurveTo(len * 0.08, -len * 0.5, 0, -len)
        c.stroke()
        if (depth <= 0) return
        for (let i = 1; i <= 3; i++) {
          const side = i % 2 ? 1 : -1
          at(c, 0, (-len * i) / 4.2, side * 0.55, () =>
            branch(len * (0.5 - i * 0.08), depth - 1, width * 0.7),
          )
          at(c, 0, (-len * i) / 4.2 - 1.5, -side * 0.6, () =>
            branch(len * (0.32 - i * 0.05), 0, width * 0.6),
          )
        }
      }
      branch(14 + n.seed * 6, 2, 0.9)
      const flash = Math.max(0, Math.sin(P.time * 1.6 + n.phase * 5)) ** 10
      if (flash > 0.02)
        at(c, 0, -14 - n.seed * 6, 0, () => art(c, 'diamond', P.pal.accent, 1.4, n.phase, P.time))
    },
  },
  code: {
    mode: 'float',
    edge(c, P) {
      c.strokeStyle = alpha(P.pal.accent, 0.08)
      c.lineWidth = 1
      for (let v = 1.5; v < P.thick; v += 3) line(c, 0, v, P.len, v)
      const scan = (P.time * 0.15) % 1
      along(c, P, () => scan * P.thick, P.len)
      stroke(c, alpha(P.pal.accent, 0.25), 1.4)
    },
    motif(c, P, n) {
      const words = ['0x1F', '</>', '$ _', '{ }', '0110', 'ok', '>>', '#!']
      const word = words[Math.floor(n.seed * words.length)]
      c.font = '700 9px Consolas, "Cascadia Mono", "JetBrains Mono", "Courier New", monospace'
      c.textAlign = 'center'
      c.textBaseline = 'middle'
      const flicker =
        0.75 + 0.25 * Math.sin(P.time * 9 + n.phase * 5) * Math.sin(P.time * 3 + n.phase)
      c.fillStyle = alpha(P.pal.accent, 0.9 * flicker)
      const blink = Math.floor(P.time * 1.6 + n.phase) % 2 === 0
      c.fillText(word === '$ _' && !blink ? '$  ' : word, 0, 0)
    },
  },
  moon: {
    mode: 'float',
    edge(c, P) {
      along(c, P, () => 3, P.len)
      stroke(c, alpha(P.pal.accent, 0.5), 0.6)
      c.fillStyle = alpha(P.pal.secondary, 0.7)
      for (let u = 12; u < P.len; u += 24) disc(c, u, 3, 0.9)
    },
    // A lunar calendar: each anchor shows the next phase.
    motif(c, P, n) {
      const f = n.count > 1 ? n.index / (n.count - 1) : 0.5
      const r = 5.5
      const silver = ink(P, P.pal.accent, 0.45)
      c.fillStyle = alpha(silver, 0.18)
      disc(c, 0, 0, r)
      c.fillStyle = silver
      const k = Math.cos(f * TAU)
      c.beginPath()
      if (f <= 0.5) {
        c.arc(0, 0, r, -Math.PI / 2, Math.PI / 2)
        c.ellipse(0, 0, Math.abs(k) * r, r, 0, Math.PI / 2, -Math.PI / 2, k > 0)
      } else {
        c.arc(0, 0, r, Math.PI / 2, -Math.PI / 2)
        c.ellipse(0, 0, Math.abs(k) * r, r, 0, -Math.PI / 2, Math.PI / 2, k > 0)
      }
      c.fill()
      c.strokeStyle = alpha(silver, 0.5)
      c.lineWidth = 0.5
      c.beginPath()
      c.arc(0, 0, r, 0, TAU)
      c.stroke()
    },
  },
  fish: {
    mode: 'grow',
    edge(c, P) {
      along(c, P, (u) => 3 + Math.sin(u * 0.05 + P.time * 1.1) * 1.3, 5)
      stroke(c, alpha(P.pal.secondary, 0.7), 1)
      along(c, P, (u) => P.thick - 1.5 - Math.sin(u * 0.02) * 1.2, 10)
      stroke(c, alpha(P.pal.accent, 0.4), 1.6)
    },
    motif(c, P, n) {
      const coral = (len, depth, width) => {
        c.lineWidth = width
        line(c, 0, 0, 0, -len)
        if (depth <= 0) {
          disc(c, 0, -len, width * 0.6)
          return
        }
        for (const side of [-1, 1])
          at(c, 0, -len * (side > 0 ? 0.55 : 0.75), side * 0.6, () =>
            coral(len * 0.65, depth - 1, width * 0.75),
          )
      }
      c.strokeStyle = alpha(P.pal.accent, 0.9)
      c.fillStyle = alpha(P.pal.accent, 0.9)
      coral(8 + n.seed * 3, 2, 1.8)
      c.strokeStyle = alpha(P.pal.secondary, 0.8)
      c.lineWidth = 1.1
      c.beginPath()
      c.moveTo(7, 0)
      for (let i = 1; i <= 6; i++)
        c.lineTo(7 + Math.sin(P.time * 1.6 + i * 0.9 + n.phase) * 1.8, -i * 3.2)
      c.stroke()
    },
  },
  plasma: {
    mode: 'float',
    edge(c, P) {
      const tick = Math.floor(P.time * 8)
      for (const [v, color] of [
        [2.5, P.pal.accent],
        [P.thick - 2.5, P.pal.secondary],
      ]) {
        for (let u = 0, i = 0; u < P.len; u += 60, i++) {
          if (hash(tick + i * 7.7 + v) < 0.45) continue
          jagged(c, u, v, u + 50, v, tick + i, 7, 2)
          glow(c, alpha(color, 0.85), 0.7)
        }
      }
    },
    // A small plasma globe: glass sphere, glowing core and arcs that reach the glass.
    motif(c, P, n) {
      const tick = Math.floor(P.time * 7 + n.phase * 10)
      const R = 9
      const g = c.createRadialGradient(0, 0, 0, 0, 0, R)
      g.addColorStop(0, alpha(P.pal.accent, 0.35))
      g.addColorStop(1, alpha(P.pal.accent, 0.04))
      c.fillStyle = g
      disc(c, 0, 0, R)
      c.strokeStyle = alpha(ink(P, P.pal.accent, 0.5), 0.55)
      c.lineWidth = 0.7
      c.beginPath()
      c.arc(0, 0, R, 0, TAU)
      c.stroke()
      c.fillStyle = alpha(shade(P.pal.secondary, -0.3), 0.7)
      c.beginPath()
      c.moveTo(-4, R - 1)
      c.lineTo(4, R - 1)
      c.lineTo(5.5, R + 3)
      c.lineTo(-5.5, R + 3)
      c.closePath()
      c.fill()
      for (let i = 0; i < 4; i++) {
        if (hash(tick + i * 3.3) < 0.25) continue
        const a = hash(tick * 0.7 + i * 1.9) * TAU
        jagged(c, 0, 0, Math.cos(a) * (R - 0.8), Math.sin(a) * (R - 0.8), tick + i * 5, 4, 1.6)
        glow(c, alpha(i % 2 ? P.pal.secondary : mix(P.pal.accent, '#ffffff', 0.3), 0.9), 0.5)
      }
      c.fillStyle = mix(P.pal.accent, '#ffffff', 0.65)
      disc(c, 0, 0, 1.9)
    },
  },
  gear: {
    mode: 'wall',
    edge(c, P) {
      along(c, P, () => 1.5, P.len)
      stroke(c, alpha(P.pal.accent, 0.4), 1)
      for (const v of [4, P.thick - 3])
        for (let u = 9; u < P.len; u += 18) {
          c.fillStyle = alpha(shade(P.pal.accent, -0.3), 0.6)
          disc(c, u, v, 1.3)
          c.fillStyle = alpha('#ffffff', 0.35)
          disc(c, u - 0.4, v - 0.4, 0.5)
        }
    },
    // Two meshing gears; the second turns the other way.
    motif(c, P, n) {
      const s = 2.7,
        spin = P.time * 0.6 * (n.side > 0 ? 1 : -1)
      at(c, 0, -7, spin, () => art(c, 'gear', P.pal.accent, s, 0, P.time))
      at(c, 3.95 * s * Math.sin(0.7), -7 - 3.95 * s * Math.cos(0.7), -spin + Math.PI / 10, () =>
        art(c, 'gear', P.pal.secondary, s, 0, P.time),
      )
    },
  },
  lantern: {
    mode: 'hang',
    edge(c, P) {
      const points = [0, ...P.nodes.map((n) => n.t * P.len), P.len]
      c.beginPath()
      c.moveTo(0, 1)
      for (let i = 1; i < points.length; i++) {
        const a = points[i - 1],
          b = points[i]
        c.quadraticCurveTo(
          (a + b) / 2,
          1 + Math.min(10, (b - a) * 0.08),
          b,
          i < points.length - 1 ? 3 : 1,
        )
      }
      stroke(c, alpha(shade(P.pal.accent, -0.2), 0.7), 0.8)
    },
    motif(c, P, n) {
      const len = 4 + n.seed * 8
      c.rotate(Math.sin(P.time * 0.9 + n.phase) * 0.08)
      c.strokeStyle = alpha(shade(P.pal.accent, -0.2), 0.7)
      c.lineWidth = 0.6
      line(c, 0, 3, 0, 3 + len)
      at(c, 0, 3 + len + 7, 0, () =>
        art(c, 'lantern', n.seed > 0.75 ? P.pal.secondary : P.pal.accent, 2, n.phase, P.time),
      )
    },
  },
}

export const PANEL_KINDS = Object.freeze(Object.keys(DESIGNS))

function place(c, P, n, mode, pos) {
  const { w, h } = P
  if (!P.vertical) {
    if (mode === 'hang') c.translate(pos, 0)
    else if (mode === 'grow') {
      c.translate(pos, h)
      c.rotate((n.seed - 0.5) * 0.3)
    } else if (mode === 'wall') {
      const top = n.side < 0
      c.translate(pos, top ? 0 : h)
      if (top) c.rotate(Math.PI)
    } else c.translate(pos, h * (0.35 + n.seed * 0.3))
    return
  }
  const left = n.side < 0
  if (mode === 'hang') c.translate(left ? 10 : w - 10, pos - 12)
  else if (mode === 'grow') {
    c.translate(left ? 0 : w, pos)
    c.rotate((left ? 1 : -1) * (0.35 + n.seed * 0.25))
  } else if (mode === 'wall') {
    c.translate(left ? 0 : w, pos)
    c.rotate(left ? Math.PI / 2 : -Math.PI / 2)
  } else c.translate(left ? 12 : w - 12, pos)
}

export function drawPanel(c, theme, w, h, vertical, time, nodes) {
  c.clearRect(0, 0, w, h)
  const design = DESIGNS[theme?.particle]
  if (!design || !(w > 0) || !(h > 0)) return
  const P = {
    pal: theme.palette,
    light: Boolean(theme.palette.light),
    time: Number.isFinite(time) ? time : 0,
    vertical,
    w,
    h,
    len: vertical ? h : w,
    thick: vertical ? w : h,
    nodes: nodes ?? [],
  }
  c.lineCap = 'round'
  c.lineJoin = 'round'
  // The frame pattern runs along the bar; for the sidebar u/v are swapped.
  c.save()
  if (vertical) c.transform(0, 1, 1, 0, 0, 0)
  c.globalAlpha = 0.5
  design.edge(c, P)
  c.restore()
  const mode = vertical ? design.sideMode ?? design.mode : design.mode
  for (const n of P.nodes) {
    const pos = n.t * P.len
    // The lower sidebar is mostly empty, so ornaments there may be larger and brighter.
    const lower = vertical && pos > h * 0.5
    c.save()
    c.globalAlpha = vertical ? (lower ? 0.75 : 0.5) : 0.5
    place(c, P, n, mode, pos)
    const s = (n.size ?? 1) * (vertical && !lower ? 0.8 : 1)
    c.scale(s, s)
    design.motif(c, P, n)
    c.restore()
  }
  c.globalAlpha = 1
}
