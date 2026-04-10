<script setup lang="ts">
import { NTabs, NTabPane, NButton, NIcon, NInput, NTooltip, NDropdown } from 'naive-ui'
import {
  ChevronBackOutline,
  ChevronForwardOutline,
} from '@vicons/ionicons5'
import { ref, computed, nextTick } from 'vue'
import { useRequestStore, useEnvironmentStore } from '@/stores'
import { useI18n } from '@/composables/useI18n'
import type { Request, HttpMethod, History as RequestHistory } from '@/types'

const HTTP_METHODS: HttpMethod[] = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'OPTIONS', 'HEAD']

function isValidHttpMethod(method: string): method is HttpMethod {
  return HTTP_METHODS.includes(method as HttpMethod)
}
import { HttpMethodIcon, AppIcon } from '@/components/icons'
import BatchOperationToolbar from '@/components/common/BatchOperationToolbar.vue'
import BatchMoveDialog from '@/components/common/BatchMoveDialog.vue'
import HistoryPanel from '@/components/history/HistoryPanel.vue'

defineProps<{
  collapsed?: boolean
}>()

const emit = defineEmits<{
  toggle: []
  'new-request': []
  import: [key: string]
  export: []
  settings: []
  code: [key: string]
}>()

const requestStore = useRequestStore()
const environmentStore = useEnvironmentStore()
const { t } = useI18n()

const activeTab = ref('collections')
const searchQuery = ref('')
const expandedKeys = ref<Set<string>>(new Set())
const editingId = ref<string | null>(null)
const editingName = ref('')
const editingType = ref<'collection' | 'environment' | 'request' | null>(null)

const showBatchDialog = ref(false)
const batchMode = ref<'copy' | 'move'>('move')

const showContextMenu = ref(false)
const contextMenuX = ref(0)
const contextMenuY = ref(0)
const contextMenuTarget = ref<{ id: string; type: 'collection' | 'request'; collectionId?: string } | null>(null)

const collectionContextMenuOptions = [
  { label: '重命名', key: 'rename' },
  { label: '删除', key: 'delete' },
]

const requestContextMenuOptions = [
  { label: '重命名', key: 'rename' },
  { label: '删除', key: 'delete' },
]

function handleCollectionContextMenu(e: MouseEvent, collection: { id: string; name: string }) {
  e.preventDefault()
  contextMenuTarget.value = { id: collection.id, type: 'collection' }
  contextMenuX.value = e.clientX
  contextMenuY.value = e.clientY
  showContextMenu.value = true
}

function handleRequestContextMenu(e: MouseEvent, request: { id: string; name: string }, collectionId: string) {
  e.preventDefault()
  contextMenuTarget.value = { id: request.id, type: 'request', collectionId }
  contextMenuX.value = e.clientX
  contextMenuY.value = e.clientY
  showContextMenu.value = true
}

function handleContextMenuSelect(key: string) {
  if (!contextMenuTarget.value) return

  if (contextMenuTarget.value.type === 'collection') {
    if (key === 'rename') {
      const collection = requestStore.collections.find(c => c.id === contextMenuTarget.value!.id)
      if (collection) {
        startRename(collection.id, collection.name, 'collection')
      }
    } else if (key === 'delete') {
      requestStore.deleteCollection(contextMenuTarget.value.id)
    }
  } else if (contextMenuTarget.value.type === 'request' && contextMenuTarget.value.collectionId) {
    if (key === 'rename') {
      const collection = requestStore.collections.find(c => c.id === contextMenuTarget.value!.collectionId)
      const request = collection?.requests.find(r => r.id === contextMenuTarget.value!.id)
      if (request) {
        startRename(request.id, request.name, 'request')
      }
    } else if (key === 'delete') {
      requestStore.deleteRequest(contextMenuTarget.value.collectionId, contextMenuTarget.value.id)
    }
  }

  showContextMenu.value = false
  contextMenuTarget.value = null
}

function handleContextMenuClickoutside() {
  showContextMenu.value = false
  contextMenuTarget.value = null
}

function toggleExpand(key: string) {
  if (expandedKeys.value.has(key)) {
    expandedKeys.value.delete(key)
  } else {
    expandedKeys.value.add(key)
  }
}

function toggleRequestSelection(requestId: string) {
  requestStore.toggleSelection(requestId)
}

function selectRequest(request: Request, collectionId?: string) {
  if (requestStore.isSelectionMode) {
    toggleRequestSelection(request.id)
  } else {
    requestStore.openRequest(request, collectionId)
  }
}

