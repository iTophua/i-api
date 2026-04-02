<script setup lang="ts">
import { NInput, NSelect, NDatePicker, NButton, NFlex, NTag } from 'naive-ui'
import { computed } from 'vue'
import { useHistoryStore } from '@/stores'
import { AppIcon } from '@/components/icons'

const historyStore = useHistoryStore()

const methodOptions = [
  { label: '全部方法', value: 'all' },
  { label: 'GET', value: 'GET' },
  { label: 'POST', value: 'POST' },
  { label: 'PUT', value: 'PUT' },
  { label: 'DELETE', value: 'DELETE' },
  { label: 'PATCH', value: 'PATCH' },
  { label: 'OPTIONS', value: 'OPTIONS' },
  { label: 'HEAD', value: 'HEAD' },
]

const statusOptions = [
  { label: '全部状态', value: 'all' },
  { label: '2xx 成功', value: '2xx' },
  { label: '3xx 重定向', value: '3xx' },
  { label: '4xx 客户端错误', value: '4xx' },
  { label: '5xx 服务器错误', value: '5xx' },
]

const hasActiveFilters = computed(() => {
  return (
    historyStore.searchQuery !== '' ||
    historyStore.filterMethod !== 'all' ||
    historyStore.filterStatus !== 'all' ||
    historyStore.dateRange.start ||
    historyStore.dateRange.end
  )
})

function handleClearFilters() {
  historyStore.clearFiltered()
}
</script>

<template>
  <div class="history-filters">
    <div class="filter-row">
      <NInput
        v-model:value="historyStore.searchQuery"
        placeholder="搜索 URL 或方法..."
        clearable
        size="small"
        class="search-input"
      >
        <template #prefix>
          <AppIcon type="search" :size="14" />
        </template>
      </NInput>

      <NSelect
        v-model:value="historyStore.filterMethod"
        :options="methodOptions"
        placeholder="请求方法"
        size="small"
        class="filter-select"
      />

      <NSelect
        v-model:value="historyStore.filterStatus"
        :options="statusOptions"
        placeholder="响应状态"
        size="small"
        class="filter-select"
      />
    </div>

    <div class="filter-row date-range-row">
      <NDatePicker
        v-model:value="historyStore.dateRange.start"
        type="date"
        placeholder="开始日期"
        size="small"
        class="date-picker"
        clearable
      />
      <span class="date-separator">至</span>
      <NDatePicker
        v-model:value="historyStore.dateRange.end"
        type="date"
        placeholder="结束日期"
        size="small"
        class="date-picker"
        clearable
      />

      <NFlex style="margin-left: auto">
        <NTag v-if="hasActiveFilters" type="info" size="small">
          筛选中
        </NTag>
        <NButton
          v-if="hasActiveFilters"
          size="small"
          secondary
          @click="handleClearFilters"
        >
          清除筛选
        </NButton>
      </NFlex>
    </div>
  </div>
</template>

<style scoped>
.history-filters {
  padding: var(--spacing-sm) var(--spacing-md);
  border-bottom: 1px solid var(--n-border-color);
  background: var(--n-color);
}

.filter-row {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  margin-bottom: var(--spacing-sm);
}

.filter-row:last-child {
  margin-bottom: 0;
}

.search-input {
  flex: 1;
  min-width: 200px;
}

.filter-select {
  width: 140px;
}

.date-range-row {
  flex-wrap: wrap;
}

.date-picker {
  width: 140px;
}

.date-separator {
  color: var(--n-text-color-3);
  font-size: 12px;
  padding: 0 var(--spacing-xs);
}
</style>
