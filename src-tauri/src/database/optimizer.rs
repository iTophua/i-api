#![allow(dead_code)]

use rusqlite::{Connection, Result as SqliteResult};

/// 数据库查询优化器
pub struct QueryOptimizer<'a> {
    conn: &'a Connection,
}

impl<'a> QueryOptimizer<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// 应用所有优化
    pub fn apply_all_optimizations(&self) -> SqliteResult<()> {
        self.enable_wal_mode()?;
        self.enable_foreign_keys()?;
        self.optimize_journal_size()?;
        self.set_busy_timeout()?;
        self.analyze_tables()?;
        Ok(())
    }

    /// 启用 WAL 模式（Write-Ahead Logging）
    /// 优势：读写不阻塞，提升并发性能
    fn enable_wal_mode(&self) -> SqliteResult<()> {
        self.conn.pragma_update(None, "journal_mode", &"WAL")?;
        Ok(())
    }

    /// 启用外键约束
    fn enable_foreign_keys(&self) -> SqliteResult<()> {
        self.conn.pragma_update(None, "foreign_keys", &1)?;
        Ok(())
    }

    /// 优化日志大小限制
    fn optimize_journal_size(&self) -> SqliteResult<()> {
        // 设置 WAL 自动检查点阈值为 1000 页
        self.conn.pragma_update(None, "wal_autocheckpoint", &1000)?;

        // 设置缓存大小为 2000 页（约 8MB）
        self.conn.pragma_update(None, "cache_size", &-2000)?;

        // 设置临时存储为内存
        self.conn.pragma_update(None, "temp_store", &"MEMORY")?;

        Ok(())
    }

    /// 设置忙时超时时间
    fn set_busy_timeout(&self) -> SqliteResult<()> {
        // 设置 5 秒超时
        self.conn.pragma_update(None, "busy_timeout", &5000)?;
        Ok(())
    }

    /// 分析表以优化查询计划
    fn analyze_tables(&self) -> SqliteResult<()> {
        self.conn.execute_batch("ANALYZE")?;
        Ok(())
    }

    /// 创建索引（如果不存在）
    pub fn create_indexes(&self) -> SqliteResult<()> {
        let indexes = [
            // 历史记录的 created_at 索引（加速时间范围查询）
            "CREATE INDEX IF NOT EXISTS idx_history_created_at ON history(created_at)",
            // 历史记录的 request_id 索引（加速关联查询）
            "CREATE INDEX IF NOT EXISTS idx_history_request_id ON history(request_id)",
            // 请求的 collection_id 索引（加速集合查询）
            "CREATE INDEX IF NOT EXISTS idx_requests_collection_id ON requests(collection_id)",
            // 请求的 folder_id 索引（加速文件夹查询）
            "CREATE INDEX IF NOT EXISTS idx_requests_folder_id ON requests(folder_id)",
            // 请求的 updated_at 索引（加速排序查询）
            "CREATE INDEX IF NOT EXISTS idx_requests_updated_at ON requests(updated_at)",
            // 环境的 updated_at 索引
            "CREATE INDEX IF NOT EXISTS idx_environments_updated_at ON environments(updated_at)",
        ];

        for index_sql in indexes.iter() {
            self.conn.execute_batch(index_sql)?;
        }

        Ok(())
    }

    /// 批量插入优化 - 使用事务包装
    #[allow(dead_code)]
    pub fn execute_batch_in_transaction<F>(&self, operation: F) -> SqliteResult<()>
    where
        F: FnOnce(&Connection) -> SqliteResult<()>,
    {
        let tx = self.conn.unchecked_transaction()?;
        operation(&tx)?;
        tx.commit()?;
        Ok(())
    }

    /// 使用预编译语句提升重复查询性能
    #[allow(dead_code)]
    pub fn prepare_cached_query<P, F, T>(&self, sql: &str, params: P, mapper: F) -> SqliteResult<T>
    where
        P: rusqlite::Params,
        F: FnOnce(&rusqlite::Row) -> SqliteResult<T>,
    {
        let mut stmt = self.conn.prepare_cached(sql)?;
        let row = stmt.query_row(params, |row| mapper(row))?;
        Ok(row)
    }

    /// 获取数据库统计信息
    #[allow(dead_code)]
    pub fn get_stats(&self) -> SqliteResult<DatabaseStats> {
        let mut stmt = self.conn.prepare(
            "SELECT 
                (SELECT COUNT(*) FROM collections) as collections,
                (SELECT COUNT(*) FROM requests) as requests,
                (SELECT COUNT(*) FROM folders) as folders,
                (SELECT COUNT(*) FROM environments) as environments,
                (SELECT COUNT(*) FROM history) as history",
        )?;

        let stats = stmt.query_row([], |row| {
            Ok(DatabaseStats {
                collections: row.get(0)?,
                requests: row.get(1)?,
                folders: row.get(2)?,
                environments: row.get(3)?,
                history: row.get(4)?,
            })
        })?;

        Ok(stats)
    }

    /// 执行 VACUUM 优化数据库文件大小
    #[allow(dead_code)]
    pub fn vacuum(&self) -> SqliteResult<()> {
        self.conn.execute_batch("VACUUM")?;
        Ok(())
    }

    /// 增量优化 - 只优化特定表
    #[allow(dead_code)]
    pub fn optimize_table(&self, table_name: &str) -> SqliteResult<()> {
        self.conn
            .execute_batch(&format!("ANALYZE {}", table_name))?;
        self.conn
            .execute_batch(&format!("REINDEX {}", table_name))?;
        Ok(())
    }
}

