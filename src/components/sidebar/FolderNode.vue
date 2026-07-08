<script setup lang="ts">
// 递归文件夹节点组件：自引用渲染任意深度的文件夹树
defineOptions({ name: 'FolderNode' })

import { NTooltip, NButton, NDropdown } from 'naive-ui'
import { ref, computed, nextTick, watch } from 'vue'
import { useRequestStore } from '@/stores'
import { useI18n } from '@/composables/useI18n'
import { HttpMethodIcon, AppIcon } from '@/components/icons'
import type { Folder, Request } from '@/types'

const props = withDefaults(
  defineProps<{
    folder: Folder
    collectionId: string
    depth?: number
    expandedKeys: Set<string>
    /** 当前拖拽命中的目标文件夹 id（用于高亮 drop target） */
    dropFolderId?: string
  }>(),
  { depth: 0, dropFolderId: undefined }
)

const emit = defineEmits<{
  'toggle-expand': [key: string]
  'select-request': [request: Request, collectionId: string]
  'open-request': [request: Request, collectionId: string]
  'context-request': [e: MouseEvent, request: Request, collectionId: string, folderId: string]
  'rename-request': [id: string, name: string]
  'delete-request': [collectionId: string, requestId: string]
  'drag-start': [e: MouseEvent, request: Request, collectionId: string, folderId: string]
}>()

const requestStore = useRequestStore()
const { t } = useI18n()

const editingId = ref<string | null>(null)
const editingName = ref('')
const showFolderMenu = ref(false)
const folderMenuX = ref(0)
const folderMenuY = ref(0)
const showNewFolderInput = ref(false)
const newFolderName = ref('')
const newFolderInput = ref<HTMLInputElement | null>(null)

// 新建子文件夹输入框显示时自动聚焦
watch(showNewFolderInput, (show) => {
  if (show) {
    nextTick(() => {
      newFolderInput.value?.focus()
      newFolderInput.value?.select()
    })
  }
})

// 文件夹展开 key 加前缀，避免与 collection id 冲突
function folderKey(id: string): string {
  return `folder:${id}`
}

function isExpanded(id: string): boolean {
  return props.expandedKeys.has(folderKey(id))
}

function toggle(id: string) {
  emit('toggle-expand', folderKey(id))
}

const folderMenuOptions = [
  { label: () => t('collection.newFolder'), key: 'new-folder' },
  { label: () => t('common.rename'), key: 'rename' },
  { label: () => t('common.delete'), key: 'delete' },
]

function handleFolderContextMenu(e: MouseEvent, _folder: Folder) {
  e.preventDefault()
  e.stopPropagation()
  folderMenuX.value = e.clientX
  folderMenuY.value = e.clientY
  showFolderMenu.value = true
}

function handleFolderMenuSelect(key: string) {
  showFolderMenu.value = false
  const folder = props.folder
  if (key === 'new-folder') {
    newFolderName.value = ''
    showNewFolderInput.value = true
  } else if (key === 'rename') {
    editingId.value = folder.id
    editingName.value = folder.name
  } else if (key === 'delete') {
    requestStore.deleteFolder(props.collectionId, folder.id)
  }
}

async function finishFolderRename() {
  if (editingId.value && editingName.value.trim()) {
    await requestStore.renameFolder(props.collectionId, editingId.value, editingName.value.trim())
  }
  editingId.value = null
}

async function confirmNewSubFolder() {
  const name = newFolderName.value.trim()
  if (name) {
    await requestStore.createFolder(props.collectionId, name, props.folder.id)
    // 创建后展开当前文件夹以便看到新建的子文件夹
    if (!isExpanded(props.folder.id)) {
      emit('toggle-expand', folderKey(props.folder.id))
    }
  }
  showNewFolderInput.value = false
  newFolderName.value = ''
}

function startRenameRequest(id: string, name: string) {
  emit('rename-request', id, name)
}

function isRequestActive(requestId: string): boolean {
  return requestStore.currentTab?.request.id === requestId
}

function handleMenuClickoutside() {
  showFolderMenu.value = false
}

const isDropTarget = computed(() => props.dropFolderId === props.folder.id)
</script>

