import { particleArt } from './particle-art.js'
import { EFFECT_LEVELS } from './catalog.js'

const TAU = Math.PI * 2
const clamp = (value, min, max) => Math.min(max, Math.max(min, value))
const span = (random, [min, max]) => min + random() * (max - min)
const drift = (p, t, rate = 0.45, shift = 0) => Math.sin(t * rate + p.phase + shift)
const swirl = (p, t, rate = 0.5) => Math.cos(t * rate + p.phase * 1.3)

// Each theme owns one kind. A profile describes how that kind lives on screen:
// motion (px/s), orientation, density, size and how it reacts to a click burst.
//   orient: spin | upright | sway | velocity | swim
//   burst:  fall | rise | radial | drift | swim
//   extent: half-size of the art in particle units, used for off-screen wrapping.
const PROFILES = {
  aurora: {
    size: [4, 7.5],
    alpha: [0.2, 0.42],
    density: 0.42,
    extent: 7,
    orient: 'sway',
    swing: [0.08, 0.3],
    burst: 'rise',
    move(p, t) {
      p.vx = 3 + drift(p, t, 0.25) * 5
      p.vy = swirl(p, t, 0.2) * 2
    },
  },
  rain: {
    size: [2, 4.4],
    alpha: [0.35, 0.75],
    density: 1.15,
    accent: 0.55,
    extent: 5,
    orient: 'velocity',
    angle: -Math.PI / 2,
    burst: 'fall',
    move(p) {
      p.vx = -22 * p.speed
      p.vy = 105 * p.speed
    },
  },
  ember: {
    size: [1.8, 3.6],
    alpha: [0.45, 0.85],
    variants: [3, 1],
    extent: 4,
    orient: 'velocity',
    angle: Math.PI / 2,
    burst: 'rise',
    move(p, t) {
      p.vx = drift(p, t, 0.9) * 12 + drift(p, t, 2.3, 1) * 4
      p.vy = -(16 + 24 * p.speed)
    },
    // Ash is cooler and lighter: it tumbles slowly instead of pointing along its path.
    variant: {
      1: {
        orient: 'spin',
        spin: [0.4, 1.2],
        flip: 1.4,
        move(p, t) {
          p.vx = 4 + drift(p, t, 0.6) * 9
          p.vy = -(6 + 8 * p.speed)
        },
      },
    },
  },
  petal: {
    size: [1.9, 3.6],
    alpha: [0.5, 0.88],
    extent: 3,
    orient: 'spin',
    spin: [0.5, 1.4],
    flip: 1.3,
    burst: 'fall',
    move(p, t) {
      p.vx = 10 + drift(p, t, 0.7) * 14
      p.vy = 13 + 12 * p.speed
    },
  },
  bubble: {
    size: [1.4, 4],
    alpha: [0.4, 0.78],
    extent: 2.5,
    parallax: true,
    orient: 'upright',
    burst: 'rise',
    move(p, t) {
      p.vx = drift(p, t, 1.6) * 7
      p.vy = -(9 + 16 * p.speed)
    },
  },
  orbit: {
    size: [1.4, 2.8],
    alpha: [0.5, 0.92],
    density: 0.7,
    extent: 7,
    orient: 'velocity',
    burst: 'radial',
    absolute: true,
    init(p, random) {
      p.orbitA = 0.12 + random() ** 0.8 * 0.8
      p.orbitB = 0.32 + random() * 0.18
      p.orbitTilt = -0.32 + random() * 0.18
      p.orbitSpeed = clamp(0.07 * (0.35 / p.orbitA) ** 1.5, 0.025, 0.4)
      p.orbitStart = random() * TAU
    },
    // Stars follow tilted ellipses around the centre; inner orbits are faster (Kepler-like).
    place(p, t, w, h) {
      const tall = h > w
      const long = Math.max(w, h) / 2,
        short = Math.min(w, h) / 2
      const a = p.orbitA * long * 0.92
      const b = Math.min(a * p.orbitB, short * 0.92)
      const tilt = p.orbitTilt * (short / long) + (tall ? Math.PI / 2 : 0)
      const angle = p.orbitStart + t * p.orbitSpeed
      const ca = Math.cos(angle),
        sa = Math.sin(angle),
        ct = Math.cos(tilt),
        st = Math.sin(tilt)
      p.x = w / 2 + a * ca * ct - b * sa * st
      p.y = h / 2 + a * ca * st + b * sa * ct
      p.vx = (-a * sa * ct - b * ca * st) * p.orbitSpeed
      p.vy = (-a * sa * st + b * ca * ct) * p.orbitSpeed
      p.depth = 0.72 + 0.28 * ((sa + 1) / 2)
    },
  },
  firefly: {
    size: [1.5, 2.9],
    alpha: [0.55, 0.95],
    density: 0.8,
    extent: 5.5,
    orient: 'sway',
    swing: [0.25, 0.8],
    burst: 'radial',
    move(p, t) {
      p.vx = drift(p, t, 0.6) * 12 + Math.cos(t * 1.7 + p.phase * 2) * 5
      p.vy = swirl(p, t, 0.5) * 9 - 1.5
    },
  },
  snow: {
    size: [1.5, 3.4],
    alpha: [0.45, 0.88],
    extent: 3,
    parallax: true,
    orient: 'spin',
    spin: [0.15, 0.55],
    burst: 'fall',
    move(p, t) {
      p.vx = 4 + drift(p, t, 0.5) * 9
      p.vy = 11 + 13 * p.speed
    },
  },
  sand: {
    size: [1.4, 2.6],
    alpha: [0.4, 0.78],
    density: 1.15,
    extent: 5.5,
    orient: 'velocity',
    burst: 'drift',
    move(p, t) {
      const gust = 0.7 + 0.3 * Math.sin(t * 0.35 + p.y * 0.006)
      p.vx = (26 + 28 * p.speed) * gust
      p.vy = drift(p, t, 1.1) * 3 - 1
    },
  },
  crystal: {
    size: [1.8, 3.4],
    alpha: [0.45, 0.82],
    density: 0.75,
    extent: 3,
    orient: 'sway',
    swing: [0.3, 0.35],
    burst: 'radial',
    move(p, t) {
      p.vx = drift(p, t, 0.35) * 4
      p.vy = -(2 + 5 * p.speed)
    },
  },
  corona: {
    size: [1.6, 3],
    alpha: [0.38, 0.75],
    density: 0.38,
    extent: 6,
    orient: 'spin',
    spin: [0.03, 0.1],
    burst: 'radial',
    move(p, t) {
      p.vx = 2.5 + drift(p, t, 0.2) * 3
      p.vy = -1.2 * p.speed + swirl(p, t, 0.18) * 1.5
    },
  },
  goldleaf: {
    size: [1.7, 3.2],
    alpha: [0.55, 0.92],
    extent: 3,
    orient: 'spin',
    spin: [0.2, 0.8],
    flip: 1.7,
    burst: 'fall',
    move(p, t) {
      p.vx = 2 + drift(p, t, 0.6) * 11
      p.vy = 7 + 8 * p.speed
    },
  },
  plankton: {
    size: [1.5, 2.9],
    alpha: [0.45, 0.88],
    variants: [2, 1],
    extent: 3.6,
    orient: 'spin',
    spin: [0.05, 0.25],
    twinkle: true,
    burst: 'radial',
    move(p, t) {
      p.vx = 4 + drift(p, t, 0.3) * 8
      p.vy = swirl(p, t, 0.27) * 6 - 1.5
    },
  },
  geometry: {
    size: [2, 3.8],
    alpha: [0.28, 0.55],
    density: 0.55,
    variants: [1, 1, 1, 1],
    extent: 4,
    orient: 'spin',
    spin: [0.04, 0.18],
    burst: 'radial',
    move(p, t) {
      p.vx = 4 + drift(p, t, 0.2) * 3
      p.vy = -(2.5 + 3 * p.speed)
    },
  },
  heart: {
    size: [1.5, 3],
    alpha: [0.5, 0.88],
    density: 0.8,
    extent: 3.4,
    orient: 'sway',
    swing: [0.22, 0.9],
    burst: 'rise',
    move(p, t) {
      p.vx = drift(p, t, 0.9) * 8
      p.vy = -(7 + 10 * p.speed)
    },
  },
  wave: {
    size: [1.8, 3.2],
    alpha: [0.4, 0.78],
    density: 0.7,
    extent: 5,
    orient: 'upright',
    burst: 'drift',
    move(p, t) {
      p.vx = 13 + 16 * p.speed
      p.vy = drift(p, t, 0.5) * 2
    },
  },
  tea: {
    size: [1.8, 3.2],
    alpha: [0.5, 0.88],
    variants: [3, 1],
    extent: 3.5,
    orient: 'spin',
    spin: [0.3, 0.9],
    flip: 1,
    burst: 'fall',
    move(p, t) {
      p.vx = 7 + drift(p, t, 0.65) * 14
      p.vy = 10 + 9 * p.speed
    },
  },
  berry: {
    size: [1.7, 3],
    alpha: [0.55, 0.92],
    variants: [3, 1],
    extent: 3,
    orient: 'sway',
    swing: [0.3, 0.6],
    burst: 'fall',
    move(p, t) {
      p.vx = drift(p, t, 0.5) * 6
      p.vy = 8 + 8 * p.speed
    },
  },
  cloud: {
    size: [2.8, 5.6],
    alpha: [0.32, 0.62],
    density: 0.38,
    extent: 4,
    parallax: true,
    orient: 'upright',
    burst: 'drift',
    move(p, t) {
      p.vx = 3 + 2.4 * p.size * p.speed
      p.vy = drift(p, t, 0.2) * 1.2
    },
  },
  coffee: {
    size: [1.7, 3.1],
    alpha: [0.55, 0.9],
    variants: [2, 1],
    extent: 4,
    orient: 'spin',
    spin: [0.25, 0.7],
    flip: 0.9,
    burst: 'fall',
    move(p, t) {
      p.vx = drift(p, t, 0.5) * 6
      p.vy = 7 + 7 * p.speed
    },
    // Steam rises upright while the beans fall and tumble.
    variant: {
      1: {
        orient: 'upright',
        flip: 0,
        burst: 'rise',
        move(p, t) {
          p.vx = drift(p, t, 0.8) * 4
          p.vy = -(9 + 8 * p.speed)
        },
      },
    },
  },
  lavender: {
    size: [1.9, 3.4],
    alpha: [0.5, 0.88],
    variants: [2, 1],
    extent: 4,
    orient: 'sway',
    swing: [0.55, 0.7],
    burst: 'fall',
    move(p, t) {
      p.vx = 6 + drift(p, t, 0.6) * 10
      p.vy = 9 + 7 * p.speed
    },
    variant: { 1: { orient: 'spin', spin: [0.4, 1] } },
  },
  ginkgo: {
    size: [1.8, 3.3],
    alpha: [0.5, 0.88],
    extent: 3.5,
    orient: 'spin',
    spin: [0.3, 0.9],
    flip: 1.1,
    burst: 'fall',
    move(p, t) {
      p.vx = 7 + drift(p, t, 0.7) * 13
      p.vy = 9 + 9 * p.speed
    },
  },
  comet: {
    size: [1.4, 2.6],
    alpha: [0.5, 0.92],
    density: 0.33,
    extent: 10,
    parallax: true,
    orient: 'velocity',
    burst: 'radial',
    move(p) {
      p.vx = 40 + 32 * p.speed
      p.vy = 16 + 13 * p.speed
    },
  },
  bokeh: {
    size: [2.4, 5.6],
    alpha: [0.18, 0.42],
    density: 0.6,
    variants: [1, 1],
    extent: 3,
    orient: 'upright',
    burst: 'radial',
    move(p, t) {
      p.vx = 2 + drift(p, t, 0.18) * 3
      p.vy = -(1.5 + 2 * p.speed)
    },
  },
  honeycomb: {
    size: [1.9, 3.4],
    alpha: [0.45, 0.85],
    variants: [3, 1],
    extent: 3,
    orient: 'spin',
    spin: [0.05, 0.22],
    burst: 'radial',
    move(p, t) {
      p.vx = 1 + drift(p, t, 0.25) * 4
      p.vy = -(1.5 + 3 * p.speed)
    },
    // A honey drop falls slowly with its point up.
    variant: {
      1: {
        orient: 'upright',
        burst: 'fall',
        move(p, t) {
          p.vx = drift(p, t, 0.4) * 1.5
          p.vy = 12 + 10 * p.speed
        },
      },
    },
  },
  diamond: {
    size: [1.3, 2.6],
    alpha: [0.5, 0.92],
    density: 1.25,
    extent: 4.5,
    orient: 'spin',
    spin: [0.2, 0.8],
    burst: 'radial',
    move(p, t) {
      p.vx = 2 + drift(p, t, 0.4) * 5
      p.vy = 4 + 5 * p.speed
    },
  },
  code: {
    size: [3.6, 5.6],
    alpha: [0.4, 0.82],
    density: 0.75,
    accent: 0.88,
    extent: 13,
    orient: 'upright',
    burst: 'fall',
    move(p) {
      p.vx = 0
      p.vy = 28 + 26 * p.speed
    },
  },
  moon: {
    size: [2, 3.8],
    alpha: [0.42, 0.82],
    density: 0.42,
    extent: 4,
    orient: 'upright',
    burst: 'radial',
    move(p, t) {
      p.vx = 2 + drift(p, t, 0.2) * 2
      p.vy = -1.5 * p.speed + swirl(p, t, 0.15)
    },
  },
  fish: {
    size: [1.8, 3.2],
    alpha: [0.6, 0.94],
    density: 0.6,
    variants: [1, 1],
    extent: 3.5,
    orient: 'swim',
    burst: 'swim',
    init(p, random) {
      p.dir = random() < 0.5 ? -1 : 1
    },
    move(p, t) {
      p.vx = p.dir * (12 + 16 * p.speed) * (1 + 0.25 * drift(p, t, 0.8))
      p.vy = drift(p, t, 0.6, 1) * 5
    },
  },
  plasma: {
    size: [1.8, 3.4],
    alpha: [0.5, 0.9],
    density: 0.7,
    extent: 4,
    orient: 'spin',
    spin: [0.1, 0.4],
    burst: 'radial',
    move(p, t) {
      p.vx = drift(p, t, 0.4) * 5
      p.vy = -2 + swirl(p, t, 0.3) * 3
    },
  },
  gear: {
    size: [1.8, 3.6],
    alpha: [0.35, 0.72],
    density: 0.6,
    variants: [3, 1],
    extent: 3.5,
    orient: 'spin',
    spin: [0.35, 0.9],
    burst: 'radial',
    move(p, t) {
      p.vx = 2 + drift(p, t, 0.2) * 3
      p.vy = -(2 + 2.5 * p.speed)
    },
  },
  lantern: {
    size: [1.6, 3.6],
    alpha: [0.55, 0.95],
    density: 0.55,
    accent: 0.85,
    extent: 5,
    parallax: true,
    orient: 'sway',
    swing: [0.08, 0.6],
    burst: 'rise',
    move(p, t) {
      p.vx = 3 + drift(p, t, 0.4) * 5
      p.vy = -(6 + 9 * p.speed)
    },
  },
}

