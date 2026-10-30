<script setup>
import { useTheming } from '@/store/theme.js'
import { tr, translate } from '@/i18n/index.js'
import { LibraryIcon, SearchIcon, RightArrowIcon } from '@modrinth/assets'
defineProps({ profiles: { type: Number, default: 0 } })
const theme = useTheming()
</script>
<template>
  <section class="studio-welcome" aria-labelledby="studio-welcome-title">
    <div class="welcome-copy">
      <p class="welcome-kicker"><span /> U3DRINTH <span class="edition">STUDIO</span></p>
      <h1 id="studio-welcome-title">
        {{ tr('Твоя игра.', 'Your game.') }}<br /><span>{{
          tr('Твоё пространство.', 'Your space.')
        }}</span>
      </h1>
      <p class="welcome-description">
        {{
          tr(
            'Любимые сборки, новые миры и немного твоего стиля.',
            'Favorite packs, new worlds, and a little of your own style.',
          )
        }}
      </p>
      <div class="welcome-actions">
        <RouterLink to="/library" class="studio-action main-action"
          ><LibraryIcon />{{ tr('Моя библиотека', 'My library') }}<RightArrowIcon
        /></RouterLink>
        <RouterLink to="/browse/modpack" class="studio-action secondary-action"
          ><SearchIcon />{{ tr('Найти сборку', 'Discover packs') }}</RouterLink
        >
      </div>
    </div>
    <div class="welcome-art" aria-hidden="true">
      <div class="art-grid" />
      <div class="art-orbit first" />
      <div class="art-orbit second" />
      <svg class="welcome-cube" viewBox="0 0 240 240" fill="none">
        <path
          d="M120 32 202 78v88l-82 46-82-46V78Z"
          fill="var(--color-raised-bg)"
          stroke="var(--color-brand)"
          stroke-width="2"
        />
        <path d="m38 78 82 47 82-47M120 125v87" stroke="var(--color-brand)" stroke-width="2" />
        <path d="m120 32 82 46-82 47-82-47Z" fill="var(--color-brand-highlight)" />
        <path d="m120 125 82-47v88l-82 46Z" fill="var(--u3-highlight)" />
        <path
          d="m64 95 30 18v45c0 20-30 4-30-14V95ZM148 132l25-14v19l-17 9 17 1v19l-25 14"
          stroke="var(--color-brand)"
          stroke-width="7"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
        <path
          d="m85 72 34-19 35 19-34 19Z"
          stroke="var(--theme-secondary, var(--color-brand))"
          stroke-width="1.5"
        />
      </svg>
      <div class="art-label top-label"><i />{{ translate(theme.activeTheme.name) }}</div>
      <div class="art-label bottom-label">
        <strong>{{ String(profiles).padStart(2, '0') }}</strong
        ><span>{{ tr('в твоей библиотеке', 'in your library') }}</span>
      </div>
    </div>
    <div class="welcome-footer">
      <span>01 / {{ tr('ТВОЁ ПРОСТРАНСТВО', 'YOUR SPACE') }}</span
      ><RouterLink to="/themes"
        >{{ tr('Изменить атмосферу', 'Change atmosphere') }} <span>↗</span></RouterLink
      >
    </div>
  </section>
