<script setup>
// What one mod needs (and, at the top level, who needs it), as a readable tree.
import { ref } from 'vue'
import { BoxIcon, CheckCircleIcon, XCircleIcon, IssuesIcon, InfoIcon } from '@modrinth/assets'
import { tr } from '@/i18n/index.js'
import { LOADER_LABELS } from '@/instance-health/dependencies.js'

const props = defineProps({
  entry: { type: Object, required: true },
  report: { type: Object, required: true },
  instance: { type: Object, default: null },
  visited: { type: Array, default: () => [] },
  depth: { type: Number, default: 0 },
})
const emit = defineEmits(['select'])
const open = ref({})
const nested = props.depth > 0
const BAD = new Set(['missing', 'disabled', 'version', 'wrong-platform'])
const tone = (d) => (BAD.has(d.status) ? 'bad' : d.status === 'optional-missing' ? 'muted' : 'good')
const loaderLabel = () =>
  LOADER_LABELS[String(props.instance?.loader ?? '').toLowerCase()] ||
  tr('без загрузчика', 'no loader')
function state(d) {
  const version = d.installed ? ` ${d.installed}` : ''
  switch (d.status) {
    case 'ok':
      return tr(`установлен${version}`, `installed${version}`)
    case 'bundled':
      return d.host?.path === props.entry.path
        ? tr(`встроен в этот мод${version}`, `bundled in this mod${version}`)
        : tr(
            `входит в «${title(d.host?.path)}»${version}`,
            `part of "${title(d.host?.path)}"${version}`,
          )
    case 'platform':
      if (d.id === 'java') return tr('проверяется при запуске', 'checked at launch')
      return d.installed
        ? tr(`в профиле ${d.installed}`, `profile has ${d.installed}`)
        : tr('есть в загрузчике', 'provided by the loader')
    case 'version':
      return tr(
        `установлена ${d.installed || 'без номера'}`,
        `installed ${d.installed || 'unnumbered'}`,
      )
    case 'disabled':
      return tr('отключён', 'disabled')
    case 'wrong-platform':
      return tr(`нет в профиле на ${loaderLabel()}`, `not available on ${loaderLabel()}`)
    default:
      return tr('не установлен', 'not installed')
  }
}
const title = (path) => props.report.byPath.get(path)?.title ?? path
// Only real top-level mods can be opened further; cycles and very deep chains stop.
function child(d) {
  const p = d.providers.find((x) => x.via === 'direct')
  return p && !BAD.has(d.status) ? props.report.byPath.get(p.mod.path) : null
}
const cyclic = (d) => {
  const c = child(d)
  return !!c && (c.path === props.entry.path || props.visited.includes(c.path))
}
const expandable = (d) => !!child(d) && !cyclic(d) && props.depth < 3 && child(d).required.length
</script>
<template>
  <div class="dep-tree" :class="{ nested }">
    <section class="dep-section">
      <h4 v-if="!nested">{{ tr('Требует', 'Requires') }}</h4>
      <ul v-if="entry.required.length" class="tree">
        <li v-for="d in entry.required" :key="d.id" :data-tone="tone(d)">
          <div class="node">
            <span class="node-icon" aria-hidden="true">
              <XCircleIcon v-if="tone(d) === 'bad'" />
              <BoxIcon v-else-if="d.kind === 'platform'" />
              <CheckCircleIcon v-else />
            </span>
            <span class="node-main">
              <span class="node-name">{{ d.name }}</span>
              <code v-if="d.name.toLowerCase() !== d.id.toLowerCase()" class="node-id">{{
                d.id
              }}</code>
            </span>
            <span class="node-meta">
              <span v-if="!d.anyVersion" class="node-range"
                >{{ tr('нужна', 'needs') }} {{ d.rangeText }}</span
              >
              <span class="node-state">{{ state(d) }}</span>
            </span>
            <button
              v-if="expandable(d)"
              class="node-toggle"
              :aria-expanded="!!open[d.id]"
              @click="open[d.id] = !open[d.id]"
            >
              {{ open[d.id] ? tr('Свернуть', 'Collapse') : tr('Что нужно ему', 'Its needs') }}
            </button>
            <span v-else-if="cyclic(d)" class="node-cycle">{{
              tr('взаимная зависимость', 'mutual dependency')
            }}</span>
          </div>
          <DependencyTree
            v-if="open[d.id] && expandable(d)"
            :entry="child(d)"
            :report="report"
            :instance="instance"
            :visited="[...visited, entry.path]"
            :depth="depth + 1"
            @select="emit('select', $event)"
          />
        </li>
      </ul>
      <p v-else class="muted">
        {{
          tr(
            'Отдельных зависимостей нет: нужен только сам Minecraft и загрузчик.',
            'No separate dependencies: only Minecraft and the loader.',
          )
        }}
      </p>
    </section>
    <section v-if="!nested && entry.optional.length" class="dep-section">
      <h4>{{ tr('Может работать вместе с', 'Works better with') }}</h4>
      <ul class="tree">
        <li v-for="d in entry.optional" :key="d.id" :data-tone="tone(d)">
          <div class="node">
            <span class="node-icon" aria-hidden="true">
              <InfoIcon v-if="d.status === 'optional-missing'" />
              <IssuesIcon v-else-if="tone(d) === 'bad'" />
              <CheckCircleIcon v-else />
            </span>
            <span class="node-main">
              <span class="node-name">{{ d.name }}</span>
              <code v-if="d.name.toLowerCase() !== d.id.toLowerCase()" class="node-id">{{
                d.id
              }}</code>
            </span>
            <span class="node-meta">
              <span class="node-state">{{
                d.status === 'optional-missing'
                  ? tr('не установлен, это необязательно', 'not installed, optional')
                  : state(d)
              }}</span>
            </span>
          </div>
        </li>
      </ul>
    </section>
    <section v-if="!nested" class="dep-section">
      <h4>{{ tr('Нужен для', 'Needed by') }}</h4>
      <p v-if="!entry.active && entry.dependents.some((d) => d.required)" class="waiting">
        {{
          tr(
            'Эти моды не запустятся, пока он отключён:',
            'These mods will not start while it is disabled:',
          )
        }}
      </p>
      <div v-if="entry.dependents.length" class="chips">
        <button
          v-for="d in entry.dependents"
          :key="d.path"
          class="chip"
          :class="{ soft: !d.required }"
          :title="
            d.required
              ? tr('Обязательная зависимость', 'Required')
              : tr('Необязательно', 'Optional')
          "
          @click="emit('select', d.path)"
        >
          {{ d.title }}
        </button>
      </div>
      <p v-else class="muted">
        {{
          entry.active
            ? tr(
                'Другие моды от него не зависят: его можно отключить, ничего не сломав.',
                'No other mod depends on it: disabling it breaks nothing else.',
              )
            : tr(
                'Мод отключён и сейчас ни на что не влияет.',
                'The mod is disabled and affects nothing.',
              )
        }}
      </p>
    </section>
    <details v-if="!nested && entry.bundled.length" class="dep-section bundled">
      <summary>
        {{
          tr(
            `Встроено внутри файла: ${entry.bundled.length}`,
            `Bundled inside: ${entry.bundled.length}`,
          )
        }}
      </summary>
      <ul class="bundled-list">
        <li v-for="b in entry.bundled" :key="b.id">
          <span>{{ b.name || b.id }}</span>
          <code>{{ b.id }}</code>
          <span v-if="b.version" class="muted">{{ b.version }}</span>
        </li>
      </ul>
    </details>
  </div>
