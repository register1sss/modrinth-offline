<script setup>
import { XIcon, IssuesIcon, LogInIcon } from '@modrinth/assets'
import { Modal } from '@modrinth/ui'
import { ref } from 'vue'
import { login as login_flow, set_default_user } from '@/helpers/auth.js'
import { handleError } from '@/store/notifications.js'
import mixpanel from 'mixpanel-browser'
import { handleSevereError } from '@/store/error.js'

const errorModal = ref()
const error = ref()

const title = ref('An error occurred')
const errorType = ref('unknown')
const supportLink = ref('https://support.modrinth.com')
const metadata = ref({})

defineExpose({
  async show(errorVal) {
    if (errorVal.message && errorVal.message.includes('Minecraft authentication error:')) {
      title.value = 'Unable to sign in to Minecraft'
      errorType.value = 'minecraft_auth'
      supportLink.value =
        'https://support.modrinth.com/en/articles/9038231-minecraft-sign-in-issues'

      if (
        errorVal.message.includes('existing connection was forcibly closed') ||
        errorVal.message.includes('error sending request for url')
      ) {
        metadata.value.network = true
      }
      if (errorVal.message.includes('because the target machine actively refused it')) {
        metadata.value.hostsFile = true
      }
    } else if (errorVal.message && errorVal.message.includes('User is not logged in')) {
      title.value = 'Sign in to Minecraft'
      errorType.value = 'minecraft_sign_in'
      supportLink.value = 'https://support.modrinth.com'
    } else {
      title.value = 'An error occurred'
      errorType.value = 'unknown'
      supportLink.value = 'https://support.modrinth.com'
      metadata.value = {}
    }

    error.value = errorVal
    errorModal.value.show()
  },
})

const loadingMinecraft = ref(false)
async function loginMinecraft() {
  try {
    loadingMinecraft.value = true
    const loggedIn = await login_flow()

    if (loggedIn) {
      await set_default_user(loggedIn.id).catch(handleError)
    }

    await mixpanel.track('AccountLogIn')
    loadingMinecraft.value = false
    errorModal.value.hide()
  } catch (err) {
    loadingMinecraft.value = false
    handleSevereError(err)
  }
}
</script>

<template>
  <Modal ref="errorModal" :header="title">
    <div class="modal-body">
      <div class="markdown-body">
        <template v-if="errorType === 'minecraft_auth'">
          <template v-if="metadata.network">
            <h3>{{ $u3t('Network issues') }}</h3>
            <p>
              {{
                $u3t(
                  "It looks like there were issues with the U3drinth connecting to Microsoft's servers. This is often the result of a poor connection, so we recommend trying again to see if it works. If issues continue to persist, follow the steps in",
                )
              }}
              <a
                href="https://support.modrinth.com/en/articles/9038231-minecraft-sign-in-issues#h_e71a5f805f"
              >
                {{ $u3t('our support article') }}
              </a>
              {{ $u3t('to troubleshoot.') }}
            </p>
          </template>
          <template v-else-if="metadata.hostsFile">
            <h3>{{ $u3t('Network issues') }}</h3>
            <p>
              {{
                $u3t(
                  'The U3drinth tried to connect to Microsoft / Xbox / Minecraft services, but the remote server rejected the connection. This may indicate that these services are blocked by the hosts file. Please visit',
                )
              }}
              <a
                href="https://support.modrinth.com/en/articles/9038231-minecraft-sign-in-issues#h_d694a29256"
              >
                {{ $u3t('our support article') }}
              </a>
              {{ $u3t('for steps on how to fix the issue.') }}
            </p>
          </template>
          <template v-else>
            <h3>{{ $u3t('Try another Microsoft account') }}</h3>
            <p>
              {{
                $u3t(
                  "Double check you've signed in with the right account. You may own Minecraft on a different Microsoft account.",
                )
              }}
            </p>
            <div class="cta-button">
              <button class="btn btn-primary" :disabled="loadingMinecraft" @click="loginMinecraft">
                <LogInIcon /> {{ $u3t('Try another account') }}
              </button>
            </div>
            <h3>{{ $u3t('Using PC Game Pass, coming from Bedrock, or just bought the game?') }}</h3>
            <p>
              {{ $u3t('Try signing in with the') }}
              <a href="https://www.minecraft.net/en-us/download">{{
                $u3t('official Minecraft Launcher')
              }}</a
              >{{ $u3t(" first. Once you're done, come back here and sign in!") }}
            </p>
          </template>
          <div class="cta-button">
            <button class="btn btn-primary" :disabled="loadingMinecraft" @click="loginMinecraft">
              <LogInIcon /> {{ $u3t('Try signing in again') }}
            </button>
          </div>
          <hr />
          <p>
            {{ $u3t('If nothing helps, contact support and include the debug information below.') }}
          </p>
          <details>
            <summary>{{ $u3t('Debug information') }}</summary>
            {{ error.message ?? error }}
          </details>
        </template>
        <div v-else-if="errorType === 'minecraft_sign_in'">
          <div class="warning-banner">
            <div class="warning-banner__title">
              <IssuesIcon />
              <span>{{ $u3t('Installed the app before April 23rd, 2024?') }}</span>
            </div>
            <div class="warning-banner__description">
              {{
                $u3t(
                  'Modrinth has updated our sign-in workflow to allow for better stability, security, and performance. You must sign in again so your credentials can be upgraded to this new flow.',
                )
              }}
            </div>
          </div>
          <p>
            {{
              $u3t(
                "To play this instance, you must sign in through Microsoft below. If you don't have a Minecraft account, you can purchase the game on the",
              )
            }}
            <a href="https://www.minecraft.net/en-us/store/minecraft-java-bedrock-edition-pc">{{
              $u3t('Minecraft website')
            }}</a
            >.
          </p>
          <div class="cta-button">
            <button class="btn btn-primary" :disabled="loadingMinecraft" @click="loginMinecraft">
              <LogInIcon /> {{ $u3t('Sign in to Minecraft') }}
            </button>
          </div>
        </div>
        <template v-else>
          {{ error.message ?? error }}
        </template>
      </div>
      <div class="input-group push-right">
        <button class="btn" @click="errorModal.hide()"><XIcon /> {{ $u3t('Close') }}</button>
      </div>
    </div>
  </Modal>
</template>

<style>
.light-mode {
  --color-orange-bg: rgba(255, 163, 71, 0.2);
}

.dark-mode,
.oled-mode {
  --color-orange-bg: rgba(224, 131, 37, 0.2);
}
</style>

<style scoped lang="scss">
.cta-button {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0.5rem;
}

.warning-banner {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  padding: var(--gap-lg);
  background-color: var(--color-orange-bg);
  border: 2px solid var(--color-orange);
  border-radius: var(--radius-md);
  margin-bottom: 1rem;
}

.warning-banner__title {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-weight: 700;

  svg {
    color: var(--color-orange);
    height: 1.5rem;
    width: 1.5rem;
  }
}

.modal-body {
  display: flex;
  flex-direction: column;
  gap: var(--gap-md);
  padding: var(--gap-lg);
}

.markdown-body {
  overflow: auto;
}
</style>
