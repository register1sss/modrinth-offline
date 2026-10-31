<script setup>
import { getCurrentInstance } from 'vue'
import { tr } from '@/i18n/index.js'
import { LibraryIcon, SearchIcon, RightArrowIcon } from '@modrinth/assets'
const cubeId = `u3-cube-${getCurrentInstance().uid}`
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
      <svg class="welcome-cube" viewBox="0 0 320 306" fill="none">
        <defs>
          <linearGradient
            :id="`${cubeId}-top`"
            x1="78"
            y1="66"
            x2="225"
            y2="155"
            gradientUnits="userSpaceOnUse"
          >
            <stop stop-color="var(--color-brand)" stop-opacity=".48" />
            <stop offset="1" stop-color="var(--color-brand)" stop-opacity=".08" />
          </linearGradient>
          <linearGradient
            :id="`${cubeId}-left`"
            x1="50"
            y1="115"
            x2="170"
            y2="252"
            gradientUnits="userSpaceOnUse"
          >
            <stop stop-color="var(--color-brand)" stop-opacity=".24" />
            <stop offset="1" stop-color="var(--color-bg)" stop-opacity=".7" />
          </linearGradient>
          <linearGradient
            :id="`${cubeId}-right`"
            x1="263"
            y1="111"
            x2="165"
            y2="267"
            gradientUnits="userSpaceOnUse"
          >
            <stop stop-color="var(--color-bg)" stop-opacity=".75" />
            <stop offset="1" stop-color="var(--color-brand)" stop-opacity=".16" />
          </linearGradient>
          <linearGradient
            :id="`${cubeId}-edge`"
            x1="80"
            y1="56"
            x2="252"
            y2="257"
            gradientUnits="userSpaceOnUse"
          >
            <stop stop-color="var(--color-brand)" stop-opacity=".95" />
            <stop offset=".5" stop-color="var(--color-brand)" stop-opacity=".38" />
            <stop offset="1" stop-color="var(--color-brand)" stop-opacity=".12" />
          </linearGradient>
        </defs>
        <ellipse cx="160" cy="283" rx="77" ry="9" fill="var(--color-bg)" opacity=".45" />
        <path d="m160 38 110 63v106l-110 65-110-65V101Z" fill="var(--color-raised-bg)" />
        <path d="m160 38 110 63-110 65L50 101Z" :fill="`url(#${cubeId}-top)`" />
        <path d="m50 101 110 65v106L50 207Z" :fill="`url(#${cubeId}-left)`" />
        <path d="m160 166 110-65v106l-110 65Z" :fill="`url(#${cubeId}-right)`" />
        <path
          d="m160 46 97 56-97 57-97-57Z"
          fill="var(--color-bg)"
          fill-opacity=".16"
          stroke="var(--color-brand)"
          stroke-opacity=".3"
        />
        <path
          d="m57 114 96 57v87l-96-56Z"
          fill="var(--color-bg)"
          fill-opacity=".2"
          stroke="var(--color-brand)"
          stroke-opacity=".12"
        />
        <path
          d="m167 171 96-57v88l-96 56Z"
          fill="var(--color-bg)"
          fill-opacity=".18"
          stroke="var(--color-brand)"
          stroke-opacity=".12"
        />
        <path
          d="m160 38 110 63v106l-110 65-110-65V101Zm-110 63 110 65 110-65M160 166v106"
          :stroke="`url(#${cubeId}-edge)`"
          stroke-width="2"
          stroke-linejoin="round"
        />
        <path
          d="m55 101 105-60 105 60M55 105l101 60"
          stroke="white"
          stroke-opacity=".24"
          stroke-linecap="round"
        />
        <g
          transform="matrix(1.1 .65 0 1.06 50 101)"
          stroke="var(--color-brand)"
          stroke-width="9"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M27 23v33q0 21 23 21t23-21V23" />
        </g>
        <g
          transform="matrix(1.1 -.65 0 1.06 160 166)"
          stroke="var(--color-brand)"
          stroke-width="8"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M27 24h28q19 0 19 13t-19 13H43m12 0q21 0 21 14T55 78H27" />
        </g>
        <path
          d="m160 71 47 27-47 28-47-28Z"
          stroke="var(--color-brand)"
          stroke-opacity=".7"
          stroke-width="1.5"
        />
        <path d="m160 84 24 14-24 14-24-14Z" fill="var(--color-brand)" fill-opacity=".28" />
        <circle cx="160" cy="98" r="3" fill="var(--color-brand)" />
      </svg>
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
  padding: 32px 36px;
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
  padding-bottom: 0;
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
  min-height: 274px;
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
  width: 286px;
  height: 274px;
  transform: translateY(-2px);
  filter: drop-shadow(0 16px 18px var(--u3-shadow));
}
.art-orbit {
  position: absolute;
  width: 300px;
  height: 126px;
  opacity: 0.6;
  border: 1px solid var(--u3-outline);
  border-radius: 50%;
  transform: rotate(-22deg);
}
.art-orbit.second {
  width: 280px;
  height: 212px;
  transform: rotate(48deg);
  border-style: dashed;
}
@media (max-width: 800px) {
  .studio-welcome {
    grid-template-columns: 1fr;
    padding: 26px 24px;
  }
  .welcome-art {
    display: none;
  }
}
@media (max-width: 520px) {
  .studio-welcome {
    width: calc(100% - 24px);
    margin: 12px auto;
    padding: 22px 18px;
    border-radius: 18px;
    gap: 0;
  }
  h1 {
    font-size: 31px;
  }
  .studio-action {
    width: 100%;
  }
}
</style>
