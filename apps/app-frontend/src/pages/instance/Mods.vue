<template>
  <Card v-if="projects.length > 0" class="mod-card">
    <div class="dropdown-input">
      <DropdownSelect
        v-model="selectedProjectType"
        :options="Object.keys(selectableProjectTypes)"
        default-value="All"
        name="project-type-dropdown"
        color="primary"
        :display-name="$u3t"
      />
      <div class="iconified-input">
        <SearchIcon />
        <input
          v-model="searchFilter"
          type="text"
          :placeholder="searchPlaceholder"
          class="text-input"
          autocomplete="off"
        />
        <Button class="r-btn" @click="() => (searchFilter = '')">
          <XIcon />
        </Button>
      </div>
    </div>
    <Button
      v-if="canUpdatePack"
      :disabled="installing"
      color="secondary"
      @click="modpackVersionModal.show()"
    >
      <UpdatedIcon />
      {{ installing ? $u3t('Updating') : $u3t('Update modpack') }}
    </Button>
    <Button v-else-if="!isPackLocked" @click="exportModal.show()">
      <PackageIcon />
      {{ $u3t('Export modpack') }}
    </Button>
    <Button v-if="!isPackLocked && projects.some((m) => m.outdated)" @click="updateAll">
      <UpdatedIcon /> {{ $u3t('Update all') }}
    </Button>
    <AddContentButton v-if="!isPackLocked" :instance="instance" />
  </Card>
  <Pagination
    v-if="projects.length > 0"
    :page="currentPage"
    :count="Math.ceil(search.length / 20)"
    class="pagination-before"
    :link-function="(page) => `?page=${page}`"
    @switch-page="switchPage"
  />
  <Card v-if="projects.length > 0" class="list-card">
    <div class="table">
      <div class="table-row table-head" :class="{ 'show-options': selected.length > 0 }">
        <div class="table-cell table-text">
          <Checkbox v-model="selectAll" class="select-checkbox" />
        </div>
        <div v-if="selected.length === 0" class="table-cell table-text name-cell actions-cell">
          <Button class="transparent" @click="sortProjects('Name')">
            {{ $u3t('Name') }}
            <DropdownIcon v-if="sortColumn === 'Name'" :class="{ down: ascending }" />
          </Button>
        </div>
        <div v-if="selected.length === 0" class="table-cell table-text version">
          <Button class="transparent" @click="sortProjects('Version')">
            {{ $u3t('Version') }}
            <DropdownIcon v-if="sortColumn === 'Version'" :class="{ down: ascending }" />
          </Button>
        </div>
        <div v-if="selected.length === 0" class="table-cell table-text actions-cell">
          <Button class="transparent" @click="sortProjects('Enabled')">
            {{ $u3t('Actions') }}
            <DropdownIcon v-if="sortColumn === 'Enabled'" :class="{ down: ascending }" />
          </Button>
        </div>
        <div v-else class="options table-cell name-cell">
          <div>
            <Button
              class="transparent share"
              @click="() => (showingOptions = !showingOptions)"
              @mouseover="selectedOption = 'Share'"
            >
              <MenuIcon :class="{ open: showingOptions }" />
            </Button>
          </div>
          <Button
            class="transparent share"
            @click="shareNames()"
            @mouseover="selectedOption = 'Share'"
          >
            <ShareIcon /> {{ $u3t('Share') }}
          </Button>
          <div v-tooltip="isPackLocked ? $u3t('Unlock this instance to remove mods') : ''">
            <Button
              :disabled="isPackLocked"
              class="transparent trash"
              @click="deleteSelected"
              @mouseover="selectedOption = 'Delete'"
            >
              <TrashIcon /> {{ $u3t('Delete') }}
            </Button>
          </div>
          <div v-tooltip="isPackLocked ? $u3t('Unlock this instance to update mods') : ''">
            <Button
              :disabled="isPackLocked || offline"
              class="transparent update"
              @click="updateSelected()"
              @mouseover="selectedOption = 'Update'"
            >
              <UpdatedIcon /> {{ $u3t('Update') }}
            </Button>
          </div>
          <div v-tooltip="isPackLocked ? $u3t('Unlock this instance to toggle mods') : ''">
            <Button
              :disabled="isPackLocked"
              class="transparent"
              @click="toggleSelected()"
              @mouseover="selectedOption = 'Toggle'"
            >
              <ToggleIcon />
              {{ $u3t('Toggle') }}
            </Button>
          </div>
        </div>
      </div>
      <div v-if="showingOptions && selected.length > 0" class="more-box">
        <section v-if="selectedOption === 'Share'" class="options">
          <Button class="transparent" @click="shareNames()">
            <TextInputIcon />
            {{ $u3t('Share names') }}
          </Button>
          <Button class="transparent" @click="shareUrls()">
            <GlobeIcon />
            {{ $u3t('Share URLs') }}
          </Button>
          <Button class="transparent" @click="shareFileNames()">
            <FileIcon />
            {{ $u3t('Share file names') }}
          </Button>
          <Button class="transparent" @click="shareMarkdown()">
            <CodeIcon />
            {{ $u3t('Share as markdown') }}
          </Button>
        </section>
        <section v-if="selectedOption === 'Delete'" class="options">
          <Button class="transparent" @click="deleteSelected">
            <TrashIcon />
            {{ $u3t('Delete selected') }}
          </Button>
          <Button class="transparent" @click="deleteDisabled">
            <ToggleIcon />
            {{ $u3t('Delete disabled') }}
          </Button>
        </section>
        <section v-if="selectedOption === 'Update'" class="options">
          <Button class="transparent" :disabled="offline" @click="updateAll()">
            <UpdatedIcon /> {{ $u3t('Update all') }}
          </Button>
          <Button class="transparent" @click="selectUpdatable()">
            <CheckIcon />
            {{ $u3t('Select updatable') }}
          </Button>
        </section>
        <section v-if="selectedOption === 'Toggle'" class="options">
          <Button class="transparent" @click="enableAll()">
            <CheckIcon />
            {{ $u3t('Toggle on') }}
          </Button>
          <Button class="transparent" @click="disableAll()">
            <XIcon />
            {{ $u3t('Toggle off') }}
          </Button>
          <Button class="transparent" @click="hideShowAll()">
            <EyeIcon v-if="hideNonSelected" />
            <EyeOffIcon v-else />
            {{ hideNonSelected ? $u3t('Show unselected') : $u3t('Hide unselected') }}
          </Button>
        </section>
      </div>
      <div
        v-for="mod in search.slice((currentPage - 1) * 20, currentPage * 20)"
        :key="mod.file_name"
        class="table-row"
        @contextmenu.prevent.stop="(c) => handleRightClick(c, mod)"
      >
        <div class="table-cell table-text checkbox">
          <Checkbox
            :model-value="selectionMap.get(mod.path)"
            class="select-checkbox"
            @update:model-value="(newValue) => selectionMap.set(mod.path, newValue)"
          />
        </div>
        <div class="table-cell table-text name-cell">
          <router-link
            v-if="mod.slug"
            :to="{ path: `/project/${mod.slug}/`, query: { i: props.instance.path } }"
            :disabled="offline"
            class="mod-content"
          >
            <Avatar :src="mod.icon" />
            <div v-tooltip="mod.author ? `${mod.name} (${mod.author})` : mod.name" class="mod-text">
              <div class="title">{{ mod.name }}</div>
              <span class="no-wrap">{{ $u3t('by') }} {{ mod.author }}</span>
            </div>
          </router-link>
          <div v-else class="mod-content">
            <Avatar :src="mod.icon" />
            <div v-if="mod.manifest" v-tooltip="mod.file_name" class="mod-text">
              <div class="title">{{ mod.name }}</div>
              <span class="no-wrap file-line">{{ mod.file_name }}</span>
            </div>
            <span v-else v-tooltip="`${mod.name}`" class="title">{{ mod.name }}</span>
          </div>
        </div>
        <div class="table-cell table-text version">
          <span v-tooltip="mod.version || ''">{{ mod.version }}</span>
        </div>
        <div class="table-cell table-text manage">
          <div
            v-tooltip="
              isPackLocked ? $u3t('Unlock this instance to remove mods.') : $u3t('Remove project')
            "
          >
            <Button :disabled="isPackLocked" icon-only @click="removeMod(mod)">
              <TrashIcon />
            </Button>
          </div>
          <AnimatedLogo v-if="mod.updating" class="btn icon-only updating-indicator" />
          <div
            v-else
            v-tooltip="
              isPackLocked ? $u3t('Unlock this instance to update mods.') : $u3t('Update project')
            "
          >
            <Button
              :disabled="!mod.outdated || offline || isPackLocked"
              icon-only
              @click="updateProject(mod)"
            >
              <UpdatedIcon v-if="mod.outdated" />
              <CheckIcon v-else />
            </Button>
          </div>
          <div v-tooltip="isPackLocked ? $u3t('Unlock this instance to toggle mods.') : ''">
            <input
              id="switch-1"
              :disabled="isPackLocked"
              autocomplete="off"
              type="checkbox"
              class="switch stylized-toggle"
              :checked="!mod.disabled"
              @change="toggleDisableMod(mod)"
            />
          </div>
          <Button
            v-tooltip="`${$u3t('Show in folder')}: ${mod.file_name}`"
            icon-only
            @click="highlightModInProfile(instance.path, mod.path)"
          >
            <FolderOpenIcon />
          </Button>
        </div>
      </div>
    </div>
  </Card>
  <div v-else class="empty-prompt">
    <div class="empty-icon">
      <AddProjectImage />
    </div>
    <h3>{{ $u3t('No projects found') }}</h3>
    <p class="empty-subtitle">{{ $u3t('Add a project to get started') }}</p>
    <AddContentButton :instance="instance" />
  </div>
  <Pagination
    v-if="projects.length > 0"
    :page="currentPage"
    :count="Math.ceil(search.length / 20)"
    class="pagination-after"
    :link-function="(page) => `?page=${page}`"
    @switch-page="switchPage"
  />
  <Modal ref="removalDialog" :header="tr('Удаление файлов', 'Remove files')">
    <div class="universal-modal removal-dialog">
      <p>
        {{
          pendingRemoval.length > 1
            ? tr(
                `Будут удалены (${pendingRemoval.length}):`,
                `Will be removed (${pendingRemoval.length}):`,
              )
            : tr('Будет удалён:', 'Will be removed:')
        }}
        <strong>{{ pendingRemoval.map((m) => quoteName(removalTitle(m))).join(', ') }}</strong>
      </p>
      <template v-if="impact.affected.length">
        <h4 class="removal-danger">
          {{ tr('Эти моды перестанут запускаться:', 'These mods will stop working:') }}
        </h4>
        <ul>
          <li v-for="m in impact.affected" :key="m.path">
            {{
              tr(
                `${quoteName(m.name)}: нужен ${quoteName(m.dependencyName)}`,
                `${quoteName(m.name)}: needs ${quoteName(m.dependencyName)}`,
              )
            }}
          </li>
        </ul>
      </template>
      <p v-else-if="removalGraph.length" class="removal-ok">
        {{
          tr(
            'Другие установленные моды от этих файлов не зависят.',
            'No other installed mod depends on these files.',
          )
        }}
      </p>
      <template v-if="impact.optional.length">
        <h4>
          {{
            tr('Запустятся, но потеряют часть функций:', 'Will still work, but lose some features:')
          }}
        </h4>
        <ul>
          <li v-for="(m, i) in impact.optional" :key="i">
            {{
              tr(
                `${quoteName(m.name)}: работает вместе с ${quoteName(m.dependencyName)}`,
                `${quoteName(m.name)}: works together with ${quoteName(m.dependencyName)}`,
              )
            }}
          </li>
        </ul>
      </template>
      <details v-if="impact.incomplete.length" class="removal-unknown">
        <summary>
          {{
            tr(
              `У ${impact.incomplete.length} ${pluralFiles(impact.incomplete.length)} не удалось прочитать зависимости`,
              `Dependencies of ${impact.incomplete.length} ${impact.incomplete.length === 1 ? 'file' : 'files'} could not be read`,
            )
          }}
        </summary>
        <ul>
          <li v-for="path in impact.incomplete" :key="path">{{ path.split('/').pop() }}</li>
        </ul>
      </details>
      <p v-if="removalGraph.length" class="removal-note">
        {{
          tr(
            'Связи взяты из описаний внутри файлов модов. Часть зависимостей загрузчик проверяет только при запуске игры.',
            'Links come from the descriptions inside mod files. The loader checks some dependencies only when the game starts.',
          )
        }}
      </p>
      <template v-if="impact.affected.length">
        <Checkbox
          v-model="removeDependents"
          :label="tr('Удалить и эти моды тоже', 'Remove these mods too')"
        />
        <Checkbox
          v-model="removalAcknowledged"
          :label="
            tr(
              'Понимаю, что часть модов перестанет работать',
              'I understand that some mods will stop working',
            )
          "
        />
      </template>
      <p v-if="removalError" role="alert" class="removal-danger">{{ removalError }}</p>
      <div class="button-group">
        <Button :disabled="removalBusy" @click="removalDialog.hide()">{{
          tr('Отмена', 'Cancel')
        }}</Button>
        <Button
          color="danger"
          :disabled="removalBusy || (needsAcknowledge && !removalAcknowledged) || playing"
          @click="confirmRemoval"
          ><TrashIcon />
          {{ removalBusy ? tr('Удаление...', 'Removing...') : tr('Удалить', 'Remove') }}</Button
        >
      </div>
    </div>
  </Modal>
  <ShareModal
    ref="shareModal"
    :share-title="$u3t('Sharing modpack content')"
    :share-text="$u3t('Check out the projects I\'m using in my modpack!')"
    :open-in-new-tab="false"
  />
  <ExportModal v-if="projects.length > 0" ref="exportModal" :instance="instance" />
  <ModpackVersionModal
    v-if="instance.linked_data"
    ref="modpackVersionModal"
    :instance="instance"
    :versions="props.versions"
  />
