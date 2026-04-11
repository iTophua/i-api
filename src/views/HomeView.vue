<script setup lang="ts">
import { NButton, NIcon, NDropdown, useMessage, NSplit, NModal } from 'naive-ui'
import { CloseOutline, AddOutline } from '@vicons/ionicons5'
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { useSettingsStore, useEnvironmentStore, useRequestStore } from '@/stores'
import { useShortcuts } from '@/composables/useShortcuts'
import { useI18n } from '@/composables/useI18n'
import { useTabDrag } from '@/composables/useTabDrag'
import TitleBar from '@/components/common/TitleBar.vue'
import Sidebar from '@/components/sidebar/Sidebar.vue'
import RequestPanel from '@/components/request/RequestPanel.vue'
import ResponsePanel from '@/components/response/ResponsePanel.vue'
import CurlImportModal from '@/components/common/CurlImportModal.vue'
import PostmanImportModal from '@/components/common/PostmanImportModal.vue'
import PostmanExportModal from '@/components/common/PostmanExportModal.vue'
import CodeGeneratorModal from '@/components/common/CodeGeneratorModal.vue'
import OpenApiImportModal from '@/components/common/OpenApiImportModal.vue'
import HarImportModal from '@/components/common/HarImportModal.vue'
import SaveRequestModal from '@/components/common/SaveRequestModal.vue'
import SettingsModal from '@/components/common/SettingsModal.vue'
import EnvironmentManager from '@/components/environment/EnvironmentManager.vue'
import { generateCode } from '@/utils/codeGenerator'
import type { RequestTab } from '@/types'
import { normalizeRequest } from '@/types'

const message = useMessage()
const settingsStore = useSettingsStore()
const environmentStore = useEnvironmentStore()
const requestStore = useRequestStore()
const { t } = useI18n()

const collapsed = ref(false)
const showCurlImport = ref(false)
const showPostmanImport = ref(false)
const showPostmanExport = ref(false)
const showCodeGenerator = ref(false)
const showOpenApiImport = ref(false)
const showHarImport = ref(false)
const showSaveRequest = ref(false)
const showSettings = ref(false)
const showEnvironmentManager = ref(false)

const contextMenuOptions = [
  { label: '复制', key: 'duplicate' },
  { type: 'divider', key: 'd1' },
  { label: '关闭', key: 'close' },
  { label: '关闭左边', key: 'close-left' },
  { label: '关闭右边', key: 'close-right' },
  { label: '关闭全部', key: 'close-all' },
]

const showContextMenu = ref(false)
const contextMenuX = ref(0)
const contextMenuY = ref(0)
const contextMenuTabId = ref<string | null>(null)

const { dragState, handleDragStart, cleanup: cleanupDrag } = useTabDrag(requestStore.moveTab)

function toggleSidebar() {
  collapsed.value = !collapsed.value
}

function handleImportSelect(key: string) {
  switch (key) {
    case 'curl':
      showCurlImport.value = true
      break
    case 'postman':
      showPostmanImport.value = true
      break
    case 'openapi':
      showOpenApiImport.value = true
      break
    case 'har':
      showHarImport.value = true
      break
  }
}

function handleCurlImport(request: any) {
  const normalizedRequest = normalizeRequest(request)
  requestStore.openRequest(normalizedRequest, undefined, true)
  message.success(t('import.importSuccess', { count: 1 }))
}

function handleRequestsImport(requests: any[], _source: string) {
  if (requests.length > 0) {
    const normalizedRequest = normalizeRequest(requests[0])
    requestStore.openRequest(normalizedRequest, undefined, true)
    message.success(t('import.importSuccess', { count: requests.length }))
  }
}

function handleExport() {
  showPostmanExport.value = true
}

function handleCodeSelect(key: string) {
  if (key === 'copy-curl') {
    copyAsCurl()
  } else if (key === 'generate') {
    showCodeGenerator.value = true
  }
}

function copyAsCurl() {
  const curl = generateCode(requestStore.currentRequest, 'curl')
  navigator.clipboard.writeText(curl)
  message.success(t('common.copy') + ' cURL')
}

function sendRequest() {
  window.dispatchEvent(new CustomEvent('send-request'))
}

function saveRequest() {
  showSaveRequest.value = true
}

function handleSaveRequest(data: { name: string; collectionId: string }) {
  if (requestStore.currentTab) {
    requestStore.currentTab.request.name = data.name
    requestStore.saveRequest(data.collectionId)
    message.success(t('common.success'))
  }
}

function newRequest() {
  requestStore.newRequest()
}

