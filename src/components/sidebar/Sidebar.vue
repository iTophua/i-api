<script setup lang="ts">
import { NTabs, NTabPane, NButton, NIcon, NInput, NScrollbar, NTag, NTooltip } from 'naive-ui'
import {
  AddOutline,
  FolderOutline,
  TimeOutline,
  TrashOutline,
  SearchOutline,
  EarthOutline,
  CreateOutline,
  ChevronBackOutline,
  ChevronForwardOutline,
} from '@vicons/ionicons5'
import { ref, computed, nextTick } from 'vue'
import { useRequestStore, useHistoryStore, useEnvironmentStore } from '@/stores'
import { useI18n } from '@/composables/useI18n'
import type { Request, History, Environment } from '@/types'

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
const historyStore = useHistoryStore()
const environmentStore = useEnvironmentStore()
const { t } = useI18n()

const activeTab = ref('collections')
const searchQuery = ref('')
const expandedKeys = ref<Set<string>>(new Set())
const editingId = ref<string | null>(null)
const editingName = ref('')
const editingType = ref<'collection' | 'environment' | 'request' | null>(null)

const methodTypes: Record<string, 'success' | 'info' | 'warning' | 'error' | 'primary'> = {
  GET: 'info',
  POST: 'success',
  PUT: 'warning',
  DELETE: 'error',
  PATCH: 'primary',
  OPTIONS: 'info',
  HEAD: 'info',
}

function getMethodType(method: string): 'success' | 'info' | 'warning' | 'error' | 'primary' {
  return methodTypes[method] || 'info'
}

function toggleExpand(key: string) {
  if (expandedKeys.value.has(key)) {
    expandedKeys.value.delete(key)
  } else {
    expandedKeys.value.add(key)
  }
}

function selectRequest(request: Request, collectionId?: string) {
  requestStore.openRequest(request, collectionId)
}

function selectHistory(history: History) {
  requestStore.openRequest(
    {
      id: crypto.randomUUID(),
      name: `${history.method} ${history.url}`,
      method: history.method as any,
      url: history.url,
      params: [],
      headers: [],
      body: { mode: 'none' },
      auth: { type: 'none' },
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    },
    undefined,
    true
  )
}

function selectEnvironment(env: Environment) {
  environmentStore.setCurrentEnvironment(env.id)
}

function handleNewCollection() {
  requestStore.createCollection(t('collection.newCollection'))
}

function handleNewRequest() {
  emit('new-request')
}