</template>
<script setup>
import {
  TrashIcon,
  CheckIcon,
  SearchIcon,
  UpdatedIcon,
  FolderOpenIcon,
  XIcon,
  ShareIcon,
  DropdownIcon,
  GlobeIcon,
  FileIcon,
  EyeIcon,
  EyeOffIcon,
  CodeIcon,
} from '@modrinth/assets'
import {
  Pagination,
  DropdownSelect,
  ShareModal,
  Modal,
  Checkbox,
  AnimatedLogo,
  Avatar,
  Button,
  Card,
} from '@modrinth/ui'
import { formatProjectType } from '@modrinth/utils'
import { computed, onUnmounted, ref, watch } from 'vue'
import {
  add_project_from_path,
  get,
  get_projects,
  remove_project,
  toggle_disable_project,
  update_all,
  update_project,
} from '@/helpers/profile.js'
import { handleError } from '@/store/notifications.js'
import { mixpanel_track } from '@/helpers/mixpanel'
import { listen } from '@tauri-apps/api/event'
import { highlightModInProfile } from '@/helpers/utils.js'
import { MenuIcon, ToggleIcon, TextInputIcon, AddProjectImage, PackageIcon } from '@/assets/icons'
import ExportModal from '@/components/ui/ExportModal.vue'
import ModpackVersionModal from '@/components/ui/ModpackVersionModal.vue'
import AddContentButton from '@/components/ui/AddContentButton.vue'
import {
  get_organization_many,
  get_project_many,
  get_team_many,
  get_version_many,
} from '@/helpers/cache.js'
import { profile_listener } from '@/helpers/events.js'
import { health } from '@/instance-health/api.js'
import { removalImpact, normalizePath } from '@/instance-health/model.js'
import { modTitle } from '@/instance-health/dependencies.js'
import { language, tr } from '@/i18n/index.js'