function selectHistory(history: RequestHistory) {
  const now = new Date().toISOString()
  requestStore.openRequest(
    {
      id: crypto.randomUUID(),
      name: `${history.method} ${history.url}`,
      description: '',
      method: isValidHttpMethod(history.method) ? history.method : 'GET',
      url: history.url,
      params: [],
      headers: [],
      body: { mode: 'none' },
      auth: { type: 'none' },
      preScript: '',
      postScript: '',
      createdAt: now,
      updatedAt: now,
    },
    undefined,
    true
  )
}

function handleNewCollection() {
  requestStore.createCollection(t('collection.newCollection'))
}

const filteredCollections = computed(() => {
  if (!searchQuery.value) return requestStore.collections
  const query = searchQuery.value.toLowerCase()
  return requestStore.collections.filter(
    (c) =>
      c.name.toLowerCase().includes(query) ||
      c.requests.some((r) => r.name.toLowerCase().includes(query))
  )
})

function isRequestActive(requestId: string): boolean {
  return requestStore.currentTab?.request.id === requestId
}

function startRename(id: string, name: string, type: 'collection' | 'environment' | 'request') {
  editingId.value = id
  editingName.value = name
  editingType.value = type
  nextTick(() => {
    const input = document.querySelector(`.rename-input-${id}`) as HTMLInputElement
    if (input) {
      input.focus()
      input.select()
    }
  })
}

async function finishRename() {
  if (editingId.value && editingName.value.trim()) {
    if (editingType.value === 'collection') {
      requestStore.renameCollection(editingId.value, editingName.value.trim())
    } else if (editingType.value === 'environment') {
      await environmentStore.renameEnvironment(editingId.value, editingName.value.trim())
    } else if (editingType.value === 'request') {
      await requestStore.renameRequest(editingId.value, editingName.value.trim())
    }
  }
  cancelRename()
}

function cancelRename() {
  editingId.value = null
  editingName.value = ''
  editingType.value = null
}

function handleToggle() {
  emit('toggle')
}

function openBatchDialog(mode: 'copy' | 'move') {
  batchMode.value = mode
  showBatchDialog.value = true
}

function closeBatchDialog() {
  showBatchDialog.value = false
}

function getCurrentCollectionId(): string | undefined {
  const currentTab = requestStore.currentTab
  if (!currentTab?.collectionId) return undefined
  return currentTab.collectionId
}
</script>

