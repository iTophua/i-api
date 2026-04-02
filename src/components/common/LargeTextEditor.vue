<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { NScrollbar, NButton, NTag } from 'naive-ui'
import { useVirtualScroll } from '@/composables/useEditorOptimizer'

interface Props {
  modelValue: string
  language?: string
  readOnly?: boolean
  height?: string
  maxLinesForVirtualScroll?: number
}

const props = withDefaults(defineProps<Props>(), {
  language: 'plaintext',
  readOnly: false,
  height: '100%',
  maxLinesForVirtualScroll: 1000,
})

const contentRef = ref<HTMLElement | null>(null)
const isVirtualScroll = ref(false)
const scrollTop = ref(0)

const lines = computed(() => {
  return props.modelValue.split('\n')
})

const ITEM_HEIGHT = 20
const VISIBLE_HEIGHT = 600

const { visibleItems, totalHeight, startIndex, updateVisibleRange } = useVirtualScroll(lines, {
  itemHeight: ITEM_HEIGHT,
  visibleHeight: VISIBLE_HEIGHT,
})

watch(
  () => props.modelValue,
  (newValue) => {
    const lineCount = newValue.split('\n').length
    isVirtualScroll.value = lineCount > props.maxLinesForVirtualScroll

    if (isVirtualScroll.value && contentRef.value) {
      updateVisibleRange(scrollTop.value)
    }
  },
  { immediate: true }
)

function handleScroll(event: Event) {
  const target = event.target as HTMLElement
  scrollTop.value = target.scrollTop

  if (isVirtualScroll.value) {
    updateVisibleRange(scrollTop.value)
  }
}

function copyContent() {
  navigator.clipboard.writeText(props.modelValue)
}

function downloadContent() {
  const blob = new Blob([props.modelValue], { type: 'text/plain' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `response-${Date.now()}.txt`
  a.click()
  URL.revokeObjectURL(url)
}

defineExpose({
  copyContent,
  downloadContent,
})
</script>

<template>
  <div class="large-text-editor" :style="{ height }">
    <div v-if="lines.length > 500" class="editor-toolbar">
      <div class="toolbar-info">
        <NTag size="small" :type="isVirtualScroll ? 'success' : 'info'">
          {{ lines.length }} 行
        </NTag>
        <span v-if="isVirtualScroll" class="virtual-scroll-hint"> 虚拟滚动已启用 </span>
      </div>
      <div class="toolbar-actions">
        <NButton text size="small" @click="copyContent">复制</NButton>
        <NButton text size="small" @click="downloadContent">下载</NButton>
      </div>
    </div>

    <NScrollbar
      v-if="isVirtualScroll"
      ref="contentRef"
      class="virtual-scroll-content"
      @scroll="handleScroll"
    >
      <div :style="{ height: `${totalHeight}px`, position: 'relative' }">
        <div
          class="virtual-scroll-items"
          :style="{ transform: `translateY(${startIndex * ITEM_HEIGHT}px)` }"
        >
          <div v-for="(line, index) in visibleItems" :key="startIndex + index" class="line-item">
            <span class="line-number">{{ startIndex + index + 1 }}</span>
            <pre class="line-content">{{ line }}</pre>
          </div>
        </div>
      </div>
    </NScrollbar>

    <NScrollbar v-else ref="contentRef" class="normal-content" @scroll="handleScroll">
      <div class="normal-content-inner">
        <div v-for="(line, index) in lines" :key="index" class="line-item">
          <span class="line-number">{{ index + 1 }}</span>
          <pre class="line-content">{{ line }}</pre>
        </div>
      </div>
    </NScrollbar>
  </div>
</template>

<style scoped>
.large-text-editor {
  width: 100%;
  height: 100%;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  background: var(--n-color);
}

.editor-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-xs) var(--spacing-sm);
  border-bottom: 1px solid var(--n-border-color);
  background: var(--n-color-modal);
  flex-shrink: 0;
}

.toolbar-info {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.virtual-scroll-hint {
  font-size: 11px;
  color: var(--n-text-color-3);
}

.toolbar-actions {
  display: flex;
  gap: var(--spacing-xs);
}

.virtual-scroll-content,
.normal-content {
  flex: 1;
  overflow: auto;
}

.normal-content-inner {
  min-height: 100%;
}

.line-item {
  display: flex;
  align-items: flex-start;
  min-height: 20px;
  line-height: 20px;
  padding: 0 var(--spacing-sm);
  font-family: 'Monaco', 'Menlo', 'Courier New', monospace;
  font-size: 13px;
}

.line-item:hover {
  background: var(--n-color-hover);
}

.line-number {
  display: inline-block;
  min-width: 40px;
  padding-right: var(--spacing-sm);
  text-align: right;
  color: var(--n-text-color-3);
  user-select: none;
  flex-shrink: 0;
}

.line-content {
  flex: 1;
  margin: 0;
  white-space: pre;
  word-break: break-all;
  color: var(--n-text-color-1);
}
</style>
