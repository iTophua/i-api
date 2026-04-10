<script setup lang="ts">
import { NTabs, NTabPane, NButton, NIcon, NEmpty, NScrollbar, NTag } from 'naive-ui'
import { CopyOutline, CodeOutline } from '@vicons/ionicons5'
import { computed, ref, shallowRef, onMounted } from 'vue'
import { useRequestStore } from '@/stores'
import { formatJsonString, detectLanguage } from '@/composables/useMonacoEditor'
import type { Response } from '@/types'
import ResponseStats from './ResponseStats.vue'
import TestResultsPanel from './TestResultsPanel.vue'
import LargeTextEditor from '@/components/common/LargeTextEditor.vue'

const MonacoEditor = shallowRef<any>(null)
const editorRef = ref<any>(null)

onMounted(async () => {
  const mod = await import('@/components/common/MonacoEditor.vue')
  MonacoEditor.value = mod.default
})

const requestStore = useRequestStore()

const currentTab = ref('body')

const rawResponse = computed<Response | null>(() => requestStore.currentResponse)

const isLarge = computed(() => rawResponse.value ? requestStore.isLargeResponse(rawResponse.value) : false)

const response = computed<Response | null>(() => {
  if (!rawResponse.value) return null
  return requestStore.getOptimizedResponse(rawResponse.value)
})

const status = computed(() => response.value?.status || 0)
const statusText = computed(() => response.value?.statusText || '')
const responseTime = computed(() => response.value?.responseTime || 0)
const responseSize = computed(() => response.value?.responseSize || 0)

const statusType = computed(() => {
  if (!response.value) return 'default'
  if (status.value >= 200 && status.value < 300) return 'success'
  if (status.value >= 300 && status.value < 400) return 'info'
  if (status.value >= 400 && status.value < 500) return 'warning'
  return 'error'
})

// 自动美化的响应内容
const formattedResponseContent = computed(() => {
  const content = response.value?.body || ''
  if (responseLanguage.value === 'json' && content) {
    try {
      return formatJsonString(content)
    } catch {
      return content
    }
  }
  return content
})

const responseContent = computed(() => response.value?.body || '')

const responseLanguage = computed(() => {
  if (!responseContent.value) return 'plaintext'

  const contentType = response.value?.headers?.['content-type'] || ''
  if (contentType.includes('json')) return 'json'
  if (contentType.includes('xml')) return 'xml'
  if (contentType.includes('html')) return 'html'
  if (contentType.includes('javascript')) return 'javascript'

  return detectLanguage(responseContent.value)
})