const DEFAULTS = {
  size: [1, 2],
  alpha: [0.4, 0.8],
  density: 1,
  accent: 0.7,
  variants: null,
  extent: 3,
  orient: 'spin',
  angle: 0,
  spin: [0.05, 0.2],
  swing: [0.2, 0.5],
  flip: 0,
  twinkle: false,
  parallax: false,
  burst: 'radial',
  absolute: false,
  init: null,
  place: null,
  move(p, t) {
    p.vx = drift(p, t) * 8
    p.vy = swirl(p, t) * 6 - 2
  },
}

export const PARTICLE_PROFILES = Object.freeze(
  Object.fromEntries(
    Object.entries(PROFILES).map(([kind, profile]) => [
      kind,
      Object.freeze({ ...DEFAULTS, ...profile, variant: undefined }),
    ]),
  ),
)
const merged = new Map()
// Resolved profile for a kind and variant (variant overrides inherit from the kind).
export function particleProfile(kind, variant = 0) {
  const key = `${kind}:${variant}`
  let profile = merged.get(key)
  if (!profile) {
    const base = PARTICLE_PROFILES[kind] ?? PARTICLE_PROFILES.firefly
    profile = Object.freeze({ ...base, ...(PROFILES[kind]?.variant?.[variant] ?? {}) })
    merged.set(key, profile)
  }
  return profile
}

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

