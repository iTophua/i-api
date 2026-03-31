import { onMounted, onUnmounted } from 'vue'

export interface ShortcutConfig {
  key: string
  ctrl?: boolean
  shift?: boolean
  alt?: boolean
  meta?: boolean
  handler: (e: KeyboardEvent) => void
  description?: string
}

export function useShortcuts(shortcuts: ShortcutConfig[]) {
  function handleKeyDown(e: KeyboardEvent) {
    for (const shortcut of shortcuts) {
      const ctrlMatch = shortcut.ctrl ? (e.ctrlKey || e.metaKey) : !e.ctrlKey && !e.metaKey
      const shiftMatch = shortcut.shift ? e.shiftKey : !e.shiftKey
      const altMatch = shortcut.alt ? e.altKey : !e.altKey
      const keyMatch = e.key.toLowerCase() === shortcut.key.toLowerCase()

      if (ctrlMatch && shiftMatch && altMatch && keyMatch) {
        e.preventDefault()
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
  {
    key: 'Enter',
    ctrl: true,
    handler: () => {},
    description: '发送请求',
  },
  {
    key: 's',
    ctrl: true,
    handler: () => {},
    description: '保存请求',
  },
  {
    key: 'n',
    ctrl: true,
    handler: () => {},
    description: '新建请求',
  },
  {
    key: 'i',
    ctrl: true,
    shift: true,
    handler: () => {},
    description: '导入 cURL',
  },
  {
    key: 'e',
    ctrl: true,
    shift: true,
    handler: () => {},
    description: '打开环境管理',
  },
  {
    key: 'f',
    ctrl: true,
    handler: () => {},
    description: '搜索',
  },
  {
    key: ',',
    ctrl: true,
    handler: () => {},
    description: '打开设置',
  },
]

export function formatShortcut(shortcut: ShortcutConfig): string {
  const parts: string[] = []
  
  if (shortcut.ctrl) {
    parts.push(navigator.platform.includes('Mac') ? '⌘' : 'Ctrl')
  }
  if (shortcut.shift) {
    parts.push('Shift')
  }
  if (shortcut.alt) {
    parts.push(navigator.platform.includes('Mac') ? '⌥' : 'Alt')
  }
  
  parts.push(shortcut.key.toUpperCase())
  
  return parts.join(' + ')
}