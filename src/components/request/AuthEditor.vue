<script setup lang="ts">
import { NSelect, NInput, NInputGroup, NInputGroupLabel, NButton, NIcon, NSpace } from 'naive-ui'
import { EyeOutline, EyeOffOutline, LockClosedOutline } from '@vicons/ionicons5'
import { computed, ref, watch, onMounted } from 'vue'
import type { AuthConfig, AuthType } from '@/types'
import { useAuthSecrets, maskSensitiveValue } from '@/composables/useSecretStorage'

const props = defineProps<{
  auth: AuthConfig | null
  requestId: string
}>()

const emit = defineEmits<{
  'update:auth': [auth: AuthConfig]
}>()

const safeAuth = computed<AuthConfig>(() => props.auth ?? { type: 'none' })

const {
  storeBearerToken,
  retrieveBearerToken,
  storeBasicAuth,
  retrieveBasicAuth,
  storeApiKey,
} = useAuthSecrets()

const showBearerToken = ref(false)
const showBasicPassword = ref(false)
const showApiKeyValue = ref(false)
const useSecureStorage = ref(true)

const authOptions = [
  { label: 'No Auth', value: 'none' },
  { label: 'Basic Auth', value: 'basic' },
  { label: 'Bearer Token', value: 'bearer' },
  { label: 'API Key', value: 'apikey' },
]

const authType = computed({
  get: () => safeAuth.value.type,
  set: async (type: AuthType) => {
    const newAuth: AuthConfig = { type }
    if (type === 'basic') {
      newAuth.basic = { username: '', password: '' }
      if (useSecureStorage.value && props.requestId) {
        const stored = await retrieveBasicAuth(props.requestId)
        if (stored) {
          newAuth.basic = stored
        }
      }
    }
    if (type === 'bearer') {
      newAuth.bearer = { token: '' }
      if (useSecureStorage.value && props.requestId) {
        const stored = await retrieveBearerToken(props.requestId)
        if (stored) {
          newAuth.bearer = { token: stored }
        }
      }
    }
    if (type === 'apikey') {
      newAuth.apikey = { key: '', value: '', addTo: 'header' }
    }
    emit('update:auth', newAuth)
  },
})

const basicUsername = computed({
  get: () => safeAuth.value.basic?.username || '',
  set: (username: string) => {
    emit('update:auth', { ...safeAuth.value, basic: { ...safeAuth.value.basic!, username } })
  },
})

const basicPassword = computed({
  get: () => safeAuth.value.basic?.password || '',
  set: async (password: string) => {
    emit('update:auth', { ...safeAuth.value, basic: { ...safeAuth.value.basic!, password } })
    if (useSecureStorage.value && props.requestId) {
      await storeBasicAuth(props.requestId, basicUsername.value, password)
    }
  },
})

const bearerToken = computed({
  get: () => safeAuth.value.bearer?.token || '',
  set: async (token: string) => {
    emit('update:auth', { ...safeAuth.value, bearer: { ...safeAuth.value.bearer!, token } })
    if (useSecureStorage.value && props.requestId) {
      await storeBearerToken(props.requestId, token)
    }
  },
})

const apiKeyKey = computed({
  get: () => safeAuth.value.apikey?.key || '',
  set: (key: string) => {
    emit('update:auth', { ...safeAuth.value, apikey: { ...safeAuth.value.apikey!, key } })
  },
})

const apiKeyValue = computed({
  get: () => safeAuth.value.apikey?.value || '',
  set: async (value: string) => {
    emit('update:auth', { ...safeAuth.value, apikey: { ...safeAuth.value.apikey!, value } })
    if (useSecureStorage.value && props.requestId) {
      await storeApiKey(props.requestId, apiKeyKey.value, value)
    }
  },
})

const apiKeyAddTo = computed({
  get: () => safeAuth.value.apikey?.addTo || 'header',
  set: (addTo: 'header' | 'query') => {
    emit('update:auth', { ...safeAuth.value, apikey: { ...safeAuth.value.apikey!, addTo } })
  },
})

const displayBearerToken = computed(() => {
  if (showBearerToken.value) return bearerToken.value
  return bearerToken.value ? maskSensitiveValue(bearerToken.value) : ''
})

const displayBasicPassword = computed(() => {
  if (showBasicPassword.value) return basicPassword.value
  return basicPassword.value ? maskSensitiveValue(basicPassword.value) : ''
})

const displayApiKeyValue = computed(() => {
  if (showApiKeyValue.value) return apiKeyValue.value
  return apiKeyValue.value ? maskSensitiveValue(apiKeyValue.value) : ''
})