function toggleTheme() {
  const currentTheme = settingsStore.settings.theme
  if (currentTheme === 'dark') {
    settingsStore.setTheme('light')
  } else if (currentTheme === 'light') {
    settingsStore.setTheme('dark')
  } else {
    // 如果是 system，切换到 dark
    settingsStore.setTheme('dark')
  }
}

function handleOpenEnvironmentManager() {
  showEnvironmentManager.value = true
}

function handleContextMenu(e: MouseEvent, tabId: string) {
  e.preventDefault()
  contextMenuTabId.value = tabId
  contextMenuX.value = e.clientX
  contextMenuY.value = e.clientY
  showContextMenu.value = true
}

function handleContextMenuSelect(key: string) {
  if (!contextMenuTabId.value) return

  switch (key) {
    case 'duplicate':
      requestStore.duplicateTab(contextMenuTabId.value)
      break
    case 'close':
      requestStore.closeTab(contextMenuTabId.value)
      break
    case 'close-left':
      requestStore.closeTabsToLeft(contextMenuTabId.value)
      break
    case 'close-right':
      requestStore.closeTabsToRight(contextMenuTabId.value)
      break
    case 'close-all':
      requestStore.closeAllTabs()
      break
  }

  showContextMenu.value = false
  contextMenuTabId.value = null
}

function handleContextMenuClickoutside() {
  showContextMenu.value = false
  contextMenuTabId.value = null
}

function getTabLabel(tab: RequestTab): string {
  return tab.request.name || t('request.untitledRequest')
}

function saveAppState() {
  settingsStore.updateAppState({
    currentEnvironmentId: environmentStore.currentEnvironmentId,
    sidebarCollapsed: collapsed.value,
  })
  settingsStore.saveAppState()
}

onMounted(async () => {
  await settingsStore.loadSettings()
  await environmentStore.loadEnvironments()
  await requestStore.loadCollections()

  const savedEnvId = localStorage.getItem('iapi-current-environment')
  if (savedEnvId && environmentStore.environments.some(e => e.id === savedEnvId)) {
    environmentStore.setCurrentEnvironment(savedEnvId)
  }
  collapsed.value = settingsStore.appState.sidebarCollapsed

  await requestStore.loadTabs()
  await requestStore.loadTemporaryRequest()
  requestStore.initTabsPersistence()
})

onUnmounted(() => {
  saveAppState()
  cleanupDrag()
})

watch(collapsed, (val) => {
  settingsStore.updateAppState({ sidebarCollapsed: val })
  settingsStore.saveAppState()
})

useShortcuts([
  { key: 'Enter', ctrl: true, handler: sendRequest, description: t('shortcuts.sendRequest') },
  { key: 's', ctrl: true, handler: saveRequest, description: t('shortcuts.saveRequest') },
  { key: 'n', ctrl: true, handler: newRequest, description: t('shortcuts.newRequest') },
  { key: 'b', ctrl: true, handler: toggleSidebar, description: t('shortcuts.toggleSidebar') },
  {
    key: ',',
    ctrl: true,
    handler: () => (showSettings.value = true),
    description: t('shortcuts.openSettings'),
  },
])
</script>

