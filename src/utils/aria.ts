/**
 * ARIA 工具函数
 * 提供无障碍相关的辅助函数和常量
 */

/**
 * ARIA 角色类型
 */
export type AriaRole =
  | 'button'
  | 'link'
  | 'checkbox'
  | 'radio'
  | 'tab'
  | 'tabpanel'
  | 'menu'
  | 'menuitem'
  | 'dialog'
  | 'alert'
  | 'status'
  | 'progressbar'
  | 'tree'
  | 'treeitem'
  | 'grid'
  | 'listbox'
  | 'option'
  | 'switch'
  | 'textbox'
  | 'searchbox'
  | 'spinbutton'
  | 'slider'
  | 'separator'
  | 'navigation'
  | 'main'
  | 'complementary'
  | 'contentinfo'
  | 'banner'
  | 'region'
  | 'form'
  | 'search'
  | 'application'

/**
 * ARIA 属性接口
 */
export interface AriaAttributes {
  'aria-label'?: string
  'aria-labelledby'?: string
  'aria-describedby'?: string
  'aria-hidden'?: boolean
  'aria-expanded'?: boolean
  'aria-selected'?: boolean
  'aria-checked'?: boolean
  'aria-pressed'?: boolean
  'aria-disabled'?: boolean
  'aria-readonly'?: boolean
  'aria-required'?: boolean
  'aria-invalid'?: boolean
  'aria-busy'?: boolean
  'aria-live'?: 'off' | 'polite' | 'assertive'
  'aria-atomic'?: boolean
  'aria-relevant'?: string
  'aria-current'?: boolean | 'page' | 'step' | 'location' | 'date' | 'time'
  'aria-controls'?: string
  'aria-owns'?: string
  'aria-haspopup'?: boolean | 'menu' | 'listbox' | 'tree' | 'grid' | 'dialog'
  'aria-modal'?: boolean
  'aria-multiselectable'?: boolean
  'aria-orientation'?: 'horizontal' | 'vertical'
  'aria-valuemin'?: number
  'aria-valuemax'?: number
  'aria-valuenow'?: number
  'aria-valuetext'?: string
  'aria-activedescendant'?: string
  'aria-autocomplete'?: 'none' | 'inline' | 'list' | 'both'
  'aria-placeholder'?: string
  // HTML 属性（非 ARIA，但常一起使用）
  role?: string
  id?: string
  tabIndex?: number
}

/**
 * 生成 ARIA 标签
 */
export function generateAriaLabel(label: string, context?: string): string {
  return context ? `${label}, ${context}` : label
}

/**
 * 为请求方法生成可访问的标签
 */
export function getMethodAriaLabel(method: string): string {
  const methodLabels: Record<string, string> = {
    GET: '获取请求',
    POST: '创建请求',
    PUT: '更新请求',
    DELETE: '删除请求',
    PATCH: '部分更新请求',
    OPTIONS: '选项请求',
    HEAD: '头部请求',
  }
  return methodLabels[method.toUpperCase()] || `${method} 请求`
}

/**
 * 为响应状态码生成可访问的标签
 */
export function getStatusAriaLabel(statusCode: number, statusText: string): string {
  const category =
    statusCode >= 200 && statusCode < 300
      ? '成功'
      : statusCode >= 300 && statusCode < 400
        ? '重定向'
        : statusCode >= 400 && statusCode < 500
          ? '客户端错误'
          : '服务器错误'
  return `${statusCode} ${statusText}, ${category}`
}

/**
 * 为表单字段生成 ARIA 描述
 */
export function getFieldAriaDescription(
  fieldName: string,
  required = false,
  hasError = false
): string {
  const parts = [fieldName]
  if (required) {
    parts.push('必填')
  }
  if (hasError) {
    parts.push('有错误')
  }
  return parts.join(', ')
}

/**
 * 为 Tab 组件生成 ARIA 属性
 */
export function getTabAriaProps(isSelected: boolean, tabId: string, panelId: string): AriaAttributes {
  return {
    role: 'tab',
    'aria-selected': isSelected,
    'aria-controls': panelId,
    id: tabId,
    tabIndex: isSelected ? 0 : -1,
  }
}

/**
 * 为 TabPanel 组件生成 ARIA 属性
 */