</template>
<style scoped>
.studio-welcome {
  position: relative;
  display: grid;
  grid-template-columns: 1.2fr 1fr;
  gap: 24px;
  width: calc(100% - 48px);
  max-width: 1440px;
  margin: 24px auto 12px;
  padding: 32px 36px 0;
  overflow: hidden;
  border: 1px solid var(--u3-outline);
  border-radius: 24px;
  background: radial-gradient(ellipse at 85% 30%, var(--color-brand-highlight), transparent 55%),
    var(--color-raised-bg);
  box-shadow:
    0 12px 40px var(--u3-shadow),
    inset 0 1px 0 var(--u3-highlight);
}
.welcome-copy {
  z-index: 1;
  padding-bottom: 24px;
}
.welcome-kicker {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
  font-weight: 750;
  letter-spacing: 0.16em;
  margin: 0 0 18px;
}
.welcome-kicker > span:first-child {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--color-brand);
}
.edition {
  font-size: 9px;
  letter-spacing: 0.12em;
  padding: 4px 6px;
  border-radius: 5px;
  border: 1px solid var(--u3-outline);
  color: var(--color-secondary);
}
h1 {
  margin: 0;
  font-size: clamp(28px, 3.5vw, 48px);
  letter-spacing: -0.055em;
  line-height: 1.1;
  font-weight: 750;
}
h1 span {
  color: var(--color-brand);
}
.welcome-description {
  color: var(--color-secondary);
  font-size: 14px;
  line-height: 1.6;
  max-width: 360px;
  margin: 16px 0 22px;
}
.welcome-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}
.studio-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 12px 15px;
  border-radius: 12px;
  font-size: 13px;
  font-weight: 650;
}
.studio-action svg {
  width: 17px;
  height: 17px;
}
.main-action {
  color: var(--color-accent-contrast);
  background: var(--color-brand);
  box-shadow: 0 5px 16px var(--color-brand-shadow);
}
.secondary-action {
  color: var(--color-base);
  border: 1px solid var(--u3-outline);
  background: var(--color-bg);
}
.welcome-art {
  position: relative;
  min-height: 235px;
  display: grid;
  place-items: center;
}
.art-grid {
  position: absolute;
  inset: 0;
  opacity: 0.25;
  background-image: radial-gradient(var(--color-secondary) 0.65px, transparent 0.8px);
  background-size: 16px 16px;
  mask-image: radial-gradient(ellipse, #000, transparent 72%);
}
.welcome-cube {
  width: 210px;
  height: 210px;
  transform: rotate(-7deg);
  filter: drop-shadow(0 16px 18px var(--u3-shadow));
}
.art-orbit {
  position: absolute;
  width: 265px;
  height: 110px;
  border: 1px solid var(--u3-outline);
  border-radius: 50%;
  transform: rotate(-22deg);
}
.art-orbit.second {
  width: 250px;
  height: 180px;
  transform: rotate(48deg);
  border-style: dashed;
}
.art-label {
  position: absolute;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 9px 12px;
  font-size: 11px;
  border: 1px solid var(--u3-outline);
  border-radius: 11px;
  background: var(--color-raised-bg);
  box-shadow: 0 8px 20px var(--u3-shadow);
}
.top-label {
  top: 8px;
  right: 0;
  transform: rotate(5deg);
}
.top-label i {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--color-brand);
}
.bottom-label {
  bottom: 19px;
  left: 0;
  transform: rotate(-4deg);
}
.bottom-label strong {
  font-size: 23px;
  color: var(--color-brand);
}
.bottom-label span {
  max-width: 85px;
  color: var(--color-secondary);
  line-height: 1.35;
}
.welcome-footer {
  grid-column: 1 / -1;
  border-top: 1px solid var(--u3-outline);
  display: flex;
  justify-content: space-between;
  gap: 12px;
  padding: 13px 0;
  font-size: 10px;
  letter-spacing: 0.08em;
  color: var(--color-secondary);
}
.welcome-footer a {
  color: var(--color-secondary);
}
.welcome-footer a span {
  color: var(--color-brand);
  margin-left: 8px;
}
@media (max-width: 800px) {
  .studio-welcome {
    grid-template-columns: 1fr;
    padding: 26px 24px 0;
  }
  .welcome-art {
    display: none;
  }
}
@media (max-width: 520px) {
  .studio-welcome {
    width: calc(100% - 24px);
    margin: 12px auto;
    padding: 22px 18px 0;
    border-radius: 18px;
    gap: 0;
  }
  h1 {
    font-size: 31px;
  }
  .welcome-footer {
    font-size: 9px;
    flex-wrap: wrap;
  }
  .studio-action {
    width: 100%;
  }
}
</style>
