# iApi 产品需求文档

## 1. 产品概述

### 1.1 产品定位

**产品名称**：iApi

**核心价值**：轻量、快速、中文的 API 测试工具

**目标用户**：国内开发者

**差异化优势**：
- 启动快（< 2秒）
- 体积小（< 20MB）
- 界面简洁，无臃余功能
- 纯本地，数据隐私可控
- 中文原生，无翻译感

### 1.2 竞品分析

| 产品 | 优点 | 缺点 |
|------|------|------|
| Postman | 功能全、生态完善 | 英文界面、重、国内访问慢 |
| Apifox | 中文、功能强大 | 太重、启动慢、功能臃肿 |
| ApiPost | 中文 | 界面老旧、体验一般 |
| **iApi** | 轻量、快速、中文 | 功能聚焦核心，不做大而全 |

### 1.3 版本规划

- **V1.0 MVP**：核心功能，本地化安装 ✅ **已完成**
- **V1.1**：Monaco 编辑器集成、JSON 美化、代码生成
- **V1.2**：OpenAPI/Swagger 导入、HAR 文件导入
- **V1.3**：团队协作（局域网同步）、Mock 服务
- **V2.0**：云端同步（可选）、团队版、插件系统

---

## 2. 技术方案

### 2.1 技术选型

| 组件 | 选择 | 版本 | 说明 |
|------|------|------|------|
| 桌面框架 | **Tauri** | 2.10.x | 体积小（~10MB）、启动快、内存低 |
| 前端框架 | **Vue** | 3.5.x | 响应式、生态成熟 |
| UI 组件库 | **Naive UI** | 2.44.x | 中文友好、轻量、TS 支持好 |
| 状态管理 | **Pinia** | 3.x | Vue 3 官方推荐 |
| 本地存储 | **SQLite** | 0.33.x | 请求、环境、历史记录持久化 |
| HTTP 客户端 | **reqwest** | 0.12.x | Rust 异步 HTTP 客户端 |
| 代码编辑器 | **Monaco Editor** | 0.55.x | VSCode 同款，JSON 美化/高亮 |
| 构建工具 | **Vite** | 8.x | 开发快、HMR 好 |

### 2.2 项目结构

```
iApi/
├── src-tauri/               # Tauri 后端（Rust）
│   ├── src/
│   │   ├── main.rs          # 入口
│   │   ├── lib.rs           # IPC 命令注册
│   │   ├── curl/            # cURL 解析
│   │   ├── database/        # SQLite 操作
│   │   ├── http/            # HTTP 客户端
│   │   ├── models/          # 数据模型
│   │   └── script/          # 脚本执行
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                     # Vue 前端
│   ├── main.ts
│   ├── App.vue
│   ├── components/
│   │   ├── common/          # 导入导出弹窗
│   │   ├── request/         # 请求编辑器
│   │   ├── response/        # 响应查看器
│   │   └── sidebar/         # 侧边栏
│   ├── stores/              # Pinia 状态
│   ├── types/               # TypeScript 类型
│   ├── router/              # 路由
│   └── views/               # 页面
├── docs/                    # 文档
├── package.json
└── vite.config.ts
```

### 2.3 平台支持

- **Windows**：Windows 10/11（64-bit）
- **macOS**：macOS 10.15+（Intel & Apple Silicon）

---

## 3. 功能需求

### 3.1 P0 核心功能

#### 3.1.1 请求构建 ✅ 已完成

| 功能 | 状态 | 详细说明 |
|------|------|----------|
| HTTP 方法 | ✅ | GET / POST / PUT / DELETE / PATCH / OPTIONS / HEAD |
| URL 编辑器 | ✅ | 协议选择（http/https）、路径输入、查询参数可视化编辑 |
| 请求头管理 | ✅ | Key-Value 编辑、启用/禁用、常用头快捷选择 |
| 请求体编辑 | ✅ | none / form-data / x-www-form-urlencoded / raw / binary |
| 文件上传 | ✅ | form-data 模式支持文件选择，binary 模式直接上传 |
| 认证支持 | ✅ | No Auth / Basic Auth / Bearer Token / API Key |
| cURL 导入 | ✅ | 解析 cURL 命令自动生成请求 |

