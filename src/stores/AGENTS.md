# src/stores/ - Pinia 状态管理

## OVERVIEW

4 个 Pinia store，Composition API 风格，通过 `index.ts` 统一导出。

## STRUCTURE

```
stores/
├── index.ts          # 统一导出
├── request.ts        # 核心 store (750行): 集合/标签页/请求/批量操作
├── environment.ts    # 环境变量: CRUD + 变量替换
├── history.ts        # 请求历史: CRUD + 导入导出
└── settings.ts       # 应用设置 + 主题 + 持久化
```

## WHERE TO LOOK

| 任务         | 文件             | 说明                                   |
| ------------ | ---------------- | -------------------------------------- |
| 请求/标签页  | `request.ts`     | 最复杂: tabs, collections, 批量操作    |
| 环境变量替换 | `environment.ts` | `replaceVariables()` 处理 {{var}} 语法 |
| 历史记录     | `history.ts`     | 历史 CRUD + JSON 导入导出              |
| 主题/设置    | `settings.ts`    | localStorage 持久化 + 系统主题监听     |

## CONVENTIONS

- Store 命名: `useXxxStore`
- Composition API 风格 (非 Options API)
- 异步操作通过 `invoke` 调用后端，错误 `console.error` + `throw e`
- 持久化: settings 用 localStorage，tabs 用后端 `open_tabs` 表

## ANTI-PATTERNS

- **禁止** 在 store 中直接操作 DOM (主题应用除外)
- **禁止** deep watch 整个 store — 监听具体字段
- **注意** `request.ts` 中 `responses` 使用 `shallowRef` — 避免深度响应式开销
- **注意** settings store 的 `mediaQuery` 监听器需在 `onUnmounted` 清理 (App.vue 已处理)