const props = defineProps({
  playing: Boolean,
  instance: {
    type: Object,
    default() {
      return {}
    },
  },
  options: {
    type: Object,
    default() {
      return {}
    },
  },
  offline: {
    type: Boolean,
    default() {
      return false
    },
  },
  versions: {
    type: Array,
    required: true,
  },
})

const unlistenProfiles = await profile_listener(async (event) => {
  if (
    event.profile_path_id === props.instance.path &&
    event.event === 'synced' &&
    props.instance.install_stage !== 'pack_installing'
  ) {
    await initProjects()
  }
})

onUnmounted(() => {
  unlistenProfiles()
})

const showingOptions = ref(false)
const isPackLocked = computed(() => {
  return props.instance.linked_data && props.instance.linked_data.locked
})
const canUpdatePack = computed(() => {
  if (!props.instance.linked_data || !props.versions || !props.versions[0]) return false
  return props.instance.linked_data.version_id !== props.versions[0].id
})
const exportModal = ref(null)

const projects = ref([])
const selectionMap = ref(new Map())
// Names and versions from mod manifests, for files that are not Modrinth projects.
const manifestInfo = new Map()
const manifestKey = (path) => normalizePath(path).replace(/\.disabled$/i, '')
const needsManifest = (m) =>
  !m.slug &&
  normalizePath(m.path).startsWith('mods/') &&
  /\.jar(\.disabled)?$/i.test(m.path) &&
  !manifestInfo.has(manifestKey(m.path))
