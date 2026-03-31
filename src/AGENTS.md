# src/ - 前端源码 (Vue 3 + TypeScript)

## OVERVIEW

Vue 3 + TypeScript 前端，使用 Composition API + Pinia 状态管理。组件按功能模块组织，stores 管理全局状态，composables 封装可复用逻辑。

## STRUCTURE

```
src/
├── components/
│   ├── common/           # 通用组件
│   ├── request/          # 请求编辑 (6个组件: RequestPanel, BodyEditor, AuthEditor, HeadersEditor, ParamsEditor, ScriptEditor)
│   ├── response/         # 响应展示 (ResponsePanel)
│   └── sidebar/          # 侧边栏 (Sidebar)
├── stores/               # Pinia stores (4个: request, settings, environment, history)
├── composables/          # 组合式函数 (useI18n, useMonacoEditor, useSecretStorage, useShortcuts)
├── views/                # 页面视图 (HomeView, SplashView)
├── router/               # Vue Router 配置
├── types/                # TypeScript 类型定义 (292行，含 normalize/convert)
├── utils/                # 工具函数 (codeGenerator.ts)
├── locales/              # 国际化 (zh-CN, en-US)
├── tests/                # 单元测试 (非标准位置)
└── assets/               # 静态资源 (logo 图标)
```

## WHERE TO LOOK

| 任务          | 文件                              | 说明                           |
| ------------- | --------------------------------- | ------------------------------ |
| 类型定义      | `types/index.ts`                  | 292行核心类型 + normalize 函数 |
| 状态管理      | `stores/request.ts`               | 最复杂的 store                 |
| 国际化        | `locales/index.ts`                | i18n 配置入口                  |
| 快捷键        | `composables/useShortcuts.ts`     | 默认快捷键定义                 |
| Monaco 编辑器 | `composables/useMonacoEditor.ts`  | 代码编辑器集成                 |
| 密钥存储      | `composables/useSecretStorage.ts` | 安全凭证管理                   |

## CONVENTIONS

### 组件风格

- Composition API (setup 语法糖)
- 单文件组件 (.vue)
- 组件名允许单字 (vue/multi-word-component-names off)

### 状态管理

- Pinia stores 使用 `useXxxStore` 命名
- Stores 在 `stores/index.ts` 统一导出
- 每个 store 职责单一 (request/settings/environment/history)

### 类型处理

- 严格 TypeScript (strict: true)
- `types/index.ts` 包含 normalize 函数处理前后端字段名差异 (camelCase ↔ snake_case)
- **禁止** `as any`、`@ts-ignore`

## ANTI-PATTERNS

- **禁止** 空 catch 块
- **避免** `console.log` 残留 (types/index.ts 有 2 处调试日志需清理)
- **避免** 在 composables 中直接操作 DOM
- **注意** 前后端字段名映射: 使用 `normalizeRequest()` / `toBackendRequest()` 转换

## UNIQUE PATTERNS

### 前后端字段映射

```typescript
// 前端 camelCase → 后端 snake_case
toBackendRequest(request) // bodyMode → body_mode, formData → form_data
normalizeRequest(backend) // 反向转换
```

### 国际化初始化

```typescript
// main.ts 中特殊处理
const savedLocale = localStorage.getItem('iapi-locale') || 'zh-CN'
setI18nInstance(i18n) // 全局设置，非标准 Vue i18n 模式
```
