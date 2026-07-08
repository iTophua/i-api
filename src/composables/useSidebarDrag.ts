import { ref } from 'vue'
import type { HttpMethod } from '@/types'

const DRAG_THRESHOLD = 5

interface SidebarDragState {
  dragging: boolean
  type: 'collection' | 'request' | null
  dragId: string | null
  dragName: string | null
  dragExpanded: boolean
  dragMethod?: HttpMethod
  sourceIndex: number
  currentIndex: number
  elementHeight: number
  sourceCollectionId?: string
  /** 拖拽请求的来源文件夹（拖入文件夹功能用） */
  sourceFolderId?: string
  /** 拖拽过程中悬停的目标文件夹（命中时高亮） */
  dropFolderId?: string
  mouseX: number
  mouseY: number
  sourceRect?: DOMRect
  dragStarted: boolean
}

export function useSidebarDrag(
  _getCollections: () => { id: string; name: string; requests: { id: string; name: string }[] }[],
  reorderCollection: (from: number, to: number) => void,
  reorderRequest: (collectionId: string, from: number, to: number) => void,
  /** 拖请求移入文件夹（跨容器移动） */
  moveRequestToFolder?: (
    requestId: string,
    fromCollectionId: string,
    fromFolderId: string | undefined,
    toCollectionId: string,
    toFolderId: string
  ) => void
) {
  const dragState = ref<SidebarDragState>({
    dragging: false,
    type: null,
    dragId: null,
    dragName: null,
    dragExpanded: false,
    sourceIndex: 0,
    currentIndex: 0,
    elementHeight: 0,
    sourceCollectionId: undefined,
    mouseX: 0,
    mouseY: 0,
    dragStarted: false,
  })

  let startX = 0
  let startY = 0

  function handleCollectionDragStart(e: MouseEvent, collection: { id: string; name: string }, collectionIndex: number, expanded?: boolean) {
    if (e.button !== 0) return

    startX = e.clientX
    startY = e.clientY

    const element = e.currentTarget as HTMLElement
    const rect = element.getBoundingClientRect()

    dragState.value = {
      dragging: true,
      type: 'collection',
      dragId: collection.id,
      dragName: collection.name,
      dragExpanded: expanded || false,
      sourceIndex: collectionIndex,
      currentIndex: collectionIndex,
      elementHeight: rect.height,
      mouseX: e.clientX,
      mouseY: e.clientY,
      sourceRect: rect,
      dragStarted: false,
    }

    document.addEventListener('mousemove', handleCollectionDragMove)
    document.addEventListener('mouseup', handleCollectionDragEnd)
  }

  function handleCollectionDragMove(e: MouseEvent) {
    if (!dragState.value.dragging || dragState.value.type !== 'collection') return

    const dx = Math.abs(e.clientX - startX)
    const dy = Math.abs(e.clientY - startY)

    if (!dragState.value.dragStarted && (dx > DRAG_THRESHOLD || dy > DRAG_THRESHOLD)) {
      dragState.value.dragStarted = true
    }

    if (!dragState.value.dragStarted) return

    dragState.value.mouseX = e.clientX
    dragState.value.mouseY = e.clientY

    const items = document.querySelectorAll('.collection-item[data-collection-id]')
    if (items.length === 0) return

    const firstRect = items[0].getBoundingClientRect()
    const mouseY = e.clientY
    const newIndex = Math.floor((mouseY - firstRect.top) / dragState.value.elementHeight)
    dragState.value.currentIndex = Math.max(0, Math.min(newIndex, items.length - 1))
  }

  function handleCollectionDragEnd() {
    document.removeEventListener('mousemove', handleCollectionDragMove)
    document.removeEventListener('mouseup', handleCollectionDragEnd)

    if (!dragState.value.dragging || dragState.value.type !== 'collection') return

    const { sourceIndex, currentIndex, dragStarted } = dragState.value

    if (dragStarted && sourceIndex !== currentIndex) {
      reorderCollection(sourceIndex, currentIndex)
    }

    dragState.value = {
      dragging: false,
      type: null,
      dragId: null,
      dragName: null,
      dragExpanded: false,
      dragMethod: undefined,
      sourceIndex: 0,
      currentIndex: 0,
      elementHeight: 0,
      sourceCollectionId: undefined,
      mouseX: 0,
      mouseY: 0,
      dragStarted: false,
    }
  }

  function handleRequestDragStart(
    e: MouseEvent,
    request: { id: string; name: string; method?: HttpMethod },
    collectionId: string,
    requestIndex: number,
    sourceFolderId?: string
  ) {
    if (e.button !== 0) return

    startX = e.clientX
    startY = e.clientY

    const element = e.currentTarget as HTMLElement
    const rect = element.getBoundingClientRect()

    dragState.value = {
      dragging: true,
      type: 'request',
      dragId: request.id,
      dragName: request.name,
      dragExpanded: false,
      dragMethod: request.method,
      sourceIndex: requestIndex,
      currentIndex: requestIndex,
      elementHeight: rect.height,
      sourceCollectionId: collectionId,
      sourceFolderId,
      mouseX: e.clientX,
      mouseY: e.clientY,
      sourceRect: rect,
      dragStarted: false,
    }

    document.addEventListener('mousemove', handleRequestDragMove)
    document.addEventListener('mouseup', handleRequestDragEnd)
  }

  function handleRequestDragMove(e: MouseEvent) {
    if (!dragState.value.dragging || dragState.value.type !== 'request') return

    const dx = Math.abs(e.clientX - startX)
    const dy = Math.abs(e.clientY - startY)

    if (!dragState.value.dragStarted && (dx > DRAG_THRESHOLD || dy > DRAG_THRESHOLD)) {
      dragState.value.dragStarted = true
    }

    if (!dragState.value.dragStarted) return

    dragState.value.mouseX = e.clientX
    dragState.value.mouseY = e.clientY

    // 检测是否悬停在文件夹节点上（用于拖入文件夹）
    const folderEl = (e.target as HTMLElement)?.closest('.folder-drop-target') as HTMLElement | null
    dragState.value.dropFolderId = folderEl?.dataset.folderId

    // 若命中文件夹，跳过排序计算（避免误判 index）
    if (folderEl) return

    const { sourceCollectionId } = dragState.value
    if (!sourceCollectionId) return

    const items = document.querySelectorAll(`.request-item[data-collection-id="${sourceCollectionId}"]:not([data-folder-id])`)
    if (items.length === 0) return

    const firstRect = items[0].getBoundingClientRect()
    const mouseY = e.clientY
    const newIndex = Math.floor((mouseY - firstRect.top) / dragState.value.elementHeight)
    dragState.value.currentIndex = Math.max(0, Math.min(newIndex, items.length - 1))
  }

  function handleRequestDragEnd() {
    document.removeEventListener('mousemove', handleRequestDragMove)
    document.removeEventListener('mouseup', handleRequestDragEnd)

    if (!dragState.value.dragging || dragState.value.type !== 'request') return

    const { dragId, sourceCollectionId, sourceFolderId, sourceIndex, currentIndex, dragStarted, dropFolderId } = dragState.value

    if (!dragId || !sourceCollectionId) return

    if (dragStarted) {
      // 优先处理：拖入文件夹
      if (dropFolderId && moveRequestToFolder && dropFolderId !== sourceFolderId) {
        moveRequestToFolder(dragId, sourceCollectionId, sourceFolderId, sourceCollectionId, dropFolderId)
      } else if (!dropFolderId && sourceIndex !== currentIndex) {
        // 否则：同容器内排序
        reorderRequest(sourceCollectionId, sourceIndex, currentIndex)
      }
    }

    dragState.value = {
      dragging: false,
      type: null,
      dragId: null,
      dragName: null,
      dragExpanded: false,
      dragMethod: undefined,
      sourceIndex: 0,
      currentIndex: 0,
      elementHeight: 0,
      sourceCollectionId: undefined,
      sourceFolderId: undefined,
      dropFolderId: undefined,
      mouseX: 0,
      mouseY: 0,
      dragStarted: false,
    }
  }

  function getItemClass(itemIndex: number, type: 'collection' | 'request') {
    if (!dragState.value.dragging || dragState.value.type !== type) return ''

    const { sourceIndex, currentIndex, dragStarted } = dragState.value

    if (itemIndex === sourceIndex) {
      if (dragStarted) {
        return 'is-dragging'
      }
      return 'is-pressing'
    }

    if (dragStarted) {
      const minIndex = Math.min(sourceIndex, currentIndex)
      const maxIndex = Math.max(sourceIndex, currentIndex)

      if (itemIndex >= minIndex && itemIndex <= maxIndex) {
        return 'is-displaced'
      }
    }

    return ''
  }

  function getItemStyle(itemIndex: number, type: 'collection' | 'request'): Record<string, string> | undefined {
    if (!dragState.value.dragging || !dragState.value.dragStarted || dragState.value.type !== type) return undefined

    const { sourceIndex, currentIndex, elementHeight } = dragState.value
    const style: Record<string, string> = {}

    if (itemIndex === sourceIndex) {
      return undefined
    }

    const minIndex = Math.min(sourceIndex, currentIndex)
    const maxIndex = Math.max(sourceIndex, currentIndex)

    if (itemIndex >= minIndex && itemIndex <= maxIndex) {
      if (currentIndex > sourceIndex) {
        style['transform'] = `translateY(${-(elementHeight)}px)`
      } else {
        style['transform'] = `translateY(${elementHeight}px)`
      }
      style['transition'] = 'transform 0.15s ease'
    }

    return Object.keys(style).length > 0 ? style : undefined
  }

  function getDraggingStyle(): Record<string, string> | undefined {
    if (!dragState.value.dragging || !dragState.value.dragStarted || !dragState.value.sourceRect) return undefined

    const { mouseY, elementHeight, sourceRect } = dragState.value
    if (!sourceRect) return undefined

    const style: Record<string, string> = {
      position: 'fixed',
      left: `${sourceRect.left}px`,
      top: `${mouseY - elementHeight / 2}px`,
      width: `${sourceRect.width}px`,
      height: `${elementHeight}px`,
      opacity: '0.9',
      boxShadow: '0 4px 12px rgba(0, 0, 0, 0.15)',
      zIndex: '9999',
      pointerEvents: 'none',
      background: 'var(--n-color)',
      borderRadius: '4px',
      display: 'flex',
      alignItems: 'center',
      padding: '0 8px',
    }

    return style
  }

  function getDraggingItem() {
    if (!dragState.value.dragging || !dragState.value.dragStarted || !dragState.value.sourceRect) return null
    return dragState.value
  }

  function cleanup() {
    document.removeEventListener('mousemove', handleCollectionDragMove)
    document.removeEventListener('mouseup', handleCollectionDragEnd)
    document.removeEventListener('mousemove', handleRequestDragMove)
    document.removeEventListener('mouseup', handleRequestDragEnd)
  }

  return {
    dragState,
    handleCollectionDragStart,
    handleRequestDragStart,
    getItemClass,
    getItemStyle,
    getDraggingStyle,
    getDraggingItem,
    cleanup,
  }
}
