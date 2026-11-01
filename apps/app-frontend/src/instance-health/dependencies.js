// Dependency analysis for the Diagnostics page: which mod needs which, what is missing,
// disabled, duplicated, incompatible or made for another loader. Pure and unit-tested.
import { compareVersions, describeRange, isAnyVersion, versionSatisfies } from './versions.js'
import { describeWarning } from './messages.js'
import { KNOWN, LOADER_LABELS, PLATFORM } from './names.js'

const say = (lang, ru, en) => (lang === 'en' ? en : ru)
const quote = (lang, value) => (lang === 'en' ? `"${value}"` : `«${value}»`)
const list = (lang, values) => [...new Set(values)].map((v) => quote(lang, v)).join(', ')

export { PLATFORM, LOADER_LABELS }
const FABRIC_API_IDS = new Set(['fabric-api', 'fabric', 'quilted_fabric_api'])
const QSL_IDS = new Set(['qsl', 'quilted_fabric_api'])
/** Fabric API ships many modules (fabric-resource-loader-v0, ...) inside one JAR. */
export const isFabricApiModule = (id) =>
  /^fabric-/.test(id) && !/^fabric-(api$|loader|language-|permissions-api)/.test(id)
const isQslModule = (id) => /^quilt_/.test(id) && id !== 'quilt_loader'

export const fileName = (path) =>
  String(path ?? '')
    .split(/[\\/]/)
    .pop()
/** Display name: manifest name, or the file name without .jar/.disabled. */
export function modTitle(mod) {
  const name = String(mod?.name ?? '').trim()
  if (name && !/[\\/]/.test(name) && !/\.(jar|disabled)$/i.test(name)) return name
  return fileName(mod?.path || name).replace(/(\.jar)?(\.disabled)?$/i, '') || name
}
const capitalize = (word) =>
  word === 'api' ? 'API' : word ? word[0].toUpperCase() + word.slice(1) : word
/** "fabric-resource-loader-v0" -> "Fabric Resource Loader (v0)". */
export function prettyId(id) {
  const m = String(id).match(/^(.*?)[-_](v\d+)$/)
  const base = (m ? m[1] : String(id)).split(/[-_]/).filter(Boolean).map(capitalize).join(' ')
  return m ? `${base} (${m[2]})` : base
}
const isRelease = (version) => /^\d+\.\d+(\.\d+)?$/.test(String(version ?? ''))
const legacyNeoForge = (game) => isRelease(game) && compareVersions(game, '1.20.1') <= 0

