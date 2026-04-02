import { onMounted, onUnmounted } from 'vue'

export interface ShortcutConfig {
  key: string
  ctrl?: boolean
  shift?: boolean
  alt?: boolean
  meta?: boolean
  handler: (e: KeyboardEvent) => void
  description?: string
  category?: 'global' | 'editor' | 'navigation'
  preventDefault?: boolean
}

export function useShortcuts(shortcuts: ShortcutConfig[]) {
  function handleKeyDown(e: KeyboardEvent) {
    // 忽略在输入框中的某些快捷键
    const target = e.target as HTMLElement
    const isInput = ['INPUT', 'TEXTAREA'].includes(target.tagName) || target.isContentEditable

    for (const shortcut of shortcuts) {
      // 如果是在输入框中，跳过编辑器相关的快捷键
      if (isInput && shortcut.category === 'navigation') {
        continue
      }

      const ctrlMatch = shortcut.ctrl ? (e.ctrlKey || e.metaKey) : !e.ctrlKey && !e.metaKey
      const shiftMatch = shortcut.shift ? e.shiftKey : !e.shiftKey
      const altMatch = shortcut.alt ? e.altKey : !e.altKey
      const keyMatch = e.key.toLowerCase() === shortcut.key.toLowerCase()

      if (ctrlMatch && shiftMatch && altMatch && keyMatch) {
        if (shortcut.preventDefault !== false) {
          e.preventDefault()
        }
        shortcut.handler(e)
        return
      }
    }
  }

  onMounted(() => {
    window.addEventListener('keydown', handleKeyDown)
  })

  onUnmounted(() => {
    window.removeEventListener('keydown', handleKeyDown)
  })
}

export const defaultShortcuts: ShortcutConfig[] = [
  // ===== 全局快捷键 =====
  {
    key: 'Enter',
    ctrl: true,
    handler: () => {},
    description: '发送请求',
    category: 'global',
  },
  {
    key: 's',
    ctrl: true,
    handler: () => {},
    description: '保存请求',
    category: 'global',
  },
  {
    key: 'n',
    ctrl: true,
    handler: () => {},
    description: '新建请求',
    category: 'global',
  },
  {
    key: 'w',
    ctrl: true,
    handler: () => {},
    description: '关闭当前标签页',
    category: 'global',
  },
  {
    key: 't',
    ctrl: true,
    handler: () => {},
    description: '新建标签页',
    category: 'global',
  },
  {
    key: 'Tab',
    ctrl: true,
    handler: () => {},
    description: '切换下一个标签页',
    category: 'global',
  },
  {
    key: 'Tab',
    ctrl: true,
    shift: true,
    handler: () => {},
    description: '切换上一个标签页',
    category: 'global',
  },
  {
    key: 'i',
    ctrl: true,
    shift: true,
    handler: () => {},
    description: '导入 cURL',
    category: 'global',
  },
  {
    key: 'e',
    ctrl: true,
    shift: true,
    handler: () => {},
    description: '打开环境管理',
    category: 'global',
  },
  {
    key: 'h',
    ctrl: true,
    shift: true,
    handler: () => {},
    description: '打开历史记录',
    category: 'global',
  },
  {
    key: 'f',
    ctrl: true,
    handler: () => {},
    description: '搜索',
    category: 'global',
  },
  {
    key: ',',
    ctrl: true,
    handler: () => {},
    description: '打开设置',
    category: 'global',
  },
  {
    key: 'k',
    ctrl: true,
    handler: () => {},
    description: '聚焦搜索框',
    category: 'global',
  },
  {
    key: 'Escape',
    handler: () => {},
    description: '取消/关闭弹窗',
    category: 'global',
  },
  
  // ===== 导航快捷键 =====
  {
    key: '1',
    ctrl: true,
    handler: () => {},
    description: '切换到第 1 个标签页',
    category: 'navigation',
  },
  {
    key: '2',
    ctrl: true,
    handler: () => {},
    description: '切换到第 2 个标签页',
    category: 'navigation',
  },
  {
    key: '3',
    ctrl: true,
    handler: () => {},
    description: '切换到第 3 个标签页',
    category: 'navigation',
  },
  {
    key: '4',
    ctrl: true,
    handler: () => {},
    description: '切换到第 4 个标签页',
    category: 'navigation',
  },
  {
    key: '5',
    ctrl: true,
    handler: () => {},
    description: '切换到第 5 个标签页',
    category: 'navigation',
  },
  {
    key: '9',
    ctrl: true,
    handler: () => {},
    description: '切换到最后一个标签页',
    category: 'navigation',
  },
  {
    key: 'PageDown',
    ctrl: true,
    handler: () => {},
    description: '切换到下一个标签页',
    category: 'navigation',
  },
  {
    key: 'PageUp',
    ctrl: true,
    handler: () => {},
    description: '切换到上一个标签页',
    category: 'navigation',
  },
  
  // ===== 编辑器快捷键 =====
  {
    key: '/',
    ctrl: true,
    handler: () => {},
    description: '切换注释',
    category: 'editor',
  },
  {
    key: 'd',
    ctrl: true,
    handler: () => {},
    description: '查找下一个',
    category: 'editor',
  },
  {
    key: 'd',
    ctrl: true,
    shift: true,
    handler: () => {},
    description: '查找上一个',
    category: 'editor',
  },
  {
    key: 'f',
    ctrl: true,
    handler: () => {},
    description: '查找',
    category: 'editor',
  },
  {
    key: 'h',
    ctrl: true,
    handler: () => {},
    description: '查找并替换',
    category: 'editor',
  },
  {
    key: 'z',
    ctrl: true,
    handler: () => {},
    description: '撤销',
    category: 'editor',
  },
  {
    key: 'z',
    ctrl: true,
    shift: true,
    handler: () => {},
    description: '重做',
    category: 'editor',
  },
  {
    key: 'y',
    ctrl: true,
    handler: () => {},
    description: '重做',
    category: 'editor',
  },
]

export function formatShortcut(shortcut: ShortcutConfig): string {
  const parts: string[] = []
  const isMac = navigator.platform.includes('Mac')
  
  if (shortcut.ctrl) {
    parts.push(isMac ? '⌘' : 'Ctrl')
  }
  if (shortcut.shift) {
    parts.push('Shift')
  }
  if (shortcut.alt) {
    parts.push(isMac ? '⌥' : 'Alt')
  }
  if (shortcut.meta && !shortcut.ctrl) {
    parts.push(isMac ? '⌘' : 'Win')
  }
  
  // 特殊键位处理
  const keyDisplay: Record<string, string> = {
    ' ': 'Space',
    'enter': '↵',
    'escape': 'Esc',
    'delete': 'Del',
    'backspace': '⌫',
    'arrowup': '↑',
    'arrowdown': '↓',
    'arrowleft': '←',
    'arrowright': '→',
    'tab': 'Tab',
    'pagedown': 'PgDn',
    'pageup': 'PgUp',
    'home': 'Home',
    'end': 'End',
  }
  
  const key = shortcut.key.toLowerCase()
  parts.push(keyDisplay[key] || shortcut.key.toUpperCase())
  
  return parts.join(isMac ? '' : ' + ')
}

/**
 * 获取所有快捷键分组
 */
export function getGroupedShortcuts(): Record<string, ShortcutConfig[]> {
  const groups: Record<string, ShortcutConfig[]> = {
    global: [],
    navigation: [],
    editor: [],
  }
  
  defaultShortcuts.forEach((shortcut) => {
    const category = shortcut.category || 'global'
    if (!groups[category]) {
      groups[category] = []
    }
    groups[category].push(shortcut)
  })
  
  return groups
}