<template>
  <div class="home-view">
    <TitleBar
      @import="handleImportSelect"
      @export="handleExport"
      @code="handleCodeSelect"
      @toggle-theme="toggleTheme"
      @select-environment="environmentStore.setCurrentEnvironment"
      @open-settings="showSettings = true"
      @open-environment-manager="handleOpenEnvironmentManager"
    />
    <div class="main-layout">
      <div class="sidebar-wrapper" :class="{ collapsed }">
        <Sidebar
          :collapsed="collapsed"
          @toggle="toggleSidebar"
          @new-request="newRequest"
          @import="handleImportSelect"
          @export="handleExport"
          @settings="showSettings = true"
          @code="handleCodeSelect"
        />
      </div>
      <div class="main-content">
        <div v-if="requestStore.tabs.length > 0" class="tabs-bar">
          <div class="tabs-scroll">
            <div
              v-for="(tab, index) in requestStore.tabs"
              :key="tab.id"
              class="tab-item"
              :data-tab-id="tab.id"
              :class="{
                active: tab.id === requestStore.activeTabId,
                dirty: tab.isDirty,
                dragging: dragState.dragging && dragState.tabId === tab.id,
                'drag-over':
                  dragState.dragging &&
                  dragState.currentIndex === index &&
                  dragState.tabId !== tab.id,
              }"
              :style="
                dragState.dragging && dragState.tabId === tab.id
                  ? {
                      transform: `translateX(${dragState.offsetX}px)`,
                      zIndex: 10,
                    }
                  : dragState.dragging &&
                      dragState.currentIndex !== dragState.startIndex &&
                      index !== dragState.startIndex
                    ? {
                        transform:
                          index >= Math.min(dragState.startIndex, dragState.currentIndex) &&
                          index <= Math.max(dragState.startIndex, dragState.currentIndex)
                            ? `translateX(${dragState.startIndex < dragState.currentIndex ? -dragState.tabWidth - 2 : dragState.tabWidth + 2}px)`
                            : 'translateX(0)',
                      }
                    : {}
              "
              @click="requestStore.switchTab(tab.id)"
              @contextmenu="handleContextMenu($event, tab.id)"
              @mousedown="handleDragStart($event, tab.id, requestStore.tabs)"
            >
              <span class="tab-method" :class="'method-' + tab.request.method.toLowerCase()">
                {{ tab.request.method }}
              </span>
              <span class="tab-name">{{ getTabLabel(tab) }}</span>
              <NButton text class="tab-close" @click.stop="requestStore.closeTab(tab.id)">
                <NIcon :component="CloseOutline" size="14" />
              </NButton>
            </div>
            <NButton text class="new-tab-btn" @click="newRequest">
              <NIcon :component="AddOutline" size="16" />
            </NButton>
          </div>
          <NDropdown
            :show="showContextMenu"
            :options="contextMenuOptions"
            :x="contextMenuX"
            :y="contextMenuY"
            placement="bottom-start"
            @select="handleContextMenuSelect"
            @clickoutside="handleContextMenuClickoutside"
          />
        </div>

        <div v-if="requestStore.tabs.length > 0" class="panels-container">
          <NSplit
            direction="vertical"
            :default-size="0.5"
            :min="0.15"
            :max="0.85"
            style="height: 100%"
          >
            <template #1>
              <RequestPanel />
            </template>
            <template #2>
              <ResponsePanel />
            </template>
          </NSplit>
        </div>

        <div v-else class="empty-state">
          <div class="empty-content">
            <NButton type="primary" size="large" @click="newRequest">
              <template #icon>
                <NIcon :component="AddOutline" />
              </template>
              {{ t('request.newRequest') }}
            </NButton>
            <p class="empty-hint">{{ t('request.enterUrl') }}</p>
          </div>
        </div>
      </div>
    </div>

    <CurlImportModal v-model:show="showCurlImport" @import="handleCurlImport" />

    <PostmanImportModal
      v-model:show="showPostmanImport"
      @import="(reqs: any[]) => handleRequestsImport(reqs, 'Postman')"
    />

    <PostmanExportModal
      v-model:show="showPostmanExport"
      :requests="requestStore.collections.flatMap((c) => c.requests)"
    />

    <CodeGeneratorModal v-model:show="showCodeGenerator" :request="requestStore.currentRequest" />

    <OpenApiImportModal
      v-model:show="showOpenApiImport"
      @import="(reqs: any[]) => handleRequestsImport(reqs, 'OpenAPI')"
    />

    <HarImportModal
      v-model:show="showHarImport"
      @import="(reqs: any[]) => handleRequestsImport(reqs, 'HAR')"
    />

    <SaveRequestModal
      v-model:show="showSaveRequest"
      :request="requestStore.currentRequest"
      :collections="requestStore.collections"
      :default-collection-id="requestStore.currentCollection?.id"
      @save="handleSaveRequest"
    />

    <SettingsModal v-model:show="showSettings" />
    <NModal
      v-model:show="showEnvironmentManager"
      preset="card"
      style="width: 800px; height: 600px; max-width: 90vw; max-height: 90vh"
      title="环境管理"
      :bordered="false"
      :mask-closable="true"
      :focus-lock="false"
    >
      <EnvironmentManager />
    </NModal>
  </div>
</template>

<style scoped>
.home-view {
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--n-color-modal);
}

.main-layout {
  flex: 1;
  min-height: 0;
  display: flex;
  position: relative;
}

.sidebar-wrapper {
  width: 300px;
  flex-shrink: 0;
  transition: width 0.2s ease;
  overflow: hidden;
}

.sidebar-wrapper.collapsed {
  width: 48px;
}

.main-layout :deep(.n-split) {
  height: 100%;
}

.main-layout :deep(.n-split__left),
.main-layout :deep(.n-split__right) {
  height: 100%;
  overflow: hidden;
}

.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  background: var(--n-color);
  position: relative;
}

