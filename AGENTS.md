# PROJECT KNOWLEDGE BASE

**Generated:** 2026-04-02
**Project:** iApi - 轻量、快速、中文的 API 测试工具

## OVERVIEW

Tauri 2.x 桌面应用。前端 Vue 3 + TypeScript + Vite，后端 Rust。类 Postman 的 API 测试工具，支持请求发送、环境管理、集合组织、脚本执行。

## STRUCTURE

```
i-api/
├── src/                    # 前端源码 (Vue 3 + TS)
│   ├── components/         # 按功能模块: common(16), request(6), response(4), sidebar(1), environment(4), history(5), icons(5)
│   ├── stores/             # Pinia: request(750行), settings, environment, history
│   ├── composables/        # 7个: useI18n, useMonacoEditor, useSecretStorage, useShortcuts, useEditorOptimizer, useStreamedRequest, useTabDrag
│   ├── views/              # HomeView(708行), SplashView
│   ├── router/             # Vue Router 配置
│   ├── types/              # 核心类型定义 + normalize 函数
│   ├── utils/              # codeGenerator (esbuild)
│   └── locales/            # zh-CN, en-US
├── src-tauri/              # Rust 后端
│   └── src/
│       ├── lib.rs          # Tauri command 注册 (30+ commands)
│       ├── http/           # HTTP 请求发送 + 取消
│       ├── database/       # SQLite (rusqlite) + 连接池 + 优化器
│       ├── models/         # 数据模型
│       ├── curl/           # cURL 解析
│       ├── openapi/        # OpenAPI/HAR 导入
│       ├── script/         # 前后置脚本执行
│       └── secure_storage/ # Keyring 凭证存储
├── tests/                  # E2E (Playwright) + mocks
├── src/tests/              # 单元测试 (Vitest) - 非标准位置
├── scripts/                # 构建脚本
└── docs/                   # 需求文档
```

## WHERE TO LOOK

| 任务       | 位置                              | 说明                          |
| ---------- | --------------------------------- | ----------------------------- |
| 前端组件   | `src/components/`                 | request/ 最复杂 (6个组件)     |
| 状态管理   | `src/stores/`                     | 4 个 Pinia store              |
| 类型定义   | `src/types/index.ts`              | 含 normalize/convert 函数     |
| Tauri 命令 | `src-tauri/src/lib.rs`            | 所有前端调用的后端入口        |
| HTTP 逻辑  | `src-tauri/src/http/`             | 请求发送、取消、响应处理      |
| 数据库     | `src-tauri/src/database/`         | SQLite schema + CRUD + 连接池 |
| 国际化     | `src/locales/`                    | zh-CN / en-US                 |
| 快捷键     | `src/composables/useShortcuts.ts` | 默认快捷键配置                |

## CONVENTIONS

### 代码风格

- 无分号 (semi: false)
- 单引号 (singleQuote: true)
- 2 空格缩进
- 行宽 100 字符
- 严格 TypeScript (strict: true)

### 命名规范

- 组件: PascalCase (允许单字: vue/multi-word-component-names off)
- Composables: useXxx.ts
- Stores: useXxxStore
- 类型: PascalCase interfaces, camelCase functions

### 前后端通信

- Tauri invoke 命令全部在 `lib.rs` 注册
- 前端通过 `@tauri-apps/api` 调用
- 错误处理: Rust `Result<T, String>` → JS Promise

## ANTI-PATTERNS (THIS PROJECT)

- **禁止** `as any`、`@ts-ignore` — 使用严格类型
- **禁止** 空 catch 块
- **禁止** HTTP 响应缓存 — API 测试工具必须每次发送真实请求
- **避免** `console.log` 残留
- **注意** typecheck 回退: CI 中 `npm run typecheck || npm run build` 可能隐藏类型错误
- **注意** E2E 静默失败: CI 中 `npm run test:e2e || true`

## COMMANDS

```bash
# 开发
npm run dev              # Vite dev server
npm run tauri:dev        # Tauri 开发模式

# 构建
npm run build            # TypeScript check + Vite build
npm run tauri:build      # Tauri 生产构建

# 测试
npm run test             # Vitest 单元测试
npm run test:coverage    # 覆盖率 (阈值 80%)
npm run test:e2e         # Playwright E2E

# 代码质量
npm run lint             # ESLint + auto fix
npm run format           # Prettier 格式化
```

## NOTES

- 测试目录分散: `src/tests/` (单元) + `tests/` (E2E) — 非标准
- 路径别名: `@/*` → `src/*`
- 包管理: pnpm (检测到 pnpm-lock.yaml)
- 覆盖率阈值: 语句/分支/函数/行 ≥ 80%
- Tauri 特性: devtools 启用
- 数据库: SQLite WAL 模式 + 外键约束 + 连接池(10)
