/**
 * 主题工具函数
 * 提供主题颜色相关的辅助函数
 */

/**
 * 获取 CSS 变量值
 */
export function getCssVariable(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim()
}

/**
 * 设置 CSS 变量值
 */
export function setCssVariable(name: string, value: string): void {
  document.documentElement.style.setProperty(name, value)
}

/**
 * 获取请求方法的颜色
 */
export function getMethodColor(method: string): { color: string; bg: string } {
  const methodUpper = method.toUpperCase()
  const colorVar = getCssVariable(`--color-method-${methodUpper.toLowerCase()}`)
  const bgVar = getCssVariable(`--color-method-${methodUpper.toLowerCase()}-bg`)

  if (colorVar && bgVar) {
    return { color: colorVar, bg: bgVar }
  }

  // 默认返回 GET 的颜色
  return {
    color: getCssVariable('--color-method-get') || '#10b981',
    bg: getCssVariable('--color-method-get-bg') || 'rgba(16, 185, 129, 0.1)',
  }
}

/**
 * 根据状态码获取状态颜色
 */
export function getStatusColor(statusCode: number): { color: string; bg: string } {
  if (statusCode >= 200 && statusCode < 300) {
    return {
      color: getCssVariable('--color-status-success') || '#10b981',
      bg: getCssVariable('--color-status-success-bg') || 'rgba(16, 185, 129, 0.1)',
    }
  } else if (statusCode >= 300 && statusCode < 400) {
    return {
      color: getCssVariable('--color-status-redirect') || '#3b82f6',
      bg: getCssVariable('--color-status-redirect-bg') || 'rgba(59, 130, 246, 0.1)',
    }
  } else if (statusCode >= 400 && statusCode < 500) {
    return {
      color: getCssVariable('--color-status-client-error') || '#f59e0b',
      bg: getCssVariable('--color-status-client-error-bg') || 'rgba(245, 158, 11, 0.1)',
    }
  } else {
    return {
      color: getCssVariable('--color-status-server-error') || '#ef4444',
      bg: getCssVariable('--color-status-server-error-bg') || 'rgba(239, 68, 68, 0.1)',
    }
  }
}

/**
 * 获取功能色
 */
export function getFunctionalColor(
  type: 'primary' | 'success' | 'warning' | 'error' | 'info',
  shade: number = 500
): string {
  const varName = `--color-${type}-${shade}`
  return getCssVariable(varName) || getCssVariable(`--color-${type}-500`) || '#3b82f6'
}

/**
 * 获取中性色
 */
export function getNeutralColor(type: 'text' | 'bg' | 'border', variant: string = 'primary'): string {
  const varName = `--color-${type}-${variant}`
  return getCssVariable(varName) || '#6b7280'
}

/**
 * 判断当前是否为暗色主题
 */
export function isDarkTheme(): boolean {
  return document.documentElement.getAttribute('data-theme') === 'dark'
}

/**
 * 获取当前主题
 */
export function getCurrentTheme(): 'light' | 'dark' {
  return isDarkTheme() ? 'dark' : 'light'
}
