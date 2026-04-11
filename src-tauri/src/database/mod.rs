#![allow(dead_code)]

mod optimizer;
mod repository;

use rusqlite::{Connection, Result as SqliteResult};
use std::io;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub use optimizer::QueryOptimizer;
pub use repository::DatabaseRepository;

/// 简单的数据库连接池
#[derive(Clone)]
struct ConnectionPool {
    connections: Arc<Mutex<Vec<Connection>>>,
}

impl ConnectionPool {
    fn new(db_path: PathBuf, size: usize) -> SqliteResult<Self> {
        let mut connections = Vec::with_capacity(size);

        for _ in 0..size {
            let conn = Connection::open(&db_path).map_err(|e| {
                rusqlite::Error::ToSqlConversionFailure(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    format!("无法打开数据库连接：{}", e),
                )))
            })?;
            conn.pragma_update(None, "foreign_keys", &1).ok();
            conn.pragma_update(None, "journal_mode", &"WAL").ok();
            connections.push(conn);
        }

        Ok(Self {
            connections: Arc::new(Mutex::new(connections)),
        })
    }

    fn get(&self) -> SqliteResult<PooledConn> {
        let conn = {
            let mut conns = self.connections.lock().map_err(|e| {
                rusqlite::Error::ToSqlConversionFailure(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    format!("连接池锁错误：{}", e),
                )))
            })?;
            conns.pop().ok_or_else(|| {
                rusqlite::Error::ToSqlConversionFailure(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "连接池已空",
                )))
            })?
        };

        Ok(PooledConn {
            conn: Some(conn),
            pool: self.connections.clone(),
        })
    }
}

struct PooledConn {
    conn: Option<Connection>,
    pool: Arc<Mutex<Vec<Connection>>>,
}

impl Drop for PooledConn {
    fn drop(&mut self) {
        if let Some(conn) = self.conn.take() {
            if let Ok(mut pool) = self.pool.lock() {
                pool.push(conn);
            }
        }
    }
}

impl std::ops::Deref for PooledConn {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        self.conn.as_ref().unwrap()
    }
}

pub struct Database {
    #[allow(dead_code)]
    pool: Arc<ConnectionPool>,
    pub repository: Arc<DatabaseRepository>,
}

impl Database {
    pub fn new(app_data_dir: PathBuf) -> SqliteResult<Self> {
        std::fs::create_dir_all(&app_data_dir).map_err(|e| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!("无法创建应用数据目录 {:?}: {}", app_data_dir, e),
            )))
        })?;

        let db_path = app_data_dir.join("iapi.db");
        let pool = ConnectionPool::new(db_path.clone(), 10)?;

        // 初始化表结构
        let conn = pool.get()?;

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

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS app_state (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                current_request_id TEXT,
                current_environment_id TEXT,
                sidebar_collapsed INTEGER DEFAULT 0,
                request_panel_height INTEGER,
                response_panel_height INTEGER,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS open_tabs (
                id TEXT PRIMARY KEY,
                tab_data TEXT NOT NULL,
                active_tab_id TEXT,
                updated_at TEXT NOT NULL
            );
            "#,
        )?;

        // 应用查询优化
        let optimizer = QueryOptimizer::new(&*conn);
        if let Err(e) = optimizer.apply_all_optimizations() {
            eprintln!("警告：应用数据库优化失败：{}", e);
        }
        if let Err(e) = optimizer.create_indexes() {
            eprintln!("警告：创建数据库索引失败：{}", e);
        }

        // 数据库迁移：为 history 表添加新列（如果不存在）
        let migrations = [
            "ALTER TABLE history ADD COLUMN params TEXT",
            "ALTER TABLE history ADD COLUMN headers TEXT",
            "ALTER TABLE history ADD COLUMN body TEXT",
            "ALTER TABLE history ADD COLUMN auth TEXT",
        ];
        for migration in migrations {
            let _ = conn.execute(migration, []);
        }

        // 创建 Arc 包装
        let pool_arc = Arc::new(pool);
        let repo = Arc::new(DatabaseRepository::new(pool_arc.clone()));

        Ok(Self {
            pool: pool_arc,
            repository: repo,
        })
    }
}
