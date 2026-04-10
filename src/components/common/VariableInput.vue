<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { NTooltip } from 'naive-ui'
import { useEnvironmentStore } from '@/stores/environment'

const props = defineProps<{
  value: string
  placeholder?: string
  size?: 'small' | 'medium' | 'large'
  disabled?: boolean
}>()

const emit = defineEmits<{
  'update:value': [value: string]
}>()

const envStore = useEnvironmentStore()

const inputValue = ref(props.value)
const showDropdown = ref(false)
const highlightedIndex = ref(-1)
const inputRef = ref<HTMLInputElement | null>(null)
const mirrorRef = ref<HTMLElement | null>(null)
const dropdownPosition = ref({ top: 0, left: 0, width: 0 })
const isMouseInDropdown = ref(false)

watch(() => props.value, (newVal) => {
  inputValue.value = newVal
})

const openBrace = '\u007B\u007B'
const closeBrace = '\u007D\u007D'

function hasUnclosedBrace(value: string): boolean {
  if (!value) return false
  const lastOpenBrace = value.lastIndexOf(openBrace)
  const lastCloseBrace = value.lastIndexOf(closeBrace)
  return lastOpenBrace !== -1 && (lastCloseBrace === -1 || lastCloseBrace < lastOpenBrace)
}

function getSuggestions(value: string) {
  if (!value || !value.includes(openBrace)) {
    return []
  }

  const lastOpenBrace = value.lastIndexOf(openBrace)
  const lastCloseBrace = value.lastIndexOf(closeBrace)

  if (lastCloseBrace > lastOpenBrace) {
    return []
  }

  const searchText = value.slice(lastOpenBrace + 2)

  const varKeys = Object.keys(envStore.variables)
  if (!searchText) {
    return varKeys.map(key => ({
      label: openBrace + key + closeBrace,
      value: openBrace + key + closeBrace,
      key,
      actualValue: envStore.variables[key]
    }))
  }

  return varKeys
    .filter(key => key.toLowerCase().includes(searchText.toLowerCase()))
    .map(key => ({
      label: openBrace + key + closeBrace,
      value: openBrace + key + closeBrace,
      key,
      actualValue: envStore.variables[key]
    }))
}

const suggestions = computed(() => {
  return getSuggestions(inputValue.value)
})

interface VariableInfo {
  fullMatch: string
  varName: string
  actualValue: string | undefined
  isValid: boolean
}

const variableInfos = computed((): VariableInfo[] => {
  const result: VariableInfo[] = []
  const regex = new RegExp('\\{\\{([^}]+)\\}\\}', 'g')
  let match

  while ((match = regex.exec(inputValue.value)) !== null) {
    const varName = match[1].trim()
    const colonIndex = varName.indexOf(':')
    let actualVarName = varName
    let defaultValue: string | undefined

    if (colonIndex > 0) {
      actualVarName = varName.substring(0, colonIndex).trim()
      const defVal = varName.substring(colonIndex + 1).trim()
      if ((defVal.startsWith('"') && defVal.endsWith('"')) ||
          (defVal.startsWith("'") && defVal.endsWith("'"))) {
        defaultValue = defVal.slice(1, -1)
      } else {
        defaultValue = defVal
      }
    }

    const actualValue = envStore.variables[actualVarName] ?? defaultValue
    const isValid = actualVarName in envStore.variables || defaultValue !== undefined

    result.push({
      fullMatch: match[0],
      varName: match[1],
      actualValue,
      isValid
    })
  }

  return result
})

const hasInvalidVariable = computed(() => {
  return variableInfos.value.some(v => !v.isValid)
})

const tooltipText = computed(() => {
  const infos = variableInfos.value
  if (infos.length === 0) return ''

  return infos.map(info => {
    if (info.isValid) {
      return `${info.varName}: ${info.actualValue}`
    } else {
      return `${info.varName}: 未定义`
    }
  }).join('\n')
})

const inputStatus = computed(() => {
  if (variableInfos.value.some(v => !v.isValid)) return 'error'
  if (variableInfos.value.length > 0) return 'success'
  return undefined
})

