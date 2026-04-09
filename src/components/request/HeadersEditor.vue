<script setup lang="ts">
import { NDataTable, NButton, NIcon, NInput, NCheckbox, NDropdown } from 'naive-ui'
import { AddOutline, TrashOutline } from '@vicons/ionicons5'
import { h } from 'vue'
import type { DataTableColumns } from 'naive-ui'
import type { KeyValuePair } from '@/types'

const props = defineProps<{
  headers: KeyValuePair[]
}>()

const emit = defineEmits<{
  'update:headers': [headers: KeyValuePair[]]
}>()

const commonHeaders = [
  { label: 'Content-Type: application/json', key: 'Content-Type', value: 'application/json' },
  {
    label: 'Content-Type: application/x-www-form-urlencoded',
    key: 'Content-Type',
    value: 'application/x-www-form-urlencoded',
  },
  { label: 'Content-Type: multipart/form-data', key: 'Content-Type', value: 'multipart/form-data' },
  { label: 'Accept: application/json', key: 'Accept', value: 'application/json' },
  { label: 'Accept: text/html', key: 'Accept', value: 'text/html' },
  { label: 'Authorization: Bearer', key: 'Authorization', value: 'Bearer ' },
]

function updateField(index: number, field: keyof KeyValuePair, value: string | boolean) {
  const newData = [...props.headers]
  newData[index] = { ...newData[index], [field]: value }
  emit('update:headers', newData)
}

function addRow() {
  emit('update:headers', [...props.headers, { key: '', value: '', description: '', enabled: true }])
}

function deleteRow(index: number) {
  const newData = [...props.headers]
  newData.splice(index, 1)
  emit('update:headers', newData)
}

function handleCommonHeader(header: (typeof commonHeaders)[0]) {
  emit('update:headers', [
    ...props.headers,
    { key: header.key, value: header.value, description: '', enabled: true },
  ])
}

const columns: DataTableColumns<KeyValuePair> = [
  {
    key: 'enabled',
    width: 50,
    align: 'center',
    render: (row, index) =>
      h(NCheckbox, {
        checked: row.enabled,
        onUpdateChecked: (checked: boolean) => updateField(index, 'enabled', checked),
      }),
  },
  {
    title: 'Key',
    key: 'key',
    render: (row, index) =>
      h(NInput, {
        value: row.key,
        placeholder: '请求头名称',
        size: 'small',
        onUpdateValue: (val: string) => updateField(index, 'key', val),
      }),
  },
  {
    title: 'Value',
    key: 'value',
    render: (row, index) =>
      h(NInput, {
        value: row.value,
        placeholder: '请求头值',
        size: 'small',
        onUpdateValue: (val: string) => updateField(index, 'value', val),
      }),
  },
  {
    title: '描述',
    key: 'description',
    render: (row, index) =>
      h(NInput, {
        value: row.description || '',
        placeholder: '描述（可选）',
        size: 'small',
        onUpdateValue: (val: string) => updateField(index, 'description', val),
      }),
  },
  {
    key: 'actions',
    width: 60,
    render: (_, index) =>
      h(
        NButton,
        { text: true, type: 'error', onClick: () => deleteRow(index) },
        { icon: () => h(NIcon, null, { default: () => h(TrashOutline) }) }
      ),
  },
]

const dropdownOptions = commonHeaders.map((h) => ({ label: h.label, key: h.key + h.value }))
</script>

<template>
  <div class="headers-editor">
    <NDataTable :columns="columns" :data="headers" :bordered="false" size="small" />
    <div class="editor-toolbar">
      <NDropdown
        :options="dropdownOptions"
        @select="
          (k: string) => handleCommonHeader(commonHeaders.find((h) => h.key + h.value === k)!)
        "
      >
        <NButton text size="small"> 常用请求头 </NButton>
      </NDropdown>
      <NButton text type="primary" size="small" @click="addRow">
        <template #icon>
          <NIcon :component="AddOutline" />
        </template>
        添加请求头
      </NButton>
    </div>
  </div>
</template>

<style scoped>
.headers-editor {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  overflow: hidden;
}

.headers-editor :deep(.n-data-table) {
  flex: 1;
  min-height: 0;
  overflow: auto;
}

.headers-editor :deep(.n-data-table-wrapper) {
  height: 100%;
  overflow: auto;
}

.headers-editor :deep(.n-data-table-td) {
  padding: 4px 8px;
}

.headers-editor :deep(.n-data-table-th) {
  padding: 6px 8px;
}

.headers-editor :deep(.n-input) {
  --n-height: 28px;
}

.editor-toolbar {
  padding: 6px 0;
  display: flex;
  gap: 16px;
  align-items: center;
  flex-shrink: 0;
}
</style>