function pickVariant(weights, random) {
  if (!weights?.length) return 0
  let roll = random() * weights.reduce((sum, weight) => sum + weight, 0)
  for (let i = 0; i < weights.length; i++) {
    roll -= weights[i]
    if (roll < 0) return i
  }
  return weights.length - 1
}

// (Re)initialise every kind-specific field. Used on creation and when the theme changes.
function assign(p, kind, palette, random, scale = 1) {
  const base = PARTICLE_PROFILES[kind] ?? PARTICLE_PROFILES.firefly
  p.kind = kind
  p.variant = pickVariant(base.variants, random)
  const profile = particleProfile(kind, p.variant)
  const sizeRoll = random()
  p.size = (profile.size[0] + sizeRoll * (profile.size[1] - profile.size[0])) * scale
  // Larger particles are "closer": with parallax they move faster.
  p.speed = profile.parallax ? 0.45 + sizeRoll * 0.55 + random() * 0.15 : 0.45 + random() * 0.7
  p.alpha = span(random, profile.alpha)
  const accent = random() < profile.accent
  p.color = accent ? palette.accent : palette.secondary
  p.alt = accent ? palette.secondary : palette.accent
  p.spin = span(random, profile.spin) * (random() < 0.5 ? -1 : 1)
  p.rotation = profile.orient === 'spin' ? random() * TAU : 0
  p.dir = 1
  p.flip = 1
  p.depth = 1
  p.vx = 0
  p.vy = 0
  profile.init?.(p, random)
  return profile
}