onMounted(async () => {
  if (useSecureStorage.value && props.requestId) {
    if (safeAuth.value.type === 'bearer') {
      const stored = await retrieveBearerToken(props.requestId)
      if (stored && !safeAuth.value.bearer?.token) {
        emit('update:auth', { ...safeAuth.value, bearer: { token: stored } })
      }
    }
    if (safeAuth.value.type === 'basic') {
      const stored = await retrieveBasicAuth(props.requestId)
      if (stored && !safeAuth.value.basic?.password) {
        emit('update:auth', { ...safeAuth.value, basic: stored })
      }
    }
  }
})

watch(() => props.requestId, async (newId) => {
  if (newId && useSecureStorage.value) {
    if (safeAuth.value.type === 'bearer') {
      const stored = await retrieveBearerToken(newId)
      if (stored) {
        emit('update:auth', { ...safeAuth.value, bearer: { token: stored } })
      }
    }
    if (safeAuth.value.type === 'basic') {
      const stored = await retrieveBasicAuth(newId)
      if (stored) {
        emit('update:auth', { ...safeAuth.value, basic: stored })
      }
    }
  }
})
</script>

<template>
  <div class="auth-editor">
    <div class="auth-type-selector">
      <NSelect :value="authType" :options="authOptions" @update:value="authType = $event" />
    </div>

    <div v-if="authType !== 'none'" class="secure-storage-hint">
      <NSpace align="center">
        <NIcon :component="LockClosedOutline" size="16" />
        <span class="hint-text">敏感信息将使用系统密钥库安全存储</span>
      </NSpace>
    </div>

    <div class="auth-config">
      <template v-if="authType === 'basic'">
        <NInputGroup>
          <NInputGroupLabel>用户名</NInputGroupLabel>
          <NInput v-model:value="basicUsername" placeholder="用户名" />
        </NInputGroup>
        <NInputGroup style="margin-top: 8px">
          <NInputGroupLabel>密码</NInputGroupLabel>
          <NInput
            :value="displayBasicPassword"
            :type="showBasicPassword ? 'text' : 'password'"
            placeholder="密码"
            @update:value="basicPassword = $event"
          >
            <template #suffix>
              <NButton text size="small" @click="showBasicPassword = !showBasicPassword">
                <NIcon :component="showBasicPassword ? EyeOffOutline : EyeOutline" />
              </NButton>
            </template>
          </NInput>
        </NInputGroup>
      </template>

      <template v-else-if="authType === 'bearer'">
        <NInputGroup>
          <NInputGroupLabel>Token</NInputGroupLabel>
          <NInput
            :value="displayBearerToken"
            :type="showBearerToken ? 'text' : 'password'"
            placeholder="Bearer Token"
            @update:value="bearerToken = $event"
          >
            <template #suffix>
              <NButton text size="small" @click="showBearerToken = !showBearerToken">
                <NIcon :component="showBearerToken ? EyeOffOutline : EyeOutline" />
              </NButton>
            </template>
          </NInput>
        </NInputGroup>
      </template>

      <template v-else-if="authType === 'apikey'">
        <NInputGroup style="margin-bottom: 8px">
          <NInputGroupLabel>Key</NInputGroupLabel>
          <NInput v-model:value="apiKeyKey" placeholder="API Key 参数名" />
        </NInputGroup>
        <NInputGroup style="margin-bottom: 8px">
          <NInputGroupLabel>Value</NInputGroupLabel>
          <NInput
            :value="displayApiKeyValue"
            :type="showApiKeyValue ? 'text' : 'password'"
            placeholder="API Key 值"
            @update:value="apiKeyValue = $event"
          >
            <template #suffix>
              <NButton text size="small" @click="showApiKeyValue = !showApiKeyValue">
                <NIcon :component="showApiKeyValue ? EyeOffOutline : EyeOutline" />
              </NButton>
            </template>
          </NInput>
        </NInputGroup>
        <NInputGroup>
          <NInputGroupLabel>添加到</NInputGroupLabel>
          <NSelect
            :value="apiKeyAddTo"
            :options="[
              { label: 'Header', value: 'header' },
              { label: 'Query', value: 'query' },
            ]"
            @update:value="(val: 'header' | 'query') => apiKeyAddTo = val"
          />
        </NInputGroup>
      </template>

      <template v-else>
        <p style="color: var(--n-text-color-3)">该请求不需要认证</p>
      </template>
    </div>
  </div>
</template>

<style scoped>
.auth-editor {
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.auth-type-selector {
  margin-bottom: 16px;
  max-width: 300px;
}

.auth-config {
  max-width: 500px;
}

.secure-storage-hint {
  margin-bottom: 12px;
  padding: 8px 12px;
  background-color: var(--n-color-hover);
  border-radius: 4px;
}

.hint-text {
  font-size: 12px;
  color: var(--n-text-color-3);
}
</style>
