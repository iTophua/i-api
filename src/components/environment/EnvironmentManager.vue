<script setup lang="ts">
import { NButton, NInput, NFlex, NTag, NPopconfirm } from 'naive-ui'
import { ref, computed } from 'vue'
import { useEnvironmentStore } from '@/stores'
import { AppIcon } from '@/components/icons'
import type { Environment, Variable } from '@/types'

const environmentStore = useEnvironmentStore()

const newVarKey = ref('')
const newVarValue = ref('')
const searchQuery = ref('')

const filteredEnvironments = computed(() => {
  if (!searchQuery.value) return environmentStore.environments
  const query = searchQuery.value.toLowerCase()
  return environmentStore.environments.filter(
    (e) =>
      e.name.toLowerCase().includes(query) ||
      e.variables.some((v) => v.key.toLowerCase().includes(query))
  )
})

const activeVariables = computed(() => {
  const env = environmentStore.currentEnvironment
  if (!env) return []
  if (!searchQuery.value) return env.variables
  const query = searchQuery.value.toLowerCase()
  return env.variables.filter(
    (v) => v.key.toLowerCase().includes(query) || v.value.toLowerCase().includes(query)
  )
})

function handleSelectEnv(env: Environment) {
  environmentStore.setCurrentEnvironment(env.id)
}

async function handleCreateEnv() {
  const name = prompt('请输入环境名称：')
  if (name) {
    await environmentStore.createEnvironment(name)
  }
}

async function handleDeleteEnv(envId: string) {
  await environmentStore.deleteEnvironment(envId)
}

function handleAddVariable() {
  if (!newVarKey.value.trim() || !environmentStore.currentEnvironment) return

  const newVar: Variable = {
    key: newVarKey.value.trim(),
    value: newVarValue.value,
    enabled: true,
  }

  environmentStore.addVariable(environmentStore.currentEnvironment.id, newVar)
  newVarKey.value = ''
  newVarValue.value = ''
}

async function handleDeleteVariable(index: number) {
  if (!environmentStore.currentEnvironment) return
  await environmentStore.deleteVariable(environmentStore.currentEnvironment.id, index)
}

async function handleToggleVariable(index: number, variable: Variable) {
  if (!environmentStore.currentEnvironment) return
  await environmentStore.updateVariable(environmentStore.currentEnvironment.id, index, {
    ...variable,
    enabled: !variable.enabled,
  })
}

function handleExportEnv() {
  const env = environmentStore.currentEnvironment
  if (!env) return

  const data = JSON.stringify(env, null, 2)
  const blob = new Blob([data], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `${env.name}-environment.json`
  a.click()
  URL.revokeObjectURL(url)
}

function handleImportEnv(event: Event) {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file) return

  const reader = new FileReader()
  reader.onload = async (e) => {
    try {
      const content = e.target?.result as string
      const importedEnv = JSON.parse(content) as Environment
      await environmentStore.updateEnvironment(importedEnv.id, importedEnv)
    } catch (error) {
      console.error('导入环境失败:', error)
      alert('导入失败：文件格式不正确')
    }
  }
  reader.readAsText(file)
  target.value = ''
}
</script>

<template>
  <div class="environment-manager">
    <div class="env-header">
      <NInput
        v-model:value="searchQuery"
        placeholder="搜索环境或变量..."
        clearable
        size="small"
        class="search-input"
      >
        <template #prefix>
          <AppIcon type="search" :size="14" />
        </template>
      </NInput>
      <NButton size="small" type="primary" @click="handleCreateEnv">
        <template #icon>
          <AppIcon type="plus" :size="14" />
        </template>
        新建环境
      </NButton>
    </div>

    <div class="env-content">
      <div class="env-list-panel">
        <div class="panel-title">环境列表</div>
        <div class="env-list">
          <div
            v-for="env in filteredEnvironments"
            :key="env.id"
            class="env-item"
            :class="{ active: environmentStore.currentEnvironmentId === env.id }"
            @click="handleSelectEnv(env)"
          >
            <div class="env-item-content">
              <AppIcon type="environment" :size="16" class="env-icon" />
              <span class="env-name">{{ env.name }}</span>
              <NTag size="small" :bordered="false"> {{ env.variables.length }} 个变量 </NTag>
            </div>
            <div class="env-actions">
              <NPopconfirm @positive-click.stop="handleDeleteEnv(env.id)">
                <template #trigger>
                  <NButton text size="tiny" type="error">
                    <template #icon>
                      <AppIcon type="trash" :size="12" />
                    </template>
                  </NButton>
                </template>
                确定要删除环境 "{{ env.name }}" 吗？
              </NPopconfirm>
            </div>
          </div>
        </div>
      </div>

      <div class="variables-panel">
        <div class="panel-header">
          <span class="panel-title">变量管理</span>
          <NFlex :size="4">
            <NButton size="small" secondary @click="handleExportEnv">
              <template #icon>
                <AppIcon type="download" :size="14" />
              </template>
              导出
            </NButton>
            <input type="file" accept=".json" style="display: none" @change="handleImportEnv" />
            <NButton
              size="small"
              secondary
              @click="$event.target?.previousElementSibling?.dispatchEvent(new MouseEvent('click'))"
            >
              <template #icon>
                <AppIcon type="upload" :size="14" />
              </template>
              导入
            </NButton>
          </NFlex>
        </div>

        <div v-if="environmentStore.currentEnvironment" class="variables-content">
          <div class="add-variable-form">
            <NInput
              v-model:value="newVarKey"
              placeholder="变量名"
              size="small"
              class="var-key-input"
            />
            <NInput
              v-model:value="newVarValue"
              placeholder="变量值"
              size="small"
              class="var-value-input"
            />
            <NButton size="small" type="primary" @click="handleAddVariable">
              <template #icon>
                <AppIcon type="plus" :size="14" />
              </template>
              添加
            </NButton>
          </div>

          <div class="variables-list">
            <div
              v-for="(variable, index) in activeVariables"
              :key="variable.key + '-' + variable.value + '-' + index"
              class="variable-item"
              :class="{ disabled: !variable.enabled }"
            >
              <div class="variable-toggle" @click="handleToggleVariable(index, variable)">
                <div class="toggle-indicator" :class="{ active: variable.enabled }"></div>
              </div>
              <div class="variable-key">{{ variable.key }}</div>
              <div class="variable-value">{{ variable.value }}</div>
              <NButton
                text
                size="tiny"
                type="error"
                class="delete-var-btn"
                @click="handleDeleteVariable(index)"
              >
                <template #icon>
                  <AppIcon type="trash" :size="12" />
                </template>
              </NButton>
            </div>
            <div v-if="activeVariables.length === 0" class="empty-state">
              <AppIcon type="environment" :size="48" />
              <p>暂无变量</p>
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

