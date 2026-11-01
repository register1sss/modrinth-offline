// Readable Russian/English text for messages produced by the native diagnostics module.
import { knownName } from './names.js'
import { describeRange } from './versions.js'

const say = (lang, ru, en) => (lang === 'en' ? en : ru)

// kind: risk (worth a security review), unrecognized (not a usable mod), partial (scan limits).
const WARNINGS = [
  [
    /^Quilt dependency alternatives cannot be resolved reliably/,
    'partial',
    (lang) =>
      say(
        lang,
        'В описании есть альтернативные зависимости Quilt, их проверит загрузчик.',
        'Alternative Quilt dependencies are listed; the loader checks them.',
      ),
  ],
  [
    /^Nested JAR dependencies require loader validation/,
    'partial',
    (lang) =>
      say(
        lang,
        'Часть встроенных JAR не удалось прочитать, их зависимости проверит загрузчик.',
        'Some nested JARs could not be read; the loader checks their dependencies.',
      ),
  ],
  [
    /^File exceeds 256 MiB inspection limit/,
    'partial',
    (lang) =>
      say(lang, 'Файл больше 256 МиБ, проверка пропущена.', 'File is over 256 MiB, not checked.'),
  ],
  [
    /^File changed or grew past inspection limit/,
    'partial',
    (lang) =>
      say(
        lang,
        'Файл изменился во время проверки и проверен не полностью.',
        'File changed during the check and was not fully checked.',
      ),
  ],
  [
    /^Not a readable JAR\/ZIP/,
    'unrecognized',
    (lang) =>
      say(
        lang,
        'Это не JAR-архив или файл повреждён.',
        'This is not a JAR archive or the file is damaged.',
      ),
  ],
  [
    /^Archive has too many entries/,
    'partial',
    (lang) =>
      say(
        lang,
        'В архиве слишком много файлов, проверка пропущена.',
        'The archive has too many entries, not checked.',
      ),
  ],
  [
    /^No supported Fabric\/Quilt\/Forge\/NeoForge manifest/,
    'unrecognized',
    (lang) =>
      say(
        lang,
        'Внутри нет описания мода для Fabric, Quilt, Forge или NeoForge.',
        'There is no Fabric, Quilt, Forge or NeoForge mod description inside.',
      ),
  ],
  [
    /^Unreadable\/encrypted archive entry/,
    'unrecognized',
    (lang) =>
      say(
        lang,
        'Часть архива не читается или зашифрована.',
        'Part of the archive is unreadable or encrypted.',
      ),
  ],
  [
    /^Unsafe archive path: (.*)$/,
    'risk',
    (lang, m) =>
      say(lang, `Опасный путь внутри архива: ${m[1]}`, `Unsafe path inside the archive: ${m[1]}`),
  ],
  [
    /^Contains executable\/script: (.*?)(?: \(not proof of malware\))?$/,
    'risk',
    (lang, m) =>
      say(
        lang,
        `Содержит программу или скрипт: ${m[1]}. Само по себе это не доказывает вирус.`,
        `Contains a program or script: ${m[1]}. This alone does not prove malware.`,
      ),
  ],
  [
    /^Bytecode inspection limit reached/,
    'partial',
    (lang) =>
      say(
        lang,
        'Код мода проверен частично: достигнут лимит размера.',
        'Mod code was checked partially: size limit reached.',
      ),
  ],
  [
    /^Starts a command shell: (.*?)(?: \(review recommended\))?$/,
    'risk',
    (lang, m) =>
      say(
        lang,
        `Код может запускать командную строку: ${m[1]}.`,
        `Code may start a command shell: ${m[1]}.`,
      ),
  ],
  [
    /^References token storage: (.*?)(?: \(review recommended\))?$/,
    'risk',
    (lang, m) =>
      say(
        lang,
        `Код обращается к хранилищу токенов Discord: ${m[1]}.`,
        `Code references Discord token storage: ${m[1]}.`,
      ),
  ],
  [
    /^Additional findings omitted/,
    'partial',
    (lang) => say(lang, 'Остальные находки скрыты.', 'Further findings omitted.'),
  ],
  [
    /^Mod metadata exceeds/,
    'unrecognized',
    (lang) =>
      say(
        lang,
        'Описание мода слишком большое, его не удалось прочитать.',
        'The mod description is too large to read.',
      ),
  ],
]
const PARSE_ERROR = /line \d+|column \d+|expected|invalid|unexpected|TOML parse error|EOF/i

