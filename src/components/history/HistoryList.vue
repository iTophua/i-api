<script setup lang="ts">
import { NList, NListItem, NButton, NTag, NFlex, NEmpty, NTooltip } from 'naive-ui'
import { computed } from 'vue'
import { useHistoryStore } from '@/stores'
import { AppIcon } from '@/components/icons'
import type { History } from '@/types'
import { safeParseDate, HTTP_METHOD_COLORS } from '@/types'

// 使用统一的方法颜色定义（适配 HistoryList 的 bg 字段）
const methodColors: Record<string, { color: string; bg: string }> = {
  GET: { color: HTTP_METHOD_COLORS.GET.color, bg: HTTP_METHOD_COLORS.GET.background },
  POST: { color: HTTP_METHOD_COLORS.POST.color, bg: HTTP_METHOD_COLORS.POST.background },
  PUT: { color: HTTP_METHOD_COLORS.PUT.color, bg: HTTP_METHOD_COLORS.PUT.background },
  DELETE: { color: HTTP_METHOD_COLORS.DELETE.color, bg: HTTP_METHOD_COLORS.DELETE.background },
  PATCH: { color: HTTP_METHOD_COLORS.PATCH.color, bg: HTTP_METHOD_COLORS.PATCH.background },
  OPTIONS: { color: HTTP_METHOD_COLORS.OPTIONS.color, bg: HTTP_METHOD_COLORS.OPTIONS.background },
  HEAD: { color: HTTP_METHOD_COLORS.HEAD.color, bg: HTTP_METHOD_COLORS.HEAD.background },
}

interface Props {
  histories?: History[]
}

const props = withDefaults(defineProps<Props>(), {
  histories: () => [],
})

const emit = defineEmits<{
  select: [history: History]
  delete: [id: string]
}>()

const historyStore = useHistoryStore()

const displayHistories = computed(() => {
  return props.histories.length > 0 ? props.histories : historyStore.filteredHistories
})

function getStatusType(status: number): 'success' | 'warning' | 'error' | 'info' {
  if (status >= 200 && status < 300) return 'success'
  if (status >= 300 && status < 400) return 'warning'
  if (status >= 400 && status < 500) return 'error'
  if (status >= 500) return 'error'
  return 'info'
}

function formatTime(dateString: string): string {
  const safeDateStr = safeParseDate(dateString)
  const date = new Date(safeDateStr)
  const timestamp = date.getTime()
  
  // 检查日期是否有效
  if (isNaN(timestamp)) {
    return '未知时间'
  }
  
  const now = new Date()
  const diff = now.getTime() - timestamp
  
  const minute = 60 * 1000
  const hour = 60 * minute
  const day = 24 * hour
  
  if (diff < minute) return '刚刚'
  if (diff < hour) return `${Math.floor(diff / minute)}分钟前`
  if (diff < day) return `${Math.floor(diff / hour)}小时前`
  if (diff < 7 * day) return `${Math.floor(diff / day)}天前`
  
  return date.toLocaleDateString('zh-CN', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

function handleSelect(history: History) {
  emit('select', history)
}

function handleDelete(id: string, event: Event) {
  event.stopPropagation()
  emit('delete', id)
}
</script>

<template>
  <div class="history-list">
    <NList hoverable clickable class="history-list-component">
      <NListItem
        v-for="history in displayHistories"
        :key="history.id"
        class="history-item"
        @click="handleSelect(history)"
      >
        <div class="item-content">
          <div class="item-left">
            <div class="item-main">
              <div class="item-url">{{ history.url }}</div>
              <div class="item-meta">
                <span
                  class="method-tag"
                  :style="{
                    color: methodColors[history.method]?.color || methodColors.GET.color,
                    background: methodColors[history.method]?.bg || methodColors.GET.bg,
                  }"
                >
                  {{ history.method }}
                </span>
                <NTag :type="getStatusType(history.status)" size="small" bordered>
                  {{ history.status }}
                </NTag>
                <span class="item-time">{{ formatTime(history.createdAt) }}</span>
                <span class="item-size">{{ history.responseSize }} B</span>
              </div>
            </div>
          </div>
          
          <div class="item-right">
            <NFlex :size="4">
              <NTooltip placement="top">
                <template #trigger>
                  <NButton text size="small" @click.stop="handleDelete(history.id, $event)">
                    <AppIcon type="trash" :size="14" />
                  </NButton>
                </template>
                删除记录
              </NTooltip>
            </NFlex>
          </div>
        </div>
      </NListItem>
    </NList>

    <NEmpty
      v-if="displayHistories.length === 0"
      description="暂无历史记录"
      class="empty-state"
    />
  </div>
</template>

<style scoped>
.history-list {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.history-list-component {
  background: transparent;
  flex: 1;
  min-height: 0;
}

.history-item {
  transition: all 0.2s ease;
  border-bottom: 1px solid var(--n-border-color);
}


.history-item:hover {
  background: var(--n-color-hover);
  transform: translateX(2px);
}

.item-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.item-left {
  display: flex;
  align-items: center;
  flex: 1;
  min-width: 0;
}

.item-main {
  flex: 1;
  min-width: 0;
}

.item-url {
  font-size: 12px;
  color: var(--n-text-color-1);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-bottom: 2px;
  font-weight: 500;
}

.item-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 9px;
  margin-top: 1px;
}

.method-tag {
  font-size: 9px;
  font-weight: 700;
  padding: 0 3px;
  border-radius: 2px;
  letter-spacing: 0.5px;
}

.item-time,
.item-size {
  color: var(--n-text-color-3);
  font-weight: 500;
}

.item-right {
  opacity: 0;
  transition: opacity 0.2s ease;
  flex-shrink: 0;
}

.history-item:hover .item-right {
  opacity: 1;
}

.empty-state {
  margin: auto;
  padding: var(--spacing-xl);
}
</style>