#### 3.1.2 响应查看 ✅ 已完成

| 功能 | 状态 | 详细说明 |
|------|------|----------|
| 响应体展示 | ✅ | Monaco Editor 展示，支持语法高亮 |
| 响应头查看 | ✅ | 分组展示 |
| 状态信息 | ✅ | 状态码、耗时、响应大小 |
| Cookie 查看 | ✅ | 响应 Cookie 列表 |
| JSON 美化 | ✅ | Monaco Editor 语法高亮，格式化按钮 |
| 文件下载 | ⏳ | 检测 Content-Disposition，显示保存按钮 |

#### 3.1.3 请求管理 ✅ 已完成

| 功能 | 状态 | 详细说明 |
|------|------|----------|
| 保存请求 | ✅ | 保存到集合/文件夹 |
| 文件夹组织 | ✅ | 树形结构展示 |
| 历史记录 | ✅ | 自动记录最近 100 条请求 |
| 请求搜索 | ⏳ | 按名称/URL 搜索 |
| 拖拽排序 | ⏳ | 拖拽调整顺序 |

#### 3.1.4 环境变量 ✅ 已完成

| 功能 | 状态 | 详细说明 |
|------|------|----------|
| 多环境管理 | ✅ | 创建/编辑/删除环境 |
| 变量定义 | ✅ | Key-Value 形式，支持启用/禁用 |
| 变量使用 | ✅ | `{{baseUrl}}/api/users` 语法 |
| 环境切换 | ✅ | 顶部下拉快速切换 |
| 动态变量 | ⏳ | `{{$timestamp}}`、`{{$randomInt}}` 等 |

#### 3.1.5 前置/后置脚本 ✅ 已完成

| 功能 | 状态 | 详细说明 |
|------|------|----------|
| 前置脚本 | ✅ | 请求发送前执行 |
| 后置脚本 | ✅ | 响应接收后执行 |
| 内置对象 | ✅ | pm.request、pm.response、pm.environment |
| 断言方法 | ✅ | pm.test() 支持 |
| 变量提取 | ✅ | pm.environment.set() 支持 |

#### 3.1.6 导入导出 ✅ 已完成

| 功能 | 状态 | 详细说明 |
|------|------|----------|
| cURL 导入 | ✅ | 解析 cURL 命令 |
| Postman 导入 | ✅ | 导入 Postman Collection v2.1 格式 |
| Postman 导出 | ✅ | 导出为 Postman Collection v2.1 格式 |

#### 3.1.7 基础体验 ✅ 已完成

| 功能 | 状态 | 详细说明 |
|------|------|----------|
| 明暗主题 | ✅ | 跟随系统或手动切换 |
| 中文界面 | ✅ | 原生中文 |
| 快捷键 | ⏳ | 发送请求、保存等快捷键 |

---

### 3.2 P1 重要功能

| 功能 | 状态 | 详细说明 |
|------|------|----------|
| Monaco Editor 集成 | ✅ | 请求体、响应体、脚本编辑器使用 Monaco |
| JSON 美化与高亮 | ✅ | 自动格式化、语法高亮 |
| 代码生成 | ✅ | 生成 curl / axios / fetch / Python requests / Java / Go |
| 复制为 cURL | ✅ | 一键复制请求为 cURL 命令 |
| OpenAPI/Swagger 导入 | ✅ | 导入 OpenAPI 3.0 / Swagger 2.0 文档（JSON/YAML） |
| HAR 文件导入 | ✅ | 导入浏览器导出的 HAR 网络请求 |
| 快捷键系统 | ✅ | Ctrl+Enter 发送、Ctrl+S 保存、Ctrl+N 新建等 |
| URL 自动补全 | ⏳ | 基于历史请求自动补全 URL |
| 请求头建议 | ⏳ | Content-Type、Accept 等常用值下拉选择 |
| 环境变量提示 | ⏳ | 输入 `{{` 触发变量补全 |

