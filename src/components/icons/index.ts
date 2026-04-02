/**
 * 图标导出统一入口
 */

export { default as HttpMethodIcon } from './HttpMethodIcon.vue'
export { default as StatusIcon } from './StatusIcon.vue'
export { default as AppIcon } from './AppIcon.vue'

// 图标类型定义
export type IconType =
  | 'request'
  | 'collection'
  | 'folder'
  | 'environment'
  | 'history'
  | 'settings'
  | 'send'
  | 'save'
  | 'close'
  | 'check'
  | 'plus'
  | 'minus'
  | 'chevronLeft'
  | 'chevronRight'
  | 'chevronDown'
  | 'search'
  | 'trash'
  | 'edit'
  | 'copy'
  | 'download'
  | 'upload'
  | 'refresh'
  | 'play'
  | 'stop'
  | 'menu'
  | 'moreVertical'

export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH' | 'OPTIONS' | 'HEAD'
