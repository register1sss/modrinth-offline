// Vector particles share one drawing path in the background, panels and click bursts.
// No images, network requests or per-particle DOM nodes.
export function particleArt(c, kind, color) {
  c.fillStyle = color
  c.strokeStyle = color
  c.lineWidth = 0.16
  c.lineCap = 'round'
  const line = (x, y, xx, yy) => {
    c.beginPath()
    c.moveTo(x, y)
    c.lineTo(xx, yy)
    c.stroke()
  }
  const dot = (x, y, r) => {
    c.beginPath()
    c.arc(x, y, r, 0, Math.PI * 2)
    c.fill()
  }
  const glow = (r) => {
    const g = c.createRadialGradient(0, 0, 0, 0, 0, r)
    g.addColorStop(0, color + '90')
    g.addColorStop(1, color + '00')
    c.fillStyle = g
    dot(0, 0, r)
    c.fillStyle = color
  }
  switch (kind) {
    case 'ember': {
      glow(3.8)
      const g = c.createLinearGradient(0, -3, 0, 2)
      g.addColorStop(0, color + '10')
      g.addColorStop(0.55, color)
      g.addColorStop(1, '#db4825')
      c.fillStyle = g
      c.beginPath()
      c.moveTo(0, -3)
      c.bezierCurveTo(1, -1.9, -0.1, -1.1, 1.1, -0.5)
      c.bezierCurveTo(2.1, 1.6, 0.1, 2.2, -0.9, 1.2)
      c.bezierCurveTo(-2, 0.2, -0.6, -0.5, 0, -3)
      c.fill()
      c.fillStyle = '#fff0bd'
      c.beginPath()
      c.moveTo(0, -0.7)
      c.quadraticCurveTo(1.2, 1.4, 0, 1.5)
      c.quadraticCurveTo(-0.8, 0.9, 0, -0.7)
      c.fill()
      break
    }
    case 'petal': {
      const g = c.createLinearGradient(-1, -1, 1, 2)
      g.addColorStop(0, '#fff0f6')
      g.addColorStop(0.5, color)
      g.addColorStop(1, '#bd587f')
      c.fillStyle = g
      c.beginPath()
      c.moveTo(0, -0.8)
      c.bezierCurveTo(-0.7, -2.2, -2.1, -0.4, -1, 1.1)
      c.quadraticCurveTo(-0.2, 2.1, 0.3, 2.6)
      c.bezierCurveTo(2.2, 1.1, 2, -1.6, 0.7, -1.4)
      c.lineTo(0, -0.8)
      c.fill()
      c.strokeStyle = '#fff6ff99'
      line(0.3, 2, 0, -0.4)
      line(0.1, 0.7, -0.7, 0)
      break
    }
    case 'leaf': {
      const g = c.createLinearGradient(-1, 0, 1, 1)
      g.addColorStop(0, '#304e23')
      g.addColorStop(0.5, color)
      g.addColorStop(1, '#e4ed9a')
      c.fillStyle = g
      c.beginPath()
      c.moveTo(0, -2.2)
      c.bezierCurveTo(2.4, -0.2, 1.6, 1.7, 0, 2.1)
      c.bezierCurveTo(-1.8, 1, -1.9, -0.8, 0, -2.2)
      c.fill()
      c.strokeStyle = '#334d29'
      line(0, -1.7, 0, 2.6)
      for (let i = 0; i < 3; i++) {
        const y = -0.8 + i * 0.7
        line(0, y, 0.85, y - 0.5)
        line(0, y, -0.8, y - 0.5)
      }
      break
    }
    case 'snow':
      glow(2.9)
      c.strokeStyle = color
      c.lineWidth = 0.18
      for (let a = 0; a < 6; a++) {
        line(0, 0, 0, -2.3)
        for (let j = 1; j < 3; j++) {
          const y = -j * 0.7
          line(0, y, -0.48, y - 0.42)
          line(0, y, 0.48, y - 0.42)
        }
        c.rotate(Math.PI / 3)
      }
      c.fillStyle = '#ffffff'
      dot(0, 0, 0.25)
      break
    case 'crystal': {
      const g = c.createLinearGradient(-1, -2, 1, 2)
      g.addColorStop(0, '#f7edff')
      g.addColorStop(0.4, color)
      g.addColorStop(1, '#4d316c')
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
      c.stroke()
      c.lineWidth = 0.12
      line(0, -2.8, 0.3, -0.4)
      line(0.3, -0.4, 0, 2.5)
      line(-1, -1, 0.3, -0.4)
      line(0.3, -0.4, 1.2, -0.7)
      line(-1.1, 0.5, 0.3, -0.4)
      break
    }
    case 'bubble': {
      const g = c.createRadialGradient(-0.7, -0.7, 0.1, 0, 0, 2)
      g.addColorStop(0, '#ffffff50')
      g.addColorStop(0.55, color + '08')
      g.addColorStop(1, color + '80')
      c.fillStyle = g
      dot(0, 0, 2)
      c.strokeStyle = color
      c.beginPath()
      c.arc(0, 0, 2, 0, Math.PI * 2)
      c.stroke()
      c.strokeStyle = '#ffffffc0'
      c.lineWidth = 0.24
      c.beginPath()
      c.arc(-0.1, -0.1, 1.55, 3.5, 4.7)
      c.stroke()
      c.fillStyle = '#ffffff'
      dot(-0.85, -1, 0.2)
      break
    }
    case 'star':
      glow(4)
      c.beginPath()
      c.moveTo(0, -2.8)
      c.quadraticCurveTo(0.3, -0.3, 2.8, 0)
      c.quadraticCurveTo(0.3, 0.3, 0, 2.8)
      c.quadraticCurveTo(-0.3, 0.3, -2.8, 0)
      c.quadraticCurveTo(-0.3, -0.3, 0, -2.8)
      c.fill()
      c.fillStyle = '#fffaff'
      dot(0, 0, 0.4)
      line(-1.8, -1.8, 1.8, 1.8)
      break
    case 'rain': {
      const g = c.createLinearGradient(0, -4, 0, 3)
      g.addColorStop(0, color + '00')
      g.addColorStop(0.8, color)
      g.addColorStop(1, '#ffffff')
      c.strokeStyle = g
      c.lineWidth = 0.3
      line(0.8, -4, -0.8, 3)
      c.strokeStyle = color + '80'
      c.lineWidth = 0.12
      line(1.4, -2, 0.5, 1.5)
      c.fillStyle = '#ffffff'
      dot(-0.8, 3, 0.2)
      break
    }
    case 'sand':
      c.beginPath()
      c.moveTo(-1.8, 0)
      c.lineTo(-0.8, -0.75)
      c.lineTo(1.5, -0.45)
      c.lineTo(1.1, 0.7)
      c.lineTo(-1, 0.9)
      c.closePath()
      c.fill()
      c.strokeStyle = '#fff1c6'
      line(-0.8, -0.5, 1.2, -0.3)
      c.strokeStyle = color
      line(-3.6, 0.8, -2.3, 0.6)
      break
    case 'comet': {
      const g = c.createLinearGradient(-8, -3, 0, 0)
      g.addColorStop(0, color + '00')
      g.addColorStop(1, color)
      c.strokeStyle = g
      c.lineWidth = 0.7
      line(-8, -3, 0, 0)
      c.lineWidth = 0.16
      line(-6, -3.8, -0.5, -0.4)
      line(-7, -1.7, -1, 0.2)
      glow(2.6)
      c.fillStyle = '#fff0df'
      dot(0, 0, 0.65)
      break
    }
    case 'ribbon':
      c.lineWidth = 0.22
      c.beginPath()
      c.moveTo(-4, -1)
      c.bezierCurveTo(-2, -4, 0, 3, 4, 0)
      c.stroke()
      c.strokeStyle = '#fff0c399'
      c.lineWidth = 0.1
      c.beginPath()
      c.moveTo(-4, -0.6)
      c.bezierCurveTo(-2, -3.6, 0, 3.4, 4, 0.4)
      c.stroke()
      c.fillStyle = color
      c.beginPath()
      c.moveTo(-0.8, 0)
      c.lineTo(0, -0.7)
      c.lineTo(0.8, 0)
      c.lineTo(0, 0.6)
      c.closePath()
      c.fill()
      break
    case 'plankton':
      glow(4)
      c.lineWidth = 0.12
      c.beginPath()
      c.ellipse(0, 0, 1, 1.7, 0, 0, Math.PI * 2)
      c.stroke()
      for (let i = 0; i < 5; i++) {
        const y = (i - 2) * 0.5
        line(-0.7, y, -1.7, y + 0.45)
        line(0.7, y, 1.7, y + 0.45)
      }
      c.fillStyle = '#dcfffa'
      dot(0, 0, 0.5)
      c.strokeStyle = color
      c.beginPath()
      c.moveTo(0, 1.6)
      c.bezierCurveTo(2, 2, -1, 3, 1, 3.5)
      c.stroke()
      break
    default: // winged firefly with luminous abdomen
      glow(4.5)
      c.fillStyle = '#d4f7ff90'
      c.beginPath()
      c.ellipse(-0.8, -0.4, 0.65, 1.35, -0.6, 0, Math.PI * 2)
      c.ellipse(0.8, -0.4, 0.65, 1.35, 0.6, 0, Math.PI * 2)
      c.fill()
      c.fillStyle = color
      dot(0, 0.65, 0.8)
      c.fillStyle = '#fff8c0'
      dot(0, 0.7, 0.35)
      c.strokeStyle = color
      line(0, -0.6, 0, 1.3)
      line(0, -0.5, -0.5, -1.3)
      line(0, -0.5, 0.5, -1.3)
  }
}