let manifestRequest = null
function loadManifestInfo() {
  if (manifestRequest) return manifestRequest
  manifestRequest = health('inventory', props.instance.path)
    .then((list) => {
      for (const m of list || []) {
        const raw = String(m.name ?? '').trim()
        const real = raw && !/[\\/]/.test(raw) && !/\.(jar|disabled|zip|txt)$/i.test(raw)
        manifestInfo.set(manifestKey(m.path), {
          name: real ? modTitle(m) : null,
          version: m.version || null,
        })
      }
      for (const project of projects.value) {
        const info = !project.slug && manifestInfo.get(manifestKey(project.path))
        if (!info) continue
        if (info.name) {
          project.name = info.name
          project.manifest = true
        }
        if (info.version && !project.version) project.version = info.version
      }
    })
    // Names are a convenience: on failure the list keeps file names.
    .catch(() => {})
    .finally(() => {
      manifestRequest = null
    })
  return manifestRequest
}

const initProjects = async () => {
  const newProjects = []

  const profileProjects = await get_projects(props.instance.path)
  const fetchProjects = []
  const fetchVersions = []

  for (const value of Object.values(profileProjects)) {
    if (value.metadata) {
      fetchProjects.push(value.metadata.project_id)
      fetchVersions.push(value.metadata.version_id)
    }
  }

  const [modrinthProjects, modrinthVersions] = await Promise.all([
    await get_project_many(fetchProjects).catch(handleError),
    await get_version_many(fetchVersions).catch(handleError),
  ])

  const [modrinthTeams, modrinthOrganizations] = await Promise.all([
    await get_team_many(modrinthProjects.map((x) => x.team)).catch(handleError),
    await get_organization_many(
      modrinthProjects.map((x) => x.organization).filter((x) => !!x),
    ).catch(handleError),
  ])

  for (const [path, file] of Object.entries(profileProjects)) {
    if (file.metadata) {
      const project = modrinthProjects.find((x) => file.metadata.project_id === x.id)
      const version = modrinthVersions.find((x) => file.metadata.version_id === x.id)
      const org = project.organization
        ? modrinthOrganizations.find((x) => x.id === project.organization)
        : null

      const team = modrinthTeams.find((x) => x[0].team_id === project.team)

      let owner = org ? org.name : team.find((x) => x.is_owner).user.username

      newProjects.push({
        path,
        name: project.title,
        slug: project.slug,
        author: owner,
        version: version.version_number,
        file_name: file.file_name,
        icon: project.icon_url,
        disabled: file.file_name.endsWith('.disabled'),
        updateVersion: file.update_version_id,
        outdated: !!file.update_version_id,
        project_type: project.project_type,
        id: project.id,
      })
    } else {
      const info = manifestInfo.get(manifestKey(path))
      newProjects.push({
        path,
        name: info?.name || file.file_name.replace('.disabled', ''),
        manifest: !!info?.name,
        author: '',
        version: info?.version || null,
        file_name: file.file_name,
        icon: null,
        disabled: file.file_name.endsWith('.disabled'),
        outdated: false,
        project_type: file.project_type,
      })
    }
  }

  projects.value = newProjects
  if (newProjects.some((m) => needsManifest(m))) loadManifestInfo()

  const newSelectionMap = new Map()
  for (const project of projects.value) {
    newSelectionMap.set(
      project.path,
      selectionMap.value.get(project.path) ??
        selectionMap.value.get(project.path.slice(0, -9)) ??
        selectionMap.value.get(project.path + '.disabled') ??
        false,
    )
  }
  selectionMap.value = newSelectionMap
}
await initProjects()