.env-header {
  padding: var(--spacing-sm) var(--spacing-md);
  border-bottom: 1px solid var(--n-border-color);
  display: flex;
  gap: var(--spacing-sm);
  align-items: center;
}

.search-input {
  flex: 1;
}

.env-content {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.env-list-panel,
.variables-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border-right: 1px solid var(--n-border-color);
}

.env-list-panel:last-child,
.variables-panel:last-child {
  border-right: none;
}

.panel-title {
  padding: var(--spacing-sm) var(--spacing-md);
  font-size: 13px;
  font-weight: 600;
  color: var(--n-text-color-1);
  border-bottom: 1px solid var(--n-border-color);
  background: var(--n-color-modal);
}

.env-list {
  flex: 1;
  overflow-y: auto;
  padding: var(--spacing-xs);
}

.env-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-sm) var(--spacing-md);
  margin-bottom: var(--spacing-xs);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.2s ease;
}

.env-item:hover {
  background: var(--n-color-hover);
}

.env-item.active {
  background: var(--color-brand-light);
  box-shadow: inset 3px 0 0 var(--color-brand);
}

.env-item-content {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  flex: 1;
  min-width: 0;
}

.env-icon {
  color: var(--color-brand);
  flex-shrink: 0;
}

.env-name {
  flex: 1;
  font-size: 13px;
  font-weight: 500;
  color: var(--n-text-color-1);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.env-actions {
  opacity: 0;
  transition: opacity 0.2s;
}

.env-item:hover .env-actions {
  opacity: 1;
}

.variables-panel {
  flex: 1.5;
}

.panel-header {
  padding: var(--spacing-sm) var(--spacing-md);
  border-bottom: 1px solid var(--n-border-color);
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--n-color-modal);
}

.variables-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.add-variable-form {
  padding: var(--spacing-sm);
  display: flex;
  gap: var(--spacing-sm);
  border-bottom: 1px solid var(--n-border-color);
}

.var-key-input,
.var-value-input {
  flex: 1;
}

.variables-list {
  flex: 1;
  overflow-y: auto;
  padding: var(--spacing-xs);
}

.variable-item {
  display: flex;
  align-items: center;
  padding: var(--spacing-sm) var(--spacing-md);
  margin-bottom: var(--spacing-xs);
  border-radius: var(--radius-md);
  background: var(--n-color-hover);
  transition: all 0.2s ease;
}

.variable-item.disabled {
  opacity: 0.5;
}

.variable-item:hover {
  background: var(--n-color-pressed);
}

.variable-toggle {
  cursor: pointer;
  padding: var(--spacing-xs);
  margin-right: var(--spacing-sm);
}

.toggle-indicator {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  border: 2px solid var(--n-text-color-3);
  transition: all 0.2s ease;
}

.toggle-indicator.active {
  background: var(--color-success);
  border-color: var(--color-success);
}

.variable-key {
  flex: 1;
  font-size: 13px;
  font-weight: 600;
  color: var(--n-text-color-1);
  font-family: monospace;
}

.variable-value {
  flex: 2;
  font-size: 12px;
  color: var(--n-text-color-2);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: monospace;
}

.delete-var-btn {
  opacity: 0;
  transition: opacity 0.2s;
}

.variable-item:hover .delete-var-btn {
  opacity: 1;
}

.empty-state,
.no-selection {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--spacing-xl);
  color: var(--n-text-color-3);
  text-align: center;
}

.empty-state p,
.no-selection p {
  margin-top: var(--spacing-md);
  font-size: 13px;
}
</style>