const highlightedHtml = computed(() => {
  const text = inputValue.value
  if (!text) {
    return props.placeholder ? `<span class='mirror-placeholder'>${escapeHtml(props.placeholder)}</span>` : ''
  }

  const regex = new RegExp('\\{\\{([^}]+)\\}\\}', 'g')
  let result = ''
  let lastIndex = 0
  let match

  while ((match = regex.exec(text)) !== null) {
    if (match.index > lastIndex) {
      result += escapeHtml(text.slice(lastIndex, match.index))
    }

    const varName = match[1].trim()
    const colonIndex = varName.indexOf(':')
    let actualVarName = varName
    if (colonIndex > 0) {
      actualVarName = varName.substring(0, colonIndex).trim()
    }
    const isValid = actualVarName in envStore.variables
    const cls = isValid ? 'var-chip' : 'var-chip var-invalid'
    result += `<span class="${cls}">${escapeHtml(match[0])}</span>`

    lastIndex = regex.lastIndex
  }

  if (lastIndex < text.length) {
    result += escapeHtml(text.slice(lastIndex))
  }

  return result
})

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;')
}

function updateDropdownPosition() {
  const input = inputRef.value
  if (!input) return

  const inputRect = input.getBoundingClientRect()
  dropdownPosition.value = {
    top: inputRect.bottom + 4,
    left: inputRect.left,
    width: inputRect.width
  }
}

function onInput(e: Event) {
  const v = (e.target as HTMLInputElement).value
  inputValue.value = v
  highlightedIndex.value = -1

  const currentSuggestions = getSuggestions(v)
  const shouldShow = hasUnclosedBrace(v)

  if (shouldShow) {
    showDropdown.value = true
    updateDropdownPosition()
  } else {
    showDropdown.value = false
  }

  emit('update:value', v)
}

function handleKeyDown(e: KeyboardEvent) {
  const target = e.target as HTMLInputElement
  const currentValue = target?.value ?? inputValue.value
  const currentSuggestions = getSuggestions(currentValue)
  const shouldShow = hasUnclosedBrace(currentValue)

  if (!shouldShow && currentSuggestions.length === 0) {
    showDropdown.value = false
    return
  }

  if (e.key === 'ArrowDown') {
    e.preventDefault()
    if (!showDropdown.value) {
      showDropdown.value = true
      updateDropdownPosition()
    }
    if (highlightedIndex.value === -1) {
      highlightedIndex.value = 0
    } else {
      highlightedIndex.value = Math.min(
        highlightedIndex.value + 1,
        currentSuggestions.length - 1
      )
    }
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    if (!showDropdown.value) {
      showDropdown.value = true
      updateDropdownPosition()
    }
    if (highlightedIndex.value === -1) {
      highlightedIndex.value = Math.max(0, currentSuggestions.length - 1)
    } else {
      highlightedIndex.value = Math.max(highlightedIndex.value - 1, 0)
    }
  } else if (e.key === 'Enter') {
    if (showDropdown.value && highlightedIndex.value >= 0 && currentSuggestions[highlightedIndex.value]) {
      e.preventDefault()
      const newValue = currentValue.slice(0, currentValue.lastIndexOf(openBrace)) + currentSuggestions[highlightedIndex.value].value
      inputValue.value = newValue
      emit('update:value', newValue)
      showDropdown.value = false
      highlightedIndex.value = -1
      inputRef.value?.setSelectionRange(newValue.length, newValue.length)
      inputRef.value?.focus()
    }
  } else if (e.key === 'Escape') {
    showDropdown.value = false
    highlightedIndex.value = -1
  } else if (e.key === 'Tab') {
    if (showDropdown.value && highlightedIndex.value >= 0 && currentSuggestions[highlightedIndex.value]) {
      e.preventDefault()
      const newValue = currentValue.slice(0, currentValue.lastIndexOf(openBrace)) + currentSuggestions[highlightedIndex.value].value
      inputValue.value = newValue
      emit('update:value', newValue)
      showDropdown.value = false
      highlightedIndex.value = -1
      inputRef.value?.setSelectionRange(newValue.length, newValue.length)
      inputRef.value?.focus()
    } else {
      showDropdown.value = false
    }
  }
}

function handleFocus() {
  const shouldShow = hasUnclosedBrace(inputValue.value)

  if (shouldShow) {
    showDropdown.value = true
    updateDropdownPosition()
  }

  requestAnimationFrame(() => {
    inputRef.value?.setSelectionRange(inputValue.value.length, inputValue.value.length)
  })
}

function handleBlur() {
  if (isMouseInDropdown.value) return

  setTimeout(() => {
    showDropdown.value = false
    highlightedIndex.value = -1
  }, 150)
}

