<script setup>
import { computed } from 'vue'
import ParticleCanvas from './ParticleCanvas.vue'
import { rgba } from '@/themes/catalog.js'
const props = defineProps({ theme: { type: Object, required: true }, large: Boolean })
const previewStyle = computed(() => {
  const p = props.theme.palette
  return {
    '--preview-bg': p.bg,
    '--preview-surface': p.surface,
    '--preview-raised': p.raised,
    '--preview-accent': p.accent,
    '--preview-secondary': p.secondary,
    '--preview-line': rgba(p.muted, 0.35),
    '--preview-tint': rgba(p.accent, 0.15),
    '--preview-glow': rgba(p.secondary, 0.22),
    '--preview-grid': rgba(p.accent, 0.08),
  }
})
</script>

<template>
  <div
    class="theme-preview"
    :class="{ large }"
    :style="previewStyle"
    :data-scene="theme.particle"
    :data-material="theme.material || (theme.palette.light ? 'paper' : 'glass')"
    aria-hidden="true"
  >
    <div class="preview-halo" />
    <div class="preview-landscape" />
    <ParticleCanvas :theme="theme" :animated="false" class="preview-particles" />
    <div class="mini-window">
      <div class="mini-sidebar"><i /><i /><i class="active" /><i /><i class="last" /></div>
      <div class="mini-content">
        <div class="mini-topbar"><span /><span /><b /></div>
        <div class="mini-title"><i /><i /></div>
        <div class="mini-tiles"><span /><span /><span /></div>
        <div class="mini-footer"><span /><b /></div>
      </div>
    </div>
    <span class="preview-orbit orbit-one" /><span class="preview-orbit orbit-two" />
  </div>
</template>

<style scoped>
.preview-landscape {
  position: absolute;
  inset: 0;
  background: radial-gradient(ellipse at 12% 80%, var(--preview-glow), transparent 65%),
    linear-gradient(130deg, transparent 25%, var(--preview-tint), transparent 75%);
}
[data-scene='rain'] .preview-landscape {
  background: repeating-linear-gradient(115deg, transparent 0 24px, var(--preview-grid) 25px 26px),
    linear-gradient(15deg, var(--preview-bg), var(--preview-glow));
}
[data-scene='crystal'] .preview-landscape {
  background: conic-gradient(
    from 55deg at 80% 80%,
    var(--preview-tint),
    transparent 25%,
    var(--preview-glow) 45%,
    transparent 60%,
    var(--preview-tint)
  );
}
[data-scene='bubble'] .preview-landscape,
[data-scene='plankton'] .preview-landscape {
  background: radial-gradient(ellipse at 15% -30%, var(--preview-glow), transparent 70%),
    radial-gradient(ellipse at 100% 100%, var(--preview-tint), transparent 65%);
}
[data-scene='sand'] .preview-landscape {
  background: repeating-radial-gradient(
    ellipse at 10% 180%,
    transparent 0 22px,
    var(--preview-grid) 23px 24px
  );
}
[data-scene='star'] .preview-landscape,
[data-scene='comet'] .preview-landscape {
  background: radial-gradient(ellipse at 25% 75%, var(--preview-glow), transparent 55%),
    linear-gradient(25deg, transparent 35%, var(--preview-tint), transparent 70%);
}
[data-scene='ember'] .preview-landscape {
  background: radial-gradient(ellipse at 50% 120%, var(--preview-glow), transparent 70%);
}
[data-material='velvet'] .mini-window {
  box-shadow:
    0 12px 28px #00000045,
    inset 0 1px 0 var(--preview-tint);
}
[data-material='paper'] .mini-window {
  transform: rotate(-1deg);
  box-shadow: 4px 5px 0 var(--preview-tint);
}
[data-material='obsidian'] .mini-window {
  box-shadow: none;
  border-radius: 3px;
}