function context(instance) {
  const loader = String(instance?.loader ?? '').toLowerCase() || null
  const game = instance?.game_version ? String(instance.game_version) : null
  let loaderVersion = instance?.loader_version ? String(instance.loader_version) : null
  if (loaderVersion && game && loaderVersion.startsWith(`${game}-`))
    loaderVersion = loaderVersion.slice(game.length + 1)
  if (['stable', 'latest'].includes(loaderVersion)) loaderVersion = null
  return { loader, game, loaderVersion }
}
/** Fabric/Quilt use predicates (">=1.2"), Forge/NeoForge use Maven ranges ("[1.2,)"). */
export function modStyle(mod) {
  const loaders = mod?.loaders || []
  if (loaders.some((l) => l === 'fabric' || l === 'quilt')) return 'fabric'
  if (loaders.some((l) => l === 'forge' || l === 'neoforge')) return 'maven'
  return (mod?.dependencies || []).some((d) => /^\s*[[(]/.test(String(d.version ?? '')))
    ? 'maven'
    : 'fabric'
}
/** Loaders a mod is made for, as instance loader ids. */
export function modTargets(mod, game) {
  const loaders = mod?.loaders || []
  const deps = new Set((mod?.dependencies || []).map((d) => d.id))
  const targets = new Set()
  if (loaders.includes('fabric')) targets.add('fabric').add('quilt')
  if (loaders.includes('quilt')) targets.add('quilt')
  if (loaders.includes('neoforge')) targets.add('neoforge')
  if (loaders.includes('forge')) {
    if (deps.has('neoforge')) targets.add('neoforge')
    else {
      targets.add('forge')
      // NeoForge for 1.20.1 still loads Forge mods.
      if (!deps.has('forge') || legacyNeoForge(game)) targets.add('neoforge')
    }
  }
  return targets
}
function familyLabel(mod) {
  const loaders = mod?.loaders || []
  const deps = new Set((mod?.dependencies || []).map((d) => d.id))
  const names = []
  if (loaders.includes('fabric')) names.push('Fabric')
  if (loaders.includes('quilt')) names.push('Quilt')
  if (loaders.includes('neoforge') || (loaders.includes('forge') && deps.has('neoforge')))
    names.push('NeoForge')
  else if (loaders.includes('forge')) names.push('Forge')
  return names.join(' / ')
}
const push = (map, key, value) => {
  if (!map.has(key)) map.set(key, [])
  map.get(key).push(value)
}
const SEVERITY = { error: 0, warning: 1, info: 2 }
// Within one severity the usual root causes come first.
const KIND_ORDER = [
  'no-loader',
  'missing',
  'disabled',
  'loader',
  'minecraft',
  'loader-version',
  'version',
  'duplicate',
  'conflict',
  'unrecognized',
  'risk',
  'partial',
]

/**
 * Full report for the Diagnostics page.
 * mods: native inventory; instance: profile with loader, game_version, loader_version.
 */
export function analyzeMods(mods, instance, lang = 'ru') {
  const all = Array.isArray(mods) ? mods : []
  const active = all.filter((m) => !m.disabled)
  const ctx = context(instance)
  const loaderLabel = LOADER_LABELS[ctx.loader] || say(lang, 'без загрузчика', 'no loader')
  const where = [loaderLabel, ctx.game].filter(Boolean).join(' ')
  const direct = new Map()
  const bundled = new Map()
  for (const m of all) {
    for (const id of m.ids || []) push(direct, id, m)
    for (const b of m.bundled || []) push(bundled, b.id, { mod: m, entry: b })
  }
  const fabricApi = active.find((m) => (m.ids || []).some((id) => FABRIC_API_IDS.has(id)))
  const qsl = active.find((m) => (m.ids || []).some((id) => QSL_IDS.has(id)))
  const connector = active.some((m) => (m.ids || []).includes('connector'))

  const nameOf = (id) => {
    const provider = (direct.get(id) || [])[0]
    if (provider) return modTitle(provider)
    const inner = (bundled.get(id) || []).find((b) => b.entry.name)
    if (inner) return inner.entry.name
    if (PLATFORM[id]) return PLATFORM[id]
    if (KNOWN[id]) return KNOWN[id]
    return isFabricApiModule(id) || isQslModule(id) ? prettyId(id) : id
  }
  const platform = (id, range, style) => {
    if (id === 'java') return { status: 'platform', installed: null }
    if (id === 'minecraft') {
      const ok = isRelease(ctx.game) ? versionSatisfies(range, ctx.game, style) : null
      return { status: ok === false ? 'version' : 'platform', installed: ctx.game }
    }
    if (!ctx.loader) return { status: 'platform', installed: null }
    const owners = {
      fabricloader: ['fabric', 'quilt'],
      quilt_loader: ['quilt'],
      forge: ['forge'],
      neoforge: ['neoforge'],
    }[id] || ['forge', 'neoforge']
    const fits =
      owners.includes(ctx.loader) ||
      (id === 'forge' && ctx.loader === 'neoforge' && legacyNeoForge(ctx.game)) ||
      // Sinytra Connector provides Fabric Loader on Forge and NeoForge.
      (id === 'fabricloader' && connector && ['forge', 'neoforge'].includes(ctx.loader))
    if (!fits) return { status: 'wrong-platform', installed: null }
    const own =
      (id === 'fabricloader' && ctx.loader === 'fabric') ||
      id === 'quilt_loader' ||
      (id === 'forge' && ctx.loader === 'forge') ||
      id === 'neoforge'
    const ok = own ? versionSatisfies(range, ctx.loaderVersion, style) : null
    return {
      status: ok === false ? 'version' : 'platform',
      installed: own ? ctx.loaderVersion : null,
    }
  }
  const resolve = (d, m) => {
    const id = String(d.id)
    const style = modStyle(m)
    const range = d.version ?? null
    const base = {
      id,
      required: !!d.required,
      range,
      rangeText: describeRange(range, style, lang),
      anyVersion: isAnyVersion(range),
      name: nameOf(id),
      kind: PLATFORM[id] ? 'platform' : 'mod',
      providers: [],
      installed: null,
    }
    if (PLATFORM[id]) return { ...base, ...platform(id, range, style) }
    if ((m.ids || []).includes(id)) return { ...base, status: 'self' }
    const directAll = (direct.get(id) || []).filter((p) => p !== m)
    const bundledAll = bundled.get(id) || []
    const providers = [
      ...directAll
        .filter((p) => !p.disabled)
        .map((p) => ({ mod: p, via: 'direct', version: p.version ?? null, name: modTitle(p) })),
      ...bundledAll
        .filter((b) => !b.mod.disabled)
        .map((b) => ({
          mod: b.mod,
          via: b.mod === m ? 'self' : 'bundled',
          version: b.entry.version ?? null,
          name: b.entry.name || base.name,
        })),
    ]
    const host = isFabricApiModule(id) ? fabricApi : isQslModule(id) ? qsl : null
    if (!providers.length && host && host !== m && !(host.bundled || []).length)
      providers.push({ mod: host, via: 'assumed', version: null, name: base.name })
    if (providers.length) {
      const fitting = providers.filter((p) => versionSatisfies(range, p.version, style) !== false)
      if (!fitting.length) {
        const best = providers[0]
        return { ...base, status: 'version', providers, installed: best.version, name: best.name }
      }
      const best = fitting.find((p) => p.via === 'direct') || fitting[0]
      return {
        ...base,
        status: best.via === 'direct' ? 'ok' : 'bundled',
        providers: fitting,
        installed: best.version,
        name: best.via === 'direct' ? best.name : base.name,
        host: best.via === 'direct' ? null : best.mod,
      }
    }
    const off = [
      ...directAll.filter((p) => p.disabled).map((p) => ({ mod: p, via: 'disabled' })),
      ...bundledAll.filter((b) => b.mod.disabled).map((b) => ({ mod: b.mod, via: 'disabled' })),
    ].map((p) => ({ ...p, version: p.mod.version ?? null, name: modTitle(p.mod) }))
    if (off.length) return { ...base, status: 'disabled', providers: off, name: off[0].name }
    return { ...base, status: d.required ? 'missing' : 'optional-missing' }
  }

  const entries = new Map()
  for (const m of all) {
    entries.set(m.path, {
      mod: m,
      path: m.path,
      title: modTitle(m),
      file: fileName(m.path),
      version: m.version ?? null,
      active: !m.disabled,
      family: familyLabel(m),
      required: [],
      optional: [],
      dependents: [],
      bundled: m.bundled || [],
      notes: (m.warnings || []).map((w) => describeWarning(w, lang)),
      problems: [],
      status: m.disabled ? 'disabled' : 'ok',
    })
  }
  for (const m of all) {
    const entry = entries.get(m.path)
    for (const d of m.dependencies || []) {
      const r = resolve(d, m)
      if (r.status === 'self') continue
      ;(r.required ? entry.required : entry.optional).push(r)
      if (m.disabled) continue
      for (const p of r.providers) {
        if (p.mod === m) continue
        const target = entries.get(p.mod.path)
        // A disabled provider is still listed: these mods wait for it to be enabled.
        if (target && !target.dependents.some((x) => x.path === m.path))
          target.dependents.push({
            path: m.path,
            title: entry.title,
            required: r.required,
            waiting: p.via === 'disabled',
          })
      }
    }
    const order = { missing: 0, 'wrong-platform': 1, version: 2, disabled: 3 }
    const sort = (a, b) =>
      (order[a.status] ?? 9) - (order[b.status] ?? 9) ||
      (a.kind === b.kind ? 0 : a.kind === 'platform' ? 1 : -1) ||
      a.name.localeCompare(b.name)
    entry.required.sort(sort)
    entry.optional.sort(sort)
  }

  const problems = []
  const add = (problem) => {
    problem.key = `${problem.kind}:${problem.mods.join('|')}:${problem.subject ?? ''}`
    if (!problems.some((p) => p.key === problem.key)) problems.push(problem)
  }
  if (ctx.loader === 'vanilla' && active.length)
    add({
      kind: 'no-loader',
      severity: 'warning',
      mods: [],
      title: say(lang, 'В профиле нет загрузчика модов', 'This profile has no mod loader'),
      text: say(
        lang,
        `Профиль запускает обычный Minecraft, поэтому файлы из папки mods (${active.length}) не загрузятся.`,
        `The profile runs plain Minecraft, so the files in mods (${active.length}) are not loaded.`,
      ),
      fix: say(
        lang,
        'Установи Fabric, Quilt, Forge или NeoForge: «Параметры» → «Изменить версии». Или создай профиль с загрузчиком.',
        'Install Fabric, Quilt, Forge or NeoForge in Options → Edit versions, or create a profile with a loader.',
      ),
    })
  const missing = new Map()
  const disabledNeeded = new Map()
  for (const m of active) {
    const entry = entries.get(m.path)
    let wrongLoader = false
    const targets = modTargets(m, ctx.game)
    if (ctx.loader && ctx.loader !== 'vanilla' && targets.size && !targets.has(ctx.loader)) {
      const viaConnector =
        connector && (ctx.loader === 'forge' || ctx.loader === 'neoforge') && targets.has('fabric')
      wrongLoader = !viaConnector
      add({
        kind: 'loader',
        severity: viaConnector ? 'info' : 'error',
        mods: [m.path],
        title: viaConnector
          ? say(
              lang,
              `${quote(lang, entry.title)} загрузится через Sinytra Connector`,
              `${quote(lang, entry.title)} loads through Sinytra Connector`,
            )
          : say(
              lang,
              `${quote(lang, entry.title)} сделан для ${entry.family}`,
              `${quote(lang, entry.title)} is made for ${entry.family}`,
            ),
        text: viaConnector
          ? say(
              lang,
              'Это мод для Fabric. Connector запускает многие такие моды, но не все.',
              'This is a Fabric mod. Connector runs many of them, but not all.',
            )
          : say(
              lang,
              `Профиль работает на ${loaderLabel}, поэтому мод не загрузится.`,
              `The profile uses ${loaderLabel}, so the mod will not load.`,
            ),
        fix: viaConnector
          ? say(
              lang,
              'Если игра вылетает, попробуй отключить этот мод.',
              'If the game crashes, try disabling this mod.',
            )
          : say(
              lang,
              `Скачай версию ${quote(lang, entry.title)} для ${where} или удали этот файл.`,
              `Download ${quote(lang, entry.title)} for ${where} or remove this file.`,
            ),
      })
    }
    for (const r of entry.required) {
      if (r.status === 'missing') {
        if (!missing.has(r.id)) missing.set(r.id, { name: r.name, ranges: new Set(), mods: [] })
        const item = missing.get(r.id)
        if (!r.anyVersion) item.ranges.add(r.rangeText)
        item.mods.push(m.path)
      } else if (r.status === 'disabled') {
        const provider = r.providers[0].mod
        if (!disabledNeeded.has(provider.path))
          disabledNeeded.set(provider.path, { provider, name: r.providers[0].name, mods: [] })
        disabledNeeded.get(provider.path).mods.push(m.path)
      } else if (r.status === 'wrong-platform' && !wrongLoader && ctx.loader !== 'vanilla') {
        wrongLoader = true
        add({
          kind: 'loader',
          severity: 'error',
          mods: [m.path],
          title: say(
            lang,
            `${quote(lang, entry.title)} требует ${r.name}`,
            `${quote(lang, entry.title)} requires ${r.name}`,
          ),
          text: say(
            lang,
            `Профиль работает на ${loaderLabel}, поэтому мод не загрузится.`,
            `The profile uses ${loaderLabel}, so the mod will not load.`,
          ),
          fix: say(
            lang,
            `Скачай версию ${quote(lang, entry.title)} для ${where} или удали этот файл.`,
            `Download ${quote(lang, entry.title)} for ${where} or remove this file.`,
          ),
        })
      } else if (r.status === 'version' && r.id === 'minecraft') {
        add({
          kind: 'minecraft',
          severity: 'error',
          mods: [m.path],
          title: say(
            lang,
            `${quote(lang, entry.title)} не для Minecraft ${ctx.game}`,
            `${quote(lang, entry.title)} is not made for Minecraft ${ctx.game}`,
          ),
          text: say(
            lang,
            `Эта версия мода поддерживает Minecraft: ${describeRange(r.range, modStyle(m), lang, true)}.`,
            `This mod version supports Minecraft: ${describeRange(r.range, modStyle(m), lang, true)}.`,
          ),
          fix: say(
            lang,
            `Скачай версию ${quote(lang, entry.title)} для ${where} или отключи мод.`,
            `Download ${quote(lang, entry.title)} for ${where} or disable the mod.`,
          ),
        })
      } else if (r.status === 'version' && r.kind === 'platform') {
        add({
          kind: 'loader-version',
          severity: 'error',
          mods: [m.path],
          subject: r.id,
          title: say(
            lang,
            `${quote(lang, entry.title)} требует другую версию ${r.name}`,
            `${quote(lang, entry.title)} needs a different ${r.name} version`,
          ),
          text: say(
            lang,
            `Нужна ${r.rangeText}, в профиле ${r.installed}.`,
            `Needs ${r.rangeText}, the profile has ${r.installed}.`,
          ),
          fix: say(
            lang,
            `Смени версию ${r.name}: «Параметры» → «Изменить версии». Или поставь другую версию мода.`,
            `Change the ${r.name} version in Options → Edit versions, or use another mod version.`,
          ),
        })
      } else if (r.status === 'version') {
        add({
          kind: 'version',
          severity: 'error',
          mods: [m.path, ...r.providers.map((p) => p.mod.path)],
          subject: r.id,
          title: say(
            lang,
            `${quote(lang, entry.title)} ждёт другую версию ${quote(lang, r.name)}`,
            `${quote(lang, entry.title)} needs another version of ${quote(lang, r.name)}`,
          ),
          text: say(
            lang,
            `Нужна ${r.rangeText}, установлена ${r.installed || 'версия без номера'}.`,
            `Needs ${r.rangeText}, installed ${r.installed || 'an unnumbered version'}.`,
          ),
          fix: say(
            lang,
            `Обнови ${quote(lang, r.name)} или поставь версию ${quote(lang, entry.title)}, которая работает с ${r.installed || 'установленной'}.`,
            `Update ${quote(lang, r.name)} or use a version of ${quote(lang, entry.title)} that works with ${r.installed || 'the installed one'}.`,
          ),
        })
      }
    }
  }
  const incomplete = active.some((m) => m.metadata_complete === false && (m.ids || []).length)
  for (const [id, item] of missing) {
    const names = item.mods.map((p) => entries.get(p).title)
    const ranges = [...item.ranges]
    add({
      kind: 'missing',
      severity: 'error',
      mods: item.mods,
      subject: id,
      title: say(
        lang,
        `Не установлен ${quote(lang, item.name)}`,
        `${quote(lang, item.name)} is not installed`,
      ),
      text:
        say(
          lang,
          `Его требуют: ${list(lang, names)}. Пока его нет, игра не запустится.`,
          `Required by: ${list(lang, names)}. The game will not start without it.`,
        ) +
        (ranges.length
          ? say(
              lang,
              ` Нужна версия: ${ranges.join('; ')}.`,
              ` Needed version: ${ranges.join('; ')}.`,
            )
          : '') +
        (incomplete
          ? say(
              lang,
              ' Если он встроен в мод, который проверен не полностью, загрузчик найдёт его сам.',
              ' If it is bundled in a partially checked mod, the loader will still find it.',
            )
          : ''),
      fix: say(
        lang,
        `Установи ${quote(lang, item.name)} для ${where}: «Содержимое» → «Добавить контент». Или удали моды, которым он нужен.`,
        `Install ${quote(lang, item.name)} for ${where} from Content → Add content, or remove the mods that need it.`,
      ),
      id,
    })
  }
  for (const [path, item] of disabledNeeded) {
    const names = item.mods.map((p) => entries.get(p).title)
    add({
      kind: 'disabled',
      severity: 'error',
      mods: [path, ...item.mods],
      title: say(
        lang,
        `${quote(lang, item.name)} отключён, но он нужен`,
        `${quote(lang, item.name)} is disabled but needed`,
      ),
      text: say(
        lang,
        `От него зависят: ${list(lang, names)}. Отключённый мод не загружается.`,
        `These mods depend on it: ${list(lang, names)}. A disabled mod is not loaded.`,
      ),
      fix: say(
        lang,
        `Включи ${quote(lang, item.name)} во вкладке «Содержимое».`,
        `Enable ${quote(lang, item.name)} on the Content tab.`,
      ),
    })
  }
  // Declared incompatibilities (Fabric breaks/conflicts, Forge incompatible/discouraged).
  for (const m of active) {
    const entry = entries.get(m.path)
    const style = modStyle(m)
    for (const c of m.conflicts || []) {
      const holders = [
        ...(direct.get(c.id) || [])
          .filter((p) => p !== m && !p.disabled)
          .map((p) => ({ mod: p, version: p.version })),
        ...(bundled.get(c.id) || [])
          .filter((b) => b.mod !== m && !b.mod.disabled)
          .map((b) => ({ mod: b.mod, version: b.entry.version })),
      ].filter((h) => versionSatisfies(c.version ?? null, h.version ?? null, style) !== false)
      for (const h of holders) {
        const other = entries.get(h.mod.path)
        const pair = [m.path, h.mod.path].sort()
        if (
          problems.some(
            (p) =>
              p.kind === 'conflict' &&
              p.mods.join('|') === pair.join('|') &&
              (p.severity === 'error' || !c.required),
          )
        )
          continue
        const index = problems.findIndex(
          (p) => p.kind === 'conflict' && p.mods.join('|') === pair.join('|'),
        )
        if (index >= 0) problems.splice(index, 1)
        const range = isAnyVersion(c.version) ? '' : ` (${describeRange(c.version, style, lang)})`
        add({
          kind: 'conflict',
          severity: c.required ? 'error' : 'warning',
          mods: pair,
          title: c.required
            ? say(
                lang,
                `${quote(lang, entry.title)} несовместим с ${quote(lang, other.title)}`,
                `${quote(lang, entry.title)} is incompatible with ${quote(lang, other.title)}`,
              )
            : say(
                lang,
                `${quote(lang, entry.title)} может конфликтовать с ${quote(lang, other.title)}`,
                `${quote(lang, entry.title)} may conflict with ${quote(lang, other.title)}`,
              ),
          text: c.required
            ? say(
                lang,
                `Автор ${quote(lang, entry.title)} указал, что мод не работает вместе с ${quote(lang, other.title)}${range}.`,
                `The author of ${quote(lang, entry.title)} says it does not work with ${quote(lang, other.title)}${range}.`,
              )
            : say(
                lang,
                `Автор ${quote(lang, entry.title)} не рекомендует ставить его вместе с ${quote(lang, other.title)}${range}.`,
                `The author of ${quote(lang, entry.title)} advises against using it with ${quote(lang, other.title)}${range}.`,
              ),
          fix: c.required
            ? say(
                lang,
                'Отключи один из них во вкладке «Содержимое».',
                'Disable one of them on the Content tab.',
              )
            : say(
                lang,
                'Если появятся вылеты или ошибки, отключи один из них.',
                'If you see crashes or errors, disable one of them.',
              ),
        })
      }
    }
  }
  // The same mod id in several active files.
  const seen = new Set()
  for (const [id, holders] of direct) {
    const files = holders.filter((m) => !m.disabled)
    if (files.length < 2) continue
    const key = files
      .map((m) => m.path)
      .sort()
      .join('|')
    if (seen.has(key)) continue
    seen.add(key)
    add({
      kind: 'duplicate',
      severity: 'error',
      mods: files.map((m) => m.path).sort(),
      subject: id,
      title: say(
        lang,
        `${quote(lang, nameOf(id))} установлен несколько раз`,
        `${quote(lang, nameOf(id))} is installed more than once`,
      ),
      text: say(
        lang,
        `Одинаковый мод в файлах: ${files.map((m) => fileName(m.path)).join(', ')}. С двумя копиями игра не запустится.`,
        `The same mod is in: ${files.map((m) => fileName(m.path)).join(', ')}. The game will not start with two copies.`,
      ),
      fix: say(
        lang,
        'Оставь один файл (обычно самой новой версии), остальные отключи или удали.',
        'Keep one file (usually the newest), disable or remove the others.',
      ),
    })
  }
  for (const m of active) {
    const entry = entries.get(m.path)
    const byKind = (kind) => entry.notes.filter((n) => n.kind === kind).map((n) => n.text)
    const unrecognized = byKind('unrecognized')
    if (unrecognized.length && !(m.ids || []).length)
      add({
        kind: 'unrecognized',
        severity: 'warning',
        mods: [m.path],
        title: say(
          lang,
          `${quote(lang, entry.file)} не распознан как мод`,
          `${quote(lang, entry.file)} was not recognized as a mod`,
        ),
        text: unrecognized.join(' '),
        fix: say(
          lang,
          'Если это не мод, убери файл из папки mods. Если мод, скачай его заново: файл может быть повреждён.',
          'If this is not a mod, move it out of mods. If it is, download it again: the file may be damaged.',
        ),
      })
    const risk = byKind('risk')
    if (risk.length)
      add({
        kind: 'risk',
        severity: 'warning',
        mods: [m.path],
        title: say(
          lang,
          `Подозрительные признаки в ${quote(lang, entry.title)}`,
          `Suspicious signs in ${quote(lang, entry.title)}`,
        ),
        text: risk.join(' '),
        fix: say(
          lang,
          'Это эвристика, а не вердикт. Проверь файл антивирусом в карточке мода и скачивай моды только с проверенных сайтов. Если сомневаешься, отключи мод.',
          'This is a heuristic, not a verdict. Scan the file from its card and download mods only from trusted sites. If unsure, disable the mod.',
        ),
      })
    const partial = byKind('partial')
    if (partial.length)
      add({
        kind: 'partial',
        severity: 'info',
        mods: [m.path],
        title: say(
          lang,
          `${quote(lang, entry.title)} проверен не полностью`,
          `${quote(lang, entry.title)} was checked partially`,
        ),
        text: partial.join(' '),
        fix: say(
          lang,
          'Делать ничего не нужно: окончательную проверку выполнит загрузчик при запуске.',
          'Nothing to do: the loader runs the final check at launch.',
        ),
      })
  }
  problems.sort(
    (a, b) =>
      SEVERITY[a.severity] - SEVERITY[b.severity] ||
      KIND_ORDER.indexOf(a.kind) - KIND_ORDER.indexOf(b.kind) ||
      a.title.localeCompare(b.title),
  )
  for (const p of problems)
    for (const path of p.mods) {
      const entry = entries.get(path)
      if (!entry) continue
      entry.problems.push(p)
      if (entry.active && (entry.status === 'ok' || SEVERITY[p.severity] < SEVERITY[entry.status]))
        entry.status = p.severity === 'info' && entry.status === 'ok' ? 'ok' : p.severity
    }
  const list_ = [...entries.values()].sort(
    (a, b) =>
      Number(b.active) - Number(a.active) ||
      a.title.localeCompare(b.title, undefined, { sensitivity: 'base' }),
  )
  const count = (severity) => problems.filter((p) => p.severity === severity).length
  return {
    mods: list_,
    byPath: entries,
    problems,
    stats: {
      total: all.length,
      active: active.length,
      disabled: all.length - active.length,
      libraries: list_.filter((e) => e.active && e.dependents.some((d) => d.required)).length,
      errors: count('error'),
      warnings: count('warning'),
      notes: count('info'),
    },
  }
}
