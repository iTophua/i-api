<script setup lang="ts">
import { NDataTable, NButton, NIcon, NCheckbox } from 'naive-ui'
import { AddOutline, TrashOutline } from '@vicons/ionicons5'
import { h } from 'vue'
import type { DataTableColumns } from 'naive-ui'
import type { KeyValuePair } from '@/types'
import VariableInput from '@/components/common/VariableInput.vue'

const props = defineProps<{
  params: KeyValuePair[]
}>()

const emit = defineEmits<{
  'update:params': [params: KeyValuePair[]]
}>()

function updateField(index: number, field: keyof KeyValuePair, value: string | boolean) {
  const newData = [...props.params]
  newData[index] = { ...newData[index], [field]: value }
  emit('update:params', newData)
}

function addRow() {
  emit('update:params', [
    ...props.params,
    { id: crypto.randomUUID(), key: '', value: '', description: '', enabled: true },
  ])
}

function deleteRow(index: number) {
  const newData = [...props.params]
  newData.splice(index, 1)
  emit('update:params', newData)
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
    width: 150,
    render: (row, index) =>
      h(VariableInput, {
        value: row.key,
        placeholder: '参数名',
        size: 'small',
        'onUpdate:value': (val: string) => updateField(index, 'key', val),
      }),
  },
  {
    title: 'Value',
    key: 'value',
    render: (row, index) =>
      h(VariableInput, {
        value: row.value,
        placeholder: '参数值',
        size: 'small',
        'onUpdate:value': (val: string) => updateField(index, 'value', val),
      }),
  },
  {
    title: '描述',
    key: 'description',
    render: (row, index) =>
      h(VariableInput, {
        value: row.description || '',
        placeholder: '描述（可选）',
        size: 'small',
        'onUpdate:value': (val: string) => updateField(index, 'description', val),
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
</script>

<template>
  <div class="params-editor">
    <NDataTable
      :columns="columns"
      :data="params"
      :bordered="false"
      size="small"
      :row-key="(row: KeyValuePair) => row.id || ''"
    />
    <div class="add-row">
      <NButton text type="primary" size="small" @click="addRow">
        <template #icon>
          <NIcon :component="AddOutline" />
        </template>
        添加参数
      </NButton>
    </div>
  </div>
</template>

<style scoped>
.params-editor {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  overflow: hidden;
}

.params-editor :deep(.n-data-table) {
  flex: 1;
  min-height: 0;
  overflow: auto;
}

.params-editor :deep(.n-data-table-wrapper) {
  height: 100%;
  overflow: auto;
}

.params-editor :deep(.n-data-table-td) {
  padding: 4px 8px;
  vertical-align: middle;
}

.params-editor :deep(.n-data-table-th) {
  padding: 6px 8px;
}

.params-editor :deep(.n-input) {
  --n-height: 28px;
}

.params-editor :deep(.n-data-table-td:last-child) {
  padding: 4px 8px;
}

.params-editor :deep(.n-data-table-td:last-child .n-button) {
  padding: 0;
  height: 28px;
  line-height: 28px;
}

.params-editor :deep(.n-data-table-td:last-child .n-icon) {
  font-size: 16px;
}

.add-row {
  padding: 6px 0;
  flex-shrink: 0;
}
</style>
