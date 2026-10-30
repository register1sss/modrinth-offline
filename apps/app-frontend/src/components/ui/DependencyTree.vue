<script setup>
import { ref } from 'vue'
const props = defineProps({
  mod: { type: Object, required: true },
  graph: { type: Array, required: true },
  visited: { type: Array, default: () => [] },
})
const expanded = ref({})
const providers = (id) => props.graph.filter((m) => !m.disabled && m.ids.includes(id))
const builtins = new Set([
  'minecraft',
  'java',
  'fabricloader',
  'quilt_loader',
  'forge',
  'neoforge',
  'fml',
])
</script>
<template>
  <ul class="dependency-tree">
    <li v-for="(d, index) in mod.dependencies" :key="index">
      <details @toggle="expanded[index] = $event.target.open">
        <summary>
          {{ d.id }} · {{ d.required ? 'обязательно' : 'необязательно' }} · {{ d.version }}
        </summary>
        <template v-if="expanded[index]">
          <p v-if="!providers(d.id).length">
            {{
              builtins.has(d.id)
                ? 'Компонент игры или загрузчика. Версию проверяет загрузчик.'
                : 'Активный поставщик не найден в распознанных манифестах. Возможны встроенные или нераспознанные зависимости.'
            }}
          </p>
          <div v-for="provider in providers(d.id)" :key="provider.path">
            <p>{{ provider.name }} · {{ provider.path }}</p>
            <p v-if="provider.path === mod.path || visited.includes(provider.path)">
              Циклическая ссылка
            </p>
            <DependencyTree
              v-else
              :mod="provider"
              :graph="graph"
              :visited="[...visited, mod.path]"
            />
          </div>
        </template>
      </details>
    </li>
  </ul>
</template>
<style scoped>
.dependency-tree {
  padding-left: 1.25rem;
  overflow-wrap: anywhere;
}
summary {
  cursor: pointer;
}
</style>
