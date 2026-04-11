<script setup lang="ts">
import {
  NSelect,
  NInput,
  NButton,
  NIcon,
  NDropdown,
  NBadge,
  NTabs,
  NTabPane,
  useMessage,
  NProgress,
} from 'naive-ui'
import { ChevronDownOutline, StopOutline } from '@vicons/ionicons5'
import { ref, computed, onMounted, onUnmounted, h } from 'vue'
import { useSettingsStore } from '@/stores'
import { invoke } from '@tauri-apps/api/core'
import { useRequestStore, useEnvironmentStore, useHistoryStore } from '@/stores'
import { useI18n } from '@/composables/useI18n'
import { useUrlAutocomplete } from '@/composables/useUrlAutocomplete'
import ParamsEditor from './ParamsEditor.vue'
import HeadersEditor from './HeadersEditor.vue'
import BodyEditor from './BodyEditor.vue'
import AuthEditor from './AuthEditor.vue'
import ScriptEditor from './ScriptEditor.vue'
import SaveRequestModal from '@/components/common/SaveRequestModal.vue'
import type { SelectOption } from 'naive-ui'
import type { Response, HttpMethod, Request } from '@/types'
import { HTTP_METHOD_COLORS } from '@/types'

const message = useMessage()
const settingsStore = useSettingsStore()
const requestStore = useRequestStore()
const environmentStore = useEnvironmentStore()
const historyStore = useHistoryStore()
const { t } = useI18n()
const {
  suggestions,
  showSuggestions,
  selectedIndex,
  updateInput,
  handleKeyDown: handleAutocompleteKeyDown,
  hideSuggestions,
} = useUrlAutocomplete()

const currentTab = ref('params')
const showSaveModal = ref(false)
const saveMode = ref<'save' | 'save-as'>('save')

const methodOptions = [
  { label: 'GET', value: 'GET' },
  { label: 'POST', value: 'POST' },
  { label: 'PUT', value: 'PUT' },
  { label: 'DELETE', value: 'DELETE' },
  { label: 'PATCH', value: 'PATCH' },
  { label: 'OPTIONS', value: 'OPTIONS' },
  { label: 'HEAD', value: 'HEAD' },
]

// 使用统一的方法颜色定义
const methodColors = HTTP_METHOD_COLORS

function renderMethodLabel(option: { label: string; value: string }) {
  const colors = methodColors[option.value as HttpMethod] || methodColors.GET
  return h(
    'span',
    {
      style: {
        color: colors.color,
        background: colors.background,
        fontSize: '12px',
        fontWeight: 700,
        padding: '2px 6px',
        borderRadius: '3px',
        letterSpacing: '0.5px',
      }
    },
    option.label
  )
}

function renderMethodTag(props: { option: SelectOption; handleClose: () => void }) {
  const value = typeof props.option.value === 'string' ? props.option.value : 'GET'
  const colors = methodColors[value as HttpMethod] || methodColors.GET
  const label = typeof props.option.label === 'string' ? props.option.label : value
  return h(
    'span',
    {
      style: {
        color: colors.color,
        background: colors.background,
        fontSize: '12px',
        fontWeight: 700,
        padding: '2px 6px',
        borderRadius: '3px',
        letterSpacing: '0.5px',
      }
    },
    label
  )
}

const saveOptions = computed(() => [{ label: t('request.saveAs'), key: 'save-as' }])

const sendOptions = computed(() => [{ label: t('request.sendAndDownload'), key: 'send-download' }])

const tabBadges = computed(() => ({
  params: requestStore.currentRequest.params.filter((p) => p.enabled).length,
  headers: requestStore.currentRequest.headers.filter((h) => h.enabled).length,
}))

