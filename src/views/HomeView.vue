<script setup lang="ts">
import { NButton, NIcon, NDropdown, useMessage, NSplit, NModal } from 'naive-ui'
import { CloseOutline, AddOutline } from '@vicons/ionicons5'
import { ref, onMounted, onUnmounted, watch, nextTick } from 'vue'
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
import { normalizeRequest, type RawRequest } from '@/types'

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

/* ===== 请求 tab 溢出检测与激活跟随 ===== */
const tabsScrollEl = ref<HTMLElement | null>(null)
const tabsOverflowLeft = ref(false)
const tabsOverflowRight = ref(false)

function checkTabsOverflow() {
  const el = tabsScrollEl.value
  if (!el) {
    tabsOverflowLeft.value = false
    tabsOverflowRight.value = false
    return
  }
  const hasOverflow = el.scrollWidth > el.clientWidth + 1
  tabsOverflowLeft.value = hasOverflow && el.scrollLeft > 4
  tabsOverflowRight.value = hasOverflow && el.scrollLeft + el.clientWidth < el.scrollWidth - 4
}

function scrollActiveTabIntoView() {
  nextTick(() => {
    tabsScrollEl.value
      ?.querySelector('.tab-item.active')
      ?.scrollIntoView({ block: 'nearest', inline: 'nearest', behavior: 'smooth' })
  })
}

/* ===== 侧栏拖拽调宽（220–420px，与折叠状态互斥） ===== */
const sidebarWidth = ref(300)
const sidebarResizing = ref(false)
let tabsResizeObserver: ResizeObserver | null = null

function handleSidebarResizeStart(e: MouseEvent) {
  e.preventDefault()
  sidebarResizing.value = true
  const startX = e.clientX
  const startWidth = sidebarWidth.value
  const onMove = (ev: MouseEvent) => {
    sidebarWidth.value = Math.min(420, Math.max(220, startWidth + ev.clientX - startX))
  }
  const onUp = () => {
    sidebarResizing.value = false
    window.removeEventListener('mousemove', onMove)
    window.removeEventListener('mouseup', onUp)
  }
  window.addEventListener('mousemove', onMove)
  window.addEventListener('mouseup', onUp)
}

/* tabs-bar 是 v-if 渲染的，el 挂载/销毁时同步管理 observer 生命周期 */
watch(
  tabsScrollEl,
  (el, _prev, onCleanup) => {
    tabsResizeObserver?.disconnect()
    if (el) {
      tabsResizeObserver = new ResizeObserver(checkTabsOverflow)
      tabsResizeObserver.observe(el)
      checkTabsOverflow()
    }
    onCleanup(() => tabsResizeObserver?.disconnect())
  },
  { flush: 'post' }
)

watch(
  () => requestStore.activeTabId,
  () => scrollActiveTabIntoView()
)

watch(
  () => requestStore.tabs.length,
  () => scrollActiveTabIntoView()
)

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

function handleCurlImport(request: RawRequest) {
  const normalizedRequest = normalizeRequest(request)
  requestStore.openRequest(normalizedRequest, undefined, true)
  message.success(t('import.importSuccess', { count: 1 }))
}

function handleRequestsImport(requests: RawRequest[], _source: string) {
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
  const curl = generateCode(requestStore.currentRequest, 'curl', { includeDependencies: false })
  navigator.clipboard.writeText(curl)
  message.success(t('request.copyAsCurl'))
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
  tabsResizeObserver?.disconnect()
})

watch(collapsed, (val) => {
  settingsStore.updateAppState({ sidebarCollapsed: val })
  settingsStore.saveAppState()
})

useShortcuts([
  {
    id: 'sendRequest',
    key: 'Enter',
    ctrl: true,
    handler: sendRequest,
    description: t('shortcuts.sendRequest'),
    category: 'global',
  },
  {
    id: 'saveRequest',
    key: 's',
    ctrl: true,
    handler: saveRequest,
    description: t('shortcuts.saveRequest'),
    category: 'global',
  },
  {
    id: 'newRequest',
    key: 'n',
    ctrl: true,
    handler: newRequest,
    description: t('shortcuts.newRequest'),
    category: 'global',
  },
  {
    id: 'closeTab',
    key: 'w',
    ctrl: true,
    handler: () => {
      if (requestStore.activeTabId) {
        requestStore.closeTab(requestStore.activeTabId)
      }
    },
    description: t('shortcuts.closeTab'),
    category: 'global',
  },
  {
    id: 'importCurl',
    key: 'i',
    ctrl: true,
    shift: true,
    handler: () => (showCurlImport.value = true),
    description: t('shortcuts.importCurl'),
    category: 'global',
  },
  {
    id: 'openEnvironment',
    key: 'e',
    ctrl: true,
    shift: true,
    handler: () => (showEnvironmentManager.value = true),
    description: t('shortcuts.openEnvironment'),
    category: 'global',
  },
  {
    id: 'openHistory',
    key: 'h',
    ctrl: true,
    shift: true,
    handler: () => {
      const event = new CustomEvent('toggle-history')
      window.dispatchEvent(event)
    },
    description: t('shortcuts.openHistory'),
    category: 'global',
  },
  {
    id: 'openSettings',
    key: ',',
    ctrl: true,
    handler: () => (showSettings.value = true),
    description: t('shortcuts.openSettings'),
    category: 'global',
  },
  {
    id: 'toggleSidebar',
    key: 'b',
    ctrl: true,
    handler: toggleSidebar,
    description: t('shortcuts.toggleSidebar'),
    category: 'global',
  },
  {
    id: 'nextTab',
    key: 'ArrowRight',
    ctrl: true,
    handler: () => {
      const tabs = requestStore.tabs
      const currentIndex = tabs.findIndex((t) => t.id === requestStore.activeTabId)
      if (currentIndex < tabs.length - 1) {
        requestStore.switchTab(tabs[currentIndex + 1].id)
      } else if (tabs.length > 0) {
        requestStore.switchTab(tabs[0].id)
      }
    },
    description: t('shortcuts.nextTab'),
    category: 'global',
  },
  {
    id: 'prevTab',
    key: 'ArrowLeft',
    ctrl: true,
    handler: () => {
      const tabs = requestStore.tabs
      const currentIndex = tabs.findIndex((t) => t.id === requestStore.activeTabId)
      if (currentIndex > 0) {
        requestStore.switchTab(tabs[currentIndex - 1].id)
      } else if (tabs.length > 0) {
        requestStore.switchTab(tabs[tabs.length - 1].id)
      }
    },
    description: t('shortcuts.prevTab'),
    category: 'global',
  },
])
</script>

