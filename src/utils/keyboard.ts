/**
 * 键盘导航工具函数
 * 提供通用的键盘导航功能
 */

/**
 * 焦点管理工具类
 */
export class FocusManager {
  private focusableSelectors = [
    'button:not([disabled])',
    'input:not([disabled])',
    'select:not([disabled])',
    'textarea:not([disabled])',
    'a[href]',
    '[tabindex]:not([tabindex="-1"])',
    '[contenteditable]',
  ].join(', ')

  /**
   * 获取容器内所有可聚焦元素
   */
  getFocusableElements(container: HTMLElement): HTMLElement[] {
    const elements = container.querySelectorAll(this.focusableSelectors)
    return Array.from(elements) as HTMLElement[]
  }

  /**
   * 聚焦第一个可聚焦元素
   */
  focusFirst(container: HTMLElement): void {
    const elements = this.getFocusableElements(container)
    if (elements.length > 0) {
      elements[0].focus()
    }
  }

  /**
   * 聚焦最后一个可聚焦元素
   */
  focusLast(container: HTMLElement): void {
    const elements = this.getFocusableElements(container)
    if (elements.length > 0) {
      elements[elements.length - 1].focus()
    }
  }

  /**
   * 在容器内处理 Tab 键循环聚焦
   */
  handleTabLoop(container: HTMLElement, event: KeyboardEvent): void {
    if (event.key !== 'Tab') return

    const elements = this.getFocusableElements(container)
    if (elements.length === 0) return

    const firstElement = elements[0]
    const lastElement = elements[elements.length - 1]
    const currentElement = document.activeElement as HTMLElement

    if (event.shiftKey) {
      // Shift + Tab: 反向循环
      if (currentElement === firstElement || !container.contains(currentElement)) {
        event.preventDefault()
        lastElement.focus()
      }
    } else {
      // Tab: 正向循环
      if (currentElement === lastElement || !container.contains(currentElement)) {
        event.preventDefault()
        firstElement.focus()
      }
    }
  }

  /**
   * 处理 Escape 键关闭
   */
  handleEscape(onEscape: () => void): (event: KeyboardEvent) => void {
    return (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault()
        onEscape()
      }
    }
  }

  /**
   * 处理箭头键导航
   */
  handleArrowKeys(
    items: HTMLElement[],
    options: {
      orientation?: 'horizontal' | 'vertical'
      loop?: boolean
      onSelect?: (index: number) => void
    } = {}
  ): (event: KeyboardEvent) => void {
    const { orientation = 'vertical', loop = true, onSelect } = options
    let currentIndex = -1

    return (event: KeyboardEvent) => {
      const isHorizontal = orientation === 'horizontal'
      const prevKey = isHorizontal ? 'ArrowLeft' : 'ArrowUp'
      const nextKey = isHorizontal ? 'ArrowRight' : 'ArrowDown'

      if (![prevKey, nextKey, 'Enter', ' '].includes(event.key)) return

      // 查找当前聚焦的索引
      const focusedIndex = items.findIndex((item) => item.contains(document.activeElement))
      if (focusedIndex >= 0) {
        currentIndex = focusedIndex
      }

      switch (event.key) {
        case prevKey:
          event.preventDefault()
          if (currentIndex <= 0) {
            if (loop) {
              currentIndex = items.length - 1
            }
          } else {
            currentIndex--
          }
          items[currentIndex]?.focus()
          break

        case nextKey:
          event.preventDefault()
          if (currentIndex >= items.length - 1) {
            if (loop) {
              currentIndex = 0
            }
          } else {
            currentIndex++
          }
          items[currentIndex]?.focus()
          break

        case 'Enter':
        case ' ':
          if (currentIndex >= 0 && onSelect) {
            event.preventDefault()
            onSelect(currentIndex)
          }
          break
      }
    }
  }

  /**
   * 保存当前焦点位置
   */
  saveFocus(): HTMLElement | null {
    return document.activeElement as HTMLElement
  }

  /**
   * 恢复之前保存的焦点位置
   */
  restoreFocus(element: HTMLElement | null): void {
    if (element && typeof element.focus === 'function') {
      element.focus()
    }
  }

  /**
   * 将焦点限制在容器内（用于模态框）
   */
  trapFocus(container: HTMLElement): () => void {
    const handleKeyDown = (event: KeyboardEvent) => {
      this.handleTabLoop(container, event)
    }

    container.addEventListener('keydown', handleKeyDown)

    // 自动聚焦第一个元素
    this.focusFirst(container)

    // 返回清理函数
    return () => {
      container.removeEventListener('keydown', handleKeyDown)
    }
  }
}