const modpackVersionModal = ref(null)
const installing = computed(() => props.instance.install_stage !== 'installed')

const searchFilter = ref('')
const selectAll = ref(false)
const selectedProjectType = ref('All')
const hideNonSelected = ref(false)
const selectedOption = ref('Share')
const shareModal = ref(null)
const ascending = ref(true)
const sortColumn = ref('Name')
const currentPage = ref(1)

watch(searchFilter, () => (currentPage.value = 1))

const selected = computed(() =>
  Array.from(selectionMap.value)
    .filter((args) => {
      return args[1]
    })
    .map((args) => {
      return projects.value.find((x) => x.path === args[0])
    }),
)

const functionValues = computed(() =>
  selected.value.length > 0 ? selected.value : Array.from(projects.value.values()),
)

const selectableProjectTypes = computed(() => {
  const obj = { All: 'all' }

  for (const project of projects.value) {
    obj[project.project_type ? formatProjectType(project.project_type) + 's' : 'Other'] =
      project.project_type
  }

  return obj
})

const search = computed(() => {
  const projectType = selectableProjectTypes.value[selectedProjectType.value]
  const filtered = projects.value
    .filter((mod) => {
      const needle = searchFilter.value.toLowerCase()
      return (
        (mod.name.toLowerCase().includes(needle) ||
          String(mod.file_name ?? '')
            .toLowerCase()
            .includes(needle)) &&
        (projectType === 'all' || mod.project_type === projectType)
      )
    })
    .filter((mod) => {
      if (hideNonSelected.value) {
        return !mod.disabled
      }
      return true
    })

  return updateSort(filtered)
})

const updateSort = (projects) => {
  switch (sortColumn.value) {
    case 'Version':
      return projects.slice().sort((a, b) => {
        if (a.version < b.version) {
          return ascending.value ? -1 : 1
        }
        if (a.version > b.version) {
          return ascending.value ? 1 : -1
        }
        return 0
      })
    case 'Author':
      return projects.slice().sort((a, b) => {
        if (a.author < b.author) {
          return ascending.value ? -1 : 1
        }
        if (a.author > b.author) {
          return ascending.value ? 1 : -1
        }
        return 0
      })
    case 'Enabled':
      return projects.slice().sort((a, b) => {
        if (a.disabled && !b.disabled) {
          return ascending.value ? 1 : -1
        }
        if (!a.disabled && b.disabled) {
          return ascending.value ? -1 : 1
        }
        return 0
      })
    default:
      return projects.slice().sort((a, b) => {
        const order = String(a.name).localeCompare(String(b.name), undefined, {
          sensitivity: 'base',
          numeric: true,
        })
        return ascending.value ? order : -order
      })
  }
}

