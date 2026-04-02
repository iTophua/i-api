# src-tauri/src/database/ - SQLite 数据层

## OVERVIEW

rusqlite 封装，含自定义连接池(10)、CRUD repository、查询优化器。

## STRUCTURE

```
database/
├── mod.rs            # ConnectionPool + PooledConn + Database + schema 初始化
├── repository.rs     # DatabaseRepository: 所有 CRUD 操作
└── optimizer.rs      # QueryOptimizer: WAL/索引/ANALYZE + QueryBuilder
```

## WHERE TO LOOK

| 任务          | 文件             | 说明                                                                                   |
| ------------- | ---------------- | -------------------------------------------------------------------------------------- |
| 连接池        | `mod.rs`         | ConnectionPool(10) + PooledConn                                                        |
| Schema 初始化 | `mod.rs:105-188` | 8 张表: collections/folders/requests/environments/history/settings/app_state/open_tabs |
| CRUD 操作     | `repository.rs`  | 所有表的增删改查                                                                       |
| 批量删除      | `repository.rs`  | `batch_delete_requests` (事务)                                                         |
| 级联删除      | `repository.rs`  | `delete_collection` (事务删除 requests+folders)                                        |
| 查询优化      | `optimizer.rs`   | WAL/外键/索引/ANALYZE                                                                  |

## CONVENTIONS

- 所有写操作使用事务 (`unchecked_transaction()`) 保证原子性
- 时间字段使用 RFC3339 格式 (`chrono::Utc::now().to_rfc3339()`)
- JSON 字段序列化失败返回错误 (不可 `unwrap_or_default()`)
- 每个连接启用 `PRAGMA foreign_keys = ON` + `journal_mode = WAL`

## ANTI-PATTERNS

- **禁止** 在多个独立操作中写入关联数据 — 必须使用事务
- **禁止** `unwrap_or_default()` 处理 JSON 序列化 — 会静默丢失数据
- **注意** `PooledConn` 通过 `Drop` 自动归还连接池
- **注意** `DatabaseRepository::new` 接受 `Arc<ConnectionPool>` 而非裸连接

## SCHEMA

| 表名         | 主键       | 外键                            | 说明         |
| ------------ | ---------- | ------------------------------- | ------------ |
| collections  | id (TEXT)  | -                               | 请求集合     |
| folders      | id (TEXT)  | collection_id, parent_folder_id | 文件夹(嵌套) |
| requests     | id (TEXT)  | collection_id, folder_id        | HTTP 请求    |
| environments | id (TEXT)  | -                               | 环境变量     |
| history      | id (TEXT)  | request_id                      | 请求历史     |
| settings     | key (TEXT) | -                               | 应用设置     |
| app_state    | id (INT=1) | -                               | 应用状态     |
| open_tabs    | id (TEXT)  | -                               | 打开的标签页 |