/** { kind, text } for one native warning string. */
export function describeWarning(warning, lang = 'ru') {
  const raw = String(warning ?? '')
  for (const [pattern, kind, text] of WARNINGS) {
    const m = raw.match(pattern)
    if (m) return { kind, text: text(lang, m) }
  }
  if (PARSE_ERROR.test(raw))
    return {
      kind: 'unrecognized',
      text: say(
        lang,
        `Описание мода повреждено (${raw}).`,
        `The mod description is damaged (${raw}).`,
      ),
    }
  return { kind: 'other', text: raw }
}

const ERRORS = [
  [
    /More than 1024 mod files/,
    (lang) =>
      say(
        lang,
        'В папке mods больше 1024 файлов, проверка остановлена.',
        'The mods folder has more than 1024 files; the check stopped.',
      ),
  ],
  [
    /^(Unsafe profile identifier|Unknown profile|Profile not found)/,
    (lang) =>
      say(
        lang,
        'Профиль не найден. Вернись в библиотеку и открой его заново.',
        'Profile not found. Go back to the library and open it again.',
      ),
  ],
  [
    /^(Profile links are not allowed|Links are not permitted|Profile escapes launcher directory|Path escapes profile|Unsafe relative path)/,
    (lang) =>
      say(
        lang,
        'Путь к файлам профиля ведёт за его папку или через ссылку, поэтому действие отменено.',
        'The profile path points outside its folder or through a link, so the action was cancelled.',
      ),
  ],
  [
    /^Stop the instance before changing files/,
    (lang) => say(lang, 'Сначала закрой игру.', 'Close the game first.'),
  ],
  [
    /^Another repair is already running/,
    (lang) => say(lang, 'Восстановление уже идёт.', 'A repair is already running.'),
  ],
  [
    /^Options file exceeds backup limit/,
    (lang) =>
      say(
        lang,
        'Файл options.txt слишком большой для резервной копии.',
        'options.txt is too large to back up.',
      ),
  ],
  [
    /^Repair failed; backup retained in ([^:]+)/,
    (lang, m) =>
      say(
        lang,
        `Восстановить файлы не удалось. Резервная копия сохранена в ${m[1]}.`,
        `Repair failed. The backup is kept in ${m[1]}.`,
      ),
  ],
  [
    /^Only mod files can be checked/,
    (lang) =>
      say(
        lang,
        'Проверять можно только файлы из папки mods.',
        'Only files in mods can be checked.',
      ),
  ],
  [
    /^Unsupported file or size/,
    (lang) =>
      say(
        lang,
        'Этот файл нельзя проверить: неподходящий тип или размер.',
        'This file cannot be checked: unsupported type or size.',
      ),
  ],
  [
    /^(ProgramFiles is unavailable|Microsoft Defender command-line scanner was not found)/,
    (lang) =>
      say(
        lang,
        'Microsoft Defender не найден на этом компьютере.',
        'Microsoft Defender was not found on this computer.',
      ),
  ],
  [
    /^Antivirus scan timed out/,
    (lang) =>
      say(
        lang,
        'Проверка антивирусом шла слишком долго и остановлена.',
        'The antivirus scan took too long and was stopped.',
      ),
  ],
]

/** Readable text for an error thrown by a diagnostics command. */
export function describeHealthError(error, lang = 'ru') {
  const raw = String(error?.message ?? error ?? '')
  for (const [pattern, text] of ERRORS) {
    const m = raw.match(pattern)
    if (m) return text(lang, m)
  }
  return raw || say(lang, 'Неизвестная ошибка.', 'Unknown error.')
}

