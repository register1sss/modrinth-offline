// Version ranges as written in fabric.mod.json, quilt.mod.json and mods.toml.
// Pure functions: they never guess. `null` means "cannot tell", never "compatible".
const say = (lang, ru, en) => (lang === 'en' ? en : ru)

const QUALIFIERS = {
  snapshot: -5,
  alpha: -4,
  a: -4,
  beta: -3,
  b: -3,
  milestone: -2,
  m: -2,
  rc: -1,
  cr: -1,
  pre: -1,
  preview: -1,
}
const isNumber = (token) => /^\d+$/.test(token)
const rank = (token) => QUALIFIERS[token] ?? 0

/** Splits "v1.20.1-pre2+build.7" into ['1', '20', '1', 'pre', '2'] (build metadata ignored). */
export function versionTokens(version) {
  return (
    String(version ?? '')
      .trim()
      .toLowerCase()
      .replace(/^v(?=\d)/, '')
      .split('+')[0]
      .match(/\d+|[a-z]+/g) || []
  )
}

/** -1, 0 or 1; null when either side does not start with a number. */
export function compareVersions(a, b) {
  const x = versionTokens(a)
  const y = versionTokens(b)
  if (!isNumber(x[0] ?? '') || !isNumber(y[0] ?? '')) return null
  for (let i = 0; i < Math.max(x.length, y.length); i++) {
    let p = x[i]
    let q = y[i]
    if (p === undefined) p = isNumber(q) ? '0' : 'release'
    if (q === undefined) q = isNumber(p) ? '0' : 'release'
    const pn = isNumber(p)
    const qn = isNumber(q)
    if (pn && qn) {
      const d = Number(p) - Number(q)
      if (d) return d > 0 ? 1 : -1
    } else if (pn !== qn) {
      // 1.0.1 is newer than 1.0-beta.
      return pn ? 1 : -1
    } else if (rank(p) !== rank(q)) {
      return rank(p) > rank(q) ? 1 : -1
    }
  }
  return 0
}

const numericCore = (version) => {
  const tokens = versionTokens(version)
  const core = []
  for (const t of tokens) {
    if (!isNumber(t)) break
    core.push(Number(t))
  }
  return core
}
const isWildcard = (part) => /^[xX*]$/.test(part)
const trimDash = (v) => v.replace(/-$/, '')

function fabricPredicate(predicate, version) {
  const m = predicate.match(/^(>=|<=|>|<|=|~|\^)?\s*(.+)$/)
  if (!m) return null
  const op = m[1] || '='
  const target = trimDash(m[2])
  if (target === '*') return true
  const parts = target.split('.')
  const wildcard = parts.findIndex(isWildcard)
  if (wildcard >= 0) {
    const prefix = parts.slice(0, wildcard)
    const core = numericCore(version)
    if (!core.length || prefix.some((p) => !isNumber(p))) return null
    return prefix.every((p, i) => Number(p) === (core[i] ?? 0))
  }
  const c = compareVersions(version, target)
  if (c === null) return null
  switch (op) {
    case '>=':
      return c >= 0
    case '>':
      return c > 0
    case '<=':
      return c <= 0
    case '<':
      return c < 0
    case '~': {
      const want = numericCore(target)
      const have = numericCore(version)
      const keep = Math.max(1, Math.min(2, want.length))
      return c >= 0 && want.slice(0, keep).every((n, i) => n === (have[i] ?? 0))
    }
    case '^': {
      const want = numericCore(target)
      const have = numericCore(version)
      return c >= 0 && (want[0] ?? 0) === (have[0] ?? 0)
    }
    default:
      return c === 0
  }
}

const all = (results) => (results.includes(false) ? false : results.includes(null) ? null : true)
const any = (results) =>
  results.includes(true) ? true : results.includes(null) || !results.length ? null : false