</template>
<style scoped>
.dep-tree {
  display: grid;
  gap: 0.9rem;
  min-width: 0;
}
.dep-tree.nested {
  margin: 0.35rem 0 0.2rem 0.9rem;
  padding-left: 0.6rem;
  border-left: 1px dashed color-mix(in srgb, var(--color-secondary) 45%, transparent);
}
.dep-section h4 {
  margin: 0 0 0.45rem;
  font-size: 0.78rem;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--color-secondary);
}
.tree {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 0.3rem;
}
.tree > li {
  position: relative;
  padding-left: 1.1rem;
}
.tree > li::before {
  content: '';
  position: absolute;
  left: 0.3rem;
  top: -0.3rem;
  bottom: 50%;
  width: 0.6rem;
  border-left: 1px solid color-mix(in srgb, var(--color-secondary) 45%, transparent);
  border-bottom: 1px solid color-mix(in srgb, var(--color-secondary) 45%, transparent);
  border-bottom-left-radius: 0.35rem;
}
.tree > li:has(> .dep-tree)::before {
  bottom: auto;
  height: 1.25rem;
}
.node {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.35rem 0.6rem;
  padding: 0.4rem 0.6rem;
  border-radius: var(--radius-md);
  background: color-mix(in srgb, var(--color-button-bg) 55%, transparent);
  border: 1px solid transparent;
}
[data-tone='bad'] > .node {
  border-color: color-mix(in srgb, var(--color-red) 55%, transparent);
  background: color-mix(in srgb, var(--color-red) 12%, transparent);
}
[data-tone='muted'] > .node {
  opacity: 0.75;
}
.node-icon {
  display: inline-flex;
  width: 1.05rem;
  height: 1.05rem;
  flex: none;
}
.node-icon svg {
  width: 100%;
  height: 100%;
}
[data-tone='good'] .node-icon {
  color: var(--color-green);
}
[data-tone='bad'] .node-icon {
  color: var(--color-red);
}
[data-tone='muted'] .node-icon {
  color: var(--color-secondary);
}
.node-main {
  display: inline-flex;
  align-items: baseline;
  gap: 0.45rem;
  min-width: 0;
  flex: 1 1 12rem;
}
.node-name {
  font-weight: 600;
  color: var(--color-contrast);
  overflow-wrap: anywhere;
}
.node-id,
.bundled-list code {
  font-size: 0.75rem;
  color: var(--color-secondary);
  overflow-wrap: anywhere;
}
.node-meta {
  display: inline-flex;
  flex-wrap: wrap;
  gap: 0.25rem 0.6rem;
  font-size: 0.85rem;
}
.node-range {
  color: var(--color-secondary);
}
.node-state {
  color: var(--color-base);
}
[data-tone='bad'] .node-state {
  color: var(--color-red);
  font-weight: 600;
}
.node-toggle {
  border: 1px solid var(--color-divider);
  background: transparent;
  color: var(--color-brand);
  border-radius: 999px;
  padding: 0.1rem 0.6rem;
  font-size: 0.78rem;
  cursor: pointer;
}
.node-toggle:hover,
.node-toggle:focus-visible {
  background: var(--color-button-bg);
}
.node-cycle {
  font-size: 0.78rem;
  color: var(--color-secondary);
}
.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
}
.chip {
  border: 1px solid color-mix(in srgb, var(--color-brand) 45%, transparent);
  background: color-mix(in srgb, var(--color-brand) 12%, transparent);
  color: var(--color-contrast);
  border-radius: 999px;
  padding: 0.2rem 0.7rem;
  font-size: 0.85rem;
  cursor: pointer;
}
.chip.soft {
  border-style: dashed;
  background: transparent;
}
.chip:hover,
.chip:focus-visible {
  background: color-mix(in srgb, var(--color-brand) 24%, transparent);
}
.waiting {
  margin: 0 0 0.45rem;
  color: var(--color-red);
  font-size: 0.88rem;
}
.muted {
  margin: 0;
  color: var(--color-secondary);
  font-size: 0.88rem;
}
.bundled summary {
  cursor: pointer;
  color: var(--color-secondary);
  font-size: 0.85rem;
}
.bundled-list {
  margin: 0.5rem 0 0;
  padding-left: 1.1rem;
  display: grid;
  gap: 0.2rem;
  font-size: 0.85rem;
}
.bundled-list li {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  align-items: baseline;
}
</style>
