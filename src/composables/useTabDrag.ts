import { ref } from 'vue'

interface DragState {
  dragging: boolean
  tabId: string | null
  startX: number
  startIndex: number
  currentIndex: number
  tabWidth: number
  offsetX: number
}

export function useTabDrag(moveTab: (from: number, to: number) => void) {
  const dragState = ref<DragState>({
    dragging: false,
    tabId: null,
    startX: 0,
    startIndex: 0,
    currentIndex: 0,
    tabWidth: 0,
    offsetX: 0,
  })

  function handleDragStart(e: MouseEvent, tabId: string, tabs: Array<{ id: string }>) {
    if (e.button !== 0) return

    const index = tabs.findIndex((t) => t.id === tabId)
    if (index === -1) return

    const tabElement = e.currentTarget as HTMLElement
    const rect = tabElement.getBoundingClientRect()

    dragState.value = {
      dragging: true,
      tabId,
      startX: e.clientX,
      startIndex: index,
      currentIndex: index,
      tabWidth: rect.width,
      offsetX: 0,
    }

    document.addEventListener('mousemove', handleDragMove)
    document.addEventListener('mouseup', handleDragEnd)
  }

  function handleDragMove(e: MouseEvent) {
    if (!dragState.value.dragging) return

    const tabsContainer = document.querySelector('.tabs-scroll')
    if (!tabsContainer) return

    const tabElements = Array.from(tabsContainer.querySelectorAll('.tab-item')) as HTMLElement[]
    const mouseX = e.clientX

    dragState.value.offsetX = mouseX - dragState.value.startX

    let newIndex = dragState.value.startIndex
    for (let i = 0; i < tabElements.length; i++) {
      if (tabElements[i].dataset.tabId === dragState.value.tabId) continue
      const rect = tabElements[i].getBoundingClientRect()
      const midpoint = rect.left + rect.width / 2
      if (mouseX < midpoint) {
        newIndex = i > dragState.value.startIndex ? i - 1 : i
        break
      }
      if (i === tabElements.length - 1) {
        newIndex = i
      }
    }

    if (newIndex !== dragState.value.currentIndex) {
      dragState.value.currentIndex = newIndex
    }
  }

  function handleDragEnd() {
    if (!dragState.value.dragging) return

    if (dragState.value.startIndex !== dragState.value.currentIndex) {
      moveTab(dragState.value.startIndex, dragState.value.currentIndex)
    }

    dragState.value = {
      dragging: false,
      tabId: null,
      startX: 0,
      startIndex: 0,
      currentIndex: 0,
      tabWidth: 0,
      offsetX: 0,
    }

    document.removeEventListener('mousemove', handleDragMove)
    document.removeEventListener('mouseup', handleDragEnd)
  }

  function cleanup() {
    document.removeEventListener('mousemove', handleDragMove)
    document.removeEventListener('mouseup', handleDragEnd)
  }

  return {
    dragState,
    handleDragStart,
    cleanup,
  }
}
