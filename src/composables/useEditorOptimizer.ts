import { ref, computed, onUnmounted } from 'vue'
import type { Ref } from 'vue'
import * as monaco from 'monaco-editor'

interface EditorCacheItem {
  editor: monaco.editor.IStandaloneCodeEditor
  lastUsed: number
  model: monaco.editor.ITextModel
}

interface OptimizerOptions {
  maxCacheSize?: number
  cacheTimeout?: number
  lazyLoad?: boolean
}

const DEFAULT_OPTIONS: Required<OptimizerOptions> = {
  maxCacheSize: 10,
  cacheTimeout: 5 * 60 * 1000, // 5 minutes
  lazyLoad: true,
}

export function useEditorOptimizer(options: OptimizerOptions = {}) {
  const opts = { ...DEFAULT_OPTIONS, ...options }

  const editorCache = new Map<string, EditorCacheItem>()
  const modelCache = new Map<string, monaco.editor.ITextModel>()
  const isEditorLoaded = ref(false)
  const currentEditorId = ref<string | null>(null)

  const cacheSize = computed(() => editorCache.size)
  const modelCacheSize = computed(() => modelCache.size)

  // 清理过期的编辑器实例
  function cleanupExpiredEditors() {
    const now = Date.now()
    const toDelete: string[] = []

    editorCache.forEach((item, key) => {
      if (now - item.lastUsed > opts.cacheTimeout) {
        toDelete.push(key)
      }
    })

    toDelete.forEach((key) => {
      const item = editorCache.get(key)
      if (item) {
        item.editor.dispose()
        editorCache.delete(key)
      }
    })

    // 如果缓存超出限制，移除最久未使用的
    while (editorCache.size > opts.maxCacheSize) {
      let oldestKey: string | null = null
      let oldestTime = Infinity

      editorCache.forEach((item, key) => {
        if (item.lastUsed < oldestTime) {
          oldestTime = item.lastUsed
          oldestKey = key
        }
      })

      if (oldestKey) {
        const item = editorCache.get(oldestKey)!
        item.editor.dispose()
        editorCache.delete(oldestKey)
      }
    }
  }

  // 创建或获取缓存的编辑器
  function getOrCreateEditor(
    container: HTMLElement,
    editorId: string,
    editorOptions: monaco.editor.IStandaloneEditorConstructionOptions
  ): monaco.editor.IStandaloneCodeEditor {
    const cached = editorCache.get(editorId)

    if (cached) {
      cached.lastUsed = Date.now()

      // 将编辑器移动到新的容器
      const domNode = cached.editor.getDomNode()
      if (domNode && container.firstChild) {
        container.removeChild(container.firstChild)
      }
      container.appendChild(domNode!)

      currentEditorId.value = editorId
      return cached.editor
    }

    // 创建新编辑器
    const editor = monaco.editor.create(container, editorOptions)

    editorCache.set(editorId, {
      editor,
      lastUsed: Date.now(),
      model: editor.getModel()!,
    })

    currentEditorId.value = editorId

    // 定期清理过期编辑器
    setTimeout(cleanupExpiredEditors, 60000)

    return editor
  }

  // 获取或创建模型
  function getOrCreateModel(
    modelId: string,
    value: string,
    language: string
  ): monaco.editor.ITextModel {
    const cached = modelCache.get(modelId)

    if (cached) {
      return cached
    }

    const model = monaco.editor.createModel(value, language)
    modelCache.set(modelId, model)

    // 限制模型缓存大小
    if (modelCache.size > opts.maxCacheSize * 2) {
      const toDelete = Array.from(modelCache.keys()).slice(0, modelCache.size - opts.maxCacheSize)
      toDelete.forEach((key) => {
        const model = modelCache.get(key)
        if (model) {
          model.dispose()
          modelCache.delete(key)
        }
      })
    }

    return model
  }

  // 释放编辑器（回收到缓存）
  function releaseEditor(editorId: string) {
    const cached = editorCache.get(editorId)
    if (cached) {
      cached.lastUsed = Date.now()

      // 从 DOM 中移除
      const domNode = cached.editor.getDomNode()
      if (domNode && domNode.parentNode) {
        domNode.parentNode.removeChild(domNode)
      }
    }

    if (currentEditorId.value === editorId) {
      currentEditorId.value = null
    }
  }

  // 完全销毁编辑器
  function destroyEditor(editorId: string) {
    const cached = editorCache.get(editorId)
    if (cached) {
      cached.editor.dispose()
      editorCache.delete(editorId)
    }

    if (currentEditorId.value === editorId) {
      currentEditorId.value = null
    }
  }

  // 预加载编辑器
  async function preloadEditor() {
    if (!opts.lazyLoad || isEditorLoaded.value) return

    try {
      await import('monaco-editor')
      isEditorLoaded.value = true
    } catch (error) {
      console.warn('预加载 Monaco 编辑器失败:', error)
    }
  }

  // 清空所有缓存
  function clearCache() {
    editorCache.forEach((item) => {
      item.editor.dispose()
    })
    editorCache.clear()

    modelCache.forEach((model) => {
      model.dispose()
    })
    modelCache.clear()

    currentEditorId.value = null
  }

  // 当组件卸载时自动清理
  function useCleanup() {
    onUnmounted(() => {
      clearCache()
    })
  }

  return {
    // State
    isEditorLoaded,
    currentEditorId,
    cacheSize,
    modelCacheSize,

    // Methods
    getOrCreateEditor,
    getOrCreateModel,
    releaseEditor,
    destroyEditor,
    preloadEditor,
    clearCache,
    cleanupExpiredEditors,
    useCleanup,
  }
}

