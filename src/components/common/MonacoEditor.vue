<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick, computed } from 'vue'
import * as monaco from 'monaco-editor'
import type { Language } from '@/composables/useMonacoEditor'
import { useSettingsStore } from '@/stores'

const props = withDefaults(
  defineProps<{
    modelValue: string
    language?: Language
    readOnly?: boolean
    minimap?: boolean
    lineNumbers?: boolean
    wordWrap?: 'on' | 'off'
    height?: string
    enablePerformanceOptimization?: boolean
  }>(),
  {
    language: 'plaintext',
    readOnly: false,
    minimap: false,
    lineNumbers: true,
    wordWrap: 'on',
    height: '100%',
    enablePerformanceOptimization: true,
  }
)

const settingsStore = useSettingsStore()

const isDark = computed(() => {
  const theme = settingsStore.settings.theme
  if (theme === 'dark') return true
  if (theme === 'light') return false
  return window.matchMedia('(prefers-color-scheme: dark)').matches
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const containerRef = ref<HTMLElement | null>(null)
let editor: monaco.editor.IStandaloneCodeEditor | null = null
let resizeObserver: ResizeObserver | null = null
let resizeTimer: ReturnType<typeof setTimeout> | null = null
let pendingLayout = false

function initEditor() {
  if (!containerRef.value || editor) return

  editor = monaco.editor.create(containerRef.value, {
    value: props.modelValue,
    language: props.language,
    readOnly: props.readOnly,
    minimap: { enabled: props.minimap },
    lineNumbers: props.lineNumbers ? 'on' : 'off',
    lineNumbersMinChars: 3,
    lineDecorationsWidth: 20,
    wordWrap: props.wordWrap,
    fontSize: 13,
    tabSize: 2,
    automaticLayout: false,
    scrollBeyondLastLine: false,
    folding: true,
    foldingStrategy: 'auto',
    showFoldingControls: 'always',
    unfoldOnClickAfterEndOfLine: false,
    stickyScroll: { enabled: false },
    renderWhitespace: 'selection',
    formatOnPaste: true,
    formatOnType: true,
    theme: isDark.value ? 'vs-dark' : 'vs',
    padding: { top: 8, bottom: 8 },
    scrollbar: {
      verticalScrollbarSize: 8,
      horizontalScrollbarSize: 8,
    },
    contextmenu: false,
  })

  editor.onDidChangeModelContent(() => {
    const value = editor?.getValue() || ''
    emit('update:modelValue', value)
  })

  resizeObserver = new ResizeObserver((entries) => {
    try {
      if (pendingLayout) return
      pendingLayout = true

      const entry = entries[0]
      if (!entry || entry.contentRect.width === 0 || entry.contentRect.height === 0) {
        pendingLayout = false
        setTimeout(() => {
          if (editor && containerRef.value) {
            const rect = containerRef.value.getBoundingClientRect()
            if (rect.width > 0 && rect.height > 0) {
              requestAnimationFrame(() => {
                nextTick(() => {
                  try {
                    editor?.layout()
                  } catch (e) {
                    console.warn('Monaco delayed layout error:', e)
                  }
                })
              })
            }
          }
        }, 150)
        return
      }

      if (resizeTimer) clearTimeout(resizeTimer)
      resizeTimer = setTimeout(() => {
        pendingLayout = false
        requestAnimationFrame(() => {
          nextTick(() => {
            try {
              editor?.layout()
            } catch (e) {
              console.warn('Monaco layout error:', e)
            }
          })
        })
      }, 100)
    } catch (e) {
      pendingLayout = false
      console.warn('ResizeObserver error:', e)
    }
  })

  if (containerRef.value) {
    resizeObserver.observe(containerRef.value)
  }
}

function disposeEditor() {
  if (resizeTimer) clearTimeout(resizeTimer)
  resizeTimer = null
  resizeObserver?.disconnect()
  resizeObserver = null
  editor?.dispose()
  editor = null
}

onMounted(() => {
  initEditor()
})

onUnmounted(() => {
  disposeEditor()
})

watch(
  () => props.modelValue,
  (newValue) => {
    if (editor && editor.getValue() !== newValue) {
      editor.setValue(newValue)
    }
  }
)

watch(
  () => props.language,
  (newLang) => {
    if (editor) {
      const model = editor.getModel()
      if (model) {
        monaco.editor.setModelLanguage(model, newLang)
      }
    }
  }
)

watch(isDark, (dark) => {
  if (editor) {
    monaco.editor.setTheme(dark ? 'vs-dark' : 'vs')
  }
})

function setTheme(isDark: boolean) {
  monaco.editor.setTheme(isDark ? 'vs-dark' : 'vs')
}

function format() {
  if (editor && props.language === 'json') {
    try {
      const value = editor.getValue()
      const parsed = JSON.parse(value)
      editor.setValue(JSON.stringify(parsed, null, 2))
    } catch (e) {
      console.warn('格式化失败:', e)
    }
  }
}

defineExpose({
  setTheme,
  format,
  getValue: () => editor?.getValue() || '',
  setValue: (value: string) => editor?.setValue(value),
  layout: () => {
    if (editor && containerRef.value) {
      const rect = containerRef.value.getBoundingClientRect()
      if (rect.width > 0 && rect.height > 0) {
        editor.layout()
      }
    }
  },
  forceLayout: () => {
    if (editor) {
      editor.layout()
    }
  },
})
</script>

<template>
  <div ref="containerRef" class="monaco-editor-container" :style="{ height }" />
</template>

<style>
.monaco-editor-container {
  width: 100%;
  height: 100%;
  overflow: hidden;
  position: relative;
  isolation: isolate;
}

.monaco-editor-container > .monaco-editor,
.monaco-editor-container .monaco-editor {
  position: absolute !important;
  top: 0 !important;
  left: 0 !important;
  right: 0 !important;
  bottom: 0 !important;
}

.monaco-editor-container .overflow-guard {
  overflow: hidden !important;
}

.monaco-editor-container .monaco-scrollable-element {
  overflow: hidden !important;
}

.monaco-editor-container .monaco-editor .editor-scrollable {
  overflow: hidden !important;
}
</style>
