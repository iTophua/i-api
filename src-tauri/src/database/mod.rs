use rusqlite::{Connection, Result as SqliteResult};
use std::io;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

pub struct Database {
    conn: Mutex<Connection>,
}

fn lock_conn(conn: &Mutex<Connection>) -> SqliteResult<MutexGuard<'_, Connection>> {
    conn.lock().map_err(|e| {
        rusqlite::Error::ToSqlConversionFailure(Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!("数据库锁异常: {e}"),
        )))
    })
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

        let conn = Connection::open(&db_path).map_err(|e| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!("无法打开数据库 {:?}: {}", db_path, e),
            )))
        })?;

        conn.pragma_update(None, "journal_mode", &"DELETE").ok();

        println!("数据库已成功创建/打开: {:?}", db_path);

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

            CREATE TABLE IF NOT EXISTS temporary_requests (
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
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
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

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn save_request(&self, request: &crate::models::HttpRequest) -> SqliteResult<()> {
        let conn = lock_conn(&self.conn)?;
        let params = serde_json::to_string(&request.params)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let headers = serde_json::to_string(&request.headers)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let body = request
            .body
            .as_ref()
            .map(|b| {
                serde_json::to_string(b)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
            })
            .transpose()?;
        let auth = request
            .auth
            .as_ref()
            .map(|a| {
                serde_json::to_string(a)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
            })
            .transpose()?;

        conn.execute(
            "INSERT OR REPLACE INTO requests (id, name, description, method, url, params, headers, body, auth, pre_script, post_script, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            (
                &request.id,
                &request.name,
                &request.description,
                &request.method,
                &request.url,
                &params,
                &headers,
                &body,
                &auth,
                &request.pre_script,
                &request.post_script,
                &request.created_at,
                &request.updated_at,
            ),
        )?;

        Ok(())
    }

    pub fn get_all_requests(&self) -> SqliteResult<Vec<crate::models::HttpRequest>> {
        let conn = lock_conn(&self.conn)?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, method, url, params, headers, body, auth, pre_script, post_script, created_at, updated_at FROM requests"
        )?;

        let requests = stmt.query_map([], |row| {
            Ok(crate::models::HttpRequest {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                method: row.get(3)?,
                url: row.get(4)?,
                params: {
                    let val = row.get_ref(5)?;
                    match val {
                        rusqlite::types::ValueRef::Null => vec![],
                        _ => serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                    }
                },
                headers: {
                    let val = row.get_ref(6)?;
                    match val {
                        rusqlite::types::ValueRef::Null => vec![],
                        _ => serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
                    }
                },
                body: {
                    let val = row.get_ref(7)?;
                    match val {
                        rusqlite::types::ValueRef::Null => None,
                        _ => serde_json::from_str(&row.get::<_, String>(7)?).ok(),
                    }
                },
                auth: {
                    let val = row.get_ref(8)?;
                    match val {
                        rusqlite::types::ValueRef::Null => None,
                        _ => serde_json::from_str(&row.get::<_, String>(8)?).ok(),
                    }
                },
                pre_script: row.get(9)?,
                post_script: row.get(10)?,
                return_bytes: None,
                timeout: None,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        })?;

        requests.collect()
    }

    pub fn get_all_collections(&self) -> SqliteResult<Vec<crate::models::Collection>> {
        let conn = lock_conn(&self.conn)?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, created_at, updated_at FROM collections ORDER BY created_at"
        )?;

        let collections = stmt.query_map([], |row| {
            Ok(crate::models::Collection {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?;

        collections.collect()
    }

    pub fn save_collection(&self, collection: &crate::models::Collection) -> SqliteResult<()> {
        let conn = lock_conn(&self.conn)?;
        conn.execute(
            "INSERT OR REPLACE INTO collections (id, name, description, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                &collection.id,
                &collection.name,
                &collection.description,
                &collection.created_at,
                &collection.updated_at,
            ),
        )?;
        Ok(())
    }

    pub fn delete_collection(&self, id: &str) -> SqliteResult<()> {
        let conn = lock_conn(&self.conn)?;
        conn.execute("DELETE FROM requests WHERE collection_id = ?1", [id])?;
        conn.execute("DELETE FROM folders WHERE collection_id = ?1", [id])?;
        conn.execute("DELETE FROM collections WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn save_temporary_request(&self, request: &crate::models::HttpRequest) -> SqliteResult<()> {
        let conn = lock_conn(&self.conn)?;
        let params = serde_json::to_string(&request.params)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let headers = serde_json::to_string(&request.headers)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let body = request
            .body
            .as_ref()
            .map(|b| {
                serde_json::to_string(b)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
            })
            .transpose()?;
        let auth = request
            .auth
            .as_ref()
            .map(|a| {
                serde_json::to_string(a)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
            })
            .transpose()?;

        conn.execute("DELETE FROM temporary_requests", [])?;
        conn.execute(
            "INSERT INTO temporary_requests (id, name, description, method, url, params, headers, body, auth, pre_script, post_script, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            (
                &request.id,
                &request.name,
                &request.description,
                &request.method,
                &request.url,
                &params,
                &headers,
                &body,
                &auth,
                &request.pre_script,
                &request.post_script,
                &request.created_at,
                &request.updated_at,
            ),
        )?;
        Ok(())
    }

    pub fn get_temporary_request(&self) -> SqliteResult<Option<crate::models::HttpRequest>> {
        let conn = lock_conn(&self.conn)?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, method, url, params, headers, body, auth, pre_script, post_script, created_at, updated_at FROM temporary_requests LIMIT 1"
        )?;

        let requests = stmt.query_map([], |row| {
            Ok(crate::models::HttpRequest {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                method: row.get(3)?,
                url: row.get(4)?,
                params: {
                    let val = row.get_ref(5)?;
                    match val {
                        rusqlite::types::ValueRef::Null => vec![],
                        _ => serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                    }
                },
                headers: {
                    let val = row.get_ref(6)?;
                    match val {
                        rusqlite::types::ValueRef::Null => vec![],
                        _ => serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
                    }
                },
                body: {
                    let val = row.get_ref(7)?;
                    match val {
                        rusqlite::types::ValueRef::Null => None,
                        _ => serde_json::from_str(&row.get::<_, String>(7)?).ok(),
                    }
                },
                auth: {
                    let val = row.get_ref(8)?;
                    match val {
                        rusqlite::types::ValueRef::Null => None,
                        _ => serde_json::from_str(&row.get::<_, String>(8)?).ok(),
                    }
                },
                pre_script: row.get(9)?,
                post_script: row.get(10)?,
                return_bytes: None,
                timeout: None,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        })?;

        let results: Vec<SqliteResult<crate::models::HttpRequest>> = requests.collect();
        match results.into_iter().next() {
            Some(result) => Ok(Some(result?)),
            None => Ok(None),
        }
    }

    pub fn clear_temporary_request(&self) -> SqliteResult<()> {
        let conn = lock_conn(&self.conn)?;
        conn.execute("DELETE FROM temporary_requests", [])?;
        Ok(())
    }

    pub fn rename_collection(&self, id: &str, new_name: &str) -> SqliteResult<()> {
        let conn = lock_conn(&self.conn)?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE collections SET name = ?1, updated_at = ?2 WHERE id = ?3",
            (new_name, &now, id),
        )?;
        Ok(())
    }

    pub fn save_folder(&self, folder: &crate::models::Folder) -> SqliteResult<()> {
        let conn = lock_conn(&self.conn)?;
        conn.execute(
            "INSERT OR REPLACE INTO folders (id, name, description, collection_id, parent_folder_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (
                &folder.id,
                &folder.name,
                &folder.description,
                &folder.collection_id,
                &folder.parent_folder_id,
                &folder.created_at,
                &folder.updated_at,
            ),
        )?;
        Ok(())
    }

    pub fn get_folders_by_collection(
        &self,
        collection_id: &str,
    ) -> SqliteResult<Vec<crate::models::Folder>> {
        let conn = lock_conn(&self.conn)?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, collection_id, parent_folder_id, created_at, updated_at 
             FROM folders WHERE collection_id = ?1 ORDER BY created_at",
        )?;

        let folders = stmt.query_map([collection_id], |row| {
            Ok(crate::models::Folder {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                collection_id: row.get(3)?,
                parent_folder_id: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;

        folders.collect()
    }

    pub fn delete_folder(&self, id: &str) -> SqliteResult<()> {
        let conn = lock_conn(&self.conn)?;
        conn.execute("DELETE FROM requests WHERE folder_id = ?1", [id])?;
        conn.execute("DELETE FROM folders WHERE parent_folder_id = ?1", [id])?;
        conn.execute("DELETE FROM folders WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn save_request_to_collection(
        &self,
        request: &crate::models::HttpRequest,
        collection_id: &str,
        folder_id: Option<&str>,
    ) -> SqliteResult<()> {
        let conn = lock_conn(&self.conn)?;
        let params = serde_json::to_string(&request.params)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let headers = serde_json::to_string(&request.headers)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let body = request
            .body
            .as_ref()
            .map(|b| {
                serde_json::to_string(b)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
            })
            .transpose()?;
        let auth = request
            .auth
            .as_ref()
            .map(|a| {
                serde_json::to_string(a)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
            })
            .transpose()?;

        conn.execute(
            "INSERT OR REPLACE INTO requests (id, name, description, method, url, params, headers, body, auth, pre_script, post_script, collection_id, folder_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            (
                &request.id,
                &request.name,
                &request.description,
                &request.method,
                &request.url,
                &params,
                &headers,
                &body,
                &auth,
                &request.pre_script,
                &request.post_script,
                collection_id,
                folder_id,
                &request.created_at,
                &request.updated_at,
            ),
        )?;

        Ok(())
    }

    pub fn get_requests_by_collection(
        &self,
        collection_id: &str,
    ) -> SqliteResult<Vec<crate::models::HttpRequest>> {
        let conn = lock_conn(&self.conn)?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, method, url, params, headers, body, auth, pre_script, post_script, created_at, updated_at 
             FROM requests WHERE collection_id = ?1 AND folder_id IS NULL ORDER BY created_at"
        )?;

        let requests = stmt.query_map([collection_id], |row| {
            Ok(crate::models::HttpRequest {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                method: row.get(3)?,
                url: row.get(4)?,
                params: {
                    let val = row.get_ref(5)?;
                    match val {
                        rusqlite::types::ValueRef::Null => vec![],
                        _ => serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                    }
                },
                headers: {
                    let val = row.get_ref(6)?;
                    match val {
                        rusqlite::types::ValueRef::Null => vec![],
                        _ => serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
                    }
                },
                body: {
                    let val = row.get_ref(7)?;
                    match val {
                        rusqlite::types::ValueRef::Null => None,
                        _ => serde_json::from_str(&row.get::<_, String>(7)?).ok(),
                    }
                },
                auth: {
                    let val = row.get_ref(8)?;
                    match val {
                        rusqlite::types::ValueRef::Null => None,
                        _ => serde_json::from_str(&row.get::<_, String>(8)?).ok(),
                    }
                },
                pre_script: row.get(9)?,
                post_script: row.get(10)?,
                return_bytes: None,
                timeout: None,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        })?;

        requests.collect()
    }

    pub fn get_requests_by_folder(
        &self,
        folder_id: &str,
    ) -> SqliteResult<Vec<crate::models::HttpRequest>> {
        let conn = lock_conn(&self.conn)?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, method, url, params, headers, body, auth, pre_script, post_script, created_at, updated_at 
             FROM requests WHERE folder_id = ?1 ORDER BY created_at"
        )?;

        let requests = stmt.query_map([folder_id], |row| {
            Ok(crate::models::HttpRequest {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                method: row.get(3)?,
                url: row.get(4)?,
                params: {
                    let val = row.get_ref(5)?;
                    match val {
                        rusqlite::types::ValueRef::Null => vec![],
                        _ => serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                    }
                },
                headers: {
                    let val = row.get_ref(6)?;
                    match val {
                        rusqlite::types::ValueRef::Null => vec![],
                        _ => serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
                    }
                },
                body: {
                    let val = row.get_ref(7)?;
                    match val {
                        rusqlite::types::ValueRef::Null => None,
                        _ => serde_json::from_str(&row.get::<_, String>(7)?).ok(),
                    }
                },
                auth: {
                    let val = row.get_ref(8)?;
                    match val {
                        rusqlite::types::ValueRef::Null => None,
                        _ => serde_json::from_str(&row.get::<_, String>(8)?).ok(),
                    }
                },
                pre_script: row.get(9)?,
                post_script: row.get(10)?,
                return_bytes: None,
                timeout: None,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        })?;

        requests.collect()
    }

    pub fn add_history(&self, history: &crate::models::History, limit: i64) -> SqliteResult<()> {
        let conn = lock_conn(&self.conn)?;
        conn.execute(
            "INSERT INTO history (id, request_id, method, url, status, response_time, response_size, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            (
                &history.id,
                &history.request_id,
                &history.method,
                &history.url,
                history.status,
                history.response_time,
                history.response_size,
                &history.created_at,
            ),
        )?;

        conn.execute(
            "DELETE FROM history WHERE id NOT IN (SELECT id FROM history ORDER BY created_at DESC LIMIT ?1)",
            [limit],
        )?;

        Ok(())
    }

    pub fn delete_request(&self, id: &str) -> SqliteResult<()> {
        let conn = lock_conn(&self.conn)?;
        conn.execute("DELETE FROM requests WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn get_recent_history(&self, limit: usize) -> SqliteResult<Vec<crate::models::History>> {
        let conn = lock_conn(&self.conn)?;
        let mut stmt = conn.prepare(
            "SELECT id, request_id, method, url, status, response_time, response_size, created_at FROM history ORDER BY created_at DESC LIMIT ?1"
        )?;

        let histories = stmt.query_map([limit as i64], |row| {
            Ok(crate::models::History {
                id: row.get(0)?,
                request_id: row.get(1)?,
                method: row.get(2)?,
                url: row.get(3)?,
                status: row.get(4)?,
                response_time: row.get(5)?,
                response_size: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;

        histories.collect()
    }

    pub fn rename_request(&self, id: &str, new_name: &str) -> SqliteResult<()> {
        let conn = lock_conn(&self.conn)?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE requests SET name = ?1, updated_at = ?2 WHERE id = ?3",
            (new_name, &now, id),
        )?;
        Ok(())
    }

    pub fn delete_request_from_collection(&self, id: &str) -> SqliteResult<()> {
        let conn = lock_conn(&self.conn)?;
        conn.execute("DELETE FROM requests WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn save_open_tabs(&self, tabs_data: &str, active_tab_id: Option<&str>) -> SqliteResult<()> {
        let conn = lock_conn(&self.conn)?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute("DELETE FROM open_tabs", [])?;
        conn.execute(
            "INSERT INTO open_tabs (id, tab_data, active_tab_id, updated_at) VALUES (?1, ?2, ?3, ?4)",
            ("default", tabs_data, active_tab_id, &now),
        )?;
        Ok(())
    }

    pub fn get_open_tabs(&self) -> SqliteResult<Option<(String, Option<String>)>> {
        let conn = lock_conn(&self.conn)?;
        let mut stmt = conn.prepare("SELECT tab_data, active_tab_id FROM open_tabs LIMIT 1")?;
        let result = stmt.query_row([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
        });
        match result {
            Ok(data) => Ok(Some(data)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn save_environment(&self, environment: &crate::models::Environment) -> SqliteResult<()> {
        let conn = lock_conn(&self.conn)?;
        let variables = serde_json::to_string(&environment.variables)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        conn.execute(
            "INSERT OR REPLACE INTO environments (id, name, variables, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                &environment.id,
                &environment.name,
                &variables,
                &environment.created_at,
                &environment.updated_at,
            ),
        )?;
        Ok(())
    }

    pub fn get_all_environments(&self) -> SqliteResult<Vec<crate::models::Environment>> {
        let conn = lock_conn(&self.conn)?;
        let mut stmt = conn.prepare(
            "SELECT id, name, variables, created_at, updated_at FROM environments ORDER BY created_at"
        )?;

        let envs = stmt.query_map([], |row| {
            Ok(crate::models::Environment {
                id: row.get(0)?,
                name: row.get(1)?,
                variables: serde_json::from_str(&row.get::<_, String>(2)?).unwrap_or_default(),
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?;

        envs.collect()
    }

    pub fn delete_environment(&self, id: &str) -> SqliteResult<()> {
        let conn = lock_conn(&self.conn)?;
        conn.execute("DELETE FROM environments WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn save_app_state(&self, state: &crate::models::AppState) -> SqliteResult<()> {
        let conn = lock_conn(&self.conn)?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT OR REPLACE INTO app_state (id, current_request_id, current_environment_id, sidebar_collapsed, request_panel_height, response_panel_height, updated_at)
             VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6)",
            (
                &state.current_request_id,
                &state.current_environment_id,
                state.sidebar_collapsed as i32,
                state.request_panel_height,
                state.response_panel_height,
                &now,
            ),
        )?;
        Ok(())
    }

    pub fn get_app_state(&self) -> SqliteResult<crate::models::AppState> {
        let conn = lock_conn(&self.conn)?;
        let mut stmt = conn.prepare(
            "SELECT current_request_id, current_environment_id, sidebar_collapsed, request_panel_height, response_panel_height FROM app_state WHERE id = 1"
        )?;

        let result = stmt.query_row([], |row| {
            Ok(crate::models::AppState {
                current_request_id: row.get(0)?,
                current_environment_id: row.get(1)?,
                sidebar_collapsed: row.get::<_, i32>(2)? != 0,
                request_panel_height: row.get(3)?,
                response_panel_height: row.get(4)?,
            })
        });

        match result {
            Ok(state) => Ok(state),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(crate::models::AppState::default()),
            Err(e) => Err(e),
        }
    }
}