const MAVEN_GROUP = /([[(])\s*([^,\])]*?)\s*(?:,\s*([^\])]*?)\s*)?([\])])/g
function mavenGroups(text) {
  return [...text.matchAll(MAVEN_GROUP)].map((m) => ({
    open: m[1],
    low: m[2] || null,
    // Without a comma "[1.0]" means exactly 1.0.
    high: m[3] === undefined ? m[2] || null : m[3] || null,
    exact: m[3] === undefined,
    close: m[4],
  }))
}
function mavenSatisfies(text, version) {
  // Forge treats a bare version as a recommendation: any version loads.
  if (!/[[(]/.test(text)) return true
  const groups = mavenGroups(text)
  if (!groups.length) return null
  return any(
    groups.map((g) => {
      if (g.exact) {
        const c = compareVersions(version, g.low)
        return c === null ? null : c === 0
      }
      const checks = []
      if (g.low) {
        const c = compareVersions(version, g.low)
        checks.push(c === null ? null : g.open === '[' ? c >= 0 : c > 0)
      }
      if (g.high) {
        const c = compareVersions(version, g.high)
        checks.push(c === null ? null : g.close === ']' ? c <= 0 : c < 0)
      }
      return all(checks)
    }),
  )
}

/**
 * Whether `version` satisfies a dependency range.
 * style: 'fabric' (Fabric/Quilt predicates) or 'maven' (Forge/NeoForge versionRange).
 */
export function versionSatisfies(range, version, style = 'fabric') {
  if (range === null || range === undefined) return true
  if (version === null || version === undefined || version === '') return null
  if (Array.isArray(range)) {
    return range.length ? any(range.map((r) => versionSatisfies(r, version, style))) : true
  }
  if (typeof range === 'object') {
    if (Array.isArray(range.any)) return versionSatisfies(range.any, version, style)
    if (Array.isArray(range.all))
      return all(range.all.map((r) => versionSatisfies(r, version, style)))
    return null
  }
  const text = String(range).trim()
  if (!text || text === '*') return true
  if (style === 'maven' || /^[[(]/.test(text)) return mavenSatisfies(text, version)
  return any(
    text.split(/\s*\|\|\s*/).map((part) =>
      all(
        part
          .split(/\s+/)
          .filter(Boolean)
          .map((p) => fabricPredicate(p, version)),
      ),
    ),
  )
}

export const isAnyVersion = (range) =>
  range === null ||
  range === undefined ||
  (typeof range === 'string' && ['', '*'].includes(range.trim())) ||
  (Array.isArray(range) && (range.length === 0 || range.every(isAnyVersion)))

function describePredicate(predicate, lang) {
  const m = predicate.match(/^(>=|<=|>|<|=|~|\^)?\s*(.+)$/)
  if (!m) return predicate
  const v = trimDash(m[2])
  if (v === '*') return say(lang, 'любая версия', 'any version')
  const parts = v.split('.')
  const wildcard = parts.findIndex(isWildcard)
  if (wildcard >= 0) {
    const branch = `${parts.slice(0, wildcard).join('.')}.x`
    return say(lang, `любая ${branch}`, `any ${branch}`)
  }
  const branch = (keep) => `${parts.slice(0, keep).join('.')}.x`
  switch (m[1]) {
    case '>=':
      return say(lang, `не ниже ${v}`, `${v} or newer`)
    case '>':
      return say(lang, `выше ${v}`, `newer than ${v}`)
    case '<=':
      return say(lang, `не выше ${v}`, `${v} or older`)
    case '<':
      return say(lang, `ниже ${v}`, `older than ${v}`)
    case '~':
      return parts.length > 1
        ? say(lang, `не ниже ${v} в ветке ${branch(2)}`, `${v} or newer within ${branch(2)}`)
        : say(lang, `не ниже ${v} в ветке ${branch(1)}`, `${v} or newer within ${branch(1)}`)
    case '^':
      return say(lang, `не ниже ${v} в ветке ${branch(1)}`, `${v} or newer within ${branch(1)}`)
    default:
      return say(lang, `только ${v}`, `only ${v}`)
  }
}
const exactVersion = (range) =>
  typeof range === 'string' && /^=?\s*\d[^\s*xX<>=~^|]*$/.test(range.trim())
    ? trimDash(range.trim().replace(/^=\s*/, ''))
    : null
function describeMaven(text, lang) {
  const groups = mavenGroups(text)
  if (!groups.length) return text
  return groups
    .map((g) => {
      if (g.exact) return say(lang, `только ${g.low}`, `only ${g.low}`)
      const low =
        g.low &&
        (g.open === '['
          ? say(lang, `не ниже ${g.low}`, `${g.low} or newer`)
          : say(lang, `выше ${g.low}`, `newer than ${g.low}`))
      const high =
        g.high &&
        (g.close === ']'
          ? say(lang, `не выше ${g.high}`, `${g.high} or older`)
          : say(lang, `ниже ${g.high}`, `older than ${g.high}`))
      if (low && high) return `${low}, ${high}`
      return low || high || say(lang, 'любая версия', 'any version')
    })
    .join(say(lang, ' или ', ' or '))
}

/**
 * Human-readable range: "не ниже 0.92.0", "только 1.20.1", "любая 1.20.x".
 * compact: bare versions for lists after a colon ("1.20.1", "1.19.x").
 */
export function describeRange(range, style = 'fabric', lang = 'ru', compact = false) {
  const anyText = say(lang, 'любая версия', 'any version')
  if (isAnyVersion(range)) return anyText
  if (compact && typeof range === 'string' && /^=?\s*\d[\w.+-]*$/.test(range.trim()))
    return trimDash(range.trim().replace(/^=\s*/, ''))
  if (compact && Array.isArray(range) && range.every((r) => typeof r === 'string'))
    return [...new Set(range.map((r) => describeRange(r, style, lang, true)))].join(
      say(lang, ' или ', ' or '),
    )
  if (Array.isArray(range)) {
    const exact = range.map(exactVersion)
    if (style !== 'maven' && exact.every(Boolean))
      return [...new Set(exact)].join(say(lang, ' или ', ' or '))
    return [...new Set(range.map((r) => describeRange(r, style, lang)))].join(
      say(lang, ' или ', ' or '),
    )
  }
  if (typeof range === 'object') {
    if (Array.isArray(range.any)) return describeRange(range.any, style, lang)
    if (Array.isArray(range.all))
      return range.all.map((r) => describeRange(r, style, lang)).join(', ')
    return anyText
  }
  const text = String(range).trim()
  if (/^[[(]/.test(text)) return describeMaven(text, lang)
  if (style === 'maven') return say(lang, `рекомендуется ${text}`, `${text} recommended`)
  return text
    .split(/\s*\|\|\s*/)
    .map((part) =>
      part
        .split(/\s+/)
        .filter(Boolean)
        .map((p) => describePredicate(p, lang))
        .join(', '),
    )
    .join(say(lang, ' или ', ' or '))
}