.tabs-bar {
  height: 32px;
  background: var(--n-color);
  border-bottom: 1px solid var(--n-border-color);
  display: flex;
  align-items: center;
  padding: 0 8px;
  flex-shrink: 0;
  gap: 4px;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
}

.tabs-scroll {
  flex: 1;
  display: flex;
  align-items: center;
  overflow-x: auto;
  gap: 2px;
  padding: 2px 0;
}

.tabs-scroll::-webkit-scrollbar {
  display: none;
}

.tab-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 3px 8px;
  border-radius: 4px;
  cursor: pointer;
  max-width: 200px;
  white-space: nowrap;
  color: var(--n-text-color-3);
  transition:
    transform 0.2s ease,
    background 0.2s ease,
    border-color 0.2s ease,
    box-shadow 0.2s ease;
  position: relative;
  border: 1px solid transparent;
  will-change: transform;
}

.tab-item:hover {
  background: var(--n-color-hover);
  border-color: var(--n-border-color);
}

.tab-item.active {
  background: var(--n-color-modal);
  color: var(--n-text-color-1);
  border-color: var(--n-border-color);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
}

.tab-item.dragging {
  opacity: 0.8;
  cursor: grabbing;
  transition: none;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 10;
}

.tab-item.drag-over {
  border-left: 2px solid var(--n-primary-color);
}

.tab-item.dirty .tab-name::after {
  content: '';
  width: 6px;
  height: 6px;
  background: var(--n-info-color);
  border-radius: 50%;
  margin-left: 4px;
  display: inline-block;
}

.tab-method {
  font-size: 10px;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: 3px;
  letter-spacing: 0.5px;
}

.method-get {
  color: #61affe;
  background: rgba(97, 175, 254, 0.1);
}
.method-post {
  color: #49cc90;
  background: rgba(73, 204, 144, 0.1);
}
.method-put {
  color: #fca130;
  background: rgba(252, 161, 48, 0.1);
}
.method-delete {
  color: #f93e3e;
  background: rgba(249, 62, 62, 0.1);
}
.method-patch {
  color: #50e3c2;
  background: rgba(80, 227, 194, 0.1);
}
.method-options {
  color: #9012fe;
  background: rgba(144, 18, 254, 0.1);
}
.method-head {
  color: #9012fe;
  background: rgba(144, 18, 254, 0.1);
}

.tab-name {
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  font-weight: 500;
}

.tab-close {
  opacity: 0;
  transition: opacity 0.2s;
  margin-left: auto;
}

.tab-item:hover .tab-close {
  opacity: 1;
}

.new-tab-btn {
  flex-shrink: 0;
  padding: 6px;
  margin-left: 4px;
}

.panels-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
}

.panels-container :deep(.n-split) {
  height: 100%;
}

.panels-container :deep(.n-split-pane-wrapper) {
  height: 100%;
  overflow: hidden;
  min-height: 0;
}

.panels-container :deep(.n-split-pane) {
  height: 100%;
  overflow: hidden;
  min-height: 0;
}

.panels-container :deep(.n-split__left),
.panels-container :deep(.n-split__right) {
  height: 100%;
  overflow: hidden;
}

.panels-container :deep(.n-split-bar) {
  cursor: ns-resize;
}

.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px;
}

.empty-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 24px;
  max-width: 400px;
  text-align: center;
}

.empty-content :deep(.n-button) {
  height: 48px;
  padding: 0 32px;
  font-size: 15px;
}

.empty-hint {
  color: var(--n-text-color-3);
  font-size: 14px;
  line-height: 1.6;
}

/* 响应式设计 */
@media (max-width: 1400px) {
  .panels-container {
    padding: 4px;
    gap: 4px;
  }
}

@media (max-width: 1200px) {
  .tab-item {
    max-width: 160px;
    padding: 2px 6px;
  }

  .tab-name {
    font-size: 12px;
  }
}

@media (max-width: 900px) {
  .tabs-bar {
    height: 28px;
    padding: 0 4px;
  }

  .tab-item {
    max-width: 140px;
    padding: 2px 6px;
  }

  .tab-method {
    font-size: 9px;
    padding: 1px 4px;
  }

  .panels-container {
    padding: 2px;
    gap: 2px;
  }
}

@media (min-width: 1600px) {
  .tabs-bar {
    height: 36px;
    padding: 0 10px;
  }

  .tab-item {
    max-width: 240px;
    padding: 4px 10px;
  }

  .tab-name {
    font-size: 14px;
  }

  .panels-container {
    padding: 8px;
    gap: 8px;
  }
}

@media (min-width: 2000px) {
  .tab-item {
    max-width: 280px;
  }
}
</style>
