<template>
  <div class="page-loading" :class="{ 'app-loading': appLoading }">
    <img class="initializing-icon" src="/u3drinth.svg" alt="U3drinth" />
  </div>
</template>

<script setup>
defineProps({
  appLoading: Boolean,
})
</script>

<style scoped lang="scss">
.page-loading {
  position: relative;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  height: 100%;
  width: 100%;

  &.app-loading {
    background-color: #16181c;
    height: 100vh;
  }
}

.initializing-icon {
  width: 12rem;
  height: 12rem;

  :deep(svg),
  svg {
    width: 12rem;
    height: 12rem;
    fill: var(--color-brand);
    color: var(--color-brand);
  }
}
</style>
