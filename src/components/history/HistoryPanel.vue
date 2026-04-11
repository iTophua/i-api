<script setup lang="ts">
import { NInput, NSelect, NButton, NPopconfirm, NTooltip } from 'naive-ui'
import { ref, watch } from 'vue'
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

const emit = defineEmits<{
  select: [history: History]
}>()

function handleSelectHistory(history: History) {
  emit('select', history)
}

async function handleDeleteHistory(id: string) {
  await historyStore.deleteHistory(id)
}

async function handleClearAll() {
  await historyStore.clearHistory()
  searchInput.value = ''
}
</script>

<template>
  <div class="history-panel">
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

      <NTooltip placement="top">
        <template #trigger>
          <NPopconfirm positive-text="确定" negative-text="取消" @positive-click="handleClearAll">
            <template #trigger>
              <NButton size="small" type="error" secondary class="add-btn">
                <AppIcon type="trash" :size="14" />
              </NButton>
            </template>
            确定要清空所有历史记录吗？此操作不可恢复。
          </NPopconfirm>
        </template>
        清空历史记录
      </NTooltip>
    </div>

    <div class="filters-row">
      <div class="filter-item method-filter">
        <span class="filter-label">方法</span>
        <NSelect
          v-model:value="historyStore.filterMethod"
          :options="methodOptions"
          size="small"
          class="filter-select"
        />
      </div>
      <div class="filter-item status-filter">
        <span class="filter-label">状态</span>
        <NSelect
          v-model:value="historyStore.filterStatus"
          :options="statusOptions"
          size="small"
          class="filter-select"
        />
      </div>
    </div>

    <HistoryStatistics />
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

.search-row {
  flex-shrink: 0;
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

.filters-row {
  flex-shrink: 0;
  padding: 4px;
  border-bottom: 1px solid var(--n-border-color);
  background: var(--n-color);
  display: flex;
  align-items: center;
  gap: 4px;
}

.filter-item {
  display: flex;
  align-items: center;
  gap: 4px;
}

.filter-item.method-filter {
  flex: 1.1;
  margin-right: 6px;
}

.filter-item.status-filter {
  flex: 1;
}

.filter-label {
  font-size: var(--font-size-compact-xs);
  color: var(--n-text-color-3);
  flex-shrink: 0;
}

.filter-select {
  flex: 1;
  min-width: 0;
}

.history-count-bar {
  flex-shrink: 0;
  padding: 4px;
  border-bottom: 1px solid var(--n-border-color);
  background: var(--n-color);
}

.history-count {
  font-size: var(--font-size-compact-sm);
  color: var(--n-text-color-3);
  font-weight: 500;
}
</style>
