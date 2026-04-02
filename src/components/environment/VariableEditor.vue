<script setup lang="ts">
import { NInput, NButton, NSwitch, NTag } from 'naive-ui'
import type { Variable } from '@/types'

interface Props {
  variable: Variable
  index: number
}

const props = defineProps<Props>()

const emit = defineEmits<{
  update: [index: number, variable: Variable]
  delete: [index: number]
  toggle: [index: number, enabled: boolean]
}>()

function handleKeyChange(value: string) {
  emit('update', props.index, { ...props.variable, key: value })
}

function handleValueChange(value: string) {
  emit('update', props.index, { ...props.variable, value })
}

function handleToggle(checked: boolean) {
  emit('toggle', props.index, checked)
}

function handleDelete() {
  emit('delete', props.index)
}
</script>

<template>
  <div class="variable-editor">
    <div class="variable-toggle-wrapper">
      <NSwitch
        :checked="variable.enabled"
        size="small"
        @update:checked="handleToggle"
      />
    </div>

    <NInput
      :value="variable.key"
      placeholder="变量名"
      size="small"
      class="variable-key-input"
      @update:value="handleKeyChange"
    />

    <NInput
      :value="variable.value"
      placeholder="变量值"
      size="small"
      class="variable-value-input"
      @update:value="handleValueChange"
    >
      <template #suffix>
        <NTag v-if="variable.value.includes('{{')" type="warning" size="small">
          引用
        </NTag>
      </template>
    </NInput>

    <NButton
      text
      size="small"
      type="error"
      class="delete-btn"
      @click="handleDelete"
    >
      <template #icon>
        <AppIcon type="trash" :size="14" />
      </template>
    </NButton>
  </div>
</template>

<style scoped>
.variable-editor {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-sm);
  border-radius: var(--radius-md);
  background: var(--n-color-hover);
  transition: all 0.2s ease;
}

.variable-editor:hover {
  background: var(--n-color-pressed);
}

.variable-toggle-wrapper {
  width: 40px;
  display: flex;
  justify-content: center;
}

.variable-key-input {
  flex: 1;
  min-width: 120px;
  font-family: monospace;
}

.variable-value-input {
  flex: 2;
  min-width: 200px;
  font-family: monospace;
}

.delete-btn {
  opacity: 0;
  transition: opacity 0.2s;
}

.variable-editor:hover .delete-btn {
  opacity: 1;
}
</style>