function handleMouseEnterDropdown() {
  isMouseInDropdown.value = true
}

function handleMouseLeaveDropdown() {
  isMouseInDropdown.value = false
}

function handleMouseEnter(index: number) {
  highlightedIndex.value = index
}

function handleItemClick(suggestion: { label: string; value: string }) {
  const currentValue = inputValue.value
  const newValue = currentValue.slice(0, currentValue.lastIndexOf(openBrace)) + suggestion.value
  inputValue.value = newValue
  emit('update:value', newValue)
  showDropdown.value = false
  highlightedIndex.value = -1
  isMouseInDropdown.value = false

  inputRef.value?.setSelectionRange(newValue.length, newValue.length)
  inputRef.value?.focus()
}

function handleScroll() {
  if (showDropdown.value) {
    updateDropdownPosition()
  }
}

onMounted(() => {
  window.addEventListener('scroll', handleScroll, true)
})

onUnmounted(() => {
  window.removeEventListener('scroll', handleScroll, true)
})
</script>

<template>
  <NTooltip :disabled="!tooltipText" :show-arrow="false">
    <template #trigger>
      <div class="variable-input-wrapper" :class="[`size-${size || 'small'}`, { disabled }]">
        <div
          ref="mirrorRef"
          class="input-mirror"
          aria-hidden="true"
          v-html="highlightedHtml"
        ></div>
        <input
          ref="inputRef"
          type="text"
          class="variable-native-input"
          autocomplete="off"
          spellcheck="false"
          :value="inputValue"
          :disabled="disabled"
          @input="onInput"
          @keydown="handleKeyDown"
          @focus="handleFocus"
          @blur="handleBlur"
        />
        <div v-if="variableInfos.length > 0 && hasInvalidVariable" class="var-indicator invalid"></div>
      </div>
    </template>
    <div class="variable-tooltip-content">
      <div v-for="(info, idx) in variableInfos" :key="idx" class="tooltip-line">
        <span :class="info.isValid ? 'valid' : 'invalid'" v-text="info.varName"></span>
        <span class="tooltip-arrow">→</span>
        <span :class="info.isValid ? 'valid-value' : 'invalid-value'" v-text="info.actualValue ?? '未定义'"></span>
      </div>
    </div>
  </NTooltip>
  <Teleport to="body">
    <div
      v-if="showDropdown"
      class="variable-suggest-dropdown"
      :style="{
        top: dropdownPosition.top + 'px',
        left: dropdownPosition.left + 'px',
        width: dropdownPosition.width + 'px'
      }"
      @mouseenter="handleMouseEnterDropdown"
      @mouseleave="handleMouseLeaveDropdown"
    >
      <template v-if="suggestions.length > 0">
        <div
          v-for="(item, index) in suggestions"
          :key="item.key"
          class="variable-suggest-item"
          :class="{ highlighted: index === highlightedIndex }"
          @mouseenter="handleMouseEnter(index)"
          @mousedown.prevent="handleItemClick(item)"
        >
          <span class="suggestion-label" v-text="item.label"></span>
          <span v-if="item.actualValue" class="suggestion-value" v-text="item.actualValue"></span>
        </div>
      </template>
      <div v-else class="variable-suggest-empty">
        暂无环境变量
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.variable-input-wrapper {
  position: relative;
  width: 100%;
}

.variable-input-wrapper.size-small {
  height: 32px;
}

.variable-input-wrapper.size-medium {
  height: 40px;
}

.variable-input-wrapper.size-large {
  height: 48px;
}