async function sendRequest(download = false) {
  if (!requestStore.currentRequest.url) {
    message.warning(t('validation.invalidUrl'))
    return
  }

  requestStore.isLoading = true
  requestStore.error = null
  requestStore.setPendingRequestId(requestStore.currentRequest.id)
  requestStore.resetProgress()

  try {
    const request = {
      ...requestStore.currentRequest,
      url: environmentStore.replaceVariables(requestStore.currentRequest.url),
      headers: requestStore.currentRequest.headers.map((h) => ({
        ...h,
        value: environmentStore.replaceVariables(h.value),
      })),
      params: requestStore.currentRequest.params.map((p) => ({
        ...p,
        value: environmentStore.replaceVariables(p.value),
      })),
      body: (() => {
        const body = requestStore.currentRequest.body
        if (body.mode === 'raw' && body.raw) {
          return { ...body, raw: environmentStore.replaceVariables(body.raw) }
        }
        if (body.mode === 'urlencoded' && body.urlencoded) {
          return {
            ...body,
            urlencoded: body.urlencoded.map((u) => ({
              ...u,
              value: environmentStore.replaceVariables(u.value),
            })),
          }
        }
        if (body.mode === 'form-data' && body.formData) {
          return {
            ...body,
            formData: body.formData.map((f) => ({
              ...f,
              value: environmentStore.replaceVariables(f.value),
            })),
          }
        }
        return body
      })(),
      returnBytes: download,
      timeout: settingsStore.settings?.timeout ?? 30000,
      proxy: settingsStore.settings?.proxy?.enabled ? settingsStore.settings.proxy : undefined,
      followRedirects: settingsStore.settings?.followRedirects,
      verifySsl: settingsStore.settings?.verifySsl,
    }

    const response = await invoke<Response>('send_http_request', {
      request,
      historyLimit: settingsStore.settings?.historyLimit ?? 100,
    })

    requestStore.setUploadProgress(100)
    requestStore.setDownloadProgress(100)

    historyStore.addHistory({
      requestId: requestStore.currentRequest.id,
      method: requestStore.currentRequest.method,
      url: request.url,
      status: response.status,
      responseTime: response.responseTime,
      responseSize: response.responseSize,
      params: requestStore.currentRequest.params,
      headers: requestStore.currentRequest.headers,
      body: requestStore.currentRequest.body,
      auth: requestStore.currentRequest.auth,
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
    setTimeout(() => requestStore.resetProgress(), 1000)
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

  let fileData: Uint8Array<ArrayBuffer>
  if (response.bodyBytes) {
    // 二进制响应：使用原始字节
    const buffer = new ArrayBuffer(response.bodyBytes.length)
    fileData = new Uint8Array(buffer)
    fileData.set(response.bodyBytes)
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

function handleUrlInput(value: string) {
  requestStore.updateUrl(value)
  updateInput(value)
}

function handleUrlKeyDown(event: KeyboardEvent) {
  const handled = handleAutocompleteKeyDown(event, (suggestion: { url: string; method: string }) => {
    requestStore.updateUrl(suggestion.url)
    requestStore.updateMethod(suggestion.method as Request['method'])
  })
  if (!handled && event.key === 'Enter') {
    event.preventDefault()
    sendRequest()
  }
}

function selectSuggestion(suggestion: { url: string; method: string }) {
  requestStore.updateUrl(suggestion.url)
  requestStore.updateMethod(suggestion.method as Request['method'])
  hideSuggestions()
}

function handleUrlFocus() {
  if (requestStore.currentRequest.url.length >= 2 && suggestions.value.length > 0) {
    showSuggestions.value = true
  }
}

function handleUrlBlur() {
  setTimeout(hideSuggestions, 200)
}

onMounted(() => {
  window.addEventListener('send-request', handleSendRequest)
  updateInput(requestStore.currentRequest.url, false)
})

onUnmounted(() => {
  window.removeEventListener('send-request', handleSendRequest)
})
</script>

<template>
  <div class="request-panel" role="region" aria-label="请求面板">
    <div class="url-bar" role="toolbar" aria-label="请求工具栏">
      <NSelect
        :value="requestStore.currentRequest.method"
        :options="methodOptions"
        :consistent-menu-width="false"
        :render-label="renderMethodLabel"
        :render-tag="renderMethodTag"
        style="width: 110px"
        aria-label="HTTP 方法选择"
        @update:value="requestStore.updateMethod"
      />
      <div class="url-input-container">
        <NInput
          :value="requestStore.currentRequest.url"
          :placeholder="t('request.enterUrl')"
          autocomplete="off"
          spellcheck="false"
          class="url-input"
          aria-label="请求 URL 输入"
          @update:value="handleUrlInput"
          @keydown="handleUrlKeyDown"
          @focus="handleUrlFocus"
          @blur="handleUrlBlur"
        />
        <div v-if="showSuggestions && suggestions.length > 0" class="url-suggestions">
          <div
            v-for="(suggestion, index) in suggestions"
            :key="suggestion.url"
            class="suggestion-item"
            :class="{ selected: index === selectedIndex }"
            @mousedown.prevent="selectSuggestion(suggestion)"
          >
            <span class="suggestion-method" :style="{ color: HTTP_METHOD_COLORS[suggestion.method as keyof typeof HTTP_METHOD_COLORS]?.color }">
              {{ suggestion.method }}
            </span>
            <span class="suggestion-url">{{ suggestion.url }}</span>
            <span class="suggestion-count">{{ suggestion.count }}次</span>
          </div>
        </div>
      </div>
      <NButton
        v-if="requestStore.isLoading"
        type="error"
        class="send-btn"
        aria-label="取消请求"
        @click="cancelRequest"
      >
        <template #icon>
          <NIcon :component="StopOutline" />
        </template>
        {{ t('request.cancel') }}
      </NButton>
      <div v-else class="send-btn-group" role="group" aria-label="发送操作">
        <NButton
          type="primary"
          class="send-btn"
          :loading="requestStore.isLoading"
          :aria-busy="requestStore.isLoading"
          @click="sendRequest()"
        >
          {{ t('request.send') }}
        </NButton>
        <NDropdown :options="sendOptions" placement="bottom-end" @select="handleSendSelect">
          <NButton type="primary" class="send-btn-dropdown" aria-label="发送选项">
            <NIcon :component="ChevronDownOutline" />
          </NButton>
        </NDropdown>
      </div>

      <!-- 进度指示 -->
      <div v-if="requestStore.isLoading" class="progress-container">
        <div
          v-if="requestStore.uploadProgress > 0 && requestStore.uploadProgress < 100"
          class="progress-item"
        >
          <span class="progress-label">上传</span>
          <NProgress
            :percentage="requestStore.uploadProgress"
            :show-indicator="false"
            :height="6"
            status="success"
            class="progress-bar"
          />
        </div>
        <div
          v-if="
            requestStore.downloadProgress > 0 && requestStore.downloadProgress < 100
          "
          class="progress-item"
        >
          <span class="progress-label">下载</span>
          <NProgress
            :percentage="requestStore.downloadProgress"
            :show-indicator="false"
            :height="6"
            status="success"
            class="progress-bar"
          />
        </div>
      </div>
      <div class="save-btn-group" role="group" aria-label="保存操作">
        <NButton class="save-btn" @click="handleSaveDirect">{{ t('request.save') }}</NButton>
        <NDropdown :options="saveOptions" placement="bottom-end" @select="handleSave">
          <NButton class="save-btn-dropdown" aria-label="保存选项">
            <NIcon :component="ChevronDownOutline" />
          </NButton>
        </NDropdown>
      </div>
    </div>

    <div class="request-tabs-container">
      <NTabs v-model:value="currentTab" type="line" role="tablist" aria-label="请求选项卡">
        <NTabPane name="params" display-directive="show">
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

        <NTabPane name="authorization" display-directive="show">
          <template #tab>{{ t('request.authorization') }}</template>
          <div class="tab-scroll-content">
            <AuthEditor
              :auth="requestStore.currentRequest.auth"
              :request-id="requestStore.currentRequest.id"
              @update:auth="requestStore.updateAuth"
            />
          </div>
        </NTabPane>

        <NTabPane name="headers" display-directive="show">
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

        <NTabPane name="body" display-directive="show">
          <template #tab>{{ t('request.body') }}</template>
          <div class="tab-scroll-content">
            <BodyEditor
              :body="requestStore.currentRequest.body"
              @update:body="requestStore.updateBody"
            />
          </div>
        </NTabPane>

        <NTabPane name="preScript" display-directive="show">
          <template #tab>{{ t('request.preRequestScript') }}</template>
          <div class="tab-editor-content">
            <ScriptEditor
              type="preRequest"
              :script="requestStore.currentRequest.preScript"
              @update:script="(s: string) => requestStore.updateRequest({ preScript: s })"
            />
          </div>
        </NTabPane>

        <NTabPane name="tests" display-directive="show">
          <template #tab>{{ t('request.testScript') }}</template>
          <div class="tab-editor-content">
            <ScriptEditor
              type="test"
              :script="requestStore.currentRequest.postScript"
              @update:script="(s: string) => requestStore.updateRequest({ postScript: s })"
            />
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
  font-size: var(--font-size-compact-lg);
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

/* 进度指示 */
.progress-container {
  position: absolute;
  bottom: 0;
  left: 12px;
  right: 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 4px 0;
  background: var(--n-color-modal);
  border-top: 1px solid var(--n-border-color);
}

.progress-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.progress-label {
  font-size: var(--font-size-compact-sm);
  color: var(--n-text-color-3);
  min-width: 30px;
  font-weight: 500;
}

.progress-bar {
  flex: 1;
  transition: all 0.3s ease;
}

.url-bar {
  position: relative;
}

.url-input-container {
  flex: 1;
  position: relative;
}

.url-suggestions {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  background: #fff;
  border: 1px solid var(--n-border-color);
  border-radius: 4px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 1000;
  max-height: 300px;
  overflow-y: auto;
  margin-top: 4px;
}

.suggestion-item {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  cursor: pointer;
  transition: background-color 0.15s;
  gap: 8px;
}

.suggestion-item:hover {
  background: rgba(24, 160, 88, 0.1);
}

.suggestion-item.selected {
  background: rgba(24, 160, 88, 0.15);
}

.suggestion-method {
  font-weight: 600;
  font-size: 12px;
  min-width: 50px;
}

.suggestion-url {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 13px;
}

.suggestion-count {
  font-size: 11px;
  color: var(--n-text-color-3);
  min-width: 40px;
  text-align: right;
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
  display: flex;
  flex-direction: column;
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

.tab-editor-content {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  padding: 8px;
  position: relative;
  z-index: 1;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
}

.tab-with-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.tab-with-badge :deep(.n-badge) {
  margin-left: 2px;
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
}

:global([data-theme='dark']) .url-suggestions {
  background: #2d2d2d;
}

:global([data-theme='dark']) .suggestion-item:hover {
  background: rgba(99, 226, 183, 0.15);
}

:global([data-theme='dark']) .suggestion-item.selected {
  background: rgba(99, 226, 183, 0.2);
}
</style>
