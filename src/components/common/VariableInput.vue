<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { NTooltip, NInput } from 'naive-ui'
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
const inputRef = ref<InstanceType<typeof NInput> | null>(null)
const wrapperRef = ref<HTMLElement | null>(null)
const mirrorRef = ref<HTMLElement | null>(null)
const dropdownPosition = ref({ top: 0, left: 0, width: 0 })
const isMouseInDropdown = ref(false)

watch(() => props.value, (newVal) => {
  inputValue.value = newVal
  nextTick(syncMirrorScroll)
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

const highlightedHtml = computed(() => {
  const text = inputValue.value
  if (!text) return ''

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
  const wrapper = wrapperRef.value
  if (!wrapper) return

  const rect = wrapper.getBoundingClientRect()
  dropdownPosition.value = {
    top: rect.bottom + 4,
    left: rect.left,
    width: rect.width
  }
}

function handleInput(value: string) {
  inputValue.value = value
  highlightedIndex.value = -1

  const shouldShow = hasUnclosedBrace(value)

  if (shouldShow) {
    showDropdown.value = true
    updateDropdownPosition()
  } else {
    showDropdown.value = false
  }

  emit('update:value', value)
  nextTick(syncMirrorScroll)
}

function handleKeyDown(e: KeyboardEvent) {
  const currentValue = inputValue.value
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
      nextTick(() => {
        const el = inputRef.value?.inputElRef
        if (el) {
          el.setSelectionRange(newValue.length, newValue.length)
        }
      })
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
      nextTick(() => {
        const el = inputRef.value?.inputElRef
        if (el) {
          el.setSelectionRange(newValue.length, newValue.length)
        }
      })
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

  nextTick(() => {
    const el = inputRef.value?.inputElRef
    if (el) {
      el.setSelectionRange(newValue.length, newValue.length)
      el.focus()
    }
  })
}

function handleScroll() {
  if (showDropdown.value) {
    updateDropdownPosition()
  }
}

// 同步镜像层与原生 input 的横向滚动位置
// 原生 input 文字被设为透明，超长内容靠镜像层展示，
// 因此镜像层必须跟随 input 的 scrollLeft 才能正确显示可见区域
function syncMirrorScroll() {
  const inputEl = inputRef.value?.inputElRef as HTMLInputElement | undefined
  if (inputEl && mirrorRef.value) {
    mirrorRef.value.scrollLeft = inputEl.scrollLeft
  }
}

// scroll 事件不会冒泡，必须在原生 <input> 元素上直接绑定监听器
let nativeInputEl: HTMLInputElement | null = null
function bindNativeInput() {
  const el = inputRef.value?.inputElRef as HTMLInputElement | undefined
  if (el && el !== nativeInputEl) {
    if (nativeInputEl) {
      nativeInputEl.removeEventListener('scroll', syncMirrorScroll)
    }
    nativeInputEl = el
    el.addEventListener('scroll', syncMirrorScroll, { passive: true })
  }
}

watch(() => inputRef.value?.inputElRef, () => {
  nextTick(bindNativeInput)
})

onMounted(() => {
  window.addEventListener('scroll', handleScroll, true)
  nextTick(bindNativeInput)
})

onUnmounted(() => {
  window.removeEventListener('scroll', handleScroll, true)
  if (nativeInputEl) {
    nativeInputEl.removeEventListener('scroll', syncMirrorScroll)
    nativeInputEl = null
  }
})
</script>

<template>
  <NTooltip :disabled="!tooltipText" :show-arrow="false">
    <template #trigger>
      <div ref="wrapperRef" class="variable-input-wrapper">
        <div
          ref="mirrorRef"
          class="input-mirror"
          aria-hidden="true"
          v-html="highlightedHtml || (placeholder ? `<span class='mirror-placeholder'>${escapeHtml(placeholder)}</span>` : '')"
        ></div>
        <NInput
          ref="inputRef"
          :value="inputValue"
          :placeholder="placeholder"
          :size="size || 'small'"
          :disabled="disabled"
          :input-props="{ autocomplete: 'off', spellcheck: 'false' }"
          class="variable-input"
          @update:value="handleInput"
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

.input-mirror {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 28px;
  padding: 0 9.5px;
  font-size: 14px;
  line-height: 28px;
  font-family: inherit;
  pointer-events: none;
  white-space: pre;
  z-index: 1;
  /* 允许横向滚动以显示超长内容，纵向隐藏避免撑高 */
  overflow-x: auto;
  overflow-y: hidden;
}

/* 隐藏镜像层的滚动条（靠原生 input 的滚动条即可） */
.input-mirror::-webkit-scrollbar {
  display: none;
}

.input-mirror :deep(.var-chip) {
  color: #18a058;
  background: rgba(24, 160, 88, 0.08);
  border-radius: 3px;
}

.input-mirror :deep(.var-invalid) {
  color: #d03050;
  background: rgba(208, 48, 80, 0.08);
}

.variable-input {
  position: relative;
  z-index: 0;
}

.variable-input :deep(.n-input) {
  height: 28px;
}

.variable-input :deep(.n-input .n-input__border),
.variable-input :deep(.n-input .n-input__state-border) {
  display: none;
}

.variable-input :deep(.n-input__input-el) {
  color: transparent !important;
  caret-color: #18a058;
  -webkit-text-fill-color: transparent !important;
}

.variable-input :deep(.n-input__input-el)::selection {
  background: rgba(24, 128, 56, 0.25);
  color: transparent !important;
  -webkit-text-fill-color: transparent !important;
}

.variable-input :deep(.n-input__placeholder) {
  opacity: 0;
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

:global(.input-mirror) {
  color: rgba(0, 0, 0, 0.9);
}

:global(.input-mirror .mirror-placeholder) {
  color: rgba(0, 0, 0, 0.45);
}

:global([data-theme='dark'] .input-mirror) {
  color: rgba(255, 255, 255, 0.9) !important;
}

:global([data-theme='dark'] .input-mirror .mirror-placeholder) {
  color: rgba(255, 255, 255, 0.45) !important;
}

:global([data-theme='dark'] .variable-suggest-dropdown) {
  background: var(--n-color, #333);
  border-color: var(--n-border-color, #444);
}

:global([data-theme='dark'] .input-mirror .var-chip) {
  color: #63e2b7;
  background: rgba(99, 226, 183, 0.12);
}

:global([data-theme='dark'] .input-mirror .var-invalid) {
  color: #f5a5a5;
  background: rgba(245, 165, 165, 0.12);
}

:global([data-theme='dark'] .variable-input .n-input__input-el) {
  color: transparent !important;
  caret-color: rgba(255, 255, 255, 0.9);
  -webkit-text-fill-color: transparent !important;
}

:global([data-theme='dark'] .variable-suggest-item) {
  color: var(--n-text-color, #eee);
}

:global([data-theme='dark'] .variable-suggest-item:hover),
:global([data-theme='dark'] .variable-suggest-item.highlighted) {
  background: var(--n-color-hover, #444);
}

:global([data-theme='dark'] .variable-suggest-empty) {
  color: var(--n-text-color-3, #888);
}

:global([data-theme='dark'] .suggestion-value) {
  color: var(--n-text-color-3, #888);
}

:global([data-theme='dark'] .tooltip-arrow) {
  color: var(--n-text-color-3, #888);
}
</style>
