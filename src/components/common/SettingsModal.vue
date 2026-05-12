<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
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
  NAlert,
  NIcon,
} from 'naive-ui'
import { TrashOutline } from '@vicons/ionicons5'
import { useSettingsStore } from '@/stores'
import { useI18n } from '@/composables/useI18n'
import { defaultShortcuts, formatShortcut, getGroupedShortcuts } from '@/composables/useShortcuts'
import type { Settings, ShortcutBinding } from '@/types'

defineProps<{
  show: boolean
}>()

const emit = defineEmits<{
  (e: 'update:show', value: boolean): void
}>()

const settingsStore = useSettingsStore()
const { locales } = useI18n()

const localSettings = ref<Settings>({ ...settingsStore.settings })

// 记住上次切换的 tab
const activeTab = ref<string>(localStorage.getItem('iapi-settings-tab') || 'general')

function handleTabChange(tab: string) {
  activeTab.value = tab
  localStorage.setItem('iapi-settings-tab', tab)
}

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

const followRedirects = computed({
  get: () => localSettings.value.followRedirects ?? true,
  set: (val: boolean) => {
    localSettings.value.followRedirects = val
  },
})

const verifySsl = computed({
  get: () => localSettings.value.verifySsl ?? true,
  set: (val: boolean) => {
    localSettings.value.verifySsl = val
  },
})

// 快捷键相关
const groupedShortcuts = getGroupedShortcuts()
const recordingShortcutId = ref<string | null>(null)
const pressedModifiers = ref({ ctrl: false, shift: false, alt: false })

const categoryLabels: Record<string, string> = {
  global: '全局',
  navigation: '导航',
  editor: '编辑器',
}

function getShortcutBinding(id: string): ShortcutBinding | null {
  const customBindings = localSettings.value.shortcuts
  if (customBindings && id in customBindings) {
    return customBindings[id]
  }
  const defaultShortcut = defaultShortcuts.find((s) => s.id === id)
  if (defaultShortcut) {
    return {
      key: defaultShortcut.key,
      ctrl: defaultShortcut.ctrl,
      shift: defaultShortcut.shift,
      alt: defaultShortcut.alt,
      meta: defaultShortcut.meta,
    }
  }
  return null
}

function getShortcutDisplay(id: string): string {
  const binding = getShortcutBinding(id)
  if (!binding) return '已禁用'
  return formatShortcut(binding)
}

function startRecording(id: string) {
  recordingShortcutId.value = id
  pressedModifiers.value = { ctrl: false, shift: false, alt: false }
}

function handleRecordKeyDown(e: KeyboardEvent) {
  if (!recordingShortcutId.value) return
  e.preventDefault()
  e.stopPropagation()

  if (e.key === 'Escape') {
    recordingShortcutId.value = null
    return
  }

  if (e.key === 'Backspace' || e.key === 'Delete') {
    if (!localSettings.value.shortcuts) {
      localSettings.value.shortcuts = {}
    }
    localSettings.value.shortcuts[recordingShortcutId.value] = null
    recordingShortcutId.value = null
    return
  }

  // 记录修饰键状态，不立即完成录制
  if (e.key === 'Control' || e.key === 'Meta') {
    pressedModifiers.value.ctrl = true
    return
  }
  if (e.key === 'Shift') {
    pressedModifiers.value.shift = true
    return
  }
  if (e.key === 'Alt') {
    pressedModifiers.value.alt = true
    return
  }

  // 普通按键，结合修饰键完成录制
  const binding: ShortcutBinding = {
    key: e.key,
    ctrl: pressedModifiers.value.ctrl || e.ctrlKey || e.metaKey,
    shift: pressedModifiers.value.shift || e.shiftKey,
    alt: pressedModifiers.value.alt || e.altKey,
  }

  if (!localSettings.value.shortcuts) {
    localSettings.value.shortcuts = {}
  }
  localSettings.value.shortcuts[recordingShortcutId.value] = binding
  recordingShortcutId.value = null
}

function resetShortcut(id: string) {
  if (!localSettings.value.shortcuts) {
    localSettings.value.shortcuts = {}
  }
  delete localSettings.value.shortcuts[id]
}

function resetAllShortcuts() {
  localSettings.value.shortcuts = {}
}