.theme-preview {
  position: relative;
  height: 134px;
  overflow: hidden;
  background: var(--preview-bg);
  isolation: isolate;
}
.preview-halo {
  position: absolute;
  width: 220px;
  height: 220px;
  top: -115px;
  right: -30px;
  background: radial-gradient(circle, var(--preview-tint), transparent 70%);
}
.preview-particles {
  position: absolute;
  inset: 0;
  z-index: 2;
  opacity: 0.8;
}
.mini-window {
  position: absolute;
  display: flex;
  inset: 23px 34px -9px;
  border: 1px solid var(--preview-line);
  border-radius: 8px;
  background: var(--preview-surface);
  box-shadow: 0 8px 20px #00000020;
  transform: rotate(-3deg);
}
.mini-sidebar {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 9px;
  width: 29px;
  flex-shrink: 0;
  padding: 11px 0;
  border-right: 1px solid var(--preview-line);
}
.mini-sidebar i {
  height: 7px;
  width: 7px;
  border-radius: 2px;
  background: var(--preview-raised);
}
.mini-sidebar .active {
  background: var(--preview-accent);
  box-shadow: 0 0 0 4px var(--preview-tint);
}
.mini-sidebar .last {
  margin-top: auto;
}
.mini-content {
  padding: 12px;
  flex: 1;
  min-width: 0;
}
.mini-topbar {
  display: flex;
  align-items: center;
  gap: 3px;
}
.mini-topbar span {
  width: 3px;
  height: 3px;
  border-radius: 50%;
  background: var(--preview-line);
}
.mini-topbar b {
  width: 28%;
  height: 3px;
  margin-left: auto;
  background: var(--preview-line);
  border-radius: 2px;
}
.mini-title {
  margin-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 5px;
}
.mini-title i {
  width: 44%;
  height: 5px;
  background: var(--preview-accent);
  border-radius: 3px;
}
.mini-title i + i {
  width: 66%;
  height: 3px;
  background: var(--preview-line);
}
.mini-tiles {
  display: flex;
  gap: 5px;
  margin-top: 10px;
}
.mini-tiles span {
  height: 27px;
  flex: 1;
  border: 1px solid var(--preview-line);
  background: var(--preview-raised);
  border-radius: 3px;
}
.mini-tiles span:first-child {
  background: linear-gradient(140deg, var(--preview-tint), var(--preview-raised));
  border-color: var(--preview-accent);
}
.mini-footer {
  margin-top: 8px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.mini-footer span {
  width: 35%;
  height: 3px;
  background: var(--preview-line);
}
.mini-footer b {
  width: 20%;
  height: 9px;
  border-radius: 2px;
  background: var(--preview-accent);
}
.preview-orbit {
  position: absolute;
  border: 1px solid var(--preview-line);
  border-radius: 50%;
  pointer-events: none;
}
.orbit-one {
  width: 170px;
  height: 170px;
  right: -100px;
  top: 22px;
}
.orbit-two {
  width: 100px;
  height: 100px;
  left: -60px;
  bottom: -50px;
}
.large {
  height: 230px;
  border-radius: 10px;
  border: 1px solid var(--preview-line);
}
.large .mini-window {
  inset: 36px 62px -18px;
  transform: rotate(-4deg);
}
.large .mini-sidebar {
  width: 39px;
  gap: 14px;
  padding: 18px 0;
}
.large .mini-sidebar i {
  width: 10px;
  height: 10px;
}
.large .mini-content {
  padding: 20px;
}
.large .mini-title {
  margin-top: 22px;
  gap: 9px;
}
.large .mini-title i {
  height: 8px;
}
.large .mini-title i + i {
  height: 5px;
}
.large .mini-tiles {
  margin-top: 18px;
  gap: 10px;
}
.large .mini-tiles span {
  height: 58px;
  border-radius: 5px;
}
.large .mini-footer {
  margin-top: 15px;
}
@media (max-width: 600px) {
  .large {
    height: 180px;
  }
  .large .mini-window {
    inset: 26px 40px -24px;
  }
}
</style>