export function getTabPanelAriaProps(
  tabId: string,
  panelId: string
): AriaAttributes & { role: string; id: string } {
  return {
    role: 'tabpanel',
    'aria-labelledby': tabId,
    id: panelId,
    tabIndex: 0,
  }
}

/**
 * 为按钮生成 ARIA 属性
 */
export function getButtonAriaProps(
  label: string,
  disabled = false,
  pressed?: boolean,
  expanded?: boolean
): AriaAttributes & { role?: string } {
  const props: AriaAttributes & { role?: string } = {
    'aria-label': label,
    'aria-disabled': disabled,
  }

  if (pressed !== undefined) {
    props['aria-pressed'] = pressed
  }

  if (expanded !== undefined) {
    props['aria-expanded'] = expanded
  }

  return props
}

/**
 * 为输入框生成 ARIA 属性
 */
export function getInputAriaProps(
  label: string,
  required = false,
  invalid = false,
  describedBy?: string
): AriaAttributes {
  return {
    'aria-label': label,
    'aria-required': required,
    'aria-invalid': invalid,
    'aria-describedby': describedBy,
  }
}

/**
 * 为进度条生成 ARIA 属性
 */
export function getProgressAriaProps(
  value: number,
  min: number,
  max: number,
  label?: string
): AriaAttributes & { role: string } {
  return {
    role: 'progressbar',
    'aria-label': label || '进度',
    'aria-valuemin': min,
    'aria-valuemax': max,
    'aria-valuenow': value,
    'aria-busy': value < max && value > min,
  }
}

/**
 * 为菜单项生成 ARIA 属性
 */
export function getMenuItemAriaProps(
  label: string,
  disabled = false,
  hasSubmenu = false
): AriaAttributes & { role: string } {
  return {
    role: 'menuitem',
    'aria-label': label,
    'aria-disabled': disabled,
    'aria-haspopup': hasSubmenu,
  }
}

/**
 * 为列表项生成 ARIA 属性
 */
export function getListOptionAriaProps(
  label: string,
  selected = false,
  disabled = false
): AriaAttributes & { role: string } {
  return {
    role: 'option',
    'aria-label': label,
    'aria-selected': selected,
    'aria-disabled': disabled,
  }
}

/**
 * 为树形节点生成 ARIA 属性
 */
export function getTreeItemAriaProps(
  label: string,
  expanded = false,
  selected = false,
  hasChildren = false
): AriaAttributes & { role: string } {
  return {
    role: 'treeitem',
    'aria-label': label,
    'aria-expanded': hasChildren ? expanded : undefined,
    'aria-selected': selected,
  }
}

/**
 * 为对话框生成 ARIA 属性
 */
export function getDialogAriaProps(
  titleId: string,
  descriptionId?: string,
  modal = true
): AriaAttributes & { role: string } {
  return {
    role: 'dialog',
    'aria-labelledby': titleId,
    'aria-describedby': descriptionId,
    'aria-modal': modal,
  }
}

/**
 * 为通知/警报生成 ARIA live 属性
 */
export function getLiveRegionAriaProps(
  priority: 'polite' | 'assertive' = 'polite',
  atomic = true
): AriaAttributes & { role?: string } {
  return {
    role: 'status',
    'aria-live': priority,
    'aria-atomic': atomic,
  }
}

/**
 * 为工具提示生成 ARIA 属性
 */
export function getTooltipAriaProps(targetId: string): AriaAttributes {
  return {
    role: 'tooltip',
    'aria-describedby': targetId,
  }
}

/**
 * 为搜索框生成 ARIA 属性
 */
export function getSearchBoxAriaProps(
  label: string,
  placeholder?: string
): AriaAttributes & { role: string } {
  return {
    role: 'searchbox',
    'aria-label': label,
    'aria-placeholder': placeholder,
  }
}

/**
 * 为组合框生成 ARIA 属性
 */
export function getComboboxAriaProps(
  label: string,
  expanded = false,
  controlsId?: string
): AriaAttributes & { role: string } {
  return {
    role: 'combobox',
    'aria-label': label,
    'aria-expanded': expanded,
    'aria-controls': controlsId,
    'aria-autocomplete': 'list',
  }
}
