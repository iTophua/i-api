import { onMounted, onUnmounted } from 'vue'
import { useSettingsStore } from '@/stores'
import type { ShortcutBinding } from '@/types'

export interface ShortcutConfig {
  id: string
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
  const settingsStore = useSettingsStore()

  function getEffectiveShortcut(shortcut: ShortcutConfig): ShortcutBinding | null {
    const customBindings = settingsStore.settings.shortcuts
    if (customBindings && shortcut.id in customBindings) {
      return customBindings[shortcut.id]
    }
    return {
      key: shortcut.key,
      ctrl: shortcut.ctrl,
      shift: shortcut.shift,
      alt: shortcut.alt,
      meta: shortcut.meta,
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    const target = e.target as HTMLElement
    const isInput = ['INPUT', 'TEXTAREA'].includes(target.tagName) || target.isContentEditable

    for (const shortcut of shortcuts) {
      if (isInput && shortcut.category === 'navigation') {
        continue
      }

      const binding = getEffectiveShortcut(shortcut)
      if (!binding) continue

      const ctrlMatch = binding.ctrl ? (e.ctrlKey || e.metaKey) : !e.ctrlKey && !e.metaKey
      const shiftMatch = binding.shift ? e.shiftKey : !e.shiftKey
      const altMatch = binding.alt ? e.altKey : !e.altKey
      const keyMatch = e.key.toLowerCase() === binding.key.toLowerCase()

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
    id: 'sendRequest',
    key: 'Enter',
    ctrl: true,
    handler: () => {},
    description: '发送请求',
    category: 'global',
  },
  {
    id: 'saveRequest',
    key: 's',
    ctrl: true,
    handler: () => {},
    description: '保存请求',
    category: 'global',
  },
  {
    id: 'newRequest',
    key: 'n',
    ctrl: true,
    handler: () => {},
    description: '新建请求',
    category: 'global',
  },
  {
    id: 'closeTab',
    key: 'w',
    ctrl: true,
    handler: () => {},
    description: '关闭当前标签页',
    category: 'global',
  },
  {
    id: 'importCurl',
    key: 'i',
    ctrl: true,
    shift: true,
    handler: () => {},
    description: '导入 cURL',
    category: 'global',
  },
  {
    id: 'openEnvironment',
    key: 'e',
    ctrl: true,
    shift: true,
    handler: () => {},
    description: '打开环境管理',
    category: 'global',
  },
  {
    id: 'openHistory',
    key: 'h',
    ctrl: true,
    shift: true,
    handler: () => {},
    description: '打开历史记录',
    category: 'global',
  },
  {
    id: 'openSettings',
    key: ',',
    ctrl: true,
    handler: () => {},
    description: '打开设置',
    category: 'global',
  },
  
  // ===== 导航快捷键 =====
  {
    id: 'nextTab',
    key: 'ArrowRight',
    ctrl: true,
    handler: () => {},
    description: '切换到下一个标签页',
    category: 'navigation',
  },
  {
    id: 'prevTab',
    key: 'ArrowLeft',
    ctrl: true,
    handler: () => {},
    description: '切换到上一个标签页',
    category: 'navigation',
  },
  
  // ===== 编辑器快捷键 =====
]

export function formatShortcut(shortcut: ShortcutBinding): string {
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