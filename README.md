# iApi

轻量、快速、中文的 API 测试工具。

## 简介

iApi 是一个基于 Tauri 2.x 的桌面应用，提供类似 Postman 的 API 测试功能。支持请求发送、环境管理、集合组织、脚本执行等功能。

## 技术栈

- **前端**: Vue 3 + TypeScript + Vite + Pinia
- **后端**: Rust + Tauri 2.x
- **数据库**: SQLite (rusqlite)
- **HTTP 客户端**: reqwest
- **UI 框架**: Naive UI

## 功能特性

- HTTP 请求发送与响应查看
- 环境变量管理
- 集合与文件夹组织
- 前置/后置脚本执行
- cURL 命令导入
- OpenAPI/HAR 文件导入
- Postman 集合导入/导出
- 代码生成器
- 快捷键支持
- 深色/浅色主题

## 开发

```bash
# 安装依赖
pnpm install

# 启动开发服务器
pnpm dev

# 启动 Tauri 开发模式
pnpm tauri:dev

# 构建
pnpm build

# 构建 Tauri 应用
pnpm tauri:build
```

## 测试

```bash
# 运行单元测试
pnpm test

# 运行测试覆盖率
pnpm test:coverage

# 运行 E2E 测试
pnpm test:e2e
```

## 代码质量

```bash
# ESLint 检查并修复
pnpm lint

# Prettier 格式化
pnpm format
```

## 项目结构

```
i-api/
├── src/                    # 前端源码 (Vue 3 + TS)
│   ├── components/         # 组件
│   ├── stores/             # Pinia 状态管理
│   ├── composables/        # 组合式函数
│   ├── views/              # 页面视图
│   ├── types/              # TypeScript 类型定义
│   └── locales/            # 国际化
├── src-tauri/              # Rust 后端
│   └── src/
│       ├── http/           # HTTP 请求处理
│       ├── database/       # SQLite 数据库
│       ├── models/         # 数据模型
│       ├── curl/           # cURL 解析
│       ├── openapi/        # OpenAPI/HAR 导入
│       └── script/         # 脚本执行
├── tests/                  # E2E 测试
└── docs/                   # 文档
```

## 许可证

MIT
