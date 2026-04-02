# src/composables/ - 可复用组合式函数

## OVERVIEW

7 个 composables，封装编辑器、国际化、快捷键、流式请求、拖拽等逻辑。

## STRUCTURE

```
composables/
├── useI18n.ts            # 轻量国际化 (非 vue-i18n)
├── useMonacoEditor.ts    # Monaco 编辑器语言注册
├── useSecretStorage.ts   # Tauri secure storage 封装
├── useShortcuts.ts       # 键盘快捷键管理
├── useEditorOptimizer.ts # Monaco 实例缓存 + 虚拟滚动
├── useStreamedRequest.ts # SSE 流式请求
└── useTabDrag.ts         # 标签页拖拽排序
```

## WHERE TO LOOK

| 任务            | 文件                    | 说明                         |
| --------------- | ----------------------- | ---------------------------- |
| 国际化          | `useI18n.ts`            | 自定义 i18n，非标准 vue-i18n |
| Monaco 语言注册 | `useMonacoEditor.ts`    | JSON/JS/Python/Java/Go/SQL   |
| 快捷键          | `useShortcuts.ts`       | 全局快捷键注册/卸载          |
| 编辑器性能优化  | `useEditorOptimizer.ts` | 实例缓存 LRU + 内存监控      |
| 流式请求        | `useStreamedRequest.ts` | Tauri event 监听 SSE 流      |
| 标签页拖拽      | `useTabDrag.ts`         | DOM 拖拽排序逻辑             |

## CONVENTIONS

- 文件名: `useXxx.ts`
- 返回对象包含清理方法 (如 `cleanup`, `dispose`)
- 使用 `onUnmounted` 自动清理事件监听器

## ANTI-PATTERNS

- **禁止** 在 composable 中直接操作 DOM (useTabDrag 除外 — 它只操作拖拽元素)
- **禁止** 忘记清理 Tauri event 监听器 (useStreamedRequest 必须调用 cleanup)
- **注意** `useEditorOptimizer.ts` 的编辑器缓存有大小限制 (maxCacheSize: 10)
- **注意** Monaco 实例通过 `import * as monaco from 'monaco-editor'` 静态导入

## UNIQUE PATTERNS

### 自定义国际化 (非 vue-i18n)

```typescript
// main.ts 中初始化
const i18n = createI18n(savedLocale)
setI18nInstance(i18n) // 全局设置

// 组件中使用
const { t } = useI18n()
t('request.send') // 从 locales/zh-CN.json 读取
```

### Tauri 事件监听模式

```typescript
// useStreamedRequest.ts
const unlisten = await listen(eventId, handler)
// onUnmounted 中调用 unlisten()
```