.variable-input-wrapper.disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.input-mirror {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  padding: 6px 12px;
  font-size: 14px;
  line-height: 1.5;
  font-family: inherit;
  pointer-events: none;
  white-space: pre-wrap;
  word-break: break-all;
  border-radius: 3px;
  z-index: 0;
  color: var(--n-text-color, #333);
  box-sizing: border-box;
  overflow: hidden;
  max-height: 100%;
}

.input-mirror :deep(.var-chip) {
  color: #18a058;
  font-weight: 500;
  background: rgba(24, 160, 88, 0.08);
  border-radius: 3px;
  padding: 0 2px;
}

.input-mirror :deep(.var-invalid) {
  color: #d03050;
  background: rgba(208, 48, 80, 0.08);
}

.input-mirror :deep(.mirror-placeholder) {
  color: var(--n-placeholder-color, #aaa);
}

.variable-native-input {
  position: relative;
  z-index: 1;
  width: 100%;
  height: 100%;
  padding: 6px 12px;
  border: 1px solid var(--n-border-color, #e0e0e0);
  border-radius: 3px;
  background: transparent;
  color: transparent;
  caret-color: var(--n-text-color, #333);
  font-size: 14px;
  font-family: inherit;
  line-height: 1.5;
  outline: none;
  -webkit-text-fill-color: transparent;
  cursor: text;
  box-sizing: border-box;
}

.variable-native-input::selection {
  background: rgba(24, 128, 56, 0.25);
  -webkit-text-fill-color: transparent;
}

.variable-native-input:focus {
  border-color: var(--n-color-primary, #18a058);
  box-shadow: 0 0 0 2px rgba(24, 160, 88, 0.15);
}

.variable-native-input:disabled {
  cursor: not-allowed;
}

.var-indicator {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  width: 8px;
  height: 8px;
  border-radius: 50%;
  z-index: 2;
  pointer-events: none;
}

.var-indicator.valid {
  background-color: #18a058;
}

.var-indicator.invalid {
  background-color: #d03050;
}

.variable-suggest-dropdown {
  position: fixed;
  max-height: 200px;
  overflow-y: auto;
  background: var(--n-color, #fff);
  border: 1px solid var(--n-border-color, #e0e0e0);
  border-radius: 4px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
  z-index: 9999;
}

.variable-suggest-item {
  padding: 6px 12px;
  cursor: pointer;
  font-size: 13px;
  font-family: monospace;
  color: var(--n-text-color, #333);
  transition: background-color 0.15s;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.variable-suggest-item:hover,
.variable-suggest-item.highlighted {
  background: var(--n-color-hover, #f0f0f0);
}

.variable-suggest-item:first-child {
  border-top-left-radius: 4px;
  border-top-right-radius: 4px;
}

.variable-suggest-item:last-child {
  border-bottom-left-radius: 4px;
  border-bottom-right-radius: 4px;
}

.suggestion-label {
  color: var(--n-color-primary, #18a058);
  font-weight: 500;
}

.suggestion-value {
  color: var(--n-text-color-3, #999);
  font-size: 12px;
  max-width: 150px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.variable-suggest-empty {
  padding: 8px 12px;
  font-size: 13px;
  color: var(--n-text-color-3, #999);
  text-align: center;
}

.variable-tooltip-content {
  font-size: 12px;
  line-height: 1.6;
}

.tooltip-line {
  display: flex;
  align-items: center;
  gap: 4px;
}

.tooltip-line .valid {
  color: #18a058;
  font-weight: 500;
}

.tooltip-line .invalid {
  color: #d03050;
  font-weight: 500;
}

.tooltip-arrow {
  color: var(--n-text-color-3, #999);
}

.tooltip-line .valid-value {
  color: #18a058;
}

.tooltip-line .invalid-value {
  color: #d03050;
}

:global(.dark) .variable-suggest-dropdown {
  background: var(--n-color, #333);
  border-color: var(--n-border-color, #444);
}

:global(.dark) .input-mirror :deep(.var-chip) {
  color: #63e2b7;
  background: rgba(99, 226, 183, 0.12);
}

:global(.dark) .input-mirror :deep(.var-invalid) {
  color: #f5a5a5;
  background: rgba(245, 165, 165, 0.12);
}

:global(.dark) .input-mirror :deep(.mirror-placeholder) {
  color: var(--n-placeholder-color, #666);
}

:global(.dark) .variable-native-input {
  caret-color: var(--n-text-color, #eee);
}

:global(.dark) .variable-native-input:focus {
  border-color: #63e2b7;
  box-shadow: 0 0 0 2px rgba(99, 226, 183, 0.15);
}

:global(.dark) .variable-suggest-item {
  color: var(--n-text-color, #eee);
}

:global(.dark) .variable-suggest-item:hover,
:global(.dark) .variable-suggest-item.highlighted {
  background: var(--n-color-hover, #444);
}

:global(.dark) .variable-suggest-empty {
  color: var(--n-text-color-3, #888);
}

:global(.dark) .suggestion-value {
  color: var(--n-text-color-3, #888);
}

:global(.dark) .tooltip-arrow {
  color: var(--n-text-color-3, #888);
}
</style>
