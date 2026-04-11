<script setup lang="ts">
import { NModal, NButton, NTree, NFlex } from 'naive-ui'
import { ref, computed, watch } from 'vue'
import { useRequestStore } from '@/stores'

interface Props {
  show: boolean
  collectionId?: string
  requestIds?: string[]
  mode?: 'copy' | 'move'
}

const props = withDefaults(defineProps<Props>(), {
  collectionId: '',
  requestIds: () => [],
  mode: 'move',
})

const emit = defineEmits<{
  'update:show': [value: boolean]
}>()

const requestStore = useRequestStore()

const selectedTarget = ref<{ collectionId: string; folderId?: string } | null>(null)

const treeData = computed(() => {
  return requestStore.collections.map((collection) => ({
    key: collection.id,
    label: collection.name,
    isLeaf: false,
    children: [
      {
        key: `${collection.id}::all`,
        label: '所有请求',
        isLeaf: true,
      },
      ...collection.folders.map((folder) => ({
        key: `${collection.id}::${folder.id}`,
        label: folder.name,
        isLeaf: true,
      })),
    ],
  }))
})

function handleCheck(keys: Array<string | number>) {
  if (keys.length === 0) {
    selectedTarget.value = null
    return
  }

  const key = keys[0] as string
  const parts = key.split('::')
  const collectionId = parts[0]
  const folderId = parts[1]

  selectedTarget.value = {
    collectionId,
    folderId: folderId && folderId !== 'all' ? folderId : undefined,
  }
}

async function handleConfirm() {
  if (!selectedTarget.value || !props.collectionId) return

  try {
    if (props.mode === 'copy') {
      await requestStore.batchCopyRequests(
        selectedTarget.value.collectionId,
        selectedTarget.value.folderId ?? null,
        props.requestIds
      )
    } else {
      await requestStore.batchMoveRequests(
        props.collectionId,
        selectedTarget.value.collectionId,
        selectedTarget.value.folderId ?? null,
        props.requestIds
      )
    }
    emit('update:show', false)
  } catch (error) {
    console.error('批量操作失败:', error)
  }
}

function handleClose() {
  selectedTarget.value = null
  emit('update:show', false)
}

const title = computed(() => {
  return props.mode === 'copy' ? '批量复制请求' : '批量移动请求'
})

// 重置选择状态当对话框打开时
watch(
  () => props.show,
  (newVal) => {
    if (newVal) {
      selectedTarget.value = null
    }
  }
)
</script>

<template>
  <NModal :show="show" preset="card" :title="title" style="width: 400px" @close="handleClose">
    <div class="batch-move-dialog">
      <p class="dialog-description">
        选择要将这 {{ requestIds.length }} 个请求{{ mode === 'copy' ? '复制到' : '移动到' }}的位置：
      </p>

      <NTree
        :data="treeData"
        :checked-keys="selectedTarget ? [selectedTarget.collectionId + '::' + (selectedTarget.folderId || 'all')] : []"
        :checkable="true"
        :selectable="false"
        :default-expand-all="true"
        block-line
        @update:checked-keys="handleCheck"
      />

      <NFlex justify="end" style="margin-top: 16px">
        <NButton @click="handleClose">取消</NButton>
        <NButton
          type="primary"
          :disabled="!selectedTarget"
          @click="handleConfirm"
        >
          确定
        </NButton>
      </NFlex>
    </div>
  </NModal>
</template>

<style scoped>
.batch-move-dialog {
  padding: 8px 0;
}

.dialog-description {
  margin-bottom: 16px;
  color: var(--n-text-color-2);
  font-size: 13px;
  line-height: 1.5;
}

:deep(.n-tree-node-content) {
  border-radius: 4px;
}

:deep(.n-tree-node-content:hover) {
  background-color: var(--n-color-hover);
}
</style>
