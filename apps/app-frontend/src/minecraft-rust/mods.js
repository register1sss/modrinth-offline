// Native code consent is bound to both the selected download and every existing native bundle.
export function installRequest(entry, catalog, profile, consent) {
  if (!consent || !entry?.available || !catalog?.trust_token) return null
  return {
    project: entry.id,
    checksum: entry.checksum,
    profile: profile || 'horizon',
    trust_token: catalog.trust_token,
    trust_native_code: true,
  }
}
export function modInstalled(entry, catalog) {
  return catalog?.ready && catalog.files?.[`${entry.id}.hnmod`] === entry.checksum
}

const summaries = {
  u3_fixes:
    'Исправление зависающих трезубцев: возобновление движения от сервера и падение после удаления опорного блока. Не исправляет все баги из видео. Требует пересборки Horizon.',
  biomecompass: 'Компас для поиска биомов.',
  blockscope: 'Информация о блоке под прицелом.',
  carryblock: 'Перенос сундуков с предметами и небольших мобов.',
  classic_zoom: 'Приближение на V и прокрутку колеса мыши.',
  compass_bar: 'Компас и отметки мест смерти.',
  coords_hud: 'Координаты и направление на экране.',
  enchantmenthints: 'Краткие описания зачарований.',
  fps_overlay: 'Счётчик FPS и показатели плавности.',
  itemnew: 'Подсветка недавно подобранных предметов.',
  keystrokes: 'Нажатые клавиши, кнопки мыши и счётчик кликов.',
  lookat: 'Информация о блоке или мобе под прицелом.',
  minimap: 'Мини-карта мира.',
  mod_menu: 'Список установленных модов.',
  mousetricks: 'Перенос и сортировка предметов мышью.',
  recipes: 'Рецепты и способы применения предметов.',
  smartf3: 'Настраиваемый экран отладки F3.',
  treefall: 'Рубка всего дерева топором.',
  veinminer: 'Добыча связанных блоков руды и деревьев.',
  zoom: 'Приближение по клавише и колесу мыши.',
}
export function modSummary(entry, language) {
  return language === 'ru' ? summaries[entry.id] ?? entry.summary : entry.summary
}