async function handleNewEnvironment() {
  await environmentStore.createEnvironment(t('environment.newEnvironment'))
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

const filteredHistories = computed(() => {
  if (!searchQuery.value) return historyStore.histories.slice(0, 50)
  const query = searchQuery.value.toLowerCase()
  return historyStore.histories
    .filter((h) => h.url.toLowerCase().includes(query) || h.method.toLowerCase().includes(query))
    .slice(0, 50)
})

const filteredEnvironments = computed(() => {
  if (!searchQuery.value) return environmentStore.environments
  const query = searchQuery.value.toLowerCase()
  return environmentStore.environments.filter((e) => e.name.toLowerCase().includes(query))
})

function deleteCollection(id: string) {
  requestStore.deleteCollection(id)
}

function deleteRequest(collectionId: string, requestId: string) {
  requestStore.deleteRequest(collectionId, requestId)
}

function deleteHistory(id: string) {
  historyStore.deleteHistory(id)
}

async function deleteEnvironment(id: string) {
  await environmentStore.deleteEnvironment(id)
}

function isRequestActive(requestId: string): boolean {
  return requestStore.currentTab?.request.id === requestId
}

function startRename(id: string, name: string, type: 'collection' | 'environment') {
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
</script>

<template>
  <div class="sidebar" :class="{ collapsed }">
    <div class="sidebar-tabs">
      <NTabs v-model:value="activeTab" type="segment" size="small">
        <NTabPane name="collections">
          <template #tab>
            <span class="tab-label">
              <NIcon :component="FolderOutline" />
              {{ t('collection.collections') }}
            </span>
          </template>
        </NTabPane>
        <NTabPane name="history">
          <template #tab>
            <span class="tab-label">
              <NIcon :component="TimeOutline" />
              {{ t('history.history') }}
            </span>
          </template>
        </NTabPane>
        <NTabPane name="environments">
          <template #tab>
            <span class="tab-label">
              <NIcon :component="EarthOutline" />
              {{ t('environment.environments') }}
            </span>
          </template>
        </NTabPane>
      </NTabs>
    </div>

    <div class="sidebar-search-row">
      <NInput
        v-model:value="searchQuery"
        :placeholder="t('common.search')"
        size="small"
        clearable
        class="search-input"
      >
        <template #prefix>
          <NIcon :component="SearchOutline" />
        </template>
      </NInput>
      <NButton
        size="small"
        type="primary"
        class="add-btn"
        @click="
          activeTab === 'collections'
            ? handleNewCollection()
            : activeTab === 'environments'
              ? handleNewEnvironment()
              : handleNewRequest()
        "
      >
        <template #icon>
          <NIcon :component="AddOutline" />
        </template>
      </NButton>
    </div>

    <NScrollbar class="sidebar-scroll">
      <div v-if="activeTab === 'collections'" class="list-container">
        <template v-for="collection in filteredCollections" :key="collection.id">
          <div class="list-item group" @click="toggleExpand(collection.id)">
            <div class="item-content">
              <NIcon :component="FolderOutline" class="item-icon" />
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
                <NTag size="small" :bordered="false">{{ collection.requests.length }}</NTag>
              </template>
            </div>
            <div class="action-buttons">
              <NButton
                text
                size="tiny"
                class="action-btn"
                @click.stop="startRename(collection.id, collection.name, 'collection')"
              >
                <NIcon :component="CreateOutline" />
              </NButton>
              <NButton
                text
                size="tiny"
                class="action-btn"
                @click.stop="deleteCollection(collection.id)"
              >
                <NIcon :component="TrashOutline" />
              </NButton>
            </div>
          </div>
          <div v-if="expandedKeys.has(collection.id)" class="sub-list">
            <div
              v-for="request in collection.requests"
              :key="request.id"
              class="list-item request-item"
              :class="{ active: isRequestActive(request.id) }"
              @click="selectRequest(request, collection.id)"
            >
              <NTag :type="getMethodType(request.method)" size="small">
                {{ request.method }}
              </NTag>
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
              <div class="action-buttons">
                <NButton
                  text
                  size="tiny"
                  class="action-btn"
                  @click.stop="startRename(request.id, request.name, 'request')"
                >
                  <NIcon :component="CreateOutline" />
                </NButton>
                <NButton
                  text
                  size="tiny"
                  class="action-btn"
                  @click.stop="deleteRequest(collection.id, request.id)"
                >
                  <NIcon :component="TrashOutline" />
                </NButton>
              </div>
            </div>
          </div>
        </template>
        <div v-if="filteredCollections.length === 0" class="empty-state">
          {{ t('collection.noCollections') }}
        </div>
      </div>

      <div v-if="activeTab === 'history'" class="list-container">
        <div
          v-for="history in filteredHistories"
          :key="history.id"
          class="list-item history-item"
          @click="selectHistory(history)"
        >
          <NTag :type="getMethodType(history.method)" size="small">
            {{ history.method }}
          </NTag>
          <div class="history-info">
            <span class="history-url">{{ history.url }}</span>
            <span class="history-meta">
              <NTag :type="history.status < 400 ? 'success' : 'error'" size="small">
                {{ history.status }}
              </NTag>
              <span class="time">{{ history.responseTime }}ms</span>
            </span>
          </div>
          <NButton text size="tiny" class="delete-btn" @click.stop="deleteHistory(history.id)">
            <NIcon :component="TrashOutline" />
          </NButton>
        </div>
        <div v-if="filteredHistories.length === 0" class="empty-state">
          {{ t('history.noHistory') }}
        </div>
      </div>

      <div v-if="activeTab === 'environments'" class="list-container">
        <div
          v-for="env in filteredEnvironments"
          :key="env.id"
          class="list-item env-item"
          :class="{ active: environmentStore.currentEnvironmentId === env.id }"
          @click="selectEnvironment(env)"
        >
          <div class="item-content">
            <NIcon :component="EarthOutline" class="item-icon" />
            <template v-if="editingId === env.id && editingType === 'environment'">
              <input
                v-model="editingName"
                :class="`rename-input-${env.id}`"
                class="rename-input"
                @blur="finishRename"
                @keyup.enter="finishRename"
                @keyup.escape="cancelRename"
                @click.stop
              />
            </template>
            <template v-else>
              <span class="item-name">{{ env.name }}</span>
              <NTag size="small" :bordered="false"
                >{{ env.variables.length }} {{ t('environment.variables') }}</NTag
              >
            </template>
          </div>
          <div class="action-buttons">
            <NButton
              text
              size="tiny"
              class="action-btn"
              @click.stop="startRename(env.id, env.name, 'environment')"
            >
              <NIcon :component="CreateOutline" />
            </NButton>
            <NButton text size="tiny" class="action-btn" @click.stop="deleteEnvironment(env.id)">
              <NIcon :component="TrashOutline" />
            </NButton>
          </div>
        </div>
        <div v-if="filteredEnvironments.length === 0" class="empty-state">
          {{ t('environment.noEnvironment') }}
        </div>
      </div>
    </NScrollbar>

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
</template>

<style scoped>
.sidebar {
  height: 100%;
  display: flex;
  flex-direction: column;
  position: relative;
  background: var(--n-color);
  border-right: 1px solid var(--n-border-color);
}

.sidebar.collapsed {
  width: 48px;
  min-width: 48px;
}

.sidebar.collapsed .sidebar-tabs,
.sidebar.collapsed .sidebar-search-row,
.sidebar.collapsed .sidebar-scroll,
.sidebar.collapsed .list-container,
.sidebar.collapsed .empty-state {
  display: none;
}

.sidebar-tabs {
  padding: 8px 6px 6px;
  border-bottom: 1px solid var(--n-border-color);
}

.tab-label {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  font-weight: 500;
}

.sidebar-search-row {
  padding: 6px;
  display: flex;
  align-items: center;
  gap: 6px;
}

.search-input {
  flex: 1;
}

.add-btn {
  flex-shrink: 0;
  width: 28px;
  height: 28px;
  padding: 0;
}

.sidebar-scroll {
  flex: 1;
  overflow: hidden;
}

.list-container {
  padding: 4px;
}

.list-item {
  display: flex;
  align-items: center;
  padding: 6px 8px;
  margin: 1px 2px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s ease;
  position: relative;
}

.list-item:hover {
  background: var(--n-color-hover);
  transform: translateX(2px);
}

.list-item.active {
  background: var(--n-color-hover);
  box-shadow: inset 3px 0 0 var(--n-primary-color);
}

.item-content {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
}

.item-icon {
  flex-shrink: 0;
  opacity: 0.7;
  font-size: 14px;
}

.item-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 12px;
  font-weight: 500;
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
  gap: 4px;
  opacity: 0;
  transition: opacity 0.2s;
}

.list-item:hover .action-buttons {
  opacity: 1;
}

.action-btn {
  padding: 4px;
  border-radius: 4px;
}

.action-btn:hover {
  background: var(--n-color-pressed);
}

.rename-input {
  flex: 1;
  min-width: 0;
  padding: 4px 8px;
  border: 1px solid var(--n-primary-color);
  border-radius: 4px;
  background: var(--n-color);
  color: var(--n-text-color);
  font-size: 13px;
  outline: none;
  box-shadow: 0 0 0 2px rgba(24, 160, 88, 0.1);
}

.sub-list {
  padding-left: 12px;
  margin-top: 1px;
  border-left: 2px solid var(--n-border-color);
  margin-left: 14px;
}

.request-item {
  gap: 8px;
  padding-left: 8px;
}

.request-item :deep(.n-tag) {
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.5px;
}

.history-item {
  flex-wrap: wrap;
  gap: 6px;
  padding: 6px 8px;
}

.history-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.history-url {
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 500;
}

.history-meta {
  display: flex;
  align-items: center;
  gap: 8px;
}

.time {
  font-size: 11px;
  color: var(--n-text-color-3);
  font-weight: 500;
}

.env-item {
  flex-wrap: wrap;
}

.empty-state {
  padding: 24px 12px;
  text-align: center;
  color: var(--n-text-color-3);
  font-size: 12px;
  line-height: 1.6;
}

.sidebar-footer {
  height: 36px;
  padding: 0 8px;
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
  gap: 6px;
}

.logo-text {
  font-size: 13px;
  font-weight: 700;
  color: var(--n-text-color-1);
  letter-spacing: 0.5px;
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
  .sidebar-tabs {
    padding: 6px 4px 4px;
  }

  .tab-label {
    font-size: 11px;
  }

  .sidebar-search-row {
    padding: 4px;
  }

  .list-item {
    padding: 4px 6px;
    margin: 1px 2px;
  }

  .item-name {
    font-size: 11px;
  }
}

@media (max-width: 900px) {
  .sidebar {
    width: 48px !important;
    min-width: 48px !important;
  }
}

@media (min-width: 1600px) {
  .sidebar-tabs {
    padding: 10px 8px 8px;
  }

  .tab-label {
    font-size: 13px;
  }

  .sidebar-search-row {
    padding: 8px;
  }

  .list-item {
    padding: 8px 10px;
  }

  .item-name {
    font-size: 13px;
  }

  .sidebar-footer {
    height: 40px;
  }

  .logo-text {
    font-size: 14px;
  }
}
</style>
