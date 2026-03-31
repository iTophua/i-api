<script setup lang="ts">
import { NModal, NCard, NUpload, NButton, NSpace, NAlert, NTabs, NTabPane, NInput } from 'naive-ui'
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
const currentTab = ref('file')
const yamlContent = ref('')

async function handleFileSelect(options: { file: { file: File | null } }) {
  const file = options.file.file
  if (!file) return

  isLoading.value = true
  error.value = ''

  try {
    const text = await file.text()
    const requests = await invoke<Request[]>('import_openapi', { content: text })
    emit('imported', requests)
    handleClose()
  } catch (e) {
    error.value = String(e)
  } finally {
    isLoading.value = false
  }
}

async function handleYamlImport() {
  if (!yamlContent.value.trim()) {
    error.value = '请输入 OpenAPI/Swagger 内容'
    return
  }

  isLoading.value = true
  error.value = ''

  try {
    const requests = await invoke<Request[]>('import_openapi', { content: yamlContent.value })
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
  yamlContent.value = ''
  emit('update:show', false)
}
</script>

<template>
  <NModal :show="props.show" @update:show="emit('update:show', $event)">
    <NCard style="width: 600px" title="导入 OpenAPI/Swagger" :bordered="false" size="medium">
      <NAlert v-if="error" type="error" style="margin-bottom: 16px">
        {{ error }}
      </NAlert>

      <NTabs v-model:value="currentTab" type="line">
        <NTabPane name="file" tab="文件导入">
          <div class="upload-area">
            <NUpload
              accept=".json,.yaml,.yml"
              :show-file-list="false"
              :custom-request="handleFileSelect"
            >
              <NButton>选择 OpenAPI/Swagger 文件</NButton>
            </NUpload>
            <p class="hint">支持 .json、.yaml、.yml 格式</p>
          </div>
        </NTabPane>
        <NTabPane name="paste" tab="粘贴内容">
          <div class="paste-area">
            <NInput
              v-model:value="yamlContent"
              type="textarea"
              :rows="12"
              placeholder="粘贴 OpenAPI/Swagger JSON 或 YAML 内容"
            />
            <NButton
              type="primary"
              :loading="isLoading"
              style="margin-top: 12px"
              @click="handleYamlImport"
            >
              导入
            </NButton>
          </div>
        </NTabPane>
      </NTabs>

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
  margin-top: 12px;
  color: var(--n-text-color-3);
  font-size: 12px;
}

.paste-area {
  padding: 12px 0;
}
</style>
