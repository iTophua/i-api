<script setup lang="ts">
import { NModal, NCard, NInput, NButton, NSpace, NSelect } from 'naive-ui'
import { ref, computed, watch } from 'vue'
import type { Request, Collection } from '@/types'

const props = defineProps<{
  show: boolean
  request: Request | null
  collections: Collection[]
  defaultCollectionId?: string
}>()

const emit = defineEmits<{
  'update:show': [show: boolean]
  save: [data: { name: string; collectionId: string }]
  'create-collection': [name: string]
}>()

const requestName = ref('')
const selectedCollectionId = ref('')
const showNewCollectionInput = ref(false)
const newCollectionName = ref('')

const collectionOptions = computed(() =>
  props.collections.map((c) => ({
    label: c.name,
    value: c.id,
  }))
)

watch(
  () => props.show,
  (show) => {
    if (show) {
      requestName.value = props.request?.name || '未命名请求'
      selectedCollectionId.value = props.defaultCollectionId || props.collections[0]?.id || ''
      showNewCollectionInput.value = false
      newCollectionName.value = ''
    }
  }
)

watch(
  () => props.collections.length,
  () => {
    if (props.collections.length > 0 && !selectedCollectionId.value) {
      selectedCollectionId.value = props.collections[props.collections.length - 1].id
    }
  }
)

function handleSave() {
  if (requestName.value.trim() && selectedCollectionId.value) {
    emit('save', {
      name: requestName.value.trim(),
      collectionId: selectedCollectionId.value,
    })
    emit('update:show', false)
  }
}

function handleClose() {
  emit('update:show', false)
}

function handleCreateCollection() {
  if (newCollectionName.value.trim()) {
    emit('create-collection', newCollectionName.value.trim())
    newCollectionName.value = ''
    showNewCollectionInput.value = false
  }
}

function cancelCreateCollection() {
  newCollectionName.value = ''
  showNewCollectionInput.value = false
}
</script>

<template>
  <NModal :show="props.show" @update:show="emit('update:show', $event)">
    <NCard style="width: 500px" title="保存请求" :bordered="false" size="medium">
      <div class="save-request-form">
        <div class="form-item">
          <label>请求名称</label>
          <NInput
            v-model:value="requestName"
            placeholder="请输入请求名称"
            @keyup.enter="handleSave"
          />
        </div>

        <div class="form-item">
          <label>保存到集合</label>
          <div class="collection-selector">
            <NSelect
              v-model:value="selectedCollectionId"
              :options="collectionOptions"
              placeholder="选择集合"
              class="collection-select"
            />
            <NButton
              v-if="!showNewCollectionInput"
              type="primary"
              ghost
              @click="showNewCollectionInput = true"
            >
              新建
            </NButton>
          </div>
          <div v-if="showNewCollectionInput" class="new-collection-input">
            <NInput
              v-model:value="newCollectionName"
              placeholder="输入新集合名称"
              @keyup.enter="handleCreateCollection"
            />
            <NSpace>
              <NButton size="small" @click="cancelCreateCollection">取消</NButton>
              <NButton
                size="small"
                type="primary"
                :disabled="!newCollectionName.trim()"
                @click="handleCreateCollection"
              >
                确定
              </NButton>
            </NSpace>
          </div>
        </div>
      </div>

      <template #footer>
        <NSpace justify="end">
          <NButton @click="handleClose">取消</NButton>
          <NButton
            type="primary"
            :disabled="!requestName.trim() || !selectedCollectionId"
            @click="handleSave"
          >
            保存
          </NButton>
        </NSpace>
      </template>
    </NCard>
  </NModal>
</template>

<style scoped>
.save-request-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-item label {
  font-size: 13px;
  font-weight: 500;
  color: var(--n-text-color-2);
}

.collection-selector {
  display: flex;
  gap: 8px;
}

.collection-select {
  flex: 1;
}

.new-collection-input {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 8px;
  background: var(--n-color-embedded);
  border-radius: 4px;
}

.new-collection-input :deep(.n-input) {
  margin-bottom: 4px;
}
</style>
