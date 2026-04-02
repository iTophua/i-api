<script setup lang="ts">
import { NModal, NCard, NInput, NButton, NSpace, NAlert } from 'naive-ui'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Request } from '@/types'

const props = defineProps<{
  show: boolean
}>()

const emit = defineEmits<{
  'update:show': [show: boolean]
  import: [request: Request]
}>()

const curlCommand = ref('')
const error = ref('')
const isLoading = ref(false)

async function handleImport() {
  if (!curlCommand.value.trim()) {
    error.value = '请输入 cURL 命令'
    return
  }

  isLoading.value = true
  error.value = ''

  try {
    const request = await invoke<Request>('parse_curl_command', {
      curl: curlCommand.value,
    })

    await invoke('save_temporary_request', { request })
    emit('import', request)
    handleClose()
  } catch (e) {
    error.value = String(e)
  } finally {
    isLoading.value = false
  }
}

function handleClose() {
  curlCommand.value = ''
  error.value = ''
  emit('update:show', false)
}
</script>

<template>
  <NModal :show="props.show" @update:show="emit('update:show', $event)">
    <NCard style="width: 600px" title="导入 cURL" :bordered="false" size="medium">
      <NAlert v-if="error" type="error" style="margin-bottom: 16px">
        {{ error }}
      </NAlert>

      <NInput
        v-model:value="curlCommand"
        type="textarea"
        :rows="10"
        placeholder="粘贴 cURL 命令，例如：
curl 'https://api.example.com/users' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer token' \
  -d '{&quot;name&quot;: &quot;test&quot;}'"
      />

      <template #footer>
        <NSpace justify="end">
          <NButton @click="handleClose">取消</NButton>
          <NButton type="primary" :loading="isLoading" @click="handleImport"> 导入 </NButton>
        </NSpace>
      </template>
    </NCard>
  </NModal>
</template>