<template>
  <div class="sidebar" :class="{ collapsed }">
    <div class="sidebar-tabs">
      <NTabs v-model:value="activeTab" type="segment" size="small">
        <NTabPane name="collections">
          <template #tab>
            <span class="tab-label">
              <AppIcon type="collection" :size="16" />
              {{ t('collection.collections') }}
            </span>
          </template>
        </NTabPane>
        <NTabPane name="history">
          <template #tab>
            <span class="tab-label">
              <AppIcon type="history" :size="16" />
              {{ t('history.history') }}
            </span>
          </template>
        </NTabPane>
        
      </NTabs>
    </div>

    <div v-if="activeTab !== 'history'" class="sidebar-search-row">
      <NInput
        v-model:value="searchQuery"
        :placeholder="t('common.search')"
        size="small"
        clearable
        class="search-input"
      >
        <template #prefix>
          <AppIcon type="search" :size="14" />
        </template>
      </NInput>
      <NButton
          size="small"
          type="primary"
          class="add-btn"
          @click="handleNewCollection()"
        >
        <template #icon>
          <AppIcon type="plus" :size="14" />
        </template>
      </NButton>
    </div>

    <div class="sidebar-content">
      <!-- Collections Tab -->
      <div v-if="activeTab === 'collections'" class="list-container">
        <BatchOperationToolbar
          :collection="filteredCollections[0]"
          @copy="openBatchDialog('copy')"
          @move="openBatchDialog('move')"
        />
        <template v-for="collection in filteredCollections" :key="collection.id">
          <div
            class="list-item group"
            @click="toggleExpand(collection.id)"
            @contextmenu="handleCollectionContextMenu($event, collection)"
          >
            <div class="item-content">
              <AppIcon type="folder" :size="16" class="item-icon" />
              <template v-if="editingId === collection.id && editingType === 'collection'">
                <input
                  v-model="editingName"
                  :class="`rename-input-${collection.id}`"
                  class="rename-input"
                  @blur="finishRename"
                  @keyup.enter="finishRename"
                  @keyup.escape="cancelRename"
                  @click.stop
                />
              </template>
              <template v-else>
                <span class="item-name">{{ collection.name }}</span>
                <span class="item-count">{{ collection.requests.length }}</span>
              </template>
            </div>
          </div>
          <div v-if="expandedKeys.has(collection.id)" class="sub-list">
            <div
              v-for="request in collection.requests"
              :key="request.id"
              class="list-item request-item"
              :class="{
                active: isRequestActive(request.id),
                selected: requestStore.isInSelection(request.id),
              }"
              @click="selectRequest(request, collection.id)"
              @contextmenu="handleRequestContextMenu($event, request, collection.id)"
            >
              <div
                v-if="requestStore.isSelectionMode"
                class="selection-checkbox"
                @click.stop="toggleRequestSelection(request.id)"
              >
                <AppIcon
                  :type="requestStore.isInSelection(request.id) ? 'check' : 'plus'"
                  :size="14"
                />
              </div>
              <HttpMethodIcon :method="request.method" :size="12" filled />
              <template v-if="editingId === request.id && editingType === 'request'">
                <input
                  v-model="editingName"
                  :class="`rename-input-${request.id}`"
                  class="rename-input"
                  @blur="finishRename"
                  @keyup.enter="finishRename"
                  @keyup.escape="cancelRename"
                  @click.stop
                />
              </template>
              <template v-else>
                <span class="item-name">{{ request.name }}</span>
              </template>
            </div>
          </div>
        </template>
        <div v-if="filteredCollections.length === 0" class="empty-state">
          {{ t('collection.noCollections') }}
        </div>
      </div>

      <!-- History Tab -->
      <div v-if="activeTab === 'history'" class="history-tab-container">
        <HistoryPanel @select="selectHistory" />
      </div>


    </div>

    <div class="sidebar-footer">
      <div class="sidebar-logo">
        <svg width="20" height="20" viewBox="0 0 32 32" fill="none">
          <rect width="32" height="32" rx="6" fill="#18A058" />
          <path
            d="M8 12L16 8L24 12V20L16 24L8 20V12Z"
            stroke="white"
            stroke-width="2"
            fill="none"
          />
          <circle cx="16" cy="16" r="3" fill="white" />
        </svg>
        <span class="logo-text">iApi</span>
      </div>
      <div class="footer-actions">
        <NTooltip :show-arrow="false" placement="top">
          <template #trigger>
            <NButton text circle size="small" @click="handleToggle">
              <template #icon>
                <NIcon :component="collapsed ? ChevronForwardOutline : ChevronBackOutline" />
              </template>
            </NButton>
          </template>
          {{ collapsed ? t('common.expand') : t('common.collapse') }}
        </NTooltip>
      </div>
    </div>

    <NDropdown
      :show="showContextMenu"
      :options="contextMenuTarget?.type === 'request' ? requestContextMenuOptions : collectionContextMenuOptions"
      :x="contextMenuX"
      :y="contextMenuY"
      @select="handleContextMenuSelect"
      @clickoutside="handleContextMenuClickoutside"
    />

    <BatchMoveDialog
      v-model:show="showBatchDialog"
      :collection-id="getCurrentCollectionId()"
      :request-ids="Array.from(requestStore.selectedRequestIds)"
      :mode="batchMode"
      @close="closeBatchDialog"
    />
  </div>
</template>

<style scoped>
.sidebar {
  height: 100%;
  display: flex;
  flex-direction: column;
  position: relative;
  background: var(--n-color);
}

.sidebar::after {
  content: '';
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  width: 1px;
  background: var(--n-border-color);
  pointer-events: none;
}

.sidebar-content,
.sidebar-scroll {
  position: relative;
  z-index: 1;
}

.sidebar.collapsed {
  width: 48px;
  min-width: 48px;
  max-width: 48px;
}

.sidebar.collapsed .sidebar-tabs,
.sidebar.collapsed .sidebar-search-row,
.sidebar.collapsed .sidebar-scroll,
.sidebar.collapsed .sidebar-content,
.sidebar.collapsed .list-container,
.sidebar.collapsed .empty-state,
.sidebar.collapsed .history-tab-container,
.sidebar.collapsed .env-tab-container {
  display: none;
}

.sidebar-tabs {
  padding: 6px 4px 4px;
  border-bottom: 1px solid var(--n-border-color);
}