const formattedSize = computed(() => {
  const size = responseSize.value
  if (size < 1024) return `${size} B`
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`
  return `${(size / 1024 / 1024).toFixed(2)} MB`
})

// 判断是否使用大型文本编辑器
const shouldUseLargeEditor = computed(() => {
  const content = responseContent.value
  if (!content) return false
  const lineCount = content.split('\n').length
  return lineCount > 1000 || content.length > 500000
})

const isFormatted = ref(false)

function formatResponse() {
  if (responseLanguage.value === 'json' && responseContent.value) {
    if (isFormatted.value) {
      // 压缩：移除多余空白
      const compressed = responseContent.value.replace(/\s+/g, ' ').trim()
      if (editorRef.value) {
        editorRef.value.setValue(compressed)
      }
      isFormatted.value = false
    } else {
      // 美化
      const formatted = formatJsonString(responseContent.value)
      if (formatted !== responseContent.value && editorRef.value) {
        editorRef.value.setValue(formatted)
      }
      isFormatted.value = true
    }
  }
}

function copyResponse() {
  const contentToCopy = editorRef.value?.getValue?.() || responseContent.value
  if (contentToCopy) {
    navigator.clipboard.writeText(contentToCopy)
  }
}

const headerList = computed(() => {
  if (!response.value?.headers) return []
  return Object.entries(response.value.headers).map(([key, value]) => ({ key, value }))
})

const cookieList = computed(() => response.value?.cookies || [])

const testResults = computed(() => {
  const currentTab = requestStore.currentTab
  if (!currentTab?.testResults) return null
  return currentTab.testResults
})
</script>

<template>
  <div class="response-panel">
    <!-- 状态栏 -->
    <div v-if="response" class="response-status-bar">
      <div class="status-info">
        <NTag :type="statusType" size="small"> {{ status }} {{ statusText }} </NTag>
        <span class="meta-item">
          <span class="meta-label">Time</span>
          <span class="meta-value">{{ responseTime }} ms</span>
        </span>
        <span class="meta-item">
          <span class="meta-label">Size</span>
          <span class="meta-value">{{ formattedSize }}</span>
        </span>
        <NTag v-if="isLarge" type="warning" size="small">
          大文件响应（已优化）
        </NTag>
      </div>
      <div class="status-actions">
        <!-- 按钮已移动到 Body tab 中 -->
      </div>
    </div>

    <!-- 无响应状态 -->
    <div v-if="!response" class="empty-response">
      <div class="empty-icon">
        <svg viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
          <circle
            cx="32"
            cy="32"
            r="30"
            stroke="currentColor"
            stroke-width="2"
            stroke-dasharray="8 4"
          />
          <path
            d="M20 28L28 36L44 20"
            stroke="currentColor"
            stroke-width="3"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </div>
      <p class="empty-text">发送请求后查看响应</p>
      <p class="empty-hint">按 Enter 发送请求，或点击发送按钮</p>
    </div>

    <!-- 响应内容 -->
    <div v-if="response" class="response-content">
      <NTabs v-model:value="currentTab" type="line">
        <NTabPane name="stats">
          <template #tab>概览</template>
          <ResponseStats />
        </NTabPane>

        <NTabPane name="body">
          <template #tab>Body</template>
          <div class="body-actions">
            <NButton v-if="responseLanguage === 'json'" text size="tiny" @click="formatResponse">
              <template #icon>
                <NIcon :component="CodeOutline" size="14" />
              </template>
              {{ isFormatted ? '压缩' : '美化' }}
            </NButton>
            <NButton text size="tiny" @click="copyResponse">
              <template #icon>
                <NIcon :component="CopyOutline" size="14" />
              </template>
              复制
            </NButton>
          </div>
          <div class="editor-wrapper">
            <LargeTextEditor
              v-if="shouldUseLargeEditor"
              :model-value="formattedResponseContent"
              :language="responseLanguage"
            />
            <MonacoEditor
              v-else-if="MonacoEditor"
              ref="editorRef"
              :model-value="formattedResponseContent"
              :language="responseLanguage"
              read-only
              height="100%"
              :enable-performance-optimization="true"
            />
          </div>
        </NTabPane>

        <NTabPane name="headers">
          <template #tab>
            <span class="tab-with-count">
              Headers
              <NTag v-if="headerList.length" size="small" :bordered="false">{{
                headerList.length
              }}</NTag>
            </span>
          </template>
          <NScrollbar class="table-scroll">
            <div class="header-table">
              <div class="table-row header">
                <div class="table-cell key">Key</div>
                <div class="table-cell value">Value</div>
              </div>
              <div v-for="header in headerList" :key="header.key" class="table-row">
                <div class="table-cell key">{{ header.key }}</div>
                <div class="table-cell value">{{ header.value }}</div>
              </div>
            </div>
          </NScrollbar>
        </NTabPane>

        <NTabPane name="cookies">
          <template #tab>
            <span class="tab-with-count">
              Cookies
              <NTag v-if="cookieList.length" size="small" :bordered="false">{{
                cookieList.length
              }}</NTag>
            </span>
          </template>
          <NScrollbar class="table-scroll">
            <div v-if="cookieList.length" class="header-table">
              <div class="table-row header">
                <div class="table-cell key">Name</div>
                <div class="table-cell value">Value</div>
              </div>
              <div v-for="cookie in cookieList" :key="cookie.name" class="table-row">
                <div class="table-cell key">{{ cookie.name }}</div>
                <div class="table-cell value">{{ cookie.value }}</div>
              </div>
            </div>
            <NEmpty v-else description="无 Cookies" style="padding: 40px" />
          </NScrollbar>
        </NTabPane>

        <NTabPane name="tests">
          <template #tab>
            <span class="tab-with-count">
              Tests
              <NTag
                v-if="testResults"
                :type="testResults.passed === testResults.total ? 'success' : 'error'"
                size="small"
                :bordered="false"
              >
                {{ testResults?.passed || 0 }}/{{ testResults?.total || 0 }}
              </NTag>
            </span>
          </template>
          <TestResultsPanel />
        </NTabPane>
      </NTabs>
    </div>
  </div>
</template>

<style scoped>
.response-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
  background: var(--n-color);
}

.response-status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-bottom: 1px solid var(--n-border-color);
  flex-shrink: 0;
  min-height: 40px;
  background: var(--n-color-modal);
}

.status-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.meta-item {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: 4px;
}

.meta-label {
  font-size: 11px;
  color: var(--n-text-color-3);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  font-weight: 500;
}

.meta-value {
  font-size: 13px;
  font-weight: 600;
  color: var(--n-primary-color);
}

.status-actions {
  display: flex;
  gap: 8px;
}

.empty-response {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--n-text-color-3);
  padding: 40px;
}

.empty-icon {
  width: 80px;
  height: 80px;
  margin-bottom: 20px;
  opacity: 0.4;
}

.empty-icon svg {
  width: 100%;
  height: 100%;
}

.empty-text {
  font-size: 15px;
  font-weight: 600;
  margin-bottom: 10px;
}

.empty-hint {
  font-size: 13px;
  color: var(--n-text-color-3);
  line-height: 1.6;
}

.response-content {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.response-content :deep(.n-tabs) {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.response-content :deep(.n-tabs-nav) {
  flex-shrink: 0;
  padding: 0 8px;
  background: var(--n-color-modal);
  border-bottom: 1px solid var(--n-border-color);
}

.response-content :deep(.n-tabs-tab) {
  padding: 6px 10px;
  font-weight: 500;
  font-size: 13px;
  transition: all 0.2s ease;
}

.response-content :deep(.n-tabs-tab:hover) {
  color: var(--n-primary-color);
}

.response-content :deep(.n-tabs-pane-wrapper) {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.response-content :deep(.n-tab-pane) {
  height: 100%;
  overflow: hidden;
  padding: 0;
  position: relative;
  display: flex;
  flex-direction: column;
}

.body-actions {
  position: absolute;
  top: 8px;
  right: 8px;
  display: flex;
  gap: 4px;
  padding: 4px 8px;
  background: var(--n-color-modal);
  border-radius: 4px;
  border: 1px solid var(--n-border-color);
  z-index: 10;
}

.editor-wrapper {
  flex: 1;
  min-height: 0;
  border: 1px solid var(--n-border-color);
  border-radius: 6px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  position: relative;
  margin: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}

.tab-with-count {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.tab-with-count :deep(.n-tag) {
  font-size: 11px;
  font-weight: 600;
}

.table-scroll {
  height: 100%;
  padding: 6px 8px;
}

.header-table {
  width: 100%;
  border-collapse: collapse;
}

.table-row {
  display: flex;
  border-bottom: 1px solid var(--n-border-color);
  transition: background-color 0.15s;
}

.table-row.header {
  background: var(--n-color-modal);
  font-weight: 600;
  font-size: 11px;
  color: var(--n-text-color-2);
  position: sticky;
  top: 0;
  z-index: 1;
}

.table-row:not(.header):hover {
  background: var(--n-color-hover);
}

.table-cell {
  flex: 1;
  padding: 6px 10px;
  font-size: 12px;
  word-break: break-all;
}

.table-cell.key {
  font-weight: 600;
  color: var(--n-text-color);
  max-width: 240px;
  flex: none;
  width: 240px;
  padding-right: 16px;
}

.table-cell.value {
  flex: 1;
  padding-left: 16px;
}

/* 响应式设计 */
@media (max-width: 1200px) {
  .response-status-bar {
    padding: 6px 10px;
    min-height: 36px;
  }

  .status-info {
    gap: 10px;
  }

  .meta-value {
    font-size: 12px;
  }

  .editor-wrapper {
    margin: 6px;
  }

  .table-scroll {
    padding: 4px 6px;
  }

  .table-cell {
    padding: 4px 6px;
    font-size: 11px;
  }

  .table-cell.key {
    max-width: 200px;
    width: 200px;
    padding-right: 12px;
  }

  .table-cell.value {
    padding-left: 12px;
  }

  .response-content :deep(.n-tabs-tab) {
    padding: 5px 8px;
    font-size: 12px;
  }
}

@media (max-width: 900px) {
  .response-status-bar {
    padding: 4px 8px;
    min-height: 32px;
  }

  .status-info {
    gap: 8px;
  }

  .meta-label {
    font-size: 9px;
  }

  .meta-value {
    font-size: 11px;
  }

  .empty-response {
    padding: 20px;
  }

  .empty-icon {
    width: 60px;
    height: 60px;
    margin-bottom: 12px;
  }

  .empty-text {
    font-size: 14px;
  }

  .empty-hint {
    font-size: 12px;
  }

  .editor-wrapper {
    margin: 4px;
  }

  .table-scroll {
    padding: 4px 8px;
  }

  .table-cell {
    padding: 4px 6px;
    font-size: 11px;
  }

  .table-cell.key {
    max-width: 150px;
    width: 150px;
  }

  .response-content :deep(.n-tabs-tab) {
    padding: 4px 6px;
    font-size: 11px;
  }
}

@media (min-width: 1600px) {
  .response-status-bar {
    padding: 10px 14px;
    min-height: 44px;
  }

  .status-info {
    gap: 16px;
  }

  .meta-label {
    font-size: 11px;
  }

  .meta-value {
    font-size: 14px;
  }

  .empty-response {
    padding: 40px;
  }

  .empty-icon {
    width: 80px;
    height: 80px;
    margin-bottom: 16px;
  }

  .empty-text {
    font-size: 16px;
  }

  .empty-hint {
    font-size: 14px;
  }

  .editor-wrapper {
    margin: 12px;
  }

  .table-scroll {
    padding: 10px 14px;
  }

  .table-cell {
    padding: 10px 12px;
    font-size: 14px;
  }

  .table-cell.key {
    max-width: 250px;
    width: 250px;
  }

  .response-content :deep(.n-tabs-tab) {
    padding: 8px 12px;
    font-size: 14px;
  }
}
</style>