---

## 4. 数据结构设计

### 4.1 集合（Collection）

```typescript
interface Collection {
  id: string
  name: string
  description?: string
  folders: Folder[]
  requests: Request[]
  createdAt: string
  updatedAt: string
}

interface Folder {
  id: string
  name: string
  description?: string
  folders: Folder[]
  requests: Request[]
  createdAt: string
  updatedAt: string
}

interface Request {
  id: string
  name: string
  description?: string
  method: HttpMethod
  url: string
  params: KeyValuePair[]
  headers: KeyValuePair[]
  body: RequestBody
  auth: AuthConfig
  preScript?: string
  postScript?: string
  createdAt: string
  updatedAt: string
}

type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH' | 'OPTIONS' | 'HEAD'

interface KeyValuePair {
  key: string
  value: string
  description?: string
  enabled: boolean
}

interface RequestBody {
  mode: 'none' | 'form-data' | 'urlencoded' | 'raw' | 'binary'
  raw?: string
  rawType?: 'json' | 'xml' | 'text' | 'html'
  formData?: FormDatum[]
  urlencoded?: KeyValuePair[]
  binary?: string
}

interface FormDatum extends KeyValuePair {
  type: 'text' | 'file'
  filePath?: string
}

interface AuthConfig {
  type: 'none' | 'basic' | 'bearer' | 'apikey'
  basic?: { username: string; password: string }
  bearer?: { token: string }
  apikey?: { key: string; value: string; addTo: 'header' | 'query' }
}
```

### 4.2 环境变量（Environment）

```typescript
interface Environment {
  id: string
  name: string
  variables: Variable[]
  createdAt: string
  updatedAt: string
}

type Variable = KeyValuePair
```

### 4.3 历史记录（History）

```typescript
interface History {
  id: string
  requestId?: string
  method: HttpMethod
  url: string
  status: number
  responseTime: number
  responseSize: number
  createdAt: string
}
```

### 4.4 响应（Response）

```typescript
interface Response {
  status: number
  statusText: string
  headers: Record<string, string>
  cookies: Cookie[]
  body: string
  responseTime: number
  responseSize: number
}

interface Cookie {
  name: string
  value: string
  domain?: string
  path?: string
  expires?: string
  httpOnly?: boolean
  secure?: boolean
}
```

### 4.5 全局设置（Settings）

```typescript
interface Settings {
  theme: 'light' | 'dark' | 'system'
  language: 'zh-CN'
  historyLimit: number
  timeout: number
  proxy?: {
    enabled: boolean
    host: string
    port: number
    username?: string
    password?: string
  }
}
```

---

## 5. 界面设计

### 5.1 整体布局

```
┌─────────────────────────────────────────────────────────────────────┐
│  工具栏：环境切换  导入  导出              主题切换  设置           │
├────────────┬────────────────────────────────────────────────────────┤
│            │  URL栏：[GET ▼] [________________________] [发送] [保存]│
│  侧边栏    ├────────────────────────────────────────────────────────┤
│            │  标签页：Params | Headers | Body | Auth | 前置 | 后置  │
│  ┌─集合    ├────────────────────────────────────────────────────────┤
│  │ ├用户API│                                                        │
│  │ │ ├列表 │              请求编辑区                                │
│  │ │ └详情 │                                                        │
│  │ └订单API│                                                        │
│  ├─历史    ├────────────────────────────────────────────────────────┤
│  │ GET /use│  状态: 200 OK  |  耗时: 123ms  |  大小: 1.2KB          │
│  │ POST /lo├────────────────────────────────────────────────────────┤
│  │ GET /ord│  标签页：响应体 | 响应头 | Cookies                     │
│  └─        ├────────────────────────────────────────────────────────┤
│            │              响应查看区                                │
└────────────┴────────────────────────────────────────────────────────┘
```

