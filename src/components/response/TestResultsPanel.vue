<script setup lang="ts">
import { NTag, NProgress, NFlex, NCollapse, NCollapseItem, NScrollbar } from 'naive-ui'
import { computed } from 'vue'
import { useRequestStore } from '@/stores'

interface TestResult {
  id: string
  name: string
  status: 'pass' | 'fail' | 'error'
  message?: string
  duration?: number
}

interface TestSuite {
  name: string
  total: number
  passed: number
  failed: number
  errors: number
  duration: number
  results: TestResult[]
}

const requestStore = useRequestStore()

const testResults = computed<TestSuite | null>(() => {
  const currentTab = requestStore.currentTab
  if (!currentTab?.testResults) return null

  const results = currentTab.testResults as TestResult[]
  const passed = results.filter((r) => r.status === 'pass').length
  const failed = results.filter((r) => r.status === 'fail').length
  const errors = results.filter((r) => r.status === 'error').length
  const total = results.length
  const duration = results.reduce((sum, r) => sum + (r.duration || 0), 0)

  return {
    name: 'API Tests',
    total,
    passed,
    failed,
    errors,
    duration,
    results,
  }
})

const passRate = computed(() => {
  if (!testResults.value || testResults.value.total === 0) return 0
  return Math.round((testResults.value.passed / testResults.value.total) * 100)
})

const statusColor = (status: string) => {
  switch (status) {
    case 'pass':
      return 'success'
    case 'fail':
      return 'error'
    case 'error':
      return 'warning'
    default:
      return 'default'
  }
}

const statusIcon = (status: string) => {
  switch (status) {
    case 'pass':
      return '✓'
    case 'fail':
      return '✗'
    case 'error':
      return '!'
    default:
      return '?'
  }
}
</script>

<template>
  <div class="test-results-panel">
    <template v-if="testResults">
      <!-- 统计概览 -->
      <div class="test-overview">
        <NFlex justify="space-between" align="center" class="overview-header">
          <span class="overview-title">测试结果</span>
          <NTag :type="passRate === 100 ? 'success' : passRate > 50 ? 'warning' : 'error'" size="medium">
            通过率 {{ passRate }}%
          </NTag>
        </NFlex>

        <div class="overview-stats">
          <div class="stat-item">
            <div class="stat-value">{{ testResults.total }}</div>
            <div class="stat-label">总计</div>
          </div>
          <div class="stat-item success">
            <div class="stat-value">{{ testResults.passed }}</div>
            <div class="stat-label">通过</div>
          </div>
          <div class="stat-item error">
            <div class="stat-value">{{ testResults.failed }}</div>
            <div class="stat-label">失败</div>
          </div>
          <div class="stat-item warning">
            <div class="stat-value">{{ testResults.errors }}</div>
            <div class="stat-label">错误</div>
          </div>
          <div class="stat-item">
            <div class="stat-value">{{ testResults.duration }}ms</div>
            <div class="stat-label">耗时</div>
          </div>
        </div>

        <NProgress
          :percentage="passRate"
          :color="passRate === 100 ? '#18A058' : passRate > 50 ? '#F0A020' : '#D03050'"
          :show-indicator="false"
          height="8px"
        />
      </div>

      <!-- 详细结果 -->
      <div class="test-details">
        <NCollapse arrow-placement="right">
          <NCollapseItem title="测试详情" name="details">
            <NScrollbar style="max-height: 400px">
              <div class="test-list">
                <div
                  v-for="(result, index) in testResults.results"
                  :key="result.id"
                  class="test-item"
                  :class="result.status"
                >
                  <div class="test-status">
                    <span class="status-icon" :class="result.status">
                      {{ statusIcon(result.status) }}
                    </span>
                  </div>
                  <div class="test-content">
                    <div class="test-name">
                      <span class="test-index">{{ index + 1 }}</span>
                      {{ result.name }}
                    </div>
                    <div v-if="result.message" class="test-message">
                      {{ result.message }}
                    </div>
                  </div>
                  <div class="test-meta">
                    <NTag :type="statusColor(result.status)" size="small" bordered>
                      {{ result.status.toUpperCase() }}
                    </NTag>
                    <span v-if="result.duration" class="test-duration">
                      {{ result.duration }}ms
                    </span>
                  </div>
                </div>
              </div>
            </NScrollbar>
          </NCollapseItem>
        </NCollapse>
      </div>
    </template>

    <div v-else class="no-tests">
      <div class="no-tests-icon">
        <svg viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
          <rect x="12" y="8" width="40" height="48" rx="4" stroke="currentColor" stroke-width="2" />
          <path d="M20 24L28 32L44 16" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" />
          <circle cx="32" cy="44" r="4" fill="currentColor" />
        </svg>
      </div>
      <p class="no-tests-text">暂无测试结果</p>
      <p class="no-tests-hint">在请求的 Scripts 标签页添加测试脚本</p>
    </div>
  </div>
