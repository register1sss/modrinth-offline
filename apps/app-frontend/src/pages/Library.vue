<script setup>
import { onUnmounted, ref, shallowRef } from 'vue'
import { tr } from '@/i18n/index.js'
import GridDisplay from '@/components/GridDisplay.vue'
import { list } from '@/helpers/profile.js'
import { useRoute } from 'vue-router'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { profile_listener } from '@/helpers/events.js'
import { Button } from '@modrinth/ui'
import { PlusIcon } from '@modrinth/assets'
import InstanceCreationModal from '@/components/ui/InstanceCreationModal.vue'
import { NewInstanceImage } from '@/assets/icons'

const route = useRoute()
const breadcrumbs = useBreadcrumbs()

breadcrumbs.setRootContext({ name: 'Library', link: route.path })

const instances = shallowRef([])
const error = ref(''),
  refreshing = ref(false)
let disposed = false,
  serial = 0
async function refresh() {
  const current = ++serial
  refreshing.value = true
  error.value = ''
  try {
    const result = await list()
    if (!disposed && current === serial)
      instances.value = Array.isArray(result) ? result : Object.values(result ?? {})
  } catch (e) {
    if (!disposed && current === serial) error.value = String(e?.message ?? e)
  } finally {
    if (!disposed && current === serial) refreshing.value = false
  }
}
await refresh()

const offline = ref(!navigator.onLine)
const syncOnline = () => {
  offline.value = !navigator.onLine
}
window.addEventListener('offline', syncOnline)
window.addEventListener('online', syncOnline)

const unlistenProfile = await profile_listener(refresh)
onUnmounted(() => {
  disposed = true
  ++serial
  unlistenProfile()
  window.removeEventListener('offline', syncOnline)
  window.removeEventListener('online', syncOnline)
})
</script>

<template>
  <section v-if="error" class="library-error" role="alert">
    <h2>{{ tr('Не удалось обновить библиотеку', 'Could not refresh the library') }}</h2>
    <p>
      {{
        tr(
          'Твои профили не удалены. Попробуй загрузить список ещё раз.',
          'Your profiles have not been deleted. Try loading the list again.',
        )
      }}
    </p>
    <details>
      <summary>{{ tr('Подробности', 'Details') }}</summary>
      {{ error }}
    </details>
    <Button :disabled="refreshing" @click="refresh">{{ tr('Повторить', 'Retry') }}</Button>
  </section>
  <GridDisplay v-if="instances.length > 0" label="Instances" :instances="instances" />
  <div v-else-if="!error" class="no-instance">
    <div class="icon">
      <NewInstanceImage />
    </div>
    <h3>{{ $u3t('No instances found') }}</h3>
    <Button color="primary" :disabled="offline" @click="$refs.installationModal.show()">
      <PlusIcon /> {{ $u3t('Create new instance') }}
    </Button>
    <InstanceCreationModal ref="installationModal" />
  </div>
</template>

<style lang="scss" scoped>
.library-error {
  margin: 24px;
  padding: 24px;
  border: 1px solid var(--color-red);
  border-radius: 14px;
  background: var(--color-raised-bg);
  overflow-wrap: anywhere;
  h2 {
    margin-top: 0;
    font-size: 22px;
  }
  details {
    margin-bottom: 16px;
  }
}

.no-instance {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: var(--gap-md);

  p,
  h3 {
    margin: 0;
  }

  .icon {
    svg {
      width: 10rem;
      height: 10rem;
    }
  }
}
</style>
