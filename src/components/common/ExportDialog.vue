<script setup lang="ts">
import { NModal, NCard, NButton, NCheckbox, NTree, NSpace, NAlert, useMessage } from 'naive-ui'
import type { TreeOption } from 'naive-ui'
import { computed, h, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { writeFile } from '@tauri-apps/plugin-fs'
import { useI18n } from '@/composables/useI18n'
import type { Collection, Folder, Request, HttpMethod } from '@/types'

/** 传给后端的最小导出节点（后端按 camelCase 反序列化） */
interface ExportTreeNode {
  name: string
  description?: string
  folders: ExportTreeNode[]
  requests: Request[]
}

const props = defineProps<{
  show: boolean
  collections: Collection[]
  /** 从集合右键打开时预选该集合 */
  preselectedCollectionId?: string
}>()

const emit = defineEmits<{
  'update:show': [show: boolean]
}>()

const message = useMessage()
const { t } = useI18n()

const error = ref('')
const isLoading = ref(false)

const checkedKeys = ref<string[]>([])
const includeParams = ref(true)
const includeAuth = ref(true)
const includeScripts = ref(true)
const includeDescription = ref(true)

/* 树节点 key：c/f/r 前缀区分集合、文件夹、请求 */
const collectionKey = (id: string) => `c:${id}`
const folderKey = (id: string) => `f:${id}`
const requestKey = (id: string) => `r:${id}`

function folderToTree(folder: Folder): TreeOption {
  return {
    key: folderKey(folder.id),
    label: folder.name,
    type: 'folder',
    children: [
      ...folder.folders.map(folderToTree),
      ...folder.requests.map((r) => ({
        key: requestKey(r.id),
        label: r.name || r.url,
        type: 'request',
        method: r.method,
      })),
    ],
  }
}

const treeData = computed<TreeOption[]>(() =>
  props.collections.map((c) => ({
    key: collectionKey(c.id),
    label: c.name,
    type: 'collection',
    children: [
      ...c.folders.map(folderToTree),
      ...c.requests.map((r) => ({
        key: requestKey(r.id),
        label: r.name || r.url,
        type: 'request',
        method: r.method,
      })),
    ],
  }))
)

const allRequestKeys = computed(() => {
  const keys: string[] = []
  const walkFolders = (folders: Folder[]) => {
    for (const f of folders) {
      f.requests.forEach((r) => keys.push(requestKey(r.id)))
      walkFolders(f.folders)
    }
  }
  props.collections.forEach((c) => {
    c.requests.forEach((r) => keys.push(requestKey(r.id)))
    walkFolders(c.folders)
  })
  return keys
})

const selectedRequestCount = computed(
  () => checkedKeys.value.filter((k) => k.startsWith('r:')).length
)

/* 弹窗打开时初始化勾选：预选集合则只勾它，否则全选 */
watch(
  () => props.show,
  (show) => {
    if (!show) return
    error.value = ''
    if (props.preselectedCollectionId) {
      const collection = props.collections.find((c) => c.id === props.preselectedCollectionId)
      if (collection) {
        const keys: string[] = []
        const walkFolders = (folders: Folder[]) => {
          for (const f of folders) {
            keys.push(folderKey(f.id))
            f.requests.forEach((r) => keys.push(requestKey(r.id)))
            walkFolders(f.folders)
          }
        }
        keys.push(collectionKey(collection.id))
        collection.requests.forEach((r) => keys.push(requestKey(r.id)))
        walkFolders(collection.folders)
        checkedKeys.value = keys
        return
      }
    }
    checkedKeys.value = allRequestKeys.value
  }
)

/* 防御：集合数据在弹窗打开后才就绪时（异步加载），树挂载同步会丢弃无效勾选，
   数据到位且当前无勾选时自动恢复全选 */
watch(
  treeData,
  (nodes) => {
    if (props.show && nodes.length > 0 && checkedKeys.value.length === 0) {
      checkedKeys.value = allRequestKeys.value
    }
  }
)

function handleSelectAll() {
  const keys = [...allRequestKeys.value]
  props.collections.forEach((c) => keys.push(collectionKey(c.id)))
  const walkFolders = (folders: Folder[]) => {
    for (const f of folders) {
      keys.push(folderKey(f.id))
      walkFolders(f.folders)
    }
  }
  props.collections.forEach((c) => walkFolders(c.folders))
  checkedKeys.value = keys
}

function handleSelectNone() {
  checkedKeys.value = []
}

/** 按勾选构建导出树：剔除未勾选的请求，空文件夹/空集合整体跳过 */
function buildExportTree(): ExportTreeNode[] {
  const checked = new Set(checkedKeys.value)

  const buildFolder = (folder: Folder): ExportTreeNode | null => {
    const folders = folder.folders.map(buildFolder).filter((f): f is ExportTreeNode => f !== null)
    const requests = folder.requests.filter((r) => checked.has(requestKey(r.id)))
    if (folders.length === 0 && requests.length === 0) return null
    return { name: folder.name, description: folder.description, folders, requests }
  }

  const result: ExportTreeNode[] = []
  for (const c of props.collections) {
    const folders = c.folders.map(buildFolder).filter((f): f is ExportTreeNode => f !== null)
    const requests = c.requests.filter((r) => checked.has(requestKey(r.id)))
    if (folders.length === 0 && requests.length === 0) continue
    result.push({ name: c.name, description: c.description, folders, requests })
  }
  return result
}

async function handleExport() {
  const tree = buildExportTree()
  if (tree.length === 0 || selectedRequestCount.value === 0) {
    error.value = t('export.selectAtLeastOne')
    return
  }

  isLoading.value = true
  error.value = ''
  try {
    const json = await invoke<string>('export_postman_collection', {
      collections: tree,
      options: {
        includeParams: includeParams.value,
        includeAuth: includeAuth.value,
        includeScripts: includeScripts.value,
        includeDescription: includeDescription.value,
      },
    })

    const defaultName =
      tree.length === 1 ? `${tree[0].name}.postman_collection.json` : 'iapi-collection.json'

    if ('__TAURI_INTERNALS__' in window) {
      const path = await save({
        defaultPath: defaultName,
        filters: [{ name: 'Postman Collection', extensions: ['json'] }],
      })
      if (!path) return
      await writeFile(path, new TextEncoder().encode(json))
    } else {
      // 浏览器环境（开发/E2E）回退为网页下载
      const blob = new Blob([json], { type: 'application/json' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = defaultName
      a.click()
      URL.revokeObjectURL(url)
    }

    message.success(t('export.exportSuccess'))
    handleClose()
  } catch (e) {
    error.value = e instanceof Error && 'path' in e ? t('export.saveFailed') : String(e)
  } finally {
    isLoading.value = false
  }
}

function handleClose() {
  error.value = ''
  emit('update:show', false)
}

function renderTreeLabel({ option }: { option: TreeOption }) {
  if (option.type !== 'request') return option.label as string
  const method = (option.method as HttpMethod) || 'GET'
  return h('span', { class: 'tree-request-label' }, [
    h(
      'span',
      { class: 'tree-request-method', style: { color: `var(--color-method-${method.toLowerCase()})` } },
      method
    ),
    h('span', { class: 'tree-request-name' }, option.label as string),
  ])
}
</script>

<template>
  <NModal :show="props.show" @update:show="emit('update:show', $event)">
    <NCard style="width: 520px" :title="t('export.title')" :bordered="false" size="medium">
      <NAlert v-if="error" type="error" style="margin-bottom: 12px">
        {{ error }}
      </NAlert>

      <div class="dialog-section">
        <div class="section-header">
          <span class="section-title">{{ t('export.scope') }}</span>
          <div class="section-actions">
            <NButton text size="tiny" @click="handleSelectAll">{{ t('export.selectAll') }}</NButton>
            <NButton text size="tiny" @click="handleSelectNone">
              {{ t('export.selectNone') }}
            </NButton>
          </div>
        </div>
        <div class="tree-scroll">
          <NTree
            v-model:checked-keys="checkedKeys"
            :data="treeData"
            :render-label="renderTreeLabel"
            checkable
            cascade
            block-line
            expand-on-click
            :default-expanded-keys="treeData.map((n) => n.key as string)"
          />
        </div>
      </div>

      <div class="dialog-section">
        <span class="section-title">{{ t('export.includeContent') }}</span>
        <NSpace size="large">
          <NCheckbox v-model:checked="includeParams">{{ t('export.includeParams') }}</NCheckbox>
          <NCheckbox v-model:checked="includeAuth">{{ t('export.includeAuth') }}</NCheckbox>
          <NCheckbox v-model:checked="includeScripts">{{ t('export.includeScripts') }}</NCheckbox>
          <NCheckbox v-model:checked="includeDescription">
            {{ t('export.includeDescription') }}
          </NCheckbox>
        </NSpace>
      </div>

      <template #footer>
        <div class="dialog-footer">
          <span class="request-count">{{ t('export.requestCount', { count: selectedRequestCount }) }}</span>
          <NSpace>
            <NButton @click="handleClose">{{ t('common.cancel') }}</NButton>
            <NButton type="primary" :loading="isLoading" @click="handleExport">
              {{ t('export.export') }}
            </NButton>
          </NSpace>
        </div>
      </template>
    </NCard>
  </NModal>
</template>

<style scoped>
.dialog-section {
  margin-bottom: 16px;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.section-title {
  font-size: var(--font-size-compact-md);
  font-weight: 500;
  color: var(--n-text-color-2);
}

.section-actions {
  display: flex;
  gap: 4px;
}

.tree-scroll {
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: 6px;
  max-height: 260px;
  overflow-y: auto;
}

.tree-request-label {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.tree-request-method {
  font-size: var(--font-size-compact-xs);
  font-weight: 700;
  font-family: var(--font-family-mono);
  flex-shrink: 0;
  width: 48px;
}

.tree-request-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dialog-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.request-count {
  font-size: var(--font-size-compact-sm);
  color: var(--n-text-color-3);
}
</style>
