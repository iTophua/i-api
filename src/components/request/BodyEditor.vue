<script setup lang="ts">
import {
  NRadioGroup,
  NRadioButton,
  NButton,
  NDataTable,
  NInput,
  NCheckbox,
  NSelect,
  NIcon,
} from 'naive-ui'
import { AddOutline, TrashOutline } from '@vicons/ionicons5'
import { computed, shallowRef, onMounted, h, ref } from 'vue'
import type { DataTableColumns } from 'naive-ui'
import type { RequestBody, BodyMode, KeyValuePair, FormDatum } from '@/types'
import { formatJsonString } from '@/composables/useMonacoEditor'
import { open } from '@tauri-apps/plugin-dialog'
import { readFile } from '@tauri-apps/plugin-fs'

const props = defineProps<{
  body: RequestBody | null
}>()

const emit = defineEmits<{
  'update:body': [body: RequestBody]
}>()

const safeBody = computed<RequestBody>(() => {
  const body = props.body ?? { mode: 'none' }
  console.log('BodyEditor - props.body:', JSON.stringify(props.body, null, 2))
  console.log('BodyEditor - safeBody:', JSON.stringify(body, null, 2))
  console.log('BodyEditor - bodyMode:', body.mode)
  console.log('BodyEditor - rawContent:', body.raw)
  return body
})

const MonacoEditor = shallowRef<any>(null)
const binaryFileName = ref<string>('')
const binaryFileSize = ref<number>(0)

onMounted(async () => {
  const mod = await import('@/components/common/MonacoEditor.vue')
  MonacoEditor.value = mod.default
})

const bodyMode = computed({
  get: () => safeBody.value.mode,
  set: (mode: BodyMode) => emit('update:body', { ...safeBody.value, mode }),
})

const rawContent = computed({
  get: () => safeBody.value.raw || '',
  set: (raw: string) => emit('update:body', { ...safeBody.value, raw }),
})

const rawType = computed({
  get: () => safeBody.value.rawType || 'json',
  set: (rawType: RequestBody['rawType']) => emit('update:body', { ...safeBody.value, rawType }),
})

const editorLanguage = computed(() => {
  const type = safeBody.value.rawType || 'json'
  const langMap: Record<string, string> = {
    json: 'json',
    xml: 'xml',
    html: 'html',
    text: 'plaintext',
  }
  return langMap[type] || 'plaintext'
})

function handleBeautify() {
  if (rawType.value === 'json' && rawContent.value) {
    const formatted = formatJsonString(rawContent.value)
    if (formatted !== rawContent.value) {
      rawContent.value = formatted
    }
  }
}

async function handleBinaryFileSelect() {
  try {
    const selected = await open({
      multiple: false,
      title: '选择文件',
    })

    if (selected) {
      const filePath = selected as string
      const fileName = filePath.split('/').pop() || filePath.split('\\').pop() || 'file'
      binaryFileName.value = fileName

      const fileData = await readFile(filePath)
      binaryFileSize.value = fileData.length

      const base64 = btoa(
        new Uint8Array(fileData).reduce((data, byte) => data + String.fromCharCode(byte), '')
      )

      emit('update:body', {
        ...safeBody.value,
        binary: base64,
      })
    }
  } catch (e) {
    console.error('文件读取失败:', e)
  }
}