const sortProjects = (filter) => {
  if (sortColumn.value === filter) {
    ascending.value = !ascending.value
  } else {
    sortColumn.value = filter
    ascending.value = true
  }
}

const updateAll = async () => {
  const setProjects = []
  for (const [i, project] of projects.value.entries()) {
    if (project.outdated) {
      project.updating = true
      setProjects.push(i)
    }
  }

  const paths = await update_all(props.instance.path).catch(handleError)

  for (const [oldVal, newVal] of Object.entries(paths)) {
    const index = projects.value.findIndex((x) => x.path === oldVal)
    projects.value[index].path = newVal
    projects.value[index].outdated = false

    if (projects.value[index].updateVersion) {
      projects.value[index].version = projects.value[index].updateVersion.version_number
      projects.value[index].updateVersion = null
    }
  }
  for (const project of setProjects) {
    projects.value[project].updating = false
  }

  mixpanel_track('InstanceUpdateAll', {
    loader: props.instance.loader,
    game_version: props.instance.game_version,
    count: setProjects.length,
    selected: selected.value.length > 1,
  })
}

const selectUpdatable = () => {
  for (const project of projects.value) {
    if (project.outdated) {
      selectionMap.value.set(project.path, true)
    }
  }
}

const updateProject = async (mod) => {
  mod.updating = true
  await new Promise((resolve) => setTimeout(resolve, 0))
  mod.path = await update_project(props.instance.path, mod.path).catch(handleError)
  mod.updating = false

  mod.outdated = false
  mod.version = mod.updateVersion.version_number
  mod.updateVersion = null

  mixpanel_track('InstanceProjectUpdate', {
    loader: props.instance.loader,
    game_version: props.instance.game_version,
    id: mod.id,
    name: mod.name,
    project_type: mod.project_type,
  })
}

let locks = {}

const toggleDisableMod = async (mod) => {
  // Use mod's id as the key for the lock. If mod doesn't have a unique id, replace `mod.id` with some unique property.
  if (!locks[mod.id]) {
    locks[mod.id] = ref(null)
  }

  let lock = locks[mod.id]

  while (lock.value) {
    await lock.value
  }

  lock.value = toggle_disable_project(props.instance.path, mod.path)
    .then((newPath) => {
      mod.path = newPath
      mod.disabled = !mod.disabled
      mixpanel_track('InstanceProjectDisable', {
        loader: props.instance.loader,
        game_version: props.instance.game_version,
        id: mod.id,
        name: mod.name,
        project_type: mod.project_type,
        disabled: mod.disabled,
      })
    })
    .catch(handleError)
    .finally(() => {
      lock.value = null
    })

  await lock.value
}

const removalDialog = ref(null)
const pendingRemoval = ref([])
const removalGraph = ref([])
const impact = ref({ affected: [], optional: [], incomplete: [] })
const removeDependents = ref(false)
const removalAcknowledged = ref(false)
const removalBusy = ref(false)
const removalError = ref('')
const needsAcknowledge = computed(() => impact.value.affected.length > 0)
const searchPlaceholder = computed(() => {
  const n = search.value.length
  if (language.value !== 'en') {
    const one = n % 10 === 1 && n % 100 !== 11
    return `Поиск среди ${n} ${one ? 'проекта' : 'проектов'}...`
  }
  const type = ['All', 'Other'].includes(selectedProjectType.value)
    ? 'projects'
    : selectedProjectType.value.toLowerCase()
  return `Search ${n} ${n === 1 ? type.replace(/s$/, '') : type}...`
})
const pluralFiles = (n) => (n % 10 === 1 && n % 100 !== 11 ? 'файла' : 'файлов')
/** Manifest name from the inventory when the file is not a Modrinth project. */
const removalTitle = (item) => {
  const entry = removalGraph.value.find((m) => normalizePath(m.path) === normalizePath(item.path))
  return entry && !item.slug ? modTitle(entry) : item.name
}
const quoteName = (value) => (language.value === 'en' ? `"${value}"` : `«${value}»`)
async function requestRemoval(items) {
  if (!items.length || removalBusy.value) return
  removalError.value = ''
  if (props.playing) {
    handleError(
      new Error(
        tr('Сначала закрой игру, потом удаляй файлы.', 'Close the game before removing files.'),
      ),
    )
    return
  }
  pendingRemoval.value = [...items]
  removeDependents.value = false
  removalAcknowledged.value = false
  removalBusy.value = true
  try {
    removalGraph.value = items.some((m) => normalizePath(m.path).startsWith('mods/'))
      ? await health('inventory', props.instance.path)
      : []
    impact.value = removalImpact(
      removalGraph.value,
      items.map((m) => m.path),
    )
    removalDialog.value.show()
  } catch (e) {
    handleError(e)
  } finally {
    removalBusy.value = false
  }
}
async function confirmRemoval() {
  if (removalBusy.value || (needsAcknowledge.value && !removalAcknowledged.value) || props.playing)
    return
  removalBusy.value = true
  removalError.value = ''
  try {
    if (removalGraph.value.length) {
      const fresh = await health('inventory', props.instance.path)
      if (JSON.stringify(fresh) !== JSON.stringify(removalGraph.value)) {
        removalGraph.value = fresh
        impact.value = removalImpact(
          fresh,
          pendingRemoval.value.map((m) => m.path),
        )
        removalAcknowledged.value = false
        throw new Error(
          tr(
            'Пока окно было открыто, файлы изменились. Проверь список ещё раз.',
            'Files changed while this window was open. Check the list again.',
          ),
        )
      }
    }
    const paths = new Set(pendingRemoval.value.map((m) => m.path))
    if (removeDependents.value) {
      for (const m of impact.value.affected) paths.add(m.path)
    }
    // Remove one at a time and refresh even after a partial failure. No hidden success.
    for (const path of paths) await remove_project(props.instance.path, path)
    removalDialog.value.hide()
  } catch (e) {
    removalError.value = String(e?.message || e)
  } finally {
    await initProjects().catch(handleError)
    removalBusy.value = false
  }
}
const removeMod = (mod) => requestRemoval([mod])
const deleteSelected = () => requestRemoval(functionValues.value)
const deleteDisabled = () => requestRemoval(projects.value.filter((x) => x.disabled))

