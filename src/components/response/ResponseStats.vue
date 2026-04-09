<script setup lang="ts">
import { NTag, NProgress, NFlex } from 'naive-ui'
import { computed } from 'vue'
import { useRequestStore } from '@/stores'

const requestStore = useRequestStore()

const response = computed(() => requestStore.currentResponse)

const statusType = computed(() => {
  if (!response.value) return 'default'
  const status = response.value.status
  if (status >= 200 && status < 300) return 'success'
  if (status >= 300 && status < 400) return 'info'
  if (status >= 400 && status < 500) return 'warning'
  return 'error'
})

const performanceScore = computed(() => {
  if (!response.value) return 0
  const time = response.value.responseTime
  if (time < 100) return 100
  if (time < 300) return 90
  if (time < 500) return 70
  if (time < 1000) return 50
  if (time < 2000) return 30
  return 10
})

const performanceLevel = computed(() => {
  const score = performanceScore.value
  if (score >= 90) return { text: '优秀', color: '#18A058' }
  if (score >= 70) return { text: '良好', color: '#F0A020' }
  if (score >= 50) return { text: '一般', color: '#FF6B00' }
  return { text: '较慢', color: '#D03050' }
})

const sizeFormatted = computed(() => {
  if (!response.value) return '0 B'
  const size = response.value.responseSize
  if (size < 1024) return `${size} B`
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`
  return `${(size / 1024 / 1024).toFixed(2)} MB`
})
</script>

<template>
  <div v-if="response" class="response-stats">
    <NFlex justify="space-between" align="center" class="stats-row">
      <div class="stat-group">
        <NTag :type="statusType" size="medium" bordered>
          {{ response.status }} {{ response.statusText }}
        </NTag>
      </div>

      <div class="stat-group">
        <span class="stat-label">性能</span>
        <div class="performance-indicator">
          <span class="performance-level" :style="{ color: performanceLevel.color }">
            {{ performanceLevel.text }}
          </span>
          <NProgress
            :percentage="performanceScore"
            :color="performanceLevel.color"
            :show-indicator="false"
            height="6px"
            style="width: 80px"
          />
        </div>
      </div>

      <div class="stat-group">
        <span class="stat-label">响应时间</span>
        <span class="stat-value">{{ response.responseTime }} ms</span>
      </div>

      <div class="stat-group">
        <span class="stat-label">响应大小</span>
        <span class="stat-value">{{ sizeFormatted }}</span>
      </div>
    </NFlex>
  </div>
</template>

<style scoped>
.response-stats {
  padding: 8px 12px;
  border-bottom: 1px solid var(--n-border-color);
  background: var(--n-color-modal);
}

.stats-row {
  flex-wrap: wrap;
  gap: 12px;
}

.stat-group {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 80px;
}

.stat-label {
  font-size: 10px;
  color: var(--n-text-color-3);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  font-weight: 500;
}

.stat-value {
  font-size: 13px;
  font-weight: 600;
  color: var(--n-text-color-1);
}

.performance-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
}

.performance-level {
  font-size: 11px;
  font-weight: 600;
  min-width: 36px;
}
</style>