// 简化的编辑器性能监控
export function useEditorPerformance() {
  const renderTime = ref(0)
  const layoutTime = ref(0)
  const memoryUsage = ref(0)

  const performanceMetrics = computed(() => ({
    avgRenderTime: renderTime.value,
    avgLayoutTime: layoutTime.value,
    memoryUsage: memoryUsage.value,
  }))

  function markRenderStart() {
    if (typeof performance !== 'undefined') {
      performance.mark('editor-render-start')
    }
  }

  function markRenderEnd() {
    if (typeof performance !== 'undefined') {
      performance.mark('editor-render-end')
      const measure = performance.measure(
        'editor-render',
        'editor-render-start',
        'editor-render-end'
      )
      renderTime.value = measure.duration
      performance.clearMarks('editor-render-start')
      performance.clearMarks('editor-render-end')
      performance.clearMeasures('editor-render')
    }
  }

  function markLayoutStart() {
    if (typeof performance !== 'undefined') {
      performance.mark('editor-layout-start')
    }
  }

  function markLayoutEnd() {
    if (typeof performance !== 'undefined') {
      performance.mark('editor-layout-end')
      const measure = performance.measure(
        'editor-layout',
        'editor-layout-start',
        'editor-layout-end'
      )
      layoutTime.value = measure.duration
      performance.clearMarks('editor-layout-start')
      performance.clearMarks('editor-layout-end')
      performance.clearMeasures('editor-layout')
    }
  }

  function updateMemoryUsage() {
    if ('memory' in performance && (performance as any).memory) {
      memoryUsage.value =
        Math.round(((performance as any).memory.usedJSHeapSize / 1048576) * 100) / 100
    }
  }

  return {
    performanceMetrics,
    markRenderStart,
    markRenderEnd,
    markLayoutStart,
    markLayoutEnd,
    updateMemoryUsage,
  }
}

// 虚拟滚动优化（用于大型响应体）
export function useVirtualScroll<T>(
  items: Ref<T[]>,
  options: { itemHeight: number; visibleHeight: number }
) {
  const startIndex = ref(0)
  const endIndex = ref(0)
  const totalHeight = computed(() => items.value.length * options.itemHeight)
  const visibleCount = computed(() => Math.ceil(options.visibleHeight / options.itemHeight))

  const visibleItems = computed(() => {
    return items.value.slice(startIndex.value, endIndex.value + 1)
  })

  function updateVisibleRange(scrollTop: number) {
    const start = Math.floor(scrollTop / options.itemHeight)
    const end = Math.min(start + visibleCount.value + 5, items.value.length - 1)

    startIndex.value = Math.max(0, start - 5)
    endIndex.value = end
  }

  return {
    visibleItems,
    totalHeight,
    startIndex,
    endIndex,
    updateVisibleRange,
  }
}
