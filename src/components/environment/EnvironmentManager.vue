<script setup lang="ts">
import { NButton, NInput, NModal, NDataTable, NIcon, NCheckbox, NPopconfirm, useMessage } from 'naive-ui'
import { ref, computed, onMounted, h } from 'vue'
import type { DataTableColumns } from 'naive-ui'
import { useEnvironmentStore } from '@/stores'
import { AppIcon } from '@/components/icons'
import { TrashOutline, AddOutline } from '@vicons/ionicons5'
import type { Environment, Variable } from '@/types'

const environmentStore = useEnvironmentStore()
const message = useMessage()

const searchQuery = ref('')
const editingEnvName = ref('')
const isEditingName = ref(false)
const showCreateModal = ref(false)
const newEnvName = ref('')

const filteredEnvironments = computed(() => {
  const envs = environmentStore.environments
  const defaultEnv = envs.find((e) => e.id === 'default')
  const otherEnvs = envs.filter((e) => e.id !== 'default')
  const sortedEnvs = defaultEnv ? [defaultEnv, ...otherEnvs] : otherEnvs

  if (!searchQuery.value) return sortedEnvs
  const query = searchQuery.value.toLowerCase()
  return sortedEnvs.filter(
    (e) =>
      e.name.toLowerCase().includes(query) ||
      e.variables.some((v) => v.key.toLowerCase().includes(query))
  )
})

function handleSelectEnv(env: Environment) {
  environmentStore.setManagerEnvironment(env.id)
}

function handleCreateEnv() {
  showCreateModal.value = true
  newEnvName.value = ''
}

async function confirmCreateEnv() {
  if (!newEnvName.value.trim()) return

  try {
    const newEnv = await environmentStore.createEnvironment(newEnvName.value.trim())
    environmentStore.setManagerEnvironment(newEnv.id)
    showCreateModal.value = false
    newEnvName.value = ''
    message.success('环境创建成功')
  } catch (error) {
    console.error('环境创建失败:', error)
    message.error(`创建环境失败: ${error}`)
  }
}

function cancelCreateEnv() {
  showCreateModal.value = false
  newEnvName.value = ''
}

async function handleDeleteEnv(envId: string) {
  try {
    await environmentStore.deleteEnvironment(envId)
    message.success('环境删除成功')
  } catch (error) {
    console.error('删除环境失败:', error)
    message.error(`删除环境失败: ${error}`)
  }
}

async function handleDuplicateEnv() {
  if (!environmentStore.managerEnvironment) return
  try {
    const newEnv = await environmentStore.duplicateEnvironment(
      environmentStore.managerEnvironment.id
    )
    if (newEnv) {
      environmentStore.setManagerEnvironment(newEnv.id)
      message.success('环境复制成功')
    }
  } catch (error) {
    console.error('复制环境失败:', error)
    message.error(`复制环境失败: ${error}`)
  }
}

function startEditingName() {
  if (!environmentStore.managerEnvironment) return
  editingEnvName.value = environmentStore.managerEnvironment.name
  isEditingName.value = true
}

async function saveEnvName() {
  if (!environmentStore.managerEnvironment || !editingEnvName.value.trim()) {
    isEditingName.value = false
    return
  }

  const newName = editingEnvName.value.trim()
  if (newName !== environmentStore.managerEnvironment.name) {
    try {
      await environmentStore.renameEnvironment(environmentStore.managerEnvironment.id, newName)
      message.success('环境重命名成功')
    } catch (error) {
      console.error('重命名环境失败:', error)
      message.error(`重命名环境失败: ${error}`)
    }
  }
  isEditingName.value = false
}

function cancelEditingName() {
  isEditingName.value = false
  editingEnvName.value = ''
}

function updateField(index: number, field: keyof Variable, value: string | boolean) {
  if (!environmentStore.managerEnvironment) return
  const variable = environmentStore.managerEnvironment.variables[index]
  environmentStore.updateVariable(environmentStore.managerEnvironment.id, index, {
    ...variable,
    [field]: value,
  })
}

function addRow() {
  if (!environmentStore.managerEnvironment) return
  environmentStore.addVariable(environmentStore.managerEnvironment.id, {
    key: '',
    value: '',
    enabled: true,
  })
}

async function deleteRow(index: number) {
  if (!environmentStore.managerEnvironment) return
  try {
    await environmentStore.deleteVariable(environmentStore.managerEnvironment.id, index)
  } catch (error) {
    console.error('删除变量失败:', error)
    message.error(`删除变量失败: ${error}`)
  }
}

const columns: DataTableColumns<Variable> = [
  {
    key: 'enabled',
    width: 50,
    align: 'center',
    render: (row, index) =>
      h(NCheckbox, {
        checked: row.enabled,
        onUpdateChecked: (checked: boolean) => updateField(index, 'enabled', checked),
      }),
  },
  {
    title: '变量名',
    key: 'key',
    render: (row, index) =>
      h(NInput, {
        value: row.key,
        placeholder: '变量名',
        size: 'small',
        onUpdateValue: (val: string) => updateField(index, 'key', val),
      }),
  },
  {
    title: '变量值',
    key: 'value',
    render: (row, index) =>
      h(NInput, {
        value: row.value,
        placeholder: '变量值',
        size: 'small',
        onUpdateValue: (val: string) => updateField(index, 'value', val),
      }),
  },
  {
    key: 'actions',
    width: 60,
    render: (_, index) =>
      h(
        NButton,
        { text: true, type: 'error', onClick: () => deleteRow(index) },
        { icon: () => h(NIcon, null, { default: () => h(TrashOutline) }) }
      ),
  },
]