function orient(p, profile, t, dt) {
  switch (profile.orient) {
    case 'velocity':
      if (p.vx || p.vy) p.rotation = Math.atan2(p.vy, p.vx) + profile.angle
      break
    case 'sway':
      p.rotation = profile.swing[0] * Math.sin(t * profile.swing[1] + p.phase)
      break
    case 'swim':
      p.rotation = clamp(Math.atan2(p.vy, Math.abs(p.vx) || 1), -0.4, 0.4) * p.dir
      break
    case 'upright':
      p.rotation = 0
      break
    default:
      p.rotation += p.spin * dt
  }
  // A 3D tumble: the art is squashed horizontally and its sheen follows the turn.
  p.flip = profile.flip ? Math.cos(t * profile.flip * (0.6 + 0.5 * p.speed) + p.phase * 1.7) : 1
}

export function createParticles(
  theme,
  width,
  height,
  quality = 'balanced',
  preview = false,
  sizeScale = 1,
) {
  if (
    !theme?.particle ||
    !Number.isFinite(width) ||
    !Number.isFinite(height) ||
    width <= 0 ||
    height <= 0
  )
    return []
  const kind = theme.particle
  const base = PARTICLE_PROFILES[kind] ?? PARTICLE_PROFILES.firefly
  const random = seededRandom(`${theme.id}:${quality}:${preview}`)
  const budget = EFFECT_LEVELS.find((level) => level.id === quality)?.count ?? 52
  const count = preview
    ? clamp(Math.round(16 * base.density), 5, 24)
    : Math.max(
        Math.round(12 * Math.min(1, base.density)),
        Math.round(budget * base.density * Math.min(1, (width * height) / 900000)),
      )
  const scale = (preview ? 0.72 : 1) * (Number.isFinite(sizeScale) && sizeScale > 0 ? sizeScale : 1)
  return Array.from({ length: count }, () => {
    const p = { x: random() * width, y: random() * height, phase: random() * TAU }
    const profile = assign(p, kind, theme.palette, random, scale)
    p.scale = scale
    if (profile.absolute) profile.place(p, 0, width, height)
    else profile.move(p, 0, width, height)
    orient(p, profile, 0, 0)
    return p
  })
}