onMounted(() => {
  window.addEventListener('keydown', handleRecordKeyDown, true)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleRecordKeyDown, true)
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
    style="width: 700px; max-width: 90vw; min-height: 450px"
    title="设置"
    :bordered="false"
    :mask-closable="false"
    @update:show="$emit('update:show', $event)"
  >
    <NTabs v-model:value="activeTab" type="line" @update:value="handleTabChange">
      <NTabPane name="general" tab="常规">
        <div class="tab-content">
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
        </div>
      </NTabPane>

      <NTabPane name="shortcuts" tab="快捷键">
        <div class="tab-content">
          <div class="shortcuts-container">
            <div class="shortcuts-header">
              <span class="shortcuts-hint">
                点击快捷键区域进行修改，按 Backspace/Delete 禁用，Esc 取消
              </span>
              <NButton size="small" @click="resetAllShortcuts">恢复默认</NButton>
            </div>
            <div v-for="(shortcuts, category) in groupedShortcuts" :key="category" class="shortcut-group">
              <template v-if="shortcuts.length > 0">
                <div class="shortcut-group-title">{{ categoryLabels[category] || category }}</div>
                <div v-for="shortcut in shortcuts" :key="shortcut.id" class="shortcut-item">
                  <span class="shortcut-desc">{{ shortcut.description }}</span>
                  <div
                    class="shortcut-binding"
                    :class="{ recording: recordingShortcutId === shortcut.id }"
                    @click="startRecording(shortcut.id)"
                  >
                    <span v-if="recordingShortcutId === shortcut.id" class="recording-text">
                      请按下快捷键...
                    </span>
                    <span v-else>{{ getShortcutDisplay(shortcut.id) }}</span>
                    <NButton
                      v-if="getShortcutBinding(shortcut.id) !== null"
                      text
                      size="tiny"
                      class="reset-btn"
                      @click.stop="resetShortcut(shortcut.id)"
                    >
                      <NIcon :component="TrashOutline" size="12" />
                    </NButton>
                  </div>
                </div>
              </template>
            </div>
          </div>
        </div>
      </NTabPane>

      <NTabPane name="network" tab="网络">
        <div class="tab-content">
          <NForm label-placement="left" label-width="120">
            <NFormItem label="跟随重定向">
              <NSwitch v-model:value="followRedirects" />
              <span class="setting-hint">关闭后 3xx 响应不会自动跳转</span>
            </NFormItem>
            <NFormItem label="验证 SSL 证书">
              <NSwitch v-model:value="verifySsl" />
              <span class="setting-hint">关闭后可访问自签名证书的站点</span>
            </NFormItem>
            <NAlert title="安全提示" type="warning" style="margin-top: 12px">
              关闭 SSL 验证存在安全风险，请在测试环境中使用。
            </NAlert>
          </NForm>
        </div>
      </NTabPane>

      <NTabPane name="proxy" tab="代理">
        <div class="tab-content">
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
        </div>
      </NTabPane>

      <NTabPane name="download" tab="下载">
        <div class="tab-content">
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
        </div>
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
.tab-content {
  min-height: 320px;
  overflow-y: auto;
  padding-right: 4px;
}

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

.setting-hint {
  margin-left: 8px;
  font-size: 12px;
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

.shortcuts-container {
  max-height: 320px;
  overflow-y: auto;
}

.shortcuts-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.shortcuts-hint {
  font-size: 12px;
  color: var(--n-text-color-3);
}

.shortcut-group {
  margin-bottom: 16px;
}

.shortcut-group-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--n-text-color);
  margin-bottom: 8px;
  padding-bottom: 4px;
  border-bottom: 1px solid var(--n-border-color);
}

.shortcut-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 0;
}

.shortcut-desc {
  font-size: 13px;
  color: var(--n-text-color);
}

.shortcut-binding {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 160px;
  padding: 4px 12px;
  border: 1px solid var(--n-border-color);
  border-radius: 4px;
  background: var(--n-color);
  cursor: pointer;
  font-size: 12px;
  font-family: monospace;
  transition: all 0.2s;
}

.shortcut-binding:hover {
  border-color: var(--n-color-target);
}

.shortcut-binding.recording {
  border-color: var(--n-primary-color);
  background: var(--n-primary-color-hover);
  animation: pulse 1s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.8; }
}

.recording-text {
  color: var(--n-primary-color);
  font-family: inherit;
}

.reset-btn {
  opacity: 0.5;
  transition: opacity 0.2s;
}

.reset-btn:hover {
  opacity: 1;
}
</style>