onMounted(async () => {
  await environmentStore.loadEnvironments()
})

</script>

<template>
  <div class="environment-manager">
    <div class="env-content">
      <!-- 左侧环境列表 -->
      <div class="env-list-panel">
        <div class="list-header">
          <NInput
            v-model:value="searchQuery"
            placeholder="搜索环境..."
            clearable
            size="small"
            class="search-input"
          >
            <template #prefix>
              <AppIcon type="search" :size="14" />
            </template>
          </NInput>
          <NButton size="small" type="primary" class="add-env-btn" @click.stop="handleCreateEnv">
            <template #icon>
              <AppIcon type="plus" :size="14" />
            </template>
          </NButton>
        </div>
        
        <!-- 创建环境模态框 -->
        <NModal
          v-model:show="showCreateModal"
          preset="dialog"
          title="创建新环境"
          positive-text="创建"
          negative-text="取消"
          @positive-click="confirmCreateEnv"
          @negative-click="cancelCreateEnv"
        >
          <NInput
            v-model:value="newEnvName"
            placeholder="请输入环境名称"
            @keyup.enter="confirmCreateEnv"
          />
        </NModal>
        
        <div class="env-list">
          <div
            v-for="env in filteredEnvironments"
            :key="env.id"
            class="env-item"
            :class="{ active: environmentStore.managerEnvironmentId === env.id }"
            @click="handleSelectEnv(env)"
          >
            <span class="env-name">{{ env.name }}</span>
            <span class="env-info">{{ env.variables.length }}</span>
          </div>
          <div v-if="filteredEnvironments.length === 0" class="empty-list">
            <AppIcon type="environment" :size="32" />
            <p>未找到环境</p>
          </div>
        </div>
      </div>

      <!-- 右侧环境详情 -->
      <div class="variables-panel">
        <div v-if="environmentStore.managerEnvironment" class="detail-panel">
          <div class="env-detail-header">
            <div class="env-title-section">
              <div v-if="!isEditingName" class="env-name-display">
                <h3 class="env-title">{{ environmentStore.managerEnvironment.name }}</h3>
                <NButton 
                  v-if="environmentStore.managerEnvironment.id !== 'default'"
                  text 
                  size="tiny" 
                  class="edit-name-btn"
                  @click="startEditingName"
                >
                  <template #icon>
                    <AppIcon type="edit" :size="14" />
                  </template>
                </NButton>
              </div>
              <div v-else class="env-name-edit">
                <NInput
                  v-model:value="editingEnvName"
                  size="small"
                  placeholder="环境名称"
                  @keyup.enter="saveEnvName"
                  @keyup.esc="cancelEditingName"
                />
                <NButton size="small" type="primary" @click="saveEnvName">保存</NButton>
                <NButton size="small" @click="cancelEditingName">取消</NButton>
              </div>
            </div>
            
            <div class="env-actions">
              <NButton size="small" style="margin-right: 8px;" @click.stop="handleDuplicateEnv">
                <template #icon>
                  <AppIcon type="copy" :size="14" />
                </template>
                <span>复制环境</span>
              </NButton>
              <NPopconfirm
                v-if="environmentStore.managerEnvironment.id !== 'default'"
                positive-text="确定"
                negative-text="取消"
                @positive-click="handleDeleteEnv(environmentStore.managerEnvironment.id)"
              >
                <template #trigger>
                  <NButton size="small" style="background-color: #fff1f0; border-color: #ff4d4f; color: #cf1322;" @click.stop>
                    <NIcon :component="TrashOutline" :size="14" />
                    <span>删除环境</span>
                  </NButton>
                </template>
                确定要删除环境 "{{ environmentStore.managerEnvironment.name }}" 吗？此操作不可恢复。
              </NPopconfirm>
            </div>
          </div>

          <!-- 变量管理区域 -->
          <div class="variables-section">
            <div class="variables-content">
              <NDataTable
                :columns="columns"
                :data="environmentStore.managerEnvironment?.variables || []"
                :bordered="false"
                size="small"
                :row-key="(row: Variable) => row.id || row.key"
                class="variables-table"
              />
              <div class="editor-toolbar">
                <NButton text type="primary" size="small" @click="addRow">
                  <template #icon>
                    <NIcon :component="AddOutline" />
                  </template>
                  添加变量
                </NButton>
              </div>
            </div>
          </div>
        </div>

        <div v-else class="no-selection">
          <AppIcon type="environment" :size="48" />
          <p>请选择一个环境</p>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.environment-manager {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--n-color);
  overflow: hidden;
}

