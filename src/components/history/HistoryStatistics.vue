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
    <NFlex justify="space-between" align="center" class="stats-header">
      <span class="stats-title">历史记录统计</span>
      <NTag type="info" size="small">成功率 {{ successRate }}%</NTag>
    </NFlex>

    <NFlex class="stats-content">
      <div class="stat-item">
        <div class="stat-value">{{ stats.total }}</div>
        <div class="stat-label">总请求数</div>
      </div>

      <div class="stat-item success">
        <div class="stat-value">{{ stats.successCount }}</div>
        <div class="stat-label">成功</div>
      </div>

      <div class="stat-item error">
        <div class="stat-value">{{ stats.errorCount }}</div>
        <div class="stat-label">失败</div>
      </div>

      <div class="stat-item">
        <div class="stat-value">{{ stats.avgResponseTime }}ms</div>
        <div class="stat-label">平均响应时间</div>
      </div>
    </NFlex>
  </div>
</template>

<style scoped>
.history-statistics {
  padding: var(--spacing-md);
  border-bottom: 1px solid var(--n-border-color);
  background: linear-gradient(135deg, var(--color-brand-light) 0%, var(--n-color) 100%);
}

.stats-header {
  margin-bottom: var(--spacing-sm);
}

.stats-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--n-text-color-1);
}

.stats-content {
  flex-wrap: wrap;
}

.stat-item {
  text-align: center;
  padding: var(--spacing-sm) var(--spacing-md);
  border-radius: var(--radius-md);
  background: var(--n-color);
  min-width: 100px;
  transition: all 0.2s ease;
}

.stat-item:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-sm);
}

.stat-item.success {
  background: rgba(24, 160, 88, 0.1);
}

.stat-item.error {
  background: rgba(214, 48, 49, 0.1);
}

.stat-value {
  font-size: 24px;
  font-weight: 700;
  color: var(--n-text-color-1);
  line-height: 1.2;
}

.stat-item.success .stat-value {
  color: var(--color-success);
}

.stat-item.error .stat-value {
  color: var(--color-error);
}

.stat-label {
  font-size: 12px;
  color: var(--n-text-color-3);
  margin-top: 4px;
  font-weight: 500;
}
</style>
