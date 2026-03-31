<script setup lang="ts">
import { NModal, NCard, NButton, NSpace, NAlert } from 'naive-ui'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Request } from '@/types'

const props = defineProps<{
  show: boolean
  requests: Request[]
}>()

const emit = defineEmits<{
  'update:show': [show: boolean]
}>()

const error = ref('')
const isLoading = ref(false)
const exportUrl = ref('')

async function handleExport() {
  isLoading.value = true
  error.value = ''

  try {
    const json = await invoke<string>('export_postman_collection', {
      requests: props.requests,
    })
    
    const blob = new Blob([JSON.stringify(json, null, 2)], { type: 'application/json' })
    exportUrl.value = URL.createObjectURL(blob)
    
    const a = document.createElement('a')
    a.href = exportUrl.value
    a.download = 'iapi-collection.json'
    a.click()
    
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
    <NCard style="width: 400px" title="导出 Postman 集合" :bordered="false" size="medium">
      <NAlert v-if="error" type="error" style="margin-bottom: 16px">
        {{ error }}
      </NAlert>

      <p>将导出 {{ requests.length }} 个请求为 Postman Collection v2.1 格式</p>

      <template #footer>
        <NSpace justify="end">
          <NButton @click="handleClose">取消</NButton>
          <NButton type="primary" :loading="isLoading" @click="handleExport">
            导出
          </NButton>
        </NSpace>
      </template>
    </NCard>
  </NModal>
</template>