const shareNames = async () => {
  await shareModal.value.show(functionValues.value.map((x) => x.name).join('\n'))
}

const shareFileNames = async () => {
  await shareModal.value.show(functionValues.value.map((x) => x.file_name).join('\n'))
}

const shareUrls = async () => {
  await shareModal.value.show(
    functionValues.value
      .filter((x) => x.slug)
      .map((x) => `https://modrinth.com/${x.project_type}/${x.slug}`)
      .join('\n'),
  )
}

const shareMarkdown = async () => {
  await shareModal.value.show(
    functionValues.value
      .map((x) => {
        if (x.slug) {
          return `[${x.name}](https://modrinth.com/${x.project_type}/${x.slug})`
        }
        return x.name
      })
      .join('\n'),
  )
}

const toggleSelected = async () => {
  for (const project of functionValues.value) {
    await toggleDisableMod(project, !project.disabled)
  }
}

const updateSelected = async () => {
  const promises = []
  for (const project of functionValues.value) {
    if (project.outdated) promises.push(updateProject(project))
  }
  await Promise.all(promises).catch(handleError)
}

const enableAll = async () => {
  for (const project of functionValues.value) {
    if (project.disabled) {
      await toggleDisableMod(project, false)
    }
  }
}

const disableAll = async () => {
  for (const project of functionValues.value) {
    if (!project.disabled) {
      await toggleDisableMod(project, false)
    }
  }
}

const hideShowAll = async () => {
  hideNonSelected.value = !hideNonSelected.value
}

const handleRightClick = (event, mod) => {
  if (mod.slug && mod.project_type) {
    props.options.showMenu(
      event,
      {
        link: `https://modrinth.com/${mod.project_type}/${mod.slug}`,
      },
      [{ name: 'open_link' }, { name: 'copy_link' }],
    )
  }
}

watch(selectAll, () => {
  for (const [key, value] of Array.from(selectionMap.value)) {
    if (value !== selectAll.value) {
      selectionMap.value.set(key, selectAll.value)
    }
  }
})

const unlisten = await listen('tauri://file-drop', async (event) => {
  for (const file of event.payload) {
    if (file.endsWith('.mrpack')) continue
    await add_project_from_path(props.instance.path, file).catch(handleError)
  }
  initProjects(await get(props.instance.path).catch(handleError))
})

const switchPage = (page) => {
  currentPage.value = page
}

onUnmounted(() => {
  unlisten()
})
</script>

<style scoped lang="scss">
.removal-dialog {
  display: grid;
  gap: 0.6rem;
  padding: var(--gap-lg);
  p,
  h4,
  ul {
    margin: 0;
  }
  ul {
    padding-left: 1.2rem;
  }
  .removal-danger {
    color: var(--color-red);
  }
  .removal-ok {
    color: var(--color-green);
  }
  .removal-note,
  .removal-unknown summary {
    color: var(--color-secondary);
    font-size: 0.9rem;
  }
  .removal-unknown summary {
    cursor: pointer;
  }
  .button-group {
    display: flex;
    gap: 0.6rem;
    justify-content: flex-end;
    margin-top: 0.4rem;
  }
}
.text-input {
  width: 100%;
}