</template>

<style scoped>
.test-results-panel {
  padding: var(--spacing-md);
  background: var(--n-color);
  height: 100%;
  overflow-y: auto;
}

.test-overview {
  margin-bottom: var(--spacing-md);
}

.overview-header {
  margin-bottom: var(--spacing-sm);
}

.overview-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--n-text-color-1);
}

.overview-stats {
  display: flex;
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-sm);
  flex-wrap: wrap;
}

.stat-item {
  text-align: center;
  padding: var(--spacing-sm) var(--spacing-md);
  border-radius: var(--radius-md);
  background: var(--n-color-hover);
  min-width: 60px;
}

.stat-item.success {
  background: rgba(24, 160, 88, 0.1);
}

.stat-item.error {
  background: rgba(214, 48, 49, 0.1);
}

.stat-item.warning {
  background: rgba(240, 160, 32, 0.1);
}

.stat-value {
  font-size: 20px;
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

.stat-item.warning .stat-value {
  color: var(--color-warning);
}

.stat-label {
  font-size: 11px;
  color: var(--n-text-color-3);
  margin-top: 2px;
  font-weight: 500;
  text-transform: uppercase;
}

.test-details {
  margin-top: var(--spacing-md);
}

.test-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.test-item {
  display: flex;
  align-items: flex-start;
  gap: var(--spacing-sm);
  padding: var(--spacing-sm) var(--spacing-md);
  border-radius: var(--radius-md);
  border-left: 3px solid transparent;
  transition: all 0.2s ease;
}

.test-item:hover {
  background: var(--n-color-hover);
}

.test-item.pass {
  border-left-color: var(--color-success);
}

.test-item.fail {
  border-left-color: var(--color-error);
}

.test-item.error {
  border-left-color: var(--color-warning);
}

.test-status {
  flex-shrink: 0;
  padding-top: 2px;
}

.status-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  font-size: 12px;
  font-weight: 700;
}

.status-icon.pass {
  background: rgba(24, 160, 88, 0.1);
  color: var(--color-success);
}

.status-icon.fail {
  background: rgba(214, 48, 49, 0.1);
  color: var(--color-error);
}

.status-icon.error {
  background: rgba(240, 160, 32, 0.1);
  color: var(--color-warning);
}

.test-content {
  flex: 1;
  min-width: 0;
}

.test-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--n-text-color-1);
  margin-bottom: 4px;
  display: flex;
  align-items: center;
  gap: 6px;
}

.test-index {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 18px;
  height: 18px;
  padding: 0 4px;
  font-size: 10px;
  font-weight: 600;
  color: var(--n-text-color-3);
  background: var(--n-color-pressed);
  border-radius: 3px;
}

.test-message {
  font-size: 12px;
  color: var(--n-text-color-3);
  line-height: 1.4;
  word-break: break-word;
}

.test-meta {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 4px;
}

.test-duration {
  font-size: 11px;
  color: var(--n-text-color-3);
  font-weight: 500;
}

.no-tests {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: var(--spacing-xl);
  text-align: center;
}

.no-tests-icon {
  width: 80px;
  height: 80px;
  margin-bottom: var(--spacing-md);
  opacity: 0.4;
  color: var(--n-text-color-3);
}

.no-tests-icon svg {
  width: 100%;
  height: 100%;
}

.no-tests-text {
  font-size: 15px;
  font-weight: 600;
  color: var(--n-text-color-2);
  margin-bottom: 8px;
}

.no-tests-hint {
  font-size: 13px;
  color: var(--n-text-color-3);
  line-height: 1.5;
}
</style>
