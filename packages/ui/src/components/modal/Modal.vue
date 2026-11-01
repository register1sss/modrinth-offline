<template>
  <Teleport to="body">
    <div v-if="shown">
      <div
        :class="{ shown: actuallyShown }"
        class="tauri-overlay"
        data-tauri-drag-region
        @click="() => (closable ? hide() : {})"
      />
      <div
        :class="{
          shown: actuallyShown,
          noblur: props.noblur,
        }"
        class="modal-overlay"
        @click="() => (closable ? hide() : {})"
      />
      <div class="modal-container" :class="{ shown: actuallyShown }">
        <div
          ref="dialog"
          class="modal-body"
          tabindex="-1"
          role="dialog"
          :aria-modal="actuallyShown ? 'true' : undefined"
          :aria-label="props.header || undefined"
        >
          <div v-if="props.header" class="header">
            <h1>{{ props.header }}</h1>
            <button
              v-if="closable"
              type="button"
              aria-label="Close dialog"
              class="btn icon-only transparent"
              @click="hide"
            >
              <XIcon />
            </button>
          </div>
          <div class="content">
            <slot />
          </div>
        </div>
      </div>
    </div>
    <div v-else></div>
  </Teleport>
</template>

<script setup>
import { XIcon } from '@modrinth/assets'
import { ref, nextTick, onBeforeUnmount } from 'vue'
import { registerDialog, unregisterDialog, isTopDialog } from './dialog-stack.js'

const props = defineProps({
  header: {
    type: String,
    default: null,
  },
  noblur: {
    type: Boolean,
    default: false,
  },
  closable: {
    type: Boolean,
    default: true,
  },
  onHide: {
    type: Function,
    default() {
      return () => {}
    },
  },
})

const shown = ref(false)
const actuallyShown = ref(false)
const dialog = ref(null)
const id = Symbol('dialog')
let hideTimer,
  previousFocus,
  generation = 0,
  registered = false
const focusable = () =>
  [
    ...(dialog.value?.querySelectorAll(
      'button:not([disabled]),a[href],input:not([disabled]),select:not([disabled]),textarea:not([disabled]),[tabindex]:not([tabindex="-1"])',
    ) ?? []),
  ].filter((el) => el.getClientRects().length && !el.closest('[inert]'))
function keydown(event) {
  if (!isTopDialog(id) || !actuallyShown.value) return
  if (event.key === 'Escape' && props.closable) {
    event.preventDefault()
    event.stopPropagation()
    hide()
  }
  if (event.key !== 'Tab') return
  const nodes = focusable()
  if (!nodes.length) {
    event.preventDefault()
    dialog.value?.focus()
    return
  }
  const index = nodes.indexOf(document.activeElement)
  if (
    index < 0 ||
    (event.shiftKey && index === 0) ||
    (!event.shiftKey && index === nodes.length - 1)
  ) {
    event.preventDefault()
    nodes[event.shiftKey ? nodes.length - 1 : 0]?.focus()
  }
}
function containFocus(event) {
  if (isTopDialog(id) && actuallyShown.value && !dialog.value?.contains(event.target))
    (focusable()[0] ?? dialog.value)?.focus()
}
function release() {
  if (!registered) return
  registered = false
  unregisterDialog(id)
  document.removeEventListener('keydown', keydown, true)
  document.removeEventListener('focusin', containFocus)
}

async function show() {
  if (shown.value && actuallyShown.value) return
  clearTimeout(hideTimer)
  const current = ++generation
  previousFocus = document.activeElement
  shown.value = true
  await nextTick()
  if (current !== generation || !shown.value) return
  registerDialog(id)
  registered = true
  document.addEventListener('keydown', keydown, true)
  document.addEventListener('focusin', containFocus)
  actuallyShown.value = true
  await nextTick()
  if (current === generation) (focusable()[0] ?? dialog.value)?.focus()
}

function hide() {
  if (!shown.value || (!actuallyShown.value && hideTimer)) return
  ++generation
  actuallyShown.value = false
  release()
  if (previousFocus?.isConnected) previousFocus.focus()
  props.onHide()
  clearTimeout(hideTimer)
  hideTimer = setTimeout(() => {
    shown.value = false
    hideTimer = null
  }, 300)
}
onBeforeUnmount(() => {
  ++generation
  clearTimeout(hideTimer)
  const restore = isTopDialog(id)
  release()
  if (restore && previousFocus?.isConnected) previousFocus.focus()
})

defineExpose({
  show,
  hide,
})
</script>

<style lang="scss" scoped>
.tauri-overlay {
  position: fixed;
  visibility: hidden;
  top: 0;
  left: 0;
  width: 100%;
  height: 100px;
  z-index: 20;

  &.shown {
    opacity: 1;
    visibility: visible;
  }
}

.modal-overlay {
  visibility: hidden;
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 19;
  transition: all 0.3s ease-in-out;

  @media (prefers-reduced-motion) {
    transition: none !important;
  }

  &.shown {
    opacity: 1;
    visibility: visible;
    background: hsla(0, 0%, 0%, 0.5);
    backdrop-filter: blur(3px);
  }

  &.noblur {
    backdrop-filter: none;
  }
}

.modal-container {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 21;
  visibility: hidden;
  pointer-events: none;

  &.shown {
    visibility: visible;
    .modal-body {
      opacity: 1;
      visibility: visible;
      transform: translateY(0) scale(1);
    }
  }

  .modal-body {
    position: relative;
    box-shadow: var(--shadow-raised), var(--shadow-inset);
    border-radius: var(--radius-lg);
    max-height: calc(100% - 2 * var(--gap-lg));
    overflow-y: auto;
    width: min(600px, calc(100vw - 2 * var(--gap-lg)));
    pointer-events: auto;

    .header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      background-color: var(--color-bg);
      padding: var(--gap-md) var(--gap-lg);

      h1 {
        font-weight: bold;
        font-size: 1.25rem;
      }
    }

    .content {
      background-color: var(--color-raised-bg);
    }

    transform: translateY(24px) scale(0.96);
    visibility: hidden;
    opacity: 0;
    transition:
      opacity 0.2s ease,
      transform 0.42s cubic-bezier(0.18, 0.85, 0.25, 1.12);

    @media (prefers-reduced-motion) {
      transition: none !important;
    }

    @media screen and (max-width: 650px) {
      width: calc(100% - 2 * var(--gap-lg));
    }
  }
}
:global(html[data-jelly='off']) .modal-body {
  transition: none;
}
</style>
