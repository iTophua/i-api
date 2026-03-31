<script setup lang="ts">
import { NModal, NCard, NUpload, NButton, NSpace, NAlert } from 'naive-ui'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Request } from '@/types'

const props = defineProps<{
  show: boolean
}>()

const emit = defineEmits<{
  'update:show': [show: boolean]
  imported: [requests: Request[]]
}>()

const error = ref('')
const isLoading = ref(false)

async function handleFileSelect(options: { file: { file: File | null } }) {
  const file = options.file.file
  if (!file) return

  isLoading.value = true
  error.value = ''

  try {
    const text = await file.text()
    const requests = await invoke<Request[]>('import_postman_collection', {
      json: text,
    })
    emit('imported', requests)
    handleClose()
  } catch (e) {
    error.value = String(e)
  } finally {
    isLoading.value = false
  }
}

function handleClose() {
  error.value = ''
  emit('update:show', false)
}
</script>

<template>
  <NModal :show="props.show" @update:show="emit('update:show', $event)">
    <NCard style="width: 500px" title="导入 Postman 集合" :bordered="false" size="medium">
      <NAlert v-if="error" type="error" style="margin-bottom: 16px">
        {{ error }}
      </NAlert>

      <NUpload accept=".json" :show-file-list="false" :custom-request="handleFileSelect">
        <NButton>选择 Postman Collection JSON 文件</NButton>
      </NUpload>

      <template #footer>
        <NSpace justify="end">
          <NButton @click="handleClose">取消</NButton>
        </NSpace>
      </template>
    </NCard>
  </NModal>
</template>
