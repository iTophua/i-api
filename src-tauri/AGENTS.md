# src-tauri/ - Rust 后端 (Tauri 2.x)

## OVERVIEW

Tauri 2.x 桌面应用后端。Rust 实现 HTTP 请求发送、SQLite 数据库、cURL 解析、OpenAPI 导入、前后置脚本执行、安全凭证存储。前端通过 `@tauri-apps/api` 调用 30+ 个 command。

## STRUCTURE

```
src-tauri/
├── src/
│   ├── lib.rs            # Tauri command 注册入口 (30+ commands)
│   ├── main.rs           # 应用启动入口
│   ├── error.rs          # 错误类型定义 (IApiError)
│   ├── http/             # HTTP 请求发送 + 取消 (reqwest)
│   ├── database/         # SQLite CRUD (rusqlite)
│   ├── models/           # 数据模型 (HttpRequest, HttpResponse, Collection, etc.)
│   ├── curl/             # cURL 命令解析
│   ├── openapi/          # OpenAPI/HAR 导入
│   ├── script/           # 前后置脚本执行
│   └── secure_storage.rs # Keyring 凭证存储
├── Cargo.toml            # Rust 依赖配置
├── tauri.conf.json       # Tauri 应用配置
└── gen/                  # 自动生成的 schema
```

## WHERE TO LOOK

| 任务         | 文件                  | 说明                                                     |
| ------------ | --------------------- | -------------------------------------------------------- |
| 新增 command | `src/lib.rs`          | 在 `invoke_handler` 注册 + 添加 `#[tauri::command]` 函数 |
| HTTP 逻辑    | `src/http/mod.rs`     | reqwest 发送 + tokio 取消机制                            |
| 数据库操作   | `src/database/mod.rs` | SQLite schema + CRUD                                     |
| 数据模型     | `src/models/mod.rs`   | 前后端共享的数据结构                                     |
| 错误处理     | `src/error.rs`        | IApiError + thiserror                                    |
| cURL 解析    | `src/curl/mod.rs`     | cURL 命令 → HttpRequest                                  |
| 导入功能     | `src/openapi/mod.rs`  | OpenAPI 3.x + HAR 格式                                   |
| 脚本执行     | `src/script/mod.rs`   | 前后置脚本 (JavaScript)                                  |

## CONVENTIONS

### Command 模式

```rust
#[tauri::command]
async fn command_name(
    param: Type,
    db: tauri::State<'_, Arc<Database>>
) -> Result<ReturnType, String> {
    // 实现
    db.method().map_err(|e| e.to_string())
}
```

### 错误处理

- 使用 `thiserror` 定义 `IApiError`
- Command 返回 `Result<T, String>` (Tauri 要求)
- 内部使用 `anyhow::Result` 或自定义 `IApiError`

### 数据库

- rusqlite (bundled SQLite)
- `Arc<Database>` 通过 Tauri State 共享
- schema 在 `database/mod.rs` 初始化

## ANTI-PATTERNS

- **禁止** 在 command 中 panic (使用 Result)
- **避免** 阻塞主线程 (使用 async/await)
- **注意** SQLite 连接不是线程安全的，必须使用 Arc 包装
- **注意** 前端调用是异步的，command 必须是 async

## DEPENDENCIES

| 用途   | crate                             |
| ------ | --------------------------------- |
| HTTP   | reqwest (json, multipart, stream) |
| 数据库 | rusqlite (bundled)                |
| 序列化 | serde + serde_json                |
| 异步   | tokio (full)                      |
| 错误   | thiserror + anyhow                |
| 凭证   | keyring                           |
| UUID   | uuid (v4, serde)                  |
| 时间   | chrono (serde)                    |
