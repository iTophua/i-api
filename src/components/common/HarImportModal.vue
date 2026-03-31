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
    const requests = await invoke<Request[]>('import_har', { content: text })
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
    <NCard style="width: 500px" title="导入 HAR 文件" :bordered="false" size="medium">
      <NAlert v-if="error" type="error" style="margin-bottom: 16px">
        {{ error }}
      </NAlert>

      <div class="upload-area">
        <NUpload accept=".har" :show-file-list="false" :custom-request="handleFileSelect">
          <NButton>选择 HAR 文件</NButton>
        </NUpload>
        <p class="hint">
          HAR (HTTP Archive) 文件可以从浏览器开发者工具导出<br />
          打开开发者工具 → Network → 右键 → Save all as HAR
        </p>
      </div>

      <template #footer>
        <NSpace justify="end">
          <NButton @click="handleClose">取消</NButton>
        </NSpace>
      </template>
    </NCard>
  </NModal>
</template>

<style scoped>
.upload-area {
  padding: 20px 0;
  text-align: center;
}

.hint {
  margin-top: 16px;
  color: var(--n-text-color-3);
  font-size: 12px;
  line-height: 1.6;
}
</style>
