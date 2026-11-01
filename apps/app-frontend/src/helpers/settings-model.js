// Values are passed to Java directly, never to a shell.
export function parseLaunchArgs(text = '') {
  const args = []
  let word = '',
    quote = '',
    started = false
  for (let i = 0; i < text.length; i++) {
    const char = text[i]
    if (quote && char === '\\' && [quote, '\\'].includes(text[i + 1])) {
      word += text[++i]
      started = true
    } else if (char === quote) {
      quote = ''
    } else if (!quote && (char === '"' || char === "'")) {
      quote = char
      started = true
    } else if (!quote && /\s/.test(char)) {
      if (started) args.push(word)
      word = ''
      started = false
    } else {
      word += char
      started = true
    }
  }
  if (quote) throw new Error('Закрой кавычки в аргументах Java / Close the quoted Java argument.')
  if (started) args.push(word)
  return args
}

export function formatLaunchArgs(args = []) {
  return args
    .map((arg) => {
      if (!/[\s"']/.test(arg) && arg !== '') return arg
      return '"' + arg.replaceAll('\\', '\\\\').replaceAll('"', '\\"') + '"'
    })
    .join(' ')
}

export function parseEnvVars(text = '') {
  return parseLaunchArgs(text).map((entry) => {
    const equals = entry.indexOf('=')
    if (equals < 1 || !/^[A-Za-z_][A-Za-z0-9_]*$/.test(entry.slice(0, equals)))
      throw new Error(
        'Используй NAME=value для переменных / Use NAME=value for environment variables.',
      )
    return [entry.slice(0, equals), entry.slice(equals + 1)]
  })
}

export function nativeSettings(settings) {
  const { launchArgs, envVars, ...value } = JSON.parse(JSON.stringify(settings))
  value.extra_launch_args = parseLaunchArgs(launchArgs)
  value.custom_env_vars = parseEnvVars(envVars)
  value.hooks ??= {}
  for (const key of ['pre_launch', 'wrapper', 'post_exit']) value.hooks[key] ||= null
  value.custom_dir ||= null
  return value
}

export function settingsChanges(before, after) {
  return Object.fromEntries(
    Object.entries(after).filter(
      ([key, value]) => JSON.stringify(before[key]) !== JSON.stringify(value),
    ),
  )
}

// A failed write must not poison the queue. Read *inside* it to preserve other pages' settings.
export function createSettingsUpdater(read, write) {
  let tail = Promise.resolve()
  return (patch) => {
    const snapshot = JSON.parse(JSON.stringify(patch))
    const operation = tail
      .catch(() => {})
      .then(async () => {
        const current = await read()
        return write({ ...current, ...snapshot })
      })
    tail = operation
    return operation
  }
}