/**
 * 创建全局焦点管理器实例
 */
export const focusManager = new FocusManager()

/**
 * 为列表项添加键盘导航支持
 */
export function useListNavigation<T extends HTMLElement>(
  items: T[],
  options: {
    orientation?: 'horizontal' | 'vertical'
    loop?: boolean
    activeIndex?: number
    onChange?: (index: number) => void
    onSelect?: (index: number) => void
  } = {}
) {
  const { orientation = 'vertical', loop = true, onChange, onSelect } = options
  let internalActiveIndex = options.activeIndex ?? -1

  function handleKeydown(event: KeyboardEvent) {
    const isHorizontal = orientation === 'horizontal'
    const prevKey = isHorizontal ? 'ArrowLeft' : 'ArrowUp'
    const nextKey = isHorizontal ? 'ArrowRight' : 'ArrowDown'

    if (![prevKey, nextKey, 'Enter', ' ', 'Home', 'End'].includes(event.key)) return

    event.preventDefault()

    switch (event.key) {
      case prevKey:
        if (internalActiveIndex <= 0) {
          if (loop) {
            internalActiveIndex = items.length - 1
          }
        } else {
          internalActiveIndex--
        }
        break

      case nextKey:
        if (internalActiveIndex >= items.length - 1) {
          if (loop) {
            internalActiveIndex = 0
          }
        } else {
          internalActiveIndex++
        }
        break

      case 'Home':
        internalActiveIndex = 0
        break

      case 'End':
        internalActiveIndex = items.length - 1
        break

      case 'Enter':
      case ' ':
        if (internalActiveIndex >= 0 && onSelect) {
          onSelect(internalActiveIndex)
        }
        return
    }

    // 更新聚焦状态
    if (internalActiveIndex >= 0 && internalActiveIndex < items.length) {
      items[internalActiveIndex]?.focus()
      onChange?.(internalActiveIndex)
    }
  }

  return {
    handleKeydown,
    getActiveIndex: () => internalActiveIndex,
  }
}

/**
 * 为菜单添加键盘导航支持
 */
export function useMenuNavigation(
  menuItems: HTMLElement[],
  options: {
    loop?: boolean
    onSelect?: (index: number) => void
    onClose?: () => void
  } = {}
) {
  const { onSelect, onClose } = options
  let activeIndex = 0

  function handleKeydown(event: KeyboardEvent) {
    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault()
        activeIndex = Math.min(activeIndex + 1, menuItems.length - 1)
        menuItems[activeIndex]?.focus()
        break

      case 'ArrowUp':
        event.preventDefault()
        activeIndex = Math.max(activeIndex - 1, 0)
        menuItems[activeIndex]?.focus()
        break

      case 'Home':
        event.preventDefault()
        activeIndex = 0
        menuItems[0]?.focus()
        break

      case 'End':
        event.preventDefault()
        activeIndex = menuItems.length - 1
        menuItems[menuItems.length - 1]?.focus()
        break

      case 'Enter':
      case ' ':
        event.preventDefault()
        if (activeIndex >= 0 && onSelect) {
          onSelect(activeIndex)
        }
        break

      case 'Escape':
        event.preventDefault()
        onClose?.()
        break
    }
  }

  return {
    handleKeydown,
    setActiveIndex: (index: number) => {
      activeIndex = index
      menuItems[index]?.focus()
    },
    getActiveIndex: () => activeIndex,
  }
}

/**
 * 检查元素是否可聚焦
 */
export function isFocusable(element: HTMLElement): boolean {
  if (element.tabIndex < 0) return false
  
  const focusableSelectors = [
    'button:not([disabled])',
    'input:not([disabled])',
    'select:not([disabled])',
    'textarea:not([disabled])',
    'a[href]',
    '[tabindex]',
    '[contenteditable]',
  ]
  
  return focusableSelectors.some((selector) => element.matches(selector))
}

/**
 * 获取元素的无障碍角色
 */
export function getRole(element: HTMLElement): string | null {
  return element.getAttribute('role')
}

/**
 * 设置元素的无障碍属性
 */
export function setAriaAttribute(
  element: HTMLElement,
  attribute: string,
  value: string | boolean | number
): void {
  const ariaAttr = attribute.startsWith('aria-') ? attribute : `aria-${attribute}`
  element.setAttribute(ariaAttr, String(value))
}
