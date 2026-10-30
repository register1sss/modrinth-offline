import { particleArt } from './particle-art.js'
import { seededRandom } from './particles.js'
const sessionSeed = Date.now().toString(36)
export function panelScene(theme, vertical) {
  const r = seededRandom(`${sessionSeed}:${theme.id}:${vertical}`)
  return Array.from({ length: vertical ? 9 : 12 }, (_, i) => ({
    t: (i + 0.3 + r() * 0.4) / (vertical ? 9 : 12),
    side: r() > 0.5 ? 1 : -1,
    size: 5 + r() * 6,
    phase: r() * 6.28,
  }))
}
export function drawPanel(c, theme, w, h, vertical, time, nodes) {
  c.clearRect(0, 0, w, h)
  const p = theme.palette,
    kind = theme.particle
  const line = (x, y, xx, yy, color = p.accent, width = 1) => {
    c.strokeStyle = color
    c.lineWidth = width
    c.beginPath()
    c.moveTo(x, y)
    c.lineTo(xx, yy)
    c.stroke()
  }
  // Ornament lives on the edges; controls remain unobscured in the foreground.
  c.globalAlpha = 0.45
  const len = vertical ? h : w
  const stem = (offset, color) => {
    c.strokeStyle = color
    c.lineWidth = 1.1
    c.beginPath()
    for (let i = 0; i <= len; i += 12) {
      const a = offset + Math.sin(i * 0.022) * 4
      const x = vertical ? a : i,
        y = vertical ? i : a
      i ? c.lineTo(x, y) : c.moveTo(x, y)
    }
    c.stroke()
  }
  if (['petal', 'leaf', 'ribbon', 'ember', 'bubble'].includes(kind)) {
    stem(5, p.accent)
    stem(vertical ? w - 6 : h - 4, p.secondary)
  }
  if (kind === 'rain')
    for (let i = 0; i < 3; i++) {
      const edge = 4 + i * 4
      line(
        vertical ? edge : w * 0.25,
        vertical ? h * 0.2 : edge,
        vertical ? edge : w * 0.65,
        vertical ? h * 0.85 : edge,
        i === 1 ? p.secondary : p.accent,
        0.8,
      )
    }
  if (['firefly', 'plankton'].includes(kind))
    for (let j = 0; j < 3; j++) {
      c.strokeStyle = j === 1 ? p.secondary : p.accent
      c.lineWidth = vertical ? 5 : 3
      c.globalAlpha = 0.12 + j * 0.035
      c.beginPath()
      for (let i = 0; i < len; i += 6) {
        const a = 9 + Math.sin(i * 0.015 + j + time * 0.14) * 6 + j * 5
        i
          ? c.lineTo(vertical ? a : i, vertical ? i : a)
          : c.moveTo(vertical ? a : i, vertical ? i : a)
      }
      c.stroke()
    }
  for (const n of nodes) {
    const pos = n.t * len
    // More decoration in the unused lower sidebar, finer detail in the top bar.
    const x = vertical ? (n.side > 0 ? w - 9 : 10) : pos
    const y = vertical ? pos : n.side > 0 ? h - 5 : 5
    c.save()
    c.translate(x, y)
    if (!vertical) c.rotate(-Math.PI / 2)
    c.globalAlpha = vertical && pos > h * 0.5 ? 0.8 : 0.5
    if (['leaf', 'petal', 'ember', 'ribbon', 'bubble'].includes(kind)) {
      const side = n.side > 0 ? -1 : 1
      c.strokeStyle = p.accent
      c.lineWidth = 1
      c.beginPath()
      c.moveTo(0, -20)
      c.bezierCurveTo(side * 9, -10, side * 5, 0, side * 16, 7)
      c.stroke()
      for (let j = 0; j < 3; j++) {
        c.save()
        c.translate(side * (4 + j * 4), -9 + j * 7)
        c.rotate(side * (0.6 + j * 0.2))
        c.scale(2.3, 2.3)
        if (kind === 'ribbon') {
          // Faceted ingots with a bright bevel, plus hanging gold chains.
          c.fillStyle = '#9d702a'
          c.beginPath()
          c.moveTo(-2, -1)
          c.lineTo(1.5, -1.5)
          c.lineTo(2.5, 0)
          c.lineTo(1.6, 1.2)
          c.lineTo(-2.2, 1)
          c.closePath()
          c.fill()
          c.fillStyle = p.accent
          c.beginPath()
          c.moveTo(-1.5, -0.8)
          c.lineTo(1.2, -1)
          c.lineTo(1.8, -0.1)
          c.lineTo(-1.6, 0.4)
          c.closePath()
          c.fill()
          line(-1.4, -0.5, 1, -0.75, '#fff5ce', 0.15)
        } else if (kind === 'petal') {
          for (let a = 0; a < 5; a++) {
            c.rotate((Math.PI * 2) / 5)
            particleArt(c, 'petal', p.accent, a, time)
          }
          c.fillStyle = '#ffdfad'
          c.beginPath()
          c.arc(0, 0, 0.6, 0, 6.28)
          c.fill()
        } else particleArt(c, kind === 'bubble' ? 'leaf' : kind, p.accent, n.phase, time)
        c.restore()
      }
    }
    if (kind === 'ember') {
      // Three animated flame tongues, smoke curls and sparks on the frame.
      for (let a = 0; a < 3; a++) {
        c.save()
        c.translate((a - 1) * 7, 0)
        c.rotate(Math.sin(time * 2 + n.phase + a) * 0.13)
        c.scale(4, 7 + Math.sin(time * 3 + a + n.phase) * 2)
        particleArt(c, 'ember', p.accent, a, time)
        c.restore()
      }
      c.strokeStyle = p.muted + '70'
      c.lineWidth = 3
      c.beginPath()
      c.moveTo(0, -20)
      c.bezierCurveTo(12, -30, -8, -35, 4 + Math.sin(time + n.phase) * 4, -48)
      c.stroke()
    } else if (kind === 'ribbon') {
      // Narrow gold cascade. Each strand is distinct rather than a solid overlay.
      for (let j = 0; j < 4; j++) {
        const d = (time * 14 + n.phase * 9 + j * 11) % 45
        line(j * 3 - 4, d - 18, j * 3 - 3, d - 10, p.accent, j === 2 ? 1.8 : 0.6)
      }
    } else if (kind === 'star') {
      c.strokeStyle = p.secondary + '88'
      c.lineWidth = 0.8
      c.beginPath()
      c.ellipse(0, 0, 19, 8, n.phase, 0, 6.28)
      c.stroke()
      c.fillStyle = p.accent
      c.beginPath()
      c.arc(0, 0, 5, 0, 6.28)
      c.fill()
      line(0, 0, 12, 19, p.secondary, 0.7)
    } else if (kind === 'snow') {
      c.save()
      c.scale(5, 5)
      particleArt(c, 'snow', p.accent, n.phase, time)
      c.restore()
      line(0, 0, 9, 25, p.accent, 0.7)
    } else if (kind === 'crystal') {
      for (let i = 0; i < 3; i++) {
        c.save()
        c.translate((i - 1) * 7, 0)
        c.rotate((i - 1) * 0.3)
        c.scale(3, 5 + i)
        particleArt(c, 'crystal', i === 1 ? p.secondary : p.accent, n.phase, time)
        c.restore()
      }
    } else if (kind === 'sand') {
      for (let j = 0; j < 3; j++) {
        c.strokeStyle = j === 1 ? p.secondary : p.accent
        c.lineWidth = 0.8
        c.beginPath()
        c.moveTo(-8, j * 5)
        c.quadraticCurveTo(2, -8 + j * 5, 17, j * 5)
        c.stroke()
      }
      c.save()
      c.scale(3, 3)
      particleArt(c, 'sand', p.accent, n.phase, time)
      c.restore()
    } else if (kind === 'comet') {
      c.strokeStyle = p.secondary
      c.lineWidth = 1
      c.beginPath()
      c.arc(0, 0, 12, 0, 6.28)
      c.stroke()
      c.fillStyle = p.bg
      c.beginPath()
      c.arc(4, -3, 9, 0, 6.28)
      c.fill()
      c.save()
      c.translate(9, 10)
      c.scale(2, 2)
      particleArt(c, 'comet', p.accent, n.phase, time)
      c.restore()
    } else if (['firefly', 'rain', 'plankton', 'bubble'].includes(kind)) {
      c.save()
      c.scale(2.7, 2.7)
      particleArt(c, kind, p.accent, n.phase, time)
      c.restore()
    }
    c.restore()
  }
  c.globalAlpha = 1
}
