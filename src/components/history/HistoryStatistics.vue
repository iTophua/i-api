<script setup lang="ts">
import { NFlex, NTag } from 'naive-ui'
import { computed } from 'vue'
import { useHistoryStore } from '@/stores'

const historyStore = useHistoryStore()

const stats = computed(() => historyStore.statistics)

const successRate = computed(() => {
  if (stats.value.total === 0) return 0
  return Math.round((stats.value.successCount / stats.value.total) * 100)
})
</script>

<template>
  <div class="history-statistics">
    <NFlex justify="space-between" align="center">
      <div class="stat-item">
        <span class="stat-value">{{ stats.total }}</span>
        <span class="stat-label">总请求</span>
      </div>

      <div class="stat-item success">
        <span class="stat-value">{{ stats.successCount }}</span>
        <span class="stat-label">成功</span>
      </div>

      <div class="stat-item error">
        <span class="stat-value">{{ stats.errorCount }}</span>
        <span class="stat-label">失败</span>
      </div>

      <div class="stat-item">
        <span class="stat-value">{{ stats.avgResponseTime }}<small>ms</small></span>
        <span class="stat-label">平均耗时</span>
      </div>

      <NTag type="info" size="small" class="success-rate">{{ successRate }}%</NTag>
    </NFlex>
  </div>
</template>

<style scoped>
.history-statistics {
  padding: var(--spacing-sm) var(--spacing-md);
  border-bottom: 1px solid var(--n-border-color);
  background: var(--n-color);
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
