export function diagnose(text, language = 'ru') {
  const rules = [
    [
      /OutOfMemoryError|Could not reserve enough space/i,
      'Недостаточно памяти Java. Уменьши нагрузку или проверь выделение RAM; для 16 ГБ начни с 4 ГБ для Minecraft.',
      'Java ran out of memory. Reduce load or review RAM allocation; on a 16 GB PC start with 4 GB for Minecraft.',
    ],
    [
      /UnsupportedClassVersionError|class file version/i,
      'Версия Java не подходит этой версии Minecraft или моду. Проверь Java в настройках профиля.',
      'Java is incompatible with this Minecraft version or a mod. Check the profile Java setting.',
    ],
    [
      /ModResolutionException|Incompatible mods found|requires .*version|Missing.*dependency/i,
      'Вероятно, отсутствует зависимость или установлены несовместимые версии модов. Открой «Диагностику» профиля: там видно, какого мода не хватает.',
      'A dependency may be missing or mod versions may conflict. Open the profile Diagnostics to see which mod is missing.',
    ],
    [
      /MixinApplyError|Mixin.*failed|InjectionError/i,
      'Ошибка Mixin: возможен конфликт модов. Проверь упомянутые моды и их версии; отключай по одному в копии профиля.',
      'Mixin failure: mods may conflict. Check named mods and versions; disable one at a time in a copy of the profile.',
    ],
    [
      /EXCEPTION_ACCESS_VIOLATION|GLFW error|OpenGL.*error/i,
      'Возможна проблема драйвера, рендера или нативной библиотеки. Проверь драйвер NVIDIA и временно отключи шейдеры.',
      'A driver, renderer or native library may have failed. Check the NVIDIA driver and temporarily disable shaders.',
    ],
    [
      /Caused by:|\bFATAL\b|\bERROR\b|Crash report/i,
      'В логе найдены ошибки. Это не обязательно причина вылета: открой последние строки и отправь их локальному ИИ для разбора.',
      'Errors were found. They are not necessarily the cause of a crash: review the last lines and ask the local AI to analyze them.',
    ],
  ]
  return rules
    .filter(([pattern]) => pattern.test(text))
    .map(([, ru, en]) => (language === 'ru' ? ru : en))
}
