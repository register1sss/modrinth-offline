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
[data-scene='aurora'] .preview-landscape {
  background: linear-gradient(172deg, transparent 18%, var(--preview-tint) 42%, transparent 68%),
    radial-gradient(ellipse at 30% -10%, var(--preview-glow), transparent 60%);
}
[data-scene='rain'] .preview-landscape {
  background: repeating-linear-gradient(115deg, transparent 0 24px, var(--preview-grid) 25px 26px),
    linear-gradient(15deg, var(--preview-bg), var(--preview-glow));
}
[data-scene='ember'] .preview-landscape {
  background: radial-gradient(ellipse at 50% 120%, var(--preview-glow), transparent 70%),
    radial-gradient(ellipse at 90% 110%, var(--preview-tint), transparent 50%);
}
[data-scene='petal'] .preview-landscape {
  background: radial-gradient(circle at 86% 12%, var(--preview-glow), transparent 45%),
    radial-gradient(ellipse at 8% 100%, var(--preview-tint), transparent 60%);
}
[data-scene='bubble'] .preview-landscape {
  background: radial-gradient(ellipse at 15% -30%, var(--preview-glow), transparent 70%),
    radial-gradient(ellipse at 100% 100%, var(--preview-tint), transparent 65%);
}
[data-scene='orbit'] .preview-landscape {
  background: repeating-radial-gradient(
      ellipse at 50% 55%,
      transparent 0 28px,
      var(--preview-grid) 29px 30px
    ),
    radial-gradient(circle at 50% 55%, var(--preview-glow), transparent 40%);
}
[data-scene='firefly'] .preview-landscape {
  background: radial-gradient(ellipse at 50% 125%, var(--preview-tint), transparent 65%),
    radial-gradient(circle at 82% 28%, var(--preview-glow), transparent 38%);
}
[data-scene='snow'] .preview-landscape {
  background: linear-gradient(180deg, transparent 58%, var(--preview-tint)),
    radial-gradient(ellipse at 50% -20%, var(--preview-glow), transparent 60%);
}
[data-scene='sand'] .preview-landscape {
  background: repeating-radial-gradient(
    ellipse at 10% 180%,
    transparent 0 22px,
    var(--preview-grid) 23px 24px
  );
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
[data-scene='corona'] .preview-landscape {
  background: radial-gradient(
      circle at 78% 30%,
      transparent 0 17px,
      var(--preview-glow) 19px,
      transparent 48px
    ),
    radial-gradient(ellipse at 10% 100%, var(--preview-tint), transparent 55%);
}
[data-scene='goldleaf'] .preview-landscape {
  background: repeating-linear-gradient(90deg, transparent 0 38px, var(--preview-grid) 39px 40px),
    radial-gradient(ellipse at 50% 120%, var(--preview-glow), transparent 60%);
}
[data-scene='plankton'] .preview-landscape {
  background: radial-gradient(ellipse at 50% 115%, var(--preview-glow), transparent 60%),
    repeating-linear-gradient(172deg, transparent 0 18px, var(--preview-grid) 19px 20px);
}
[data-scene='geometry'] .preview-landscape {
  background:
    linear-gradient(var(--preview-grid) 1px, transparent 1px) 0 0 / 16px 16px,
    linear-gradient(90deg, var(--preview-grid) 1px, transparent 1px) 0 0 / 16px 16px;
}
[data-scene='heart'] .preview-landscape {
  background: radial-gradient(circle at 18% 82%, var(--preview-glow), transparent 45%),
    radial-gradient(circle at 86% 18%, var(--preview-tint), transparent 45%);
}
[data-scene='wave'] .preview-landscape {
  background: radial-gradient(circle at 50% 64%, var(--preview-glow) 0 22px, transparent 23px),
    repeating-linear-gradient(180deg, transparent 0 9px, var(--preview-grid) 10px 11px);
}
[data-scene='tea'] .preview-landscape {
  background: repeating-linear-gradient(90deg, transparent 0 46px, var(--preview-grid) 47px 50px),
    radial-gradient(ellipse at 80% 0%, var(--preview-glow), transparent 55%);
}
[data-scene='berry'] .preview-landscape {
  background: radial-gradient(circle at 14% 24%, var(--preview-glow), transparent 40%),
    radial-gradient(circle at 90% 90%, var(--preview-tint), transparent 50%);
}
[data-scene='cloud'] .preview-landscape {
  background: radial-gradient(ellipse at 18% 100%, #ffffff99, transparent 55%),
    radial-gradient(ellipse at 82% 0%, var(--preview-glow), transparent 55%);
}
[data-scene='coffee'] .preview-landscape {
  background: radial-gradient(ellipse at 85% 110%, var(--preview-tint), transparent 55%),
    repeating-linear-gradient(115deg, transparent 0 30px, var(--preview-grid) 31px 32px);
}
[data-scene='lavender'] .preview-landscape {
  background: linear-gradient(0deg, var(--preview-tint), transparent 45%),
    radial-gradient(ellipse at 80% -10%, var(--preview-glow), transparent 55%);
}
[data-scene='ginkgo'] .preview-landscape {
  background: radial-gradient(circle at 86% 20%, var(--preview-glow), transparent 45%),
    linear-gradient(160deg, transparent 50%, var(--preview-tint));
}
[data-scene='comet'] .preview-landscape {
  background: radial-gradient(ellipse at 25% 75%, var(--preview-glow), transparent 55%),
    linear-gradient(25deg, transparent 35%, var(--preview-tint), transparent 70%);
}
[data-scene='bokeh'] .preview-landscape {
  background: radial-gradient(circle at 20% 30%, var(--preview-glow) 0 14px, transparent 15px),
    radial-gradient(circle at 76% 70%, var(--preview-tint) 0 22px, transparent 23px),
    radial-gradient(circle at 56% 20%, var(--preview-tint) 0 9px, transparent 10px);
}
[data-scene='honeycomb'] .preview-landscape {
  background:
    radial-gradient(circle, var(--preview-grid) 0 32%, transparent 34%) 0 0 / 18px 16px,
    radial-gradient(ellipse at 85% 0%, var(--preview-glow), transparent 55%);
}
[data-scene='diamond'] .preview-landscape {
  background: radial-gradient(ellipse at 50% -20%, var(--preview-glow), transparent 60%),
    linear-gradient(180deg, var(--preview-tint), transparent 30%);
}
[data-scene='code'] .preview-landscape {
  background: repeating-linear-gradient(180deg, transparent 0 2px, var(--preview-grid) 3px 3px),
    radial-gradient(ellipse at 50% 50%, var(--preview-tint), transparent 70%);
}
[data-scene='moon'] .preview-landscape {
  background: radial-gradient(circle at 82% 26%, var(--preview-glow) 0 16px, transparent 34px),
    linear-gradient(200deg, transparent 50%, var(--preview-tint));
}
[data-scene='fish'] .preview-landscape {
  background: linear-gradient(180deg, var(--preview-tint), transparent 40%),
    radial-gradient(ellipse at 50% 125%, var(--preview-glow), transparent 55%);
}
[data-scene='plasma'] .preview-landscape {
  background: radial-gradient(circle at 70% 40%, var(--preview-glow), transparent 35%),
    radial-gradient(circle at 20% 80%, var(--preview-tint), transparent 40%);
}
[data-scene='gear'] .preview-landscape {
  background: repeating-linear-gradient(90deg, transparent 0 3px, var(--preview-grid) 4px 4px),
    radial-gradient(ellipse at 15% 100%, var(--preview-glow), transparent 55%);
}
[data-scene='lantern'] .preview-landscape {
  background: linear-gradient(0deg, var(--preview-glow), transparent 55%),
    radial-gradient(ellipse at 80% 100%, var(--preview-tint), transparent 50%);
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