.tab-label {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: var(--font-size-compact-sm);
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.sidebar-search-row {
  padding: 4px;
  display: flex;
  align-items: center;
  gap: 4px;
}

.search-input {
  flex: 1;
  min-width: 0;
}

.add-btn {
  flex-shrink: 0;
  width: 24px;
  height: 24px;
  padding: 0;
}

.sidebar-scroll {
  flex: 1;
  overflow: hidden;
}

.sidebar-content {
  flex: 1;
  overflow: hidden;
  position: relative;
}

.list-container {
  height: 100%;
  overflow-y: auto;
  padding: 2px;
}

.history-tab-container,
.env-tab-container {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  height: 100%;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.list-item {
  display: flex;
  align-items: center;
  padding: 5px 6px;
  margin: 1px 2px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s ease;
  position: relative;
  min-height: 32px;
}

.list-item:hover {
  background: var(--n-color-hover);
  transform: translateX(2px);
}

.list-item.active {
  background: var(--n-color-hover);
  box-shadow: inset 2px 0 0 var(--n-primary-color);
}

.list-item.selected {
  background: var(--n-color-hover);
  box-shadow: inset 2px 0 0 var(--n-primary-color);
}

.selection-checkbox {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  margin-right: 4px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.selection-checkbox:hover {
  background: var(--n-color-pressed);
}

.item-content {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
  min-width: 0;
}

.item-icon {
  flex-shrink: 0;
  opacity: 0.7;
  font-size: var(--font-size-compact-md);
}

.item-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: var(--font-size-compact-md);
  font-weight: 500;
  line-height: 1.4;
}

.item-count {
  flex-shrink: 0;
  font-size: var(--font-size-compact-xs);
  color: var(--n-text-color-3);
  font-weight: 500;
  padding: 0 4px;
}

.delete-btn {
  opacity: 0;
  transition: opacity 0.2s;
}

.list-item:hover .delete-btn {
  opacity: 1;
}

.action-buttons {
  display: flex;
  gap: 2px;
  opacity: 0;
  transition: opacity 0.2s;
  flex-shrink: 0;
}

.list-item:hover .action-buttons {
  opacity: 1;
}

.action-btn {
  padding: 2px;
  border-radius: 3px;
  min-width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.action-btn:hover {
  background: var(--n-color-pressed);
}

.rename-input {
  flex: 1;
  min-width: 0;
  padding: 3px 6px;
  border: 1px solid var(--n-primary-color);
  border-radius: 3px;
  background: var(--n-color);
  color: var(--n-text-color);
  font-size: var(--font-size-compact-md);
  outline: none;
  box-shadow: 0 0 0 2px rgba(24, 160, 88, 0.1);
  height: 24px;
}

.sub-list {
  padding-left: 10px;
  margin-top: 1px;
  border-left: 1px solid var(--n-border-color);
  margin-left: 12px;
}

.request-item {
  gap: 6px;
  padding-left: 6px;
}

.request-item :deep(.n-tag) {
  font-size: var(--font-size-compact-xs);
  font-weight: 600;
  letter-spacing: 0.5px;
  padding: 1px 4px;
  min-width: 28px;
  text-align: center;
}

.history-item {
  flex-wrap: wrap;
  gap: 4px;
  padding: 5px 6px;
}

.history-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.history-url {
  font-size: var(--font-size-compact-sm);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 500;
  line-height: 1.3;
}

.history-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

.time {
  font-size: var(--font-size-compact-xs);
  color: var(--n-text-color-3);
  font-weight: 500;
  flex-shrink: 0;
}

.env-item {
  flex-wrap: wrap;
}

.empty-state {
  padding: 20px 10px;
  text-align: center;
  color: var(--n-text-color-3);
  font-size: var(--font-size-compact-sm);
  line-height: 1.6;
}

.sidebar-footer {
  height: 32px;
  padding: 0 6px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-top: 1px solid var(--n-border-color);
  flex-shrink: 0;
  background: var(--n-color-modal);
}

.sidebar-logo {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.footer-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.logo-text {
  font-size: var(--font-size-compact-sm);
  font-weight: 700;
  color: var(--n-text-color-1);
  letter-spacing: 0.5px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.sidebar.collapsed .sidebar-footer {
  justify-content: center;
  padding: 0;
}

.sidebar.collapsed .logo-text {
  display: none;
}

/* 响应式设计 */
@media (max-width: 1200px) {
  .sidebar {
    width: 200px;
  }
}

@media (max-width: 900px) {
  .sidebar {
    width: 48px !important;
    min-width: 48px !important;
    max-width: 48px !important;
  }
}

@media (min-width: 1600px) {
  .sidebar {
    width: 280px;
  }
}

.history-tab-container {
  height: 100%;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.env-tab-container {
  height: 100%;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

/* 紧凑模式下的标签页文本隐藏 */
@media (max-width: 1300px) {
  .tab-label span:not(.icon) {
    display: none;
  }
  
  .tab-label {
    justify-content: center;
    gap: 0;
  }
}


</style>
