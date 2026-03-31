<script setup lang="ts">
import {
  NSelect,
  NInput,
  NInputNumber,
  NButton,
  NIcon,
  NDropdown,
  NBadge,
  NTabs,
  NTabPane,
  NSwitch,
  NTag,
  useMessage,
} from 'naive-ui'
import { SettingsOutline, ChevronDownOutline, StopOutline } from '@vicons/ionicons5'
import { ref, computed, onMounted, onUnmounted, h } from 'vue'
import { useSettingsStore } from '@/stores'
import { invoke } from '@tauri-apps/api/core'
import { useRequestStore, useEnvironmentStore, useHistoryStore } from '@/stores'
import { useI18n } from '@/composables/useI18n'
import ParamsEditor from './ParamsEditor.vue'
import HeadersEditor from './HeadersEditor.vue'
import BodyEditor from './BodyEditor.vue'
import AuthEditor from './AuthEditor.vue'
import ScriptEditor from './ScriptEditor.vue'
import SaveRequestModal from '@/components/common/SaveRequestModal.vue'
import type { Response } from '@/types'

const message = useMessage()
const settingsStore = useSettingsStore()
const requestStore = useRequestStore()
const environmentStore = useEnvironmentStore()
const historyStore = useHistoryStore()
const { t } = useI18n()

const currentTab = ref('params')
const showSaveModal = ref(false)
const saveMode = ref<'save' | 'save-as'>('save')

const methodTypes: Record<string, 'success' | 'info' | 'warning' | 'error' | 'primary'> = {
  GET: 'info',
  POST: 'success',
  PUT: 'warning',
  DELETE: 'error',
  PATCH: 'primary',
  OPTIONS: 'info',
  HEAD: 'info',
}

const methodOptions = [
  { label: 'GET', value: 'GET' },
  { label: 'POST', value: 'POST' },
  { label: 'PUT', value: 'PUT' },
  { label: 'DELETE', value: 'DELETE' },
  { label: 'PATCH', value: 'PATCH' },
  { label: 'OPTIONS', value: 'OPTIONS' },
  { label: 'HEAD', value: 'HEAD' },
]

function renderMethodLabel(option: { label: string; value: string }) {
  const tagType = methodTypes[option.value] || 'info'
  return h(
    NTag,
    {
      type: tagType,
      size: 'small',
      bordered: false,
    },
    { default: () => option.label }
  )
}

const saveOptions = computed(() => [{ label: t('request.saveAs'), key: 'save-as' }])

const sendOptions = computed(() => [{ label: t('request.sendAndDownload'), key: 'send-download' }])

const tabBadges = computed(() => ({
  params: requestStore.currentRequest.params.filter((p) => p.enabled).length,
  headers: requestStore.currentRequest.headers.filter((h) => h.enabled).length,
}))

const timeoutValue = computed({
  get: () => settingsStore.settings?.timeout ?? 30000,
  set: (val: number) => settingsStore.updateSettings({ timeout: val }),
})

const downloadPath = computed({
  get: () => settingsStore.settings?.downloadPath ?? '',
  set: (val: string) => settingsStore.updateSettings({ downloadPath: val }),
})

const downloadAsk = computed({
  get: () => settingsStore.settings?.downloadAsk ?? true,
  set: (val: boolean) => settingsStore.updateSettings({ downloadAsk: val }),
})