/** Microsoft Defender result: { tone: ok|warn|muted, text }. */
export function describeScan(result, lang = 'ru') {
  if (!result) return null
  if (result.status === 'no_detection')
    return {
      tone: 'ok',
      text: say(
        lang,
        'Defender угроз не нашёл. Это не гарантия безопасности.',
        'Defender found no threats. This is not a safety guarantee.',
      ),
    }
  if (result.status === 'review')
    return {
      tone: 'warn',
      text: say(
        lang,
        'Defender сообщил об угрозе или ошибке проверки. Открой «Безопасность Windows» → «Журнал защиты» и отключи мод, пока не разберёшься.',
        'Defender reported a threat or a scan error. Open Windows Security → Protection history and disable the mod until you know more.',
      ),
    }
  return {
    tone: 'muted',
    text: say(
      lang,
      'Проверка антивирусом доступна только в Windows с Microsoft Defender.',
      'Antivirus scanning needs Windows with Microsoft Defender.',
    ),
  }
}

const CRASH = {
  files: [
    'Повреждены или не скачались файлы игры. Обычно помогает «Восстановить файлы».',
    'Game files are damaged or missing. "Repair files" usually helps.',
  ],
  java: [
    'Неподходящая версия Java. Выбери нужную Java в параметрах профиля: восстановление файлов это не исправит.',
    'Wrong Java version. Pick the right Java in profile options: repairing files will not fix it.',
  ],
  memory: [
    'Игре не хватило памяти. Увеличь память в параметрах профиля или убери тяжёлые моды: восстановление файлов это не исправит.',
    'The game ran out of memory. Give it more memory in profile options or remove heavy mods: repairing files will not fix it.',
  ],
  dependencies: [
    'Загрузчик нашёл проблему с зависимостями модов. Открой «Диагностику»: там видно, какого мода не хватает.',
    'The loader found a mod dependency problem. Open Diagnostics to see which mod is missing.',
  ],
  mixin: [
    'Один мод не смог изменить код игры (ошибка Mixin). Чаще всего это конфликт двух модов или неподходящая версия.',
    'A mod failed to patch the game (Mixin error). Usually two mods conflict or a version does not match.',
  ],
  crash: [
    'Игра завершилась с ошибкой. Точная причина есть в журнале и отчёте о сбое.',
    'The game stopped with an error. The exact cause is in the log and crash report.',
  ],
}
/** One plain-language sentence per crash finding kind. */
export function crashAdvice(kinds = [], lang = 'ru') {
  const list = kinds.length ? kinds : ['crash']
  return [...new Set(list)].map((kind) => say(lang, ...(CRASH[kind] || CRASH.crash)))
}