function clearBinaryFile() {
  binaryFileName.value = ''
  binaryFileSize.value = 0
  emit('update:body', {
    ...safeBody.value,
    binary: undefined,
  })
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1024 / 1024).toFixed(2)} MB`
}

function updateFormData(index: number, field: keyof FormDatum, value: string | boolean) {
  const newData = [...(safeBody.value.formData || [])]
  newData[index] = { ...newData[index], [field]: value }
  emit('update:body', { ...safeBody.value, formData: newData })
}

function addFormDataRow() {
  const current = safeBody.value.formData || []
  emit('update:body', {
    ...safeBody.value,
    formData: [...current, { key: '', value: '', description: '', enabled: true, type: 'text' }],
  })
}

function deleteFormDataRow(index: number) {
  const newData = [...(safeBody.value.formData || [])]
  newData.splice(index, 1)
  emit('update:body', { ...safeBody.value, formData: newData })
}

const formDataColumns: DataTableColumns<FormDatum> = [
  {
    key: 'enabled',
    width: 50,
    align: 'center',
    render: (row, index) =>
      h(NCheckbox, {
        checked: row.enabled,
        onUpdateChecked: (checked: boolean) => updateFormData(index, 'enabled', checked),
      }),
  },
  {
    title: 'Key',
    key: 'key',
    render: (row, index) =>
      h(NInput, {
        value: row.key,
        placeholder: '参数名',
        size: 'small',
        onUpdateValue: (val: string) => updateFormData(index, 'key', val),
      }),
  },
  {
    title: 'Value',
    key: 'value',
    render: (row, index) =>
      h(NInput, {
        value: row.value,
        placeholder: row.type === 'file' ? '选择文件' : '参数值',
        size: 'small',
        disabled: row.type === 'file',
        onUpdateValue: (val: string) => updateFormData(index, 'value', val),
      }),
  },
  {
    title: '类型',
    key: 'type',
    width: 100,
    render: (row, index) =>
      h(NSelect, {
        value: row.type,
        options: [
          { label: '文本', value: 'text' },
          { label: '文件', value: 'file' },
        ],
        size: 'small',
        onUpdateValue: (val: string) => updateFormData(index, 'type', val),
      }),
  },
  {
    key: 'actions',
    width: 60,
    render: (_, index) =>
      h(
        NButton,
        { text: true, type: 'error', onClick: () => deleteFormDataRow(index) },
        { icon: () => h(NIcon, null, { default: () => h(TrashOutline) }) }
      ),
  },
]

function updateUrlencoded(index: number, field: keyof KeyValuePair, value: string | boolean) {
  const newData = [...(safeBody.value.urlencoded || [])]
  newData[index] = { ...newData[index], [field]: value }
  emit('update:body', { ...safeBody.value, urlencoded: newData })
}

function addUrlencodedRow() {
  const current = safeBody.value.urlencoded || []
  emit('update:body', {
    ...safeBody.value,
    urlencoded: [...current, { key: '', value: '', description: '', enabled: true }],
  })
}

function deleteUrlencodedRow(index: number) {
  const newData = [...(safeBody.value.urlencoded || [])]
  newData.splice(index, 1)
  emit('update:body', { ...safeBody.value, urlencoded: newData })
}

const urlencodedColumns: DataTableColumns<KeyValuePair> = [
  {
    key: 'enabled',
    width: 50,
    align: 'center',
    render: (row, index) =>
      h(NCheckbox, {
        checked: row.enabled,
        onUpdateChecked: (checked: boolean) => updateUrlencoded(index, 'enabled', checked),
      }),
  },
  {
    title: 'Key',
    key: 'key',
    render: (row, index) =>
      h(NInput, {
        value: row.key,
        placeholder: '参数名',
        size: 'small',
        onUpdateValue: (val: string) => updateUrlencoded(index, 'key', val),
      }),
  },
  {
    title: 'Value',
    key: 'value',
    render: (row, index) =>
      h(NInput, {
        value: row.value,
        placeholder: '参数值',
        size: 'small',
        onUpdateValue: (val: string) => updateUrlencoded(index, 'value', val),
      }),
  },
  {
    key: 'actions',
    width: 60,
    render: (_, index) =>
      h(
        NButton,
        { text: true, type: 'error', onClick: () => deleteUrlencodedRow(index) },
        { icon: () => h(NIcon, null, { default: () => h(TrashOutline) }) }
      ),
  },
]
</script>

<template>
  <div class="body-editor">
    <div class="mode-selector">
      <NRadioGroup v-model:value="bodyMode" name="body-mode" size="small">
        <NRadioButton value="none">none</NRadioButton>
        <NRadioButton value="form-data">form-data</NRadioButton>
        <NRadioButton value="urlencoded">x-www-form-urlencoded</NRadioButton>
        <NRadioButton value="raw">raw</NRadioButton>
        <NRadioButton value="binary">binary</NRadioButton>
      </NRadioGroup>
      <div v-if="bodyMode === 'raw'" class="raw-options">
        <NRadioGroup v-model:value="rawType" size="small" class="raw-type-selector">
          <NRadioButton value="json">JSON</NRadioButton>
          <NRadioButton value="xml">XML</NRadioButton>
          <NRadioButton value="text">Text</NRadioButton>
          <NRadioButton value="html">HTML</NRadioButton>
        </NRadioGroup>
        <NButton v-if="rawType === 'json'" size="tiny" @click="handleBeautify"> 美化 </NButton>
      </div>
    </div>

    <div class="body-content">
      <div v-if="bodyMode === 'none'" class="content-section">
        <div class="empty-hint">该请求没有请求体</div>
      </div>

      <div v-else-if="bodyMode === 'raw'" class="content-section">
        <div class="editor-wrapper">
          <MonacoEditor
            v-if="MonacoEditor"
            v-model="rawContent"
            :language="editorLanguage"
            height="100%"
          />
        </div>
      </div>

      <div v-else-if="bodyMode === 'binary'" class="content-section">
        <div class="binary-upload">
          <div v-if="!safeBody.binary" class="upload-area">
            <NButton @click="handleBinaryFileSelect">选择文件</NButton>
            <p class="hint">支持任意类型文件</p>
          </div>
          <div v-else class="file-info">
            <div class="file-details">
              <span class="file-name">{{ binaryFileName || '已选择文件' }}</span>
              <span class="file-size">{{ formatFileSize(binaryFileSize) }}</span>
            </div>
            <NButton text type="error" @click="clearBinaryFile">
              <template #icon>
                <NIcon :component="TrashOutline" />
              </template>
            </NButton>
          </div>
        </div>
      </div>

      <div v-else-if="bodyMode === 'form-data'" class="content-section">
        <NDataTable
          :columns="formDataColumns"
          :data="safeBody.formData || []"
          :bordered="false"
          size="small"
        />
        <div class="table-toolbar">
          <NButton size="small" @click="addFormDataRow">
            <template #icon>
              <NIcon :component="AddOutline" />
            </template>
            添加字段
          </NButton>
        </div>
      </div>

      <div v-else-if="bodyMode === 'urlencoded'" class="content-section">
        <NDataTable
          :columns="urlencodedColumns"
          :data="safeBody.urlencoded || []"
          :bordered="false"
          size="small"
        />
        <div class="table-toolbar">
          <NButton size="small" @click="addUrlencodedRow">
            <template #icon>
              <NIcon :component="AddOutline" />
            </template>
            添加字段
          </NButton>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.body-editor {
  display: flex;
  flex-direction: column;
  min-height: 0;
  height: 100%;
  overflow: hidden;
}

.mode-selector {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 0;
  border-bottom: 1px solid var(--n-border-color);
  margin-bottom: 8px;
  flex-shrink: 0;
}

.raw-options {
  display: flex;
  align-items: center;
  gap: 6px;
}

.raw-type-selector {
  opacity: 0.8;
}

.body-content {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.content-section {
  height: 100%;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.content-section :deep(.n-data-table) {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.content-section :deep(.n-data-table-wrapper) {
  flex: 1;
  min-height: 0;
  overflow: auto;
}

.empty-hint {
  color: var(--n-text-color-3);
  text-align: center;
  padding: 24px;
}

.editor-wrapper {
  flex: 1;
  min-height: 0;
  border: 1px solid var(--n-border-color);
  border-radius: 4px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.table-toolbar {
  padding: 4px 0;
  flex-shrink: 0;
}

.binary-upload {
  padding: 12px 0;
}

.upload-area {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.hint {
  color: var(--n-text-color-3);
  font-size: 12px;
}

.file-info {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px;
  background: var(--n-color-hover);
  border-radius: 4px;
}

.file-details {
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex: 1;
}

.file-name {
  font-size: 13px;
}

.file-size {
  font-size: 11px;
  color: var(--n-text-color-3);
}
</style>
