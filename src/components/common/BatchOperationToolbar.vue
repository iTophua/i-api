<script setup lang="ts">
import { NButton, NPopconfirm } from 'naive-ui'
import { computed } from 'vue'
import { useRequestStore } from '@/stores'
import { AppIcon } from '@/components/icons'
import type { Collection } from '@/types'

const emit = defineEmits<{
  copy: []
  move: []
}>()

interface Props {
  collection?: Collection | null
}

const props = defineProps<Props>()

const requestStore = useRequestStore()

const selectedCount = computed(() => requestStore.selectedRequestIds.size)

const showActions = computed(() => selectedCount.value > 0)

async function handleDelete() {
  if (!props.collection || selectedCount.value === 0) return
  
  const requestIds = Array.from(requestStore.selectedRequestIds)
  await requestStore.batchDeleteRequests(props.collection.id, requestIds)
}

function handleCopy() {
  emit('copy')
}

function handleMove() {
  emit('move')
}
</script>

<template>
  <div v-if="showActions" class="batch-operation-toolbar">
    <div class="selection-info">
      <AppIcon type="check" :size="16" />
      <span>已选择 {{ selectedCount }} 个请求</span>
    </div>
    
    <div class="action-buttons">
      <NButton size="small" @click="requestStore.clearSelection()">
        取消选择
      </NButton>
      
      <NButton size="small" type="primary" @click="handleCopy">
        <template #icon>
          <AppIcon type="copy" :size="14" />
        </template>
        批量复制
      </NButton>
      
      <NButton size="small" type="info" @click="handleMove">
        <template #icon>
          <AppIcon type="upload" :size="14" />
        </template>
        批量移动
      </NButton>
      
      <NPopconfirm @positive-click="handleDelete">
        <template #trigger>
          <NButton size="small" type="error">
            <template #icon>
              <AppIcon type="trash" :size="14" />
            </template>
            批量删除
          </NButton>
        </template>
        确定要删除这 {{ selectedCount }} 个请求吗？此操作不可恢复。
      </NPopconfirm>
    </div>
  </div>
</template>

<style scoped>
.batch-operation-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-sm) var(--spacing-md);
  background-color: var(--color-brand-light);
  border-bottom: 1px solid var(--color-border);
  gap: var(--spacing-md);
  animation: slide-down 0.2s ease-out;
}

.selection-info {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  font-weight: 500;
  color: var(--color-text-primary);
}

.action-buttons {
  display: flex;
  gap: var(--spacing-sm);
}
</style>