const name = (lang, value) => (lang === 'en' ? `"${value}"` : `«${value}»`)
/** Fabric version wording ("version 1.2 or later") in plain words. */
function fabricSpec(spec, lang) {
  const s = String(spec ?? '').trim()
  let m
  if (/^any version$/.test(s)) return say(lang, 'любая версия', 'any version')
  if ((m = s.match(/^version (.+?) or later$/)))
    return say(lang, `не ниже ${m[1]}`, `${m[1]} or later`)
  if ((m = s.match(/^version (.+?) or earlier$/)))
    return say(lang, `не выше ${m[1]}`, `${m[1]} or earlier`)
  if ((m = s.match(/^version after (.+)$/))) return say(lang, `выше ${m[1]}`, `after ${m[1]}`)
  if ((m = s.match(/^version before (.+)$/))) return say(lang, `ниже ${m[1]}`, `before ${m[1]}`)
  if ((m = s.match(/^version (\S+)$/))) return say(lang, `только ${m[1]}`, `only ${m[1]}`)
  return s.replace(/^version /, '')
}
/** "mod 'Sodium' (sodium)", "'Sodium' (sodium)" or a bare id -> display name. */
function fabricTarget(text) {
  const m = String(text).match(/^(?:mod )?'(.+?)' \((.+?)\)$/)
  if (m) return m[1]
  return knownName(text) || text
}
const LOADER_RELATION = new RegExp(
  "^- Mod '(.+?)' \\((.+?)\\) (\\S+) (requires|recommends|is incompatible with|conflicts with) " +
    '(.+?) of (.+?), (which is missing|but only the wrong version is present: (.+?)|' +
    "but a version is present that doesn't support the required features: (.+?)|" +
    'but a matching version is present: (.+?))!?$',
)
/** Translates Fabric/Forge resolution lines ("- Install fabric-api, any version."). */
export function describeLoaderLine(line, lang = 'ru') {
  const raw = String(line ?? '').trim()
  let m = raw.match(/^- Install (.+?), (.+?)\.?$/)
  if (m) {
    const target = name(lang, fabricTarget(m[1]))
    return m[2] === 'any version'
      ? say(lang, `Установи ${target}.`, `Install ${target}.`)
      : say(
          lang,
          `Установи ${target}, версия ${fabricSpec(m[2], lang)}.`,
          `Install ${target}, version ${fabricSpec(m[2], lang)}.`,
        )
  }
  m = raw.match(/^- Replace mod '(.+?)' \(.+?\) (\S+) with (.+?)\.?$/)
  if (m)
    return say(
      lang,
      `Замени ${name(lang, m[1])} ${m[2]} на версию ${fabricSpec(m[3], lang)}.`,
      `Replace ${name(lang, m[1])} ${m[2]} with version ${fabricSpec(m[3], lang)}.`,
    )
  m = raw.match(/^- Remove mod '(.+?)'/)
  if (m) return say(lang, `Удали ${name(lang, m[1])}.`, `Remove ${name(lang, m[1])}.`)
  m = raw.match(LOADER_RELATION)
  if (m) {
    const [, modName, , , relation, spec, targetText, tail] = m
    const present = m[8] || m[9] || m[10]
    const who = name(lang, modName)
    const target = name(lang, fabricTarget(targetText))
    const range = spec === 'any version' ? '' : ` (${fabricSpec(spec, lang)})`
    const missing = tail === 'which is missing'
    if (relation === 'requires' || relation === 'recommends') {
      const verb = say(
        lang,
        relation === 'requires' ? 'требует' : 'советует поставить',
        relation === 'requires' ? 'requires' : 'recommends',
      )
      if (missing)
        return say(
          lang,
          `${who} ${verb} ${target}${range}, но его нет.`,
          `${who} ${verb} ${target}${range}, which is missing.`,
        )
      return say(
        lang,
        `${who} ${verb} ${target}${range}, а установлена версия ${present}.`,
        `${who} ${verb} ${target}${range}, but version ${present} is installed.`,
      )
    }
    return relation === 'is incompatible with'
      ? say(
          lang,
          `${who} несовместим с ${target}${range}, а установлена версия ${present}.`,
          `${who} is incompatible with ${target}${range}, and version ${present} is installed.`,
        )
      : say(
          lang,
          `${who} может конфликтовать с ${target}${range}, установлена версия ${present}.`,
          `${who} may conflict with ${target}${range}; version ${present} is installed.`,
        )
  }
  m = raw.match(
    /^Mod ID: '(.+?)', Requested by: '(.+?)', Expected range: '(.+?)', Actual version: '(.+?)'/,
  )
  if (m) {
    const target = name(lang, knownName(m[1]) || m[1])
    const range = describeRange(m[3], 'maven', lang)
    return m[4] === '[MISSING]'
      ? say(
          lang,
          `${name(lang, m[2])} требует ${target} (${range}), но его нет.`,
          `${name(lang, m[2])} requires ${target} (${range}), which is missing.`,
        )
      : say(
          lang,
          `${name(lang, m[2])} требует ${target} (${range}), а установлена версия ${m[4]}.`,
          `${name(lang, m[2])} requires ${target} (${range}), but version ${m[4]} is installed.`,
        )
  }
  return raw.replace(/^- /, '')
}