<template>
  <div class="folder-node">
    <!-- 文件夹节点（同时作为拖拽 drop target） -->
    <div
      class="list-item folder-item folder-drop-target"
      :class="{ 'drop-active': isDropTarget }"
      :style="{ paddingLeft: 8 + depth * 16 + 'px' }"
      :data-folder-id="folder.id"
      @click="toggle(folder.id)"
      @contextmenu="handleFolderContextMenu($event, folder)"
    >
      <div class="item-content">
        <AppIcon
          :type="isExpanded(folder.id) ? 'chevronDown' : 'chevronRight'"
          :size="14"
          class="expand-icon"
        />
        <AppIcon
          :type="isExpanded(folder.id) ? 'folderOpen' : 'folder'"
          :size="16"
          class="item-icon"
        />
        <template v-if="editingId === folder.id">
          <input
            v-model="editingName"
            class="rename-input"
            @blur="finishFolderRename"
            @keyup.enter="finishFolderRename"
            @keyup.escape="editingId = null"
            @click.stop
          />
        </template>
        <template v-else>
          <span class="item-name">{{ folder.name }}</span>
        </template>
      </div>
    </div>

    <!-- 新建子文件夹输入框 -->
    <div v-if="showNewFolderInput" class="new-folder-input-row" :style="{ paddingLeft: 8 + (depth + 1) * 16 + 'px' }">
      <AppIcon type="folder" :size="16" class="item-icon" />
      <input
        ref="newFolderInput"
        v-model="newFolderName"
        class="rename-input"
        :placeholder="t('collection.newFolder')"
        @blur="confirmNewSubFolder"
        @keyup.enter="confirmNewSubFolder"
        @keyup.escape="showNewFolderInput = false"
        @click.stop
      />
    </div>

    <!-- 文件夹展开内容：子 requests + 递归子 folders -->
    <div v-if="isExpanded(folder.id)" class="folder-children">
      <!-- 直接子 requests -->
      <div
        v-for="request in folder.requests"
        :key="request.id"
        class="list-item request-item"
        :class="{
          active: isRequestActive(request.id),
          selected: requestStore.isInSelection(request.id),
        }"
        :style="{ paddingLeft: 8 + (depth + 1) * 16 + 'px' }"
        :data-collection-id="collectionId"
        :data-folder-id="folder.id"
        @click="emit('select-request', request, collectionId)"
        @dblclick="emit('open-request', request, collectionId)"
        @contextmenu="emit('context-request', $event, request, collectionId, folder.id)"
        @mousedown="emit('drag-start', $event, request, collectionId, folder.id)"
      >
        <div class="item-content">
          <div
            v-if="requestStore.isSelectionMode"
            class="selection-checkbox"
            @click.stop="requestStore.toggleSelection(request.id)"
          >
            <AppIcon
              :type="requestStore.isInSelection(request.id) ? 'check' : 'plus'"
              :size="14"
            />
          </div>
          <HttpMethodIcon :method="request.method" :size="12" filled />
          <span class="item-name">{{ request.name }}</span>
        </div>
        <div class="item-actions">
          <NTooltip :show-arrow="false" placement="top">
            <template #trigger>
              <NButton
                size="small"
                quaternary
                class="action-btn"
                @click.stop="startRenameRequest(request.id, request.name)"
              >
                <template #icon>
                  <AppIcon type="edit" class="btn-icon edit-icon" />
                </template>
              </NButton>
            </template>
            {{ t('common.rename') }}
          </NTooltip>
          <NTooltip :show-arrow="false" placement="top">
            <template #trigger>
              <NButton
                size="small"
                quaternary
                class="action-btn delete-btn"
                @click.stop="emit('delete-request', collectionId, request.id)"
              >
                <template #icon>
                  <AppIcon type="trash" class="btn-icon delete-icon" />
                </template>
              </NButton>
            </template>
            {{ t('common.delete') }}
          </NTooltip>
        </div>
      </div>

      <!-- 递归渲染子文件夹 -->
      <FolderNode
        v-for="subFolder in folder.folders"
        :key="subFolder.id"
        :folder="subFolder"
        :collection-id="collectionId"
        :depth="depth + 1"
        :expanded-keys="expandedKeys"
        :drop-folder-id="dropFolderId"
        @toggle-expand="(key: string) => emit('toggle-expand', key)"
        @select-request="(r: Request, cid: string) => emit('select-request', r, cid)"
        @open-request="(r: Request, cid: string) => emit('open-request', r, cid)"
        @context-request="(e: MouseEvent, r: Request, cid: string, fid: string) => emit('context-request', e, r, cid, fid)"
        @rename-request="(id: string, name: string) => emit('rename-request', id, name)"
        @delete-request="(cid: string, rid: string) => emit('delete-request', cid, rid)"
        @drag-start="(e: MouseEvent, r: Request, cid: string, fid: string) => emit('drag-start', e, r, cid, fid)"
      />
    </div>

    <!-- 文件夹右键菜单 -->
    <NDropdown
      placement="bottom-start"
      trigger="manual"
      :x="folderMenuX"
      :y="folderMenuY"
      :options="folderMenuOptions"
      :show="showFolderMenu"
      @select="handleFolderMenuSelect"
      @clickoutside="handleMenuClickoutside"
    />
  </div>
</template>

<style scoped>
.folder-node {
  user-select: none;
}

.list-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 32px;
  padding-right: 8px;
  cursor: pointer;
  transition: background-color 0.15s;
}

.list-item:hover {
  background-color: var(--n-color-hover);
}

.list-item.drop-active {
  background-color: rgba(24, 160, 88, 0.15);
  outline: 1px dashed var(--n-primary-color);
  outline-offset: -1px;
}

.list-item:hover .item-actions {
  opacity: 1;
}

.list-item.active {
  background-color: var(--n-color-active);
}

.list-item.selected {
  background-color: var(--n-color-selected, rgba(24, 160, 88, 0.1));
}

.item-content {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
  flex: 1;
}

.expand-icon {
  flex-shrink: 0;
  color: var(--n-text-color-3);
}

.item-icon {
  flex-shrink: 0;
}

.item-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 13px;
}

.item-actions {
  display: flex;
  gap: 2px;
  opacity: 0;
  transition: opacity 0.15s;
}

.action-btn {
  padding: 2px;
}

.btn-icon {
  font-size: 14px;
}

.selection-checkbox {
  cursor: pointer;
  display: flex;
  align-items: center;
}

.rename-input {
  flex: 1;
  min-width: 0;
  height: 24px;
  padding: 0 4px;
  border: 1px solid var(--n-primary-color);
  border-radius: 3px;
  background: var(--n-color);
  color: var(--n-text-color);
  font-size: 13px;
  outline: none;
}

.new-folder-input-row {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 32px;
  padding-right: 8px;
}
</style>
