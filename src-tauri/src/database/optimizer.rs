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
}
