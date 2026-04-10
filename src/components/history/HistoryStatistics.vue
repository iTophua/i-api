<script setup lang="ts">
import { NTag } from 'naive-ui'
import { computed } from 'vue'
import { useHistoryStore } from '@/stores'

const historyStore = useHistoryStore()

const filteredStats = computed(() => {
  const filtered = historyStore.filteredHistories
  const total = filtered.length
  const successCount = filtered.filter((h) => h.status >= 200 && h.status < 300).length
  const errorCount = filtered.filter((h) => h.status >= 400).length
  const avgResponseTime =
    total > 0
      ? Math.round(filtered.reduce((sum, h) => sum + h.responseTime, 0) / total)
      : 0

  return { total, successCount, errorCount, avgResponseTime }
})

const successRate = computed(() => {
  if (filteredStats.value.total === 0) return 0
  return Math.round((filteredStats.value.successCount / filteredStats.value.total) * 100)
})
</script>

<template>
  <div class="history-statistics">
    <div class="stats-row">
      <div class="stat-item">
        <span class="stat-value">{{ filteredStats.avgResponseTime }}<small>ms</small></span>
        <span class="stat-label">平均耗时</span>
      </div>
      <NTag type="info" size="small" class="success-rate">{{ successRate }}% 成功率</NTag>
    </div>
    <div class="stats-row second-row">
      <div class="stat-item">
        <span class="stat-value">{{ filteredStats.total }}</span>
        <span class="stat-label">筛选结果</span>
      </div>

      <div class="stat-item success">
        <span class="stat-value">{{ filteredStats.successCount }}</span>
        <span class="stat-label">成功</span>
      </div>

      <div class="stat-item error">
        <span class="stat-value">{{ filteredStats.errorCount }}</span>
        <span class="stat-label">失败</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.history-statistics {
  padding: 4px;
  background: var(--n-color);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.stats-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.second-row {
  margin-top: 4px;
}

.stat-item {
  display: flex;
  align-items: baseline;
  gap: 4px;
}

.stat-item.success .stat-value {
  color: var(--color-success);
}

.stat-item.error .stat-value {
  color: var(--color-error);
}

.stat-value {
  font-size: 14px;
  font-weight: 600;
  color: var(--n-text-color-1);
}

.stat-value small {
  font-size: 10px;
  font-weight: 400;
  margin-left: 1px;
}

.stat-label {
  font-size: 11px;
  color: var(--n-text-color-3);
}

.success-rate {
  margin-left: auto;
}
</style>
