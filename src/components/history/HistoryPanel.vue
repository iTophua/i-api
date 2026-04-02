<script setup lang="ts">
import { NButton, NFlex, NPopconfirm } from 'naive-ui'
import { useHistoryStore } from '@/stores'
import HistoryFilters from './HistoryFilters.vue'
import HistoryStatistics from './HistoryStatistics.vue'
import HistoryList from './HistoryList.vue'
import type { History } from '@/types'

const historyStore = useHistoryStore()

const emit = defineEmits<{
  select: [history: History]
}>()

function handleSelectHistory(history: History) {
  emit('select', history)
}

function handleDeleteHistory(id: string) {
  historyStore.deleteHistory(id)
}

function handleClearAll() {
  historyStore.clearHistory()
}

function handleExport() {
  const data = historyStore.exportHistory()
  const blob = new Blob([data], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `history-${new Date().toISOString().split('T')[0]}.json`
  a.click()
  URL.revokeObjectURL(url)
}

function handleImport(event: Event) {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file) return

  const reader = new FileReader()
  reader.onload = (e) => {
    const content = e.target?.result as string
    const success = historyStore.importHistory(content)
    if (!success) {
      console.error('导入失败：文件格式不正确')
    }
  }
  reader.readAsText(file)
  target.value = ''
}
</script>

<template>
  <div class="history-panel">
    <HistoryStatistics />
    <HistoryFilters />
    
    <div class="history-actions">
      <NFlex justify="space-between" align="center">
        <span class="history-count">
          共 {{ historyStore.filteredHistories.length }} / {{ historyStore.histories.length }} 条记录
        </span>
        
        <NFlex :size="4">
          <input
            type="file"
            accept=".json"
            style="display: none"
            @change="handleImport"
          >
          <NButton size="small" secondary @click="$event.target?.previousElementSibling?.dispatchEvent(new MouseEvent('click'))">
            导入
          </NButton>
          <NButton size="small" secondary @click="handleExport">
            导出
          </NButton>
          <NPopconfirm @positive-click="handleClearAll">
            <template #trigger>
              <NButton size="small" type="error" secondary>
                清空全部
              </NButton>
            </template>
            确定要清空所有历史记录吗？此操作不可恢复。
          </NPopconfirm>
        </NFlex>
      </NFlex>
    </div>

    <HistoryList
      @select="handleSelectHistory"
      @delete="handleDeleteHistory"
    />
  </div>
</template>

<style scoped>
.history-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--n-color);
  overflow: hidden;
}

.history-statistics,
.history-filters,
.history-actions {
  flex-shrink: 0;
}

.history-actions {
  padding: var(--spacing-sm) var(--spacing-md);
  border-bottom: 1px solid var(--n-border-color);
}

.history-count {
  font-size: 12px;
  color: var(--n-text-color-3);
  font-weight: 500;
}
</style>