---

## 6. 开发计划

### 6.1 第一阶段：项目搭建 ✅ 已完成

- [x] 初始化 Tauri + Vue 3 项目
- [x] 配置 TypeScript、ESLint、Prettier
- [x] 集成 Naive UI
- [x] 设计并创建数据库表结构
- [x] 实现基础 IPC 通信

### 6.2 第二阶段：核心请求功能 ✅ 已完成

- [x] URL 编辑器组件
- [x] 请求方法选择
- [x] Params 编辑器
- [x] Headers 编辑器
- [x] Body 编辑器（各类型）
- [x] Auth 认证配置
- [x] 发送 HTTP 请求（Rust 后端）
- [x] 响应接收与展示

### 6.3 第三阶段：数据管理 ✅ 已完成

- [x] 集合/文件夹树形组件
- [x] 请求保存与加载
- [x] 历史记录
- [x] 环境变量管理

### 6.4 第四阶段：脚本与导入导出 ✅ 已完成

- [x] 脚本执行引擎
- [x] pm 对象实现
- [x] 前置/后置脚本执行
- [x] cURL 导入
- [x] Postman 导入导出

### 6.5 第五阶段：编辑器增强 ✅ 已完成

- [x] Monaco Editor 集成
- [x] JSON 美化与语法高亮
- [x] 请求体 JSON 编辑器
- [x] 脚本编辑器增强（代码片段）
- [x] 响应体代码高亮

### 6.6 第六阶段：代码生成 ✅ 已完成

- [x] 生成 curl 命令
- [x] 生成 JavaScript (axios/fetch)
- [x] 生成 Python (requests)
- [x] 生成 Java (OkHttp)
- [x] 生成 Go 代码
- [x] 复制为 cURL 功能

### 6.7 第七阶段：高级导入 ✅ 已完成

- [x] OpenAPI 3.0 导入
- [x] Swagger 2.0 导入
- [x] HAR 文件导入

### 6.8 第八阶段：优化与发布 ⏳ 进行中

- [x] 快捷键系统
- [ ] 性能优化
- [ ] 打包与分发
- [ ] 测试与 Bug 修复

---

## 7. 附录

### 7.1 快捷键列表

| 快捷键 | 功能 |
|--------|------|
| Ctrl+Enter | 发送请求 |
| Ctrl+S | 保存请求 |
| Ctrl+N | 新建请求 |
| Ctrl+Shift+I | 导入 cURL |
| Ctrl+Shift+C | 复制为 cURL |

### 7.2 HTTP 状态码颜色

| 状态码范围 | 颜色 |
|-----------|------|
| 2xx | 绿色 #52c41a |
| 3xx | 蓝色 #1890ff |
| 4xx | 橙色 #fa8c16 |
| 5xx | 红色 #f5222d |

### 7.3 HTTP 方法颜色

| 方法 | 颜色 |
|------|------|
| GET | #61affe |
| POST | #49cc90 |
| PUT | #fca130 |
| DELETE | #f93e3e |
| PATCH | #50e3c2 |
| OPTIONS | #0d5aa7 |
| HEAD | #9012fe |

---

## 8. 版本记录

| 版本 | 日期 | 说明 |
|------|------|------|
| v1.0 | 2026-03-27 | 初始需求文档 |
| v1.1 | 2026-03-27 | MVP 核心功能开发完成 |
| v1.2 | 2026-03-27 | Monaco Editor 集成、代码生成、复制为 cURL |
| v1.3 | 2026-03-27 | OpenAPI/Swagger 导入、HAR 导入、快捷键系统 |