.manage {
  display: flex;
  gap: 0.5rem;
}

.table {
  margin-block-start: 0;
  border-radius: var(--radius-lg);
  border: 2px solid var(--color-bg);
}

.table-row {
  grid-template-columns: min-content 2fr 1fr 13.25rem;

  &.show-options {
    grid-template-columns: min-content auto;

    .options {
      display: flex;
      flex-direction: row;
      align-items: center;
      gap: var(--gap-md);
    }
  }
}

.static {
  .table-row {
    grid-template-areas: 'manage name version';
    grid-template-columns: 4.25rem 1fr 1fr;
  }

  .name-cell {
    grid-area: name;
  }

  .version {
    grid-area: version;
  }

  .manage {
    justify-content: center;
    grid-area: manage;
  }
}

.table-cell {
  align-items: center;
}

.card-row {
  display: flex;
  align-items: center;
  gap: var(--gap-md);
  justify-content: space-between;
  background-color: var(--color-raised-bg);
}

.mod-card {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  gap: var(--gap-sm);
  justify-content: flex-start;
  margin-bottom: 0.5rem;
  white-space: nowrap;
  align-items: center;

  :deep(.dropdown-row) {
    .btn {
      height: 2.5rem !important;
    }
  }

  :deep(.btn) {
    height: 2.5rem;
  }

  .dropdown-input {
    flex-grow: 1;

    .animated-dropdown {
      width: unset;

      :deep(.selected) {
        border-radius: var(--radius-md) 0 0 var(--radius-md);
      }
    }

    .iconified-input {
      width: 100%;

      input {
        flex-basis: unset;
      }
    }

    :deep(.animated-dropdown) {
      .render-down {
        border-radius: var(--radius-md) 0 0 var(--radius-md) !important;
      }

      .options-wrapper {
        margin-top: 0.25rem;
        width: unset;
        border-radius: var(--radius-md);
      }

      .options {
        border-radius: var(--radius-md);
        border: 1px solid var(--color);
      }
    }
  }
}

.list-card {
  margin-top: 0.5rem;
}

.text-combo {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.name-cell {
  padding-left: 0;

  .btn {
    margin-left: var(--gap-sm);
    min-width: unset;
  }
}

.dropdown {
  width: 7rem !important;
}

.sort {
  padding-left: 0.5rem;
}

.second-row {
  display: flex;
  align-items: flex-start;
  flex-wrap: wrap;
  gap: var(--gap-sm);

  .chips {
    flex-grow: 1;
  }
}

.modal-body {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: var(--gap-lg);

  .button-group {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }

  strong {
    color: var(--color-contrast);
  }
}

.mod-content {
  display: flex;
  align-items: center;
  gap: 1rem;

  .mod-text {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .title {
    color: var(--color-contrast);
    font-weight: bolder;
  }

  .file-line {
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--color-secondary);
    font-size: 0.85rem;
  }
}

.actions-cell {
  display: flex;
  align-items: center;
  gap: 0.5rem;

  .btn {
    height: unset;
    width: unset;
    padding: 0;

    &.trash {
      color: var(--color-red);
    }

    &.update {
      color: var(--color-green);
    }

    &.share {
      color: var(--color-blue);
    }
  }
}

.more-box {
  display: flex;
  background-color: var(--color-bg);
  padding: var(--gap-lg);

  .options {
    display: flex;
    flex-wrap: wrap;
    flex-direction: row;
    gap: var(--gap-md);
    flex-grow: 1;
  }
}

.btn {
  &.transparent {
    height: unset;
    width: unset;
    padding: 0;
    color: var(--color-base);
    gap: var(--gap-xs);
    white-space: nowrap;

    svg {
      margin-right: 0 !important;
      transition: transform 0.2s ease-in-out;

      &.open {
        transform: rotate(90deg);
      }

      &.down {
        transform: rotate(180deg);
      }
    }
  }
}
.empty-prompt {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--gap-md);
  height: 100%;
  width: 100%;
  margin: auto;

  .empty-icon {
    svg {
      width: 10rem;
      height: 10rem;
      color: var(--color-contrast);
    }
  }

  p,
  h3 {
    margin: 0;
  }
}
</style>

<style lang="scss">
.updating-indicator {
  height: 2.25rem !important;
  width: 2.25rem !important;

  svg {
    height: 1.25rem !important;
    width: 1.25rem !important;
  }
}

.select-checkbox {
  button.checkbox {
    border: none;
    margin: 0;
  }
}

.dropdown-input {
  .selected {
    height: 2.5rem;
  }
}

.pagination-after {
  margin-bottom: 5rem;
}
</style>