.env-content {
  flex: 1;
  display: flex;
  overflow: hidden;
  min-width: 0;
  min-height: 0;
}

/* 左侧环境列表面板 */
.env-list-panel {
  width: 200px;
  min-width: 160px;
  max-width: 260px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border-right: 1px solid var(--n-border-color);
  flex-shrink: 0;
  background: var(--n-color-modal);
}

.list-header {
  padding: 8px;
  border-bottom: 1px solid var(--n-border-color);
  display: flex;
  gap: 6px;
  align-items: center;
  flex-shrink: 0;
}

.search-input {
  flex: 1;
}

.search-input :deep(.n-input) {
  font-size: var(--font-size-compact-sm);
  height: 28px;
}

.add-env-btn {
  min-width: 28px;
  width: 28px;
  padding: 0;
}

.env-list {
  flex: 1;
  overflow-y: auto;
  padding: 6px;
}

.env-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 10px;
  margin-bottom: 2px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s ease;
  min-height: 32px;
  box-sizing: border-box;
}

.env-item:hover {
  background: var(--n-color-hover);
}

.env-item.active {
  background: var(--color-brand-light);
  box-shadow: inset 3px 0 0 var(--color-brand);
}

.env-name {
  flex: 1;
  font-size: var(--font-size-compact-sm);
  font-weight: 500;
  color: var(--n-text-color-1);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.env-info {
  font-size: var(--font-size-compact-xs);
  color: var(--n-text-color-3);
  font-weight: 500;
  padding: 1px 5px;
  background: var(--n-color-pressed);
  border-radius: 8px;
  min-width: 20px;
  text-align: center;
  flex-shrink: 0;
}

.empty-list {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 30px 16px;
  color: var(--n-text-color-3);
  text-align: center;
  gap: 6px;
}

.empty-list p {
  font-size: var(--font-size-compact-sm);
  margin: 0;
}

/* 右侧详情面板 */
.variables-panel {
  flex: 1;
  min-width: 350px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--n-color);
}

.detail-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 环境详情头部 */
.env-detail-header {
  padding: 10px 16px;
  border-bottom: 1px solid var(--n-border-color);
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  background: var(--n-color-modal);
  flex-shrink: 0;
}

.env-title-section {
  flex: 1;
  min-width: 0;
}

.env-name-display {
  display: flex;
  align-items: center;
  gap: 6px;
}

.env-title {
  margin: 0;
  font-size: var(--font-size-compact-xl);
  font-weight: 600;
  color: var(--n-text-color-1);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.edit-name-btn {
  flex-shrink: 0;
  opacity: 0.6;
  transition: opacity 0.2s;
}

.edit-name-btn:hover {
  opacity: 1;
}

.env-name-edit {
  display: flex;
  align-items: center;
  gap: 6px;
}

.env-name-edit :deep(.n-input) {
  flex: 1;
  max-width: 250px;
}

.env-name-edit :deep(.n-input) {
  height: 28px;
  font-size: var(--font-size-compact-sm);
}

.env-name-edit :deep(.n-button) {
  height: 28px;
  font-size: var(--font-size-compact-sm);
  padding: 0 10px;
}

.env-actions {
  flex-shrink: 0;
  display: flex;
  align-items: center;
}

.env-actions :deep(.n-button) {
  height: 28px;
  font-size: var(--font-size-compact-sm);
  padding: 0 10px;
}

/* 变量管理区域 */
.variables-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 0;
  height: 0;
}

.variables-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 0;
}

.variables-table {
  flex: 1;
  min-height: 0;
}

.variables-table :deep(.n-data-table) {
  height: 100%;
}

.variables-table :deep(.n-data-table-wrapper) {
  height: 100%;
  overflow: auto;
}

.variables-table :deep(.n-data-table-td) {
  padding: 4px 8px;
}

.variables-table :deep(.n-data-table-th) {
  padding: 6px 8px;
}

.variables-table :deep(.n-input) {
  --n-height: 28px;
}

.editor-toolbar {
  padding: 8px 0;
  display: flex;
  gap: 16px;
  align-items: center;
  flex-shrink: 0;
}

/* 响应式调整 */
@media (max-width: 1200px) {
  .env-list-panel {
    width: 180px;
    min-width: 160px;
  }
  
  .variables-panel {
    min-width: 320px;
  }
}

@media (max-width: 900px) {
  .env-content {
    flex-direction: column;
  }
  
  .env-list-panel {
    width: 100%;
    max-width: none;
    height: 160px;
    border-right: none;
    border-bottom: 1px solid var(--n-border-color);
  }
  
  .variables-panel {
    min-width: auto;
  }
  
  .env-detail-header {
    padding: 8px 12px;
  }
}

@media (max-width: 600px) {
  .env-name-edit {
    flex-direction: column;
    align-items: stretch;
  }

  .env-name-edit :deep(.n-input) {
    max-width: none;
  }
}

.empty-state,
.no-selection {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  color: var(--n-text-color-3);
  text-align: center;
  gap: 10px;
}

.empty-state p,
.no-selection p {
  font-size: 12px;
  margin: 0;
}
</style>