async function sendRequest(download = false) {
  if (!requestStore.currentRequest.url) {
    message.warning(t('validation.invalidUrl'))
    return
  }

  requestStore.isLoading = true
  requestStore.error = null
  requestStore.setPendingRequestId(requestStore.currentRequest.id)

  try {
    const request = {
      ...requestStore.currentRequest,
      url: environmentStore.replaceVariables(requestStore.currentRequest.url),
      headers: requestStore.currentRequest.headers.map((h) => ({
        ...h,
        value: environmentStore.replaceVariables(h.value),
      })),
      returnBytes: download,
      timeout: settingsStore.settings?.timeout ?? 30000,
    }

    const response = await invoke<Response>('send_http_request', {
      request,
      historyLimit: settingsStore.settings?.historyLimit ?? 100,
    })

    historyStore.addHistory({
      requestId: requestStore.currentRequest.id,
      method: requestStore.currentRequest.method,
      url: request.url,
      status: response.status,
      responseTime: response.responseTime,
      responseSize: response.responseSize,
    })

    if (download) {
      await downloadResponse(response)
      // 下载模式下，清空 body 不显示在响应面板
      response.body = ''
    }

    if (requestStore.activeTabId) {
      requestStore.setResponse(requestStore.activeTabId, response)
    }
  } catch (e) {
    const errorMsg = String(e)
    if (errorMsg.includes('取消') || errorMsg.includes('cancelled')) {
      message.info(t('errors.requestCancelled'))
    } else {
      requestStore.error = errorMsg
      message.error(`${t('errors.unknownError')}: ${e}`)
    }
  } finally {
    requestStore.isLoading = false
    requestStore.setPendingRequestId(null)
  }
}