function wrap(p, profile, width, height) {
  const margin = p.size * profile.extent + 8
  if (p.x < -margin) {
    p.x = width + margin
    p.y = Math.random() * height
  } else if (p.x > width + margin) {
    p.x = -margin
    p.y = Math.random() * height
  }
  if (p.y < -margin) {
    p.y = height + margin
    p.x = Math.random() * width
  } else if (p.y > height + margin) {
    p.y = -margin
    p.x = Math.random() * width
  }
}

export function advanceParticles(particles, kind, width, height, delta, time) {
  if (!kind || !Number.isFinite(width) || !Number.isFinite(height) || width <= 0 || height <= 0)
    return
  const dt = Number.isFinite(delta) ? clamp(delta, 0, 0.08) : 0
  time = Number.isFinite(time) ? time : 0
  for (const p of particles) {
    if (p.kind !== kind) {
      // The theme changed under a live canvas: adopt the new kind without a jump.
      const palette = { accent: p.color, secondary: p.alt ?? p.color }
      assign(p, kind, palette, Math.random, p.scale ?? 1)
    }
    const profile = particleProfile(kind, p.variant)
    if (profile.absolute) profile.place(p, time, width, height)
    else {
      profile.move(p, time, width, height)
      p.x += p.vx * dt
      p.y += p.vy * dt
      wrap(p, profile, width, height)
    }
    orient(p, profile, time, dt)
    if (!Number.isFinite(p.x) || !Number.isFinite(p.y)) {
      p.x = Math.random() * width
      p.y = Math.random() * height
    }
    if (!Number.isFinite(p.rotation)) p.rotation = 0
  }
}