<template>
  <div class="home-view">
    <TitleBar
      @toggle-theme="toggleTheme"
      @select-environment="environmentStore.setCurrentEnvironment"
      @open-settings="showSettings = true"
      @open-environment-manager="handleOpenEnvironmentManager"
    />
    <div class="main-layout">
      <div
        class="sidebar-wrapper"
        :class="{ collapsed, resizing: sidebarResizing }"
        :style="collapsed ? undefined : { width: `${sidebarWidth}px` }"
      >
        <Sidebar
          :collapsed="collapsed"
          @toggle="toggleSidebar"
          @new-request="newRequest"
          @import="handleImportSelect"
          @export="handleExport"
          @settings="showSettings = true"
          @code="handleCodeSelect"
        />
        <div
          v-if="!collapsed"
          class="sidebar-resizer"
          :class="{ active: sidebarResizing }"
          @mousedown="handleSidebarResizeStart"
        ></div>
      </div>
      <div class="main-content">
        <div v-if="requestStore.tabs.length > 0" class="tabs-bar">
          <div ref="tabsScrollEl" class="tabs-scroll" @scroll.passive="checkTabsOverflow">
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
          <div v-if="tabsOverflowLeft" class="tabs-fade left" aria-hidden="true"></div>
          <div v-if="tabsOverflowRight" class="tabs-fade" aria-hidden="true"></div>
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
            :default-size="0.62"
            :min="0.15"
            :max="0.85"
            :resize-trigger-size="5"
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
            <NButton quaternary size="large" @click="showCurlImport = true">
              {{ t('import.curl') }}
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
  background: var(--color-bg-base);
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
  overflow: visible;
  position: relative;
}

.sidebar-wrapper.collapsed {
  width: 48px;
}

.sidebar-wrapper.resizing {
  transition: none;
}

.sidebar-resizer {
  position: absolute;
  top: 0;
  right: -3px;
  bottom: 0;
  width: 6px;
  cursor: col-resize;
  z-index: 20;
  transition: background-color var(--transition-fast);
}

.sidebar-resizer:hover,
.sidebar-resizer.active {
  background-color: var(--color-brand-light);
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
  background: var(--color-bg-elevated);
  position: relative;
}

.tabs-bar {
  height: 32px;
  background: var(--color-bg-elevated);
  border-bottom: 1px solid var(--color-border);
  display: flex;
  align-items: center;
  padding: 0 8px;
  flex-shrink: 0;
  gap: 4px;
  position: relative;
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

.tabs-fade {
  position: absolute;
  top: 0;
  bottom: 0;
  width: 40px;
  pointer-events: none;
  background: linear-gradient(to right, transparent, var(--color-bg-elevated));
}

.tabs-fade.left {
  left: 0;
  right: auto;
  background: linear-gradient(to left, transparent, var(--color-bg-elevated));
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
  color: var(--color-method-get);
  background: var(--color-method-get-bg);
}
.method-post {
  color: var(--color-method-post);
  background: var(--color-method-post-bg);
}
.method-put {
  color: var(--color-method-put);
  background: var(--color-method-put-bg);
}
.method-delete {
  color: var(--color-method-delete);
  background: var(--color-method-delete-bg);
}
.method-patch {
  color: var(--color-method-patch);
  background: var(--color-method-patch-bg);
}
.method-options {
  color: var(--color-method-options);
  background: var(--color-method-options-bg);
}
.method-head {
  color: var(--color-method-head);
  background: var(--color-method-head-bg);
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

/* naive-ui 分割条的真实类名是 n-split__resize-trigger（wrapper 负责事件） */
.panels-container :deep(.n-split__resize-trigger-wrapper) {
  position: relative;
}

.panels-container :deep(.n-split__resize-trigger) {
  transition: background-color var(--transition-fast);
}

.panels-container :deep(.n-split__resize-trigger-wrapper:hover .n-split__resize-trigger) {
  background-color: var(--color-border);
}

.panels-container :deep(.n-split__resize-trigger-wrapper:hover)::after {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 24px;
  height: 4px;
  border-radius: var(--radius-full);
  background: var(--color-border);
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
