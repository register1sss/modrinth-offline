// Pure logic shared by the dependency dialog, diagnostics and regression tests.
export const normalizePath = (path) => String(path).replaceAll('\\', '/')
export function removalImpact(mods, paths) {
  const removed = new Set(paths.map(normalizePath))
  const active = mods.filter((m) => !m.disabled)
  const unavailable = new Set(
    active.filter((m) => removed.has(normalizePath(m.path))).map((m) => m.path),
  )
  const affected = []
  let changed = true
  while (changed) {
    changed = false
    for (const m of active) {
      if (unavailable.has(m.path)) continue
      const cause = m.dependencies.find((d) => {
        if (!d.required) return false
        const providers = active.filter((p) => p.ids.includes(d.id))
        return providers.length > 0 && providers.every((p) => unavailable.has(p.path))
      })
      if (cause) {
        unavailable.add(m.path)
        affected.push({ path: m.path, name: m.name, dependency: cause.id })
        changed = true
      }
    }
  }
  const optional = active
    .filter((m) => !unavailable.has(m.path))
    .flatMap((m) =>
      m.dependencies
        .filter((d) => {
          const providers = active.filter((p) => p.ids.includes(d.id))
          return (
            !d.required && providers.length > 0 && providers.every((p) => unavailable.has(p.path))
          )
        })
        .map((d) => ({ path: m.path, name: m.name, dependency: d.id })),
    )
  return {
    affected,
    optional,
    incomplete: mods.filter((m) => !m.metadata_complete).map((m) => m.path),
  }
}
export function compareSamples(before, after) {
  if (!before || !after) return null
  const a = new Map(before.mods.map((m) => [m.path, m.sha256]))
  const b = new Map(after.mods.map((m) => [m.path, m.sha256]))
  const changed = [...new Set([...a.keys(), ...b.keys()])].filter((p) => a.get(p) !== b.get(p))
  return {
    changed,
    memory_delta: after.memory_bytes - before.memory_bytes,
    cpu_delta: after.cpu_percent - before.cpu_percent,
    comparable: before.processes === 1 && after.processes === 1,
    attribution: 'correlation_only',
  }
}
export function repairAdvice(kinds = []) {
  return kinds.map(
    (kind) =>
      ({
        files: 'Обнаружена ошибка файлов. Можно попробовать восстановить файлы игры и загрузчика.',
        java: 'Проверь версию Java в настройках экземпляра. Восстановление файлов не исправляет неподходящую Java.',
        memory:
          'Проверь выделенную память и нагрузку. Восстановление файлов не исправляет нехватку RAM.',
        dependencies:
          'Проверь обязательные зависимости и совместимость версий модов во вкладке диагностики.',
        mixin:
          'Возможен конфликт модов. Проверь версии и журнал; проверяй отключение модов в копии экземпляра.',
        crash: 'В журнале есть признак вылета. Причина пока не определена.',
      })[kind] || 'Причина пока не определена.',
  )
}

export function shouldAnalyzeExit(event, sinceMs) {
  return (
    !!sinceMs &&
    event.event === 'finished' &&
    event.message !== 'Stopped by user' &&
    event.exit_code !== 0
  )
}
