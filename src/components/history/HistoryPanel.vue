<script setup lang="ts">
import { NInput, NSelect, NButton } from 'naive-ui'
import { ref, watch, computed } from 'vue'
import { useHistoryStore } from '@/stores'
import HistoryStatistics from './HistoryStatistics.vue'
import HistoryList from './HistoryList.vue'
import { AppIcon } from '@/components/icons'
import type { History } from '@/types'

const historyStore = useHistoryStore()
const searchInput = ref(historyStore.searchQuery)

// 防抖搜索
let searchTimeout: ReturnType<typeof setTimeout> | null = null
watch(searchInput, (val) => {
  if (searchTimeout) clearTimeout(searchTimeout)
  searchTimeout = setTimeout(() => {
    historyStore.searchQuery = val
  }, 200)
})

const methodOptions = [
  { label: '全部', value: 'all' },
  { label: 'GET', value: 'GET' },
  { label: 'POST', value: 'POST' },
  { label: 'PUT', value: 'PUT' },
  { label: 'DELETE', value: 'DELETE' },
  { label: 'PATCH', value: 'PATCH' },
  { label: 'OPTIONS', value: 'OPTIONS' },
  { label: 'HEAD', value: 'HEAD' },
]

const statusOptions = [
  { label: '全部', value: 'all' },
  { label: '2xx', value: '2xx' },
  { label: '3xx', value: '3xx' },
  { label: '4xx', value: '4xx' },
  { label: '5xx', value: '5xx' },
]

const hasActiveFilters = computed(() => {
  return (
    historyStore.searchQuery !== '' ||
    historyStore.filterMethod !== 'all' ||
    historyStore.filterStatus !== 'all'
  )
})

const emit = defineEmits<{
  select: [history: History]
}>()

function handleSelectHistory(history: History) {
  emit('select', history)
}

function handleDeleteHistory(id: string) {
  historyStore.deleteHistory(id)
}

function handleClearFilters() {
  historyStore.clearFiltered()
  searchInput.value = ''
}

function handleClearAll() {
  historyStore.clearHistory()
  searchInput.value = ''
}
</script>

<template>
  <div class="history-panel">
    <HistoryStatistics />

    <!-- 第一行：两个下拉框 -->
    <div class="filters-row">
      <div class="filter-item method-filter">
        <NSelect
          v-model:value="historyStore.filterMethod"
          :options="methodOptions"
          size="small"
          class="filter-select"
        />
      </div>
      <div class="filter-item status-filter">
        <NSelect
          v-model:value="historyStore.filterStatus"
          :options="statusOptions"
          size="small"
          class="filter-select"
        />
      </div>
    </div>

    <!-- 第二行：搜索框 + 清空按钮 -->
    <div class="search-row">
      <NInput
        v-model:value="searchInput"
        placeholder="搜索 URL..."
        clearable
        size="small"
        class="search-input"
      >
        <template #prefix>
          <AppIcon type="search" :size="14" />
        </template>
      </NInput>

      <NButton size="small" type="error" secondary @click="handleClearAll">
        <AppIcon type="trash" :size="14" />
      </NButton>
    </div>

    <div class="history-count-bar">
      <span class="history-count">
        共 {{ historyStore.filteredHistories.length }} / {{ historyStore.histories.length }} 条记录
      </span>
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

/* 第一行：两个下拉框 */
.filters-row {
  flex-shrink: 0;
  padding: var(--spacing-xs) var(--spacing-md);
  border-bottom: 1px solid var(--n-border-color);
  background: var(--n-color);
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.filter-item {
  display: flex;
  align-items: center;
  gap: 4px;
}

.filter-item.method-filter {
  flex: 1.2;
}

.filter-item.status-filter {
  flex: 0.7;
}

.filter-label {
  font-size: 12px;
  color: var(--n-text-color-3);
  white-space: nowrap;
}

.filter-select {
  flex: 1;
  min-width: 0;
}



/* 第二行：搜索 + 按钮 */
.search-row {
  flex-shrink: 0;
  padding: var(--spacing-xs) var(--spacing-md);
  border-bottom: 1px solid var(--n-border-color);
  background: var(--n-color);
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.search-input {
  flex: 1;
  min-width: 0;
}

.history-count-bar {
  flex-shrink: 0;
  padding: var(--spacing-xs) var(--spacing-md);
  border-bottom: 1px solid var(--n-border-color);
  background: var(--n-color);
}

.history-count {
  font-size: 11px;
  color: var(--n-text-color-3);
  font-weight: 500;
}
</style>
