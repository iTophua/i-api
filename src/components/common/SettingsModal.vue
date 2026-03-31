<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  NModal,
  NTabs,
  NTabPane,
  NForm,
  NFormItem,
  NSelect,
  NInputNumber,
  NSwitch,
  NButton,
  NInput,
} from 'naive-ui'
import { useSettingsStore } from '@/stores'
import { useI18n } from '@/composables/useI18n'
import type { Settings } from '@/types'

defineProps<{
  show: boolean
}>()

const emit = defineEmits<{
  (e: 'update:show', value: boolean): void
}>()

const settingsStore = useSettingsStore()
const { locales } = useI18n()

const localSettings = ref<Settings>({ ...settingsStore.settings })

const themeOptions = [
  { label: '浅色', value: 'light' },
  { label: '深色', value: 'dark' },
  { label: '跟随系统', value: 'system' },
]

const languageOptions = computed(() =>
  locales.map((l) => ({
    label: l.label,
    value: l.value,
  }))
)

const proxyEnabled = computed({
  get: () => localSettings.value.proxy?.enabled || false,
  set: (value) => {
    if (!localSettings.value.proxy) {
      localSettings.value.proxy = {
        enabled: false,
        host: '',
        port: 8080,
      }
    }
    localSettings.value.proxy.enabled = value
  },
})

const downloadPath = computed({
  get: () => localSettings.value.downloadPath ?? '',
  set: (val: string) => {
    localSettings.value.downloadPath = val
  },
})

const downloadAsk = computed({
  get: () => localSettings.value.downloadAsk ?? true,
  set: (val: boolean) => {
    localSettings.value.downloadAsk = val
  },
})

function handleClose() {
  emit('update:show', false)
}

function handleSave() {
  const newLang = localSettings.value.language
  settingsStore.setLanguage(newLang)
  settingsStore.updateSettings(localSettings.value)
  handleClose()
}

function handleReset() {
  localSettings.value = { ...settingsStore.settings }
}
</script>

<template>
  <NModal
    :show="show"
    preset="card"
    style="width: 600px; max-width: 90vw"
    title="设置"
    :bordered="false"
    :mask-closable="false"
    @update:show="$emit('update:show', $event)"
  >
    <NTabs type="line">
      <NTabPane name="general" tab="常规">
        <NForm label-placement="left" label-width="100">
          <NFormItem label="主题">
            <NSelect
              v-model:value="localSettings.theme"
              :options="themeOptions"
              style="width: 200px"
            />
          </NFormItem>
          <NFormItem label="语言">
            <NSelect
              v-model:value="localSettings.language"
              :options="languageOptions"
              style="width: 200px"
            />
          </NFormItem>
          <NFormItem label="历史记录限制">
            <NInputNumber
              v-model:value="localSettings.historyLimit"
              :min="10"
              :max="1000"
              :step="10"
              style="width: 200px"
            />
          </NFormItem>
          <NFormItem label="请求超时">
            <NInputNumber
              v-model:value="localSettings.timeout"
              :min="1000"
              :max="300000"
              :step="1000"
              style="width: 200px"
            >
              <template #suffix>ms</template>
            </NInputNumber>
          </NFormItem>
        </NForm>
      </NTabPane>

      <NTabPane name="proxy" tab="代理">
        <NForm label-placement="left" label-width="100">
          <NFormItem label="启用代理">
            <NSwitch v-model:value="proxyEnabled" />
          </NFormItem>
          <template v-if="localSettings.proxy?.enabled">
            <NFormItem label="主机">
              <input
                v-model="localSettings.proxy!.host"
                class="proxy-input"
                placeholder="127.0.0.1"
              />
            </NFormItem>
            <NFormItem label="端口">
              <NInputNumber
                v-model:value="localSettings.proxy!.port"
                :min="1"
                :max="65535"
                style="width: 200px"
              />
            </NFormItem>
            <NFormItem label="用户名">
              <input
                v-model="localSettings.proxy!.username"
                class="proxy-input"
                placeholder="可选"
              />
            </NFormItem>
            <NFormItem label="密码">
              <input
                v-model="localSettings.proxy!.password"
                type="password"
                class="proxy-input"
                placeholder="可选"
              />
            </NFormItem>
          </template>
        </NForm>
      </NTabPane>

      <NTabPane name="download" tab="下载">
        <NForm label-placement="left" label-width="100">
          <NFormItem label="下载路径">
            <NInput
              v-model:value="downloadPath"
              placeholder="留空则默认Downloads文件夹"
              style="width: 300px"
            />
          </NFormItem>
          <NFormItem label="每次询问位置">
            <NSwitch v-model:value="downloadAsk" />
          </NFormItem>
        </NForm>
      </NTabPane>
    </NTabs>

    <template #footer>
      <div class="modal-footer">
        <NButton @click="handleReset">重置</NButton>
        <div class="footer-actions">
          <NButton @click="handleClose">取消</NButton>
          <NButton type="primary" @click="handleSave">保存</NButton>
        </div>
      </div>
    </template>
  </NModal>
</template>

<style scoped>
.proxy-input {
  width: 200px;
  padding: 6px 12px;
  border: 1px solid var(--n-border-color);
  border-radius: 4px;
  background: var(--n-color);
  color: var(--n-text-color);
  font-size: 14px;
  transition: border-color 0.2s;
}

.proxy-input:focus {
  outline: none;
  border-color: var(--n-color-target);
}

.proxy-input::placeholder {
  color: var(--n-text-color-3);
}

.modal-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.footer-actions {
  display: flex;
  gap: 8px;
}
</style>