function paintOne(ctx, p, kind, profile, time, opacity) {
  const flip = Number.isFinite(p.flip) ? p.flip : 1
  const squash = (flip < 0 ? -1 : 1) * Math.max(0.18, Math.abs(flip))
  const size = (p.size || 1) * (p.depth || 1)
  ctx.save()
  ctx.translate(p.x, p.y)
  if (p.rotation) ctx.rotate(p.rotation)
  ctx.scale(size * squash * (p.dir || 1), size)
  ctx.globalAlpha = clamp(opacity, 0, 1)
  particleArt(ctx, kind, p.color, p.phase, time, p.alt ?? p.color, p.variant ?? 0, flip)
  ctx.restore()
}

export function drawParticles(ctx, particles, kind, width, height, time = 0, subdued = false) {
  ctx.clearRect(0, 0, width, height)
  const base = subdued ? 0.65 : 1
  for (const p of particles) {
    const profile = particleProfile(kind, p.variant ?? 0)
    const shimmer = profile.twinkle
      ? 0.62 + 0.38 * (0.5 + 0.5 * Math.sin(time * 1.9 + p.phase * 3))
      : 0.9 + 0.1 * Math.sin(time * 0.8 + p.phase)
    const depth = p.depth ? 0.55 + 0.45 * ((p.depth - 0.72) / 0.28) : 1
    paintOne(ctx, p, kind, profile, time, (p.alpha ?? 0.6) * base * shimmer * depth)
  }
}

// Click bursts reuse the same art and orientation rules, so they always match the theme.
export const BURST_DURATION = 1.05
export function createBurst(theme, x, y, count, random = Math.random) {
  const kind = theme?.particle
  if (!kind) return []
  return Array.from({ length: count }, () => {
    const p = { x, y, phase: random() * TAU }
    // Large kinds (aurora, clouds, bokeh, code columns) are shrunk so a burst stays compact.
    const base = PARTICLE_PROFILES[kind] ?? PARTICLE_PROFILES.firefly
    const profile = assign(p, kind, theme.palette, random, 0.9 * Math.min(1, 3.4 / base.size[1]))
    const angle = random() * TAU,
      speed = 40 + random() * 130
    p.gravity = 35
    p.sway = 0
    switch (profile.burst) {
      case 'fall':
        p.vx = Math.cos(angle) * speed * 0.8
        p.vy = -30 - random() * 90
        p.gravity = 150
        p.sway = 16
        break
      case 'rise':
        p.vx = Math.cos(angle) * speed * 0.7
        p.vy = -40 - random() * 120
        p.gravity = -25
        p.sway = 12
        break
      case 'drift': {
        const side = random() < 0.5 ? -1 : 1
        p.vx = side * (50 + random() * 160)
        p.vy = (random() - 0.5) * 90
        p.gravity = 20
        break
      }
      case 'swim':
        p.dir = random() < 0.5 ? -1 : 1
        p.vx = p.dir * (60 + random() * 130)
        p.vy = (random() - 0.5) * 60
        p.gravity = 0
        p.sway = 8
        break
      default:
        p.vx = Math.cos(angle) * speed
        p.vy = Math.sin(angle) * speed - 15
    }
    return p
  })
}

export function drawBurst(ctx, particles, kind, t) {
  const fade = clamp((BURST_DURATION - t) * 2, 0, 1) * 0.92
  for (const p of particles) {
    const profile = particleProfile(kind, p.variant ?? 0)
    const vy = p.vy + p.gravity * t
    const view = {
      ...p,
      x: p.x + p.vx * t + Math.sin(t * 5 + p.phase) * p.sway,
      y: p.y + p.vy * t + 0.5 * p.gravity * t * t,
      vy,
      depth: 1,
    }
    switch (profile.orient) {
      case 'velocity':
        view.rotation = Math.atan2(vy, p.vx) + profile.angle
        break
      case 'sway':
        view.rotation = profile.swing[0] * Math.sin(t * 4 + p.phase)
        break
      case 'swim':
        view.rotation = clamp(Math.atan2(vy, Math.abs(p.vx) || 1), -0.4, 0.4) * p.dir
        break
      case 'upright':
        view.rotation = 0
        break
      default:
        view.rotation = p.rotation + p.spin * 3 * t
    }
    view.flip = profile.flip ? Math.cos(t * profile.flip * 3 + p.phase * 1.7) : 1
    paintOne(ctx, view, kind, profile, t, fade)
  }
}