/// 数据库统计信息
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DatabaseStats {
    pub collections: usize,
    pub requests: usize,
    pub folders: usize,
    pub environments: usize,
    pub history: usize,
}

/// 查询构建器 - 链式 API 构建复杂查询
#[allow(dead_code)]
pub struct QueryBuilder {
    table: String,
    columns: Vec<String>,
    wheres: Vec<String>,
    order_by: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
}

impl QueryBuilder {
    #[allow(dead_code)]
    pub fn new(table: &str) -> Self {
        Self {
            table: table.to_string(),
            columns: vec!["*".to_string()],
            wheres: Vec::new(),
            order_by: None,
            limit: None,
            offset: None,
        }
    }

    pub fn select(mut self, columns: &[&str]) -> Self {
        self.columns = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn where_eq(mut self, column: &str, _value: &str) -> Self {
        self.wheres.push(format!("{} = ?", column));
        self
    }

    pub fn where_like(mut self, column: &str) -> Self {
        self.wheres.push(format!("{} LIKE ?", column));
        self
    }

    pub fn order_by(mut self, column: &str, desc: bool) -> Self {
        self.order_by = Some(format!("{} {}", column, if desc { "DESC" } else { "ASC" }));
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn build(self) -> String {
        let mut sql = format!("SELECT {} FROM {}", self.columns.join(", "), self.table);

        if !self.wheres.is_empty() {
            sql.push_str(&format!(" WHERE {}", self.wheres.join(" AND ")));
        }

        if let Some(order) = self.order_by {
            sql.push_str(&format!(" ORDER BY {}", order));
        }

        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        sql
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn create_test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("无法创建内存数据库");
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS collections (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS folders (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                collection_id TEXT NOT NULL,
                parent_folder_id TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (collection_id) REFERENCES collections(id),
                FOREIGN KEY (parent_folder_id) REFERENCES folders(id)
            );

            CREATE TABLE IF NOT EXISTS requests (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                method TEXT NOT NULL,
                url TEXT NOT NULL,
                params TEXT,
                headers TEXT,
                body TEXT,
                auth TEXT,
                pre_script TEXT,
                post_script TEXT,
                collection_id TEXT,
                folder_id TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (collection_id) REFERENCES collections(id),
                FOREIGN KEY (folder_id) REFERENCES folders(id)
            );

            CREATE TABLE IF NOT EXISTS environments (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                variables TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS history (
                id TEXT PRIMARY KEY,
                request_id TEXT,
                method TEXT NOT NULL,
                url TEXT NOT NULL,
                status INTEGER NOT NULL,
                response_time INTEGER NOT NULL,
                response_size INTEGER NOT NULL,
                params TEXT,
                headers TEXT,
                body TEXT,
                auth TEXT,
                created_at TEXT NOT NULL
            );
            "#,
        )
        .expect("无法创建测试表");
        conn
    }

    #[test]
    fn test_query_builder() {
        let sql = QueryBuilder::new("history")
            .select(&["id", "method", "url"])
            .where_eq("request_id", "123")
            .order_by("created_at", true)
            .limit(10)
            .offset(0)
            .build();

        assert!(sql.contains("SELECT id, method, url FROM history"));
        assert!(sql.contains("WHERE request_id = ?"));
        assert!(sql.contains("ORDER BY created_at DESC"));
        assert!(sql.contains("LIMIT 10"));
    }

    #[test]
    fn test_query_builder_minimal() {
        let sql = QueryBuilder::new("requests").build();
        assert_eq!(sql, "SELECT * FROM requests");
    }

    #[test]
    fn test_enable_wal_mode() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("iapi_test_wal.db");
        let conn = Connection::open(&db_path).expect("无法创建临时数据库");
        let optimizer = QueryOptimizer::new(&conn);
        optimizer.enable_wal_mode().expect("启用 WAL 模式失败");
        let mode: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .expect("查询 journal_mode 失败");
        assert_eq!(mode.to_lowercase(), "wal");
        drop(conn);
        let _ = std::fs::remove_file(db_path);
    }

    #[test]
    fn test_enable_foreign_keys() {
        let conn = create_test_db();
        let optimizer = QueryOptimizer::new(&conn);
        optimizer.enable_foreign_keys().expect("启用外键约束失败");
        let enabled: i32 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .expect("查询 foreign_keys 失败");
        assert_eq!(enabled, 1);
    }

    #[test]
    fn test_set_busy_timeout() {
        let conn = create_test_db();
        let optimizer = QueryOptimizer::new(&conn);
        optimizer.set_busy_timeout().expect("设置忙时超时失败");
        let timeout: i32 = conn
            .query_row("PRAGMA busy_timeout", [], |row| row.get(0))
            .expect("查询 busy_timeout 失败");
        assert_eq!(timeout, 5000);
    }

    #[test]
    fn test_create_indexes() {
        let conn = create_test_db();
        let optimizer = QueryOptimizer::new(&conn);
        optimizer.create_indexes().expect("创建索引失败");

        let mut indexes = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='history'")
            .expect("查询索引失败");
        let index_names: Vec<String> = indexes
            .query_map([], |row| row.get(0))
            .expect("查询索引失败")
            .filter_map(|r| r.ok())
            .collect();
        assert!(index_names
            .iter()
            .any(|n| n.contains("idx_history_created_at")));
    }

    #[test]
    fn test_database_stats() {
        let conn = create_test_db();
        let optimizer = QueryOptimizer::new(&conn);
        let stats = optimizer.get_stats().expect("获取统计信息失败");
        assert_eq!(stats.collections, 0);
        assert_eq!(stats.requests, 0);
        assert_eq!(stats.history, 0);
    }

    #[test]
    fn test_optimize_journal_size() {
        let conn = create_test_db();
        let optimizer = QueryOptimizer::new(&conn);
        optimizer.optimize_journal_size().expect("优化日志大小失败");

        let wal_auto: i32 = conn
            .query_row("PRAGMA wal_autocheckpoint", [], |row| row.get(0))
            .expect("查询 wal_autocheckpoint 失败");
        assert_eq!(wal_auto, 1000);
    }

    #[test]
    fn test_execute_batch_in_transaction() {
        let conn = create_test_db();
        let optimizer = QueryOptimizer::new(&conn);

        let result = optimizer.execute_batch_in_transaction(|tx| {
            tx.execute("INSERT INTO collections (id, name, created_at, updated_at) VALUES ('1', 'Test', '2024-01-01', '2024-01-01')", [])?;
            tx.execute("INSERT INTO collections (id, name, created_at, updated_at) VALUES ('2', 'Test2', '2024-01-01', '2024-01-01')", [])?;
            Ok(())
        });

        assert!(result.is_ok());
        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM collections", [], |row| row.get(0))
            .expect("查询失败");
        assert_eq!(count, 2);
    }

    #[test]
    fn test_query_builder_with_like() {
        let sql = QueryBuilder::new("requests")
            .select(&["id", "name", "url"])
            .where_like("name")
            .order_by("updated_at", false)
            .limit(20)
            .build();

        assert!(sql.contains("LIKE ?"));
        assert!(sql.contains("ORDER BY updated_at ASC"));
        assert!(sql.contains("LIMIT 20"));
    }
}
