<script setup lang="ts">
import { NDataTable, NButton, NIcon, NInput, NCheckbox, NDropdown, NPopover } from 'naive-ui'
import { AddOutline, TrashOutline, ChevronDownOutline } from '@vicons/ionicons5'
import { ref, computed, h } from 'vue'
import type { DataTableColumns } from 'naive-ui'
import type { KeyValuePair } from '@/types'
import { COMMON_HEADERS, getHeaderSuggestions, type CommonHeader } from '@/utils/commonHeaders'
import VariableInput from '@/components/common/VariableInput.vue'

const props = defineProps<{
  headers: KeyValuePair[]
}>()

const emit = defineEmits<{
  'update:headers': [headers: KeyValuePair[]]
}>()

const searchQuery = ref('')
const showPopover = ref(false)

const filteredHeaders = computed(() => {
  return getHeaderSuggestions(searchQuery.value)
})

function updateField(index: number, field: keyof KeyValuePair, value: string | boolean) {
  const newData = [...props.headers]
  newData[index] = { ...newData[index], [field]: value }
  emit('update:headers', newData)
}

function addRow() {
  emit('update:headers', [...props.headers, { id: crypto.randomUUID(), key: '', value: '', description: '', enabled: true }])
}

function deleteRow(index: number) {
  const newData = [...props.headers]
  newData.splice(index, 1)
  emit('update:headers', newData)
}

function addHeader(header: CommonHeader) {
  const exists = props.headers.some(h => h.key.toLowerCase() === header.key.toLowerCase())
  if (!exists) {
    emit('update:headers', [
      ...props.headers,
      { id: crypto.randomUUID(), key: header.key, value: header.value || '', description: header.description, enabled: true }
    ])
  }
  showPopover.value = false
  searchQuery.value = ''
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
    width: 180,
    render: (row, index) =>
      h(VariableInput, {
        value: row.key,
        placeholder: '请求头名称',
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
        placeholder: '请求头值',
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

function renderDropdownHeader() {
  return h('div', { class: 'header-dropdown-header' }, [
    h('input', {
      class: 'header-search-input',
      placeholder: '搜索常用请求头...',
      value: searchQuery.value,
      onInput: (e: Event) => {
        searchQuery.value = (e.target as HTMLInputElement).value
      }
    })
  ])
}

function renderDropdownOption(header: CommonHeader) {
  return h('div', {
    class: 'header-option',
    onClick: () => addHeader(header)
  }, [
    h('div', { class: 'header-option-key' }, header.key),
    h('div', { class: 'header-option-value' }, header.value || '(无默认值)'),
    h('div', { class: 'header-option-desc' }, header.description)
  ])
}
</script>

<template>
  <div class="headers-editor">
    <NDataTable
      :columns="columns"
      :data="headers"
      :bordered="false"
      size="small"
      :row-key="(row: KeyValuePair, index: number) => row.id || index"
    />
    <div class="editor-toolbar">
      <NPopover
        v-model:show="showPopover"
        trigger="click"
        placement="bottom-start"
        :width="400"
      >
        <template #trigger>
          <NButton text size="small">
            <template #icon>
              <NIcon :component="ChevronDownOutline" />
            </template>
            常用请求头
          </NButton>
        </template>
        <div class="header-dropdown">
          <NInput
            v-model:value="searchQuery"
            placeholder="搜索请求头..."
            size="small"
            class="header-search"
          />
          <div class="header-list">
            <div
              v-for="header in filteredHeaders"
              :key="header.key"
              class="header-item"
              @click="addHeader(header)"
            >
              <div class="header-item-key">{{ header.key }}</div>
              <div class="header-item-value">{{ header.value || '(无默认值)' }}</div>
              <div class="header-item-desc">{{ header.description }}</div>
            </div>
            <div v-if="filteredHeaders.length === 0" class="header-empty">
              未找到匹配的请求头
            </div>
          </div>
        </div>
      </NPopover>
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

.header-dropdown {
  max-height: 400px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.header-search {
  margin-bottom: 8px;
}

.header-list {
  max-height: 320px;
  overflow-y: auto;
}

.header-item {
  padding: 8px 12px;
  cursor: pointer;
  border-radius: 4px;
  transition: background-color 0.15s;
}

.header-item:hover {
  background: var(--n-color-hover);
}

.header-item-key {
  font-weight: 600;
  font-size: 13px;
  color: var(--n-text-color);
}

.header-item-value {
  font-size: 12px;
  color: var(--n-text-color-3);
  font-family: monospace;
  margin-top: 2px;
}

.header-item-desc {
  font-size: 11px;
  color: var(--n-text-color-3);
  margin-top: 2px;
}

.header-empty {
  padding: 16px;
  text-align: center;
  color: var(--n-text-color-3);
  font-size: 13px;
}
</style>