async function downloadResponse(response: Response) {
  const contentType = response.headers['content-type'] || ''
  const contentDisposition = response.headers['content-disposition'] || ''

  let filename = 'response'

  // 解析 filename，优先使用 RFC 5987 编码的 filename*
  const filenameStarMatch = contentDisposition.match(/filename\*=([^;\n]+)/i)
  if (filenameStarMatch && filenameStarMatch[1]) {
    // 格式: UTF-8''encoded-name 或 iso-8859-1''encoded-name
    const match = filenameStarMatch[1].match(/^([^']+)''(.+)$/)
    if (match) {
      try {
        filename = decodeURIComponent(match[2])
      } catch {
        filename = match[2]
      }
    }
  } else {
    // 标准 filename="xxx" 格式
    const filenameMatch = contentDisposition.match(/filename[^;=\n]*=((['"]).*?\2|[^;\n]*)/)
    if (filenameMatch && filenameMatch[1]) {
      filename = filenameMatch[1].replace(/['"]/g, '')
      // 尝试解码 URL 编码的文件名
      try {
        filename = decodeURIComponent(filename)
      } catch {
        // URL 编码失败的保留原样
      }
    } else {
      const ext = getExtensionFromContentType(contentType)
      filename = `response${ext}`
    }
  }

  let fileData: Uint8Array
  if (response.bodyBytes) {
    // 二进制响应：使用原始字节
    fileData = new Uint8Array(response.bodyBytes)
  } else {
    // 文本响应：直接编码
    const encoder = new TextEncoder()
    fileData = encoder.encode(response.body)
  }

  const downloadAsk = settingsStore.settings?.downloadAsk ?? true
  const downloadPath = settingsStore.settings?.downloadPath ?? ''

  // 检查 Tauri 插件是否可用
  const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

  // 确定保存路径
  let savePath: string | null = null

  if (isTauri && downloadAsk) {
    // 每次询问 - 使用 save 对话框，用户可重命名
    try {
      const { save } = await import('@tauri-apps/plugin-dialog')
      savePath = await save({
        defaultPath: filename,
      })
    } catch {
      savePath = null
    }
  } else if (isTauri && downloadPath) {
    // 使用设置的路径
    savePath = downloadPath + '/' + filename
  }

  if (savePath) {
    // 使用 Tauri 保存到指定路径
    try {
      const { writeFile } = await import('@tauri-apps/plugin-fs')
      await writeFile(savePath, fileData)
      message.success(t('common.downloadSuccess'))
      return
    } catch (e) {
      console.error('保存文件失败:', e)
      message.error(String(e))
      return
    }
  } else if (!downloadAsk || !isTauri) {
    // 浏览器默认下载（非 Tauri 或关闭了每次询问）
    let blob: Blob
    try {
      blob = new Blob([fileData], { type: contentType })
    } catch {
      blob = new Blob([response.body], { type: contentType || 'text/plain' })
    }

    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = filename
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
    message.success(t('common.downloadSuccess'))
    return
  }
  // 用户取消对话框，不执行任何下载
}

function getExtensionFromContentType(contentType: string): string {
  const type = contentType.split(';')[0].trim().toLowerCase()
  const extMap: Record<string, string> = {
    'application/json': '.json',
    'text/html': '.html',
    'text/css': '.css',
    'text/javascript': '.js',
    'application/javascript': '.js',
    'text/plain': '.txt',
    'application/xml': '.xml',
    'text/xml': '.xml',
    'application/pdf': '.pdf',
    'image/png': '.png',
    'image/jpeg': '.jpg',
    'image/gif': '.gif',
    'image/svg+xml': '.svg',
    'application/zip': '.zip',
    'application/octet-stream': '.bin',
  }
  return extMap[type] || '.bin'
}

function handleSendSelect(_key?: string) {
  sendRequest(true)
}

async function cancelRequest() {
  const cancelled = await requestStore.cancelCurrentRequest()
  if (cancelled) {
    message.info(t('errors.requestCancelled'))
  } else {
    message.warning(t('errors.requestCancelled'))
  }
}

function handleSave(key: string) {
  if (key === 'save' || key === 'save-direct') {
    // 已保存过的请求直接更新原数据
    if (requestStore.currentTab?.collectionId) {
      requestStore.saveRequest(requestStore.currentTab.collectionId)
      message.success(t('common.success'))
    } else {
      // 未保存过的选择集合
      saveMode.value = 'save'
      showSaveModal.value = true
    }
  } else if (key === 'save-as') {
    // 另存为都需要选择集合
    saveMode.value = 'save-as'
    showSaveModal.value = true
  }
}

function handleSaveDirect() {
  handleSave('save-direct')
}

function handleSaveRequest(data: { name: string; collectionId: string }) {
  // 更新请求名称
  requestStore.updateRequest({ name: data.name })
  // 保存到指定集合
  requestStore.saveRequest(data.collectionId)
  message.success(t('common.success'))
}

async function handleCreateCollection(name: string) {
  const collection = await requestStore.createCollection(name)
  // 自动选中新创建的集合并保存请求
  await requestStore.saveRequest(collection.id)
  message.success(t('common.success'))
  showSaveModal.value = false
}

function handleSendRequest() {
  sendRequest()
}

onMounted(() => {
  window.addEventListener('send-request', handleSendRequest)
})

onUnmounted(() => {
  window.removeEventListener('send-request', handleSendRequest)
})
</script>

<template>
  <div class="request-panel">
    <div class="url-bar">
      <NSelect
        :value="requestStore.currentRequest.method"
        :options="methodOptions"
        :consistent-menu-width="false"
        :render-label="renderMethodLabel"
        style="width: 100px"
        @update:value="requestStore.updateMethod"
      />
      <NInput
        :value="requestStore.currentRequest.url"
        :placeholder="t('request.enterUrl')"
        class="url-input"
        @update:value="requestStore.updateUrl"
        @keyup.enter="sendRequest()"
      />
      <NButton v-if="requestStore.isLoading" type="error" class="send-btn" @click="cancelRequest">
        <template #icon>
          <NIcon :component="StopOutline" />
        </template>
        {{ t('request.cancel') }}
      </NButton>
      <div v-else class="send-btn-group">
        <NButton
          type="primary"
          class="send-btn"
          :loading="requestStore.isLoading"
          @click="sendRequest()"
        >
          {{ t('request.send') }}
        </NButton>
        <NDropdown :options="sendOptions" placement="bottom-end" @select="handleSendSelect">
          <NButton type="primary" class="send-btn-dropdown">
            <NIcon :component="ChevronDownOutline" />
          </NButton>
        </NDropdown>
      </div>
      <div class="save-btn-group">
        <NButton class="save-btn" @click="handleSaveDirect">
          {{ t('request.save') }}
        </NButton>
        <NDropdown :options="saveOptions" placement="bottom-end" @select="handleSave">
          <NButton class="save-btn-dropdown">
            <NIcon :component="ChevronDownOutline" />
          </NButton>
        </NDropdown>
      </div>
    </div>

    <div class="request-tabs-container">
      <NTabs v-model:value="currentTab" type="line">
        <NTabPane name="params">
          <template #tab>
            <span class="tab-with-badge">
              {{ t('request.params') }}
              <NBadge v-if="tabBadges.params > 0" :value="tabBadges.params" :max="99" type="info" />
            </span>
          </template>
          <div class="tab-scroll-content">
            <ParamsEditor
              :params="requestStore.currentRequest.params"
              @update:params="requestStore.updateParams"
            />
          </div>
        </NTabPane>

        <NTabPane name="authorization">
          <template #tab>{{ t('request.authorization') }}</template>
          <div class="tab-scroll-content">
            <AuthEditor
              :auth="requestStore.currentRequest.auth"
              :request-id="requestStore.currentRequest.id"
              @update:auth="requestStore.updateAuth"
            />
          </div>
        </NTabPane>

        <NTabPane name="headers">
          <template #tab>
            <span class="tab-with-badge">
              {{ t('request.headers') }}
              <NBadge
                v-if="tabBadges.headers > 0"
                :value="tabBadges.headers"
                :max="99"
                type="info"
              />
            </span>
          </template>
          <div class="tab-scroll-content">
            <HeadersEditor
              :headers="requestStore.currentRequest.headers"
              @update:headers="requestStore.updateHeaders"
            />
          </div>
        </NTabPane>

        <NTabPane name="body">
          <template #tab>{{ t('request.body') }}</template>
          <div class="tab-scroll-content">
            <BodyEditor
              :body="requestStore.currentRequest.body"
              @update:body="requestStore.updateBody"
            />
          </div>
        </NTabPane>

        <NTabPane name="preScript">
          <template #tab>{{ t('request.preRequestScript') }}</template>
          <div class="tab-scroll-content">
            <ScriptEditor
              :script="requestStore.currentRequest.preScript"
              @update:script="(s: string) => requestStore.updateRequest({ preScript: s })"
            />
          </div>
        </NTabPane>

        <NTabPane name="tests">
          <template #tab>{{ t('request.testScript') }}</template>
          <div class="tab-scroll-content">
            <ScriptEditor
              :script="requestStore.currentRequest.postScript"
              @update:script="(s: string) => requestStore.updateRequest({ postScript: s })"
            />
          </div>
        </NTabPane>

        <NTabPane name="settings">
          <template #tab>
            <NIcon :component="SettingsOutline" />
          </template>
          <div class="tab-scroll-content">
            <div class="settings-panel">
              <div class="setting-item">
                <label>{{ t('request.timeout') }}</label>
                <NInputNumber
                  v-model:value="timeoutValue"
                  :min="1000"
                  :max="300000"
                  :step="1000"
                  style="width: 150px"
                />
              </div>
              <div class="setting-item">
                <label>{{ t('request.downloadPath') }}</label>
                <NInput
                  v-model:value="downloadPath"
                  :placeholder="t('request.downloadPathPlaceholder')"
                  style="width: 250px"
                />
              </div>
              <div class="setting-item">
                <label>{{ t('request.downloadAsk') }}</label>
                <NSwitch v-model:value="downloadAsk" />
              </div>
            </div>
          </div>
        </NTabPane>
      </NTabs>
    </div>

    <SaveRequestModal
      v-model:show="showSaveModal"
      :request="requestStore.currentRequest"
      :collections="requestStore.collections"
      :default-collection-id="requestStore.currentTab?.collectionId"
      @save="handleSaveRequest"
      @create-collection="handleCreateCollection"
    />
  </div>
</template>

<style scoped>
.request-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 0;
  background: var(--n-color);
}

.url-bar {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  gap: 8px;
  border-bottom: 1px solid var(--n-border-color);
  flex-shrink: 0;
  min-height: 44px;
  background: var(--n-color-modal);
}

.url-input {
  flex: 1;
}

.url-input :deep(.n-input__input-el) {
  font-size: 14px;
  font-weight: 500;
}

.url-input :deep(.n-input__input-el textarea) {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.url-input :deep(.n-input__input-el textarea:focus) {
  overflow: visible;
  text-overflow: unset;
  white-space: normal;
}

.send-btn {
  padding: 0 24px;
  border-top-right-radius: 0;
  border-bottom-right-radius: 0;
}

.send-btn-group {
  display: flex;
  align-items: center;
}

.send-btn-group-inner {
  display: flex;
  align-items: center;
}

.send-btn-dropdown {
  padding: 0 8px;
  border-top-left-radius: 0;
  border-bottom-left-radius: 0;
  margin-left: 1px;
}

.save-btn {
  padding: 0 16px;
  border-top-right-radius: 0;
  border-bottom-right-radius: 0;
}

.save-btn-group {
  display: flex;
  align-items: center;
}

.save-btn-dropdown {
  padding: 0 6px;
  border-top-left-radius: 0;
  border-bottom-left-radius: 0;
  margin-left: 1px;
}

.save-btn-group :deep(.n-dropdown-menu) {
  transform: translateX(-100%);
}

/* 调整下拉菜单位置，使其显示在发送按钮正下方 */
.send-btn-group :deep(.n-dropdown-menu) {
  transform: translateX(-100%);
}

.send-btn-group :deep(.n-popover) {
  padding: 4px;
}

.send-btn-group :deep(.n-popover .n-button) {
  width: 100%;
  min-width: 120px;
}

.send-btn-group :deep(.n-popover .n-button) {
  width: 100%;
  min-width: 120px;
}

.request-tabs-container {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.request-tabs-container :deep(.n-tabs) {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.request-tabs-container :deep(.n-tabs-nav) {
  flex-shrink: 0;
  padding: 0 8px;
  background: var(--n-color-modal);
  border-bottom: 1px solid var(--n-border-color);
}

.request-tabs-container :deep(.n-tabs-tab) {
  padding: 6px 10px;
  font-weight: 500;
  font-size: 13px;
  transition: all 0.2s ease;
}

.request-tabs-container :deep(.n-tabs-tab:hover) {
  color: var(--n-primary-color);
}

.request-tabs-container :deep(.n-tabs-pane-wrapper) {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.request-tabs-container :deep(.n-tab-pane) {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  padding: 0;
  position: relative;
  display: flex;
  flex-direction: column;
}

.tab-scroll-content {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 8px;
  position: relative;
  z-index: 1;
  box-sizing: border-box;
}

.tab-with-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.tab-with-badge :deep(.n-badge) {
  margin-left: 2px;
}

.settings-panel {
  padding: 0;
}

.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 0;
  border-bottom: 1px solid var(--n-border-color);
}

.setting-item label {
  font-size: 14px;
  font-weight: 500;
}

/* 响应式设计 */
@media (max-width: 1200px) {
  .url-bar {
    padding: 6px 10px;
    gap: 6px;
    min-height: 40px;
  }

  .tab-scroll-content {
    padding: 6px;
  }

  .request-tabs-container :deep(.n-tabs-tab) {
    padding: 5px 8px;
    font-size: 12px;
  }
}

@media (max-width: 900px) {
  .url-bar {
    padding: 4px 8px;
    gap: 4px;
    min-height: 36px;
  }

  .url-input :deep(.n-input__input-el) {
    font-size: 13px;
  }

  .tab-scroll-content {
    padding: 4px;
  }

  .request-tabs-container :deep(.n-tabs-tab) {
    padding: 4px 6px;
    font-size: 11px;
  }
}

@media (min-width: 1600px) {
  .url-bar {
    padding: 10px 14px;
    gap: 10px;
    min-height: 48px;
  }

  .url-input :deep(.n-input__input-el) {
    font-size: 15px;
  }

  .tab-scroll-content {
    padding: 10px;
  }

  .request-tabs-container :deep(.n-tabs-tab) {
    padding: 8px 12px;
    font-size: 14px;
  }

  .setting-item {
    padding: 12px 0;
  }

  .setting-item label {
    font-size: 15px;
  }
}
</style>
