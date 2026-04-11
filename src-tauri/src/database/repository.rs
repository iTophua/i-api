use super::{ConnectionPool, PooledConn};
use crate::models::{AppState, Collection, Environment, Folder, History, HttpRequest};
use rusqlite::Result as SqliteResult;
use std::sync::Arc;

/// 数据库仓库层 - 实现所有 CRUD 操作
pub struct DatabaseRepository {
    pool: Arc<ConnectionPool>,
}

impl DatabaseRepository {
    pub fn new(pool: Arc<ConnectionPool>) -> Self {
        Self { pool }
    }

    fn get_conn(&self) -> SqliteResult<PooledConn> {
        self.pool.get()
    }

    // ==================== History 操作 ====================

    pub fn add_history(&self, history: &History, limit: i64) -> SqliteResult<()> {
        let conn = self.get_conn()?;
        let tx = conn.unchecked_transaction()?;

        let params_json = history.params.as_ref().map(|p| {
            serde_json::to_string(p).unwrap_or_default()
        });
        let headers_json = history.headers.as_ref().map(|h| {
            serde_json::to_string(h).unwrap_or_default()
        });
        let body_json = history.body.as_ref().map(|b| {
            serde_json::to_string(b).unwrap_or_default()
        });
        let auth_json = history.auth.as_ref().map(|a| {
            serde_json::to_string(a).unwrap_or_default()
        });

        tx.execute(
            "INSERT INTO history (id, request_id, method, url, status, response_time, response_size, params, headers, body, auth, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            (
                &history.id,
                &history.request_id,
                &history.method,
                &history.url,
                &history.status,
                &history.response_time,
                &history.response_size,
                &params_json,
                &headers_json,
                &body_json,
                &auth_json,
                &history.created_at,
            ),
        )?;

        tx.execute(
            "DELETE FROM history WHERE id NOT IN (
                SELECT id FROM history ORDER BY created_at DESC LIMIT ?
            )",
            [limit],
        )?;

        tx.commit()
    }

    pub fn get_recent_history(&self, limit: i64) -> SqliteResult<Vec<History>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, request_id, method, url, status, response_time, response_size, params, headers, body, auth, created_at
             FROM history ORDER BY created_at DESC LIMIT ?",
        )?;

        let histories = stmt.query_map([limit], |row| {
            let params_json: Option<String> = row.get(7)?;
            let headers_json: Option<String> = row.get(8)?;
            let body_json: Option<String> = row.get(9)?;
            let auth_json: Option<String> = row.get(10)?;

            Ok(History {
                id: row.get(0)?,
                request_id: row.get(1)?,
                method: row.get(2)?,
                url: row.get(3)?,
                status: row.get(4)?,
                response_time: row.get(5)?,
                response_size: row.get(6)?,
                params: params_json.and_then(|s| serde_json::from_str(&s).ok()),
                headers: headers_json.and_then(|s| serde_json::from_str(&s).ok()),
                body: body_json.and_then(|s| serde_json::from_str(&s).ok()),
                auth: auth_json.and_then(|s| serde_json::from_str(&s).ok()),
                created_at: row.get(11)?,
            })
        })?;

        let mut result = Vec::new();
        for history in histories {
            result.push(history?);
        }
        Ok(result)
    }

    pub fn delete_history(&self, id: &str) -> SqliteResult<()> {
        let conn = self.get_conn()?;
        conn.execute("DELETE FROM history WHERE id = ?", [id])?;
        Ok(())
    }

    pub fn clear_history(&self) -> SqliteResult<()> {
        let conn = self.get_conn()?;
        conn.execute("DELETE FROM history", [])?;
        Ok(())
    }

    // ==================== Request 操作 ====================

    pub fn save_request(&self, request: &HttpRequest) -> SqliteResult<()> {
        let conn = self.get_conn()?;
        let params_json = serde_json::to_string(&request.params).map_err(|e| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("序列化 params 失败: {}", e),
            )))
        })?;
        let headers_json = serde_json::to_string(&request.headers).map_err(|e| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("序列化 headers 失败: {}", e),
            )))
        })?;
        let body_json = serde_json::to_string(&request.body).map_err(|e| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("序列化 body 失败: {}", e),
            )))
        })?;
        let auth_json = serde_json::to_string(&request.auth).map_err(|e| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("序列化 auth 失败: {}", e),
            )))
        })?;

        conn.execute(
            "INSERT OR REPLACE INTO requests 
             (id, name, description, method, url, params, headers, body, auth, pre_script, post_script, 
              collection_id, folder_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            (
                &request.id,
                &request.name,
                &request.description,
                &request.method,
                &request.url,
                params_json,
                headers_json,
                body_json,
                auth_json,
                &request.pre_script,
                &request.post_script,
                &request.collection_id,
                &request.folder_id,
                &request.created_at,
                &request.updated_at,
            ),
        )?;
        Ok(())
    }

    pub fn get_all_requests(&self) -> SqliteResult<Vec<HttpRequest>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, method, url, params, headers, body, auth, pre_script, post_script,
                    collection_id, folder_id, created_at, updated_at
             FROM requests ORDER BY updated_at DESC"
        )?;

        let requests = stmt.query_map([], |row| {
            Ok(HttpRequest {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                method: row.get(3)?,
                url: row.get(4)?,
                params: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                headers: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
                body: serde_json::from_str(&row.get::<_, Option<String>>(7)?.unwrap_or_default())
                    .ok(),
                auth: serde_json::from_str(&row.get::<_, Option<String>>(8)?.unwrap_or_default())
                    .ok(),
                pre_script: row.get(9)?,
                post_script: row.get(10)?,
                return_bytes: None,
                timeout: None,
                collection_id: row.get(11)?,
                folder_id: row.get(12)?,
                proxy: None,
                follow_redirects: None,
                verify_ssl: None,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        })?;

        let mut result = Vec::new();
        for request in requests {
            result.push(request?);
        }
        Ok(result)
    }

    pub fn delete_request(&self, id: &str) -> SqliteResult<()> {
        let conn = self.get_conn()?;
        conn.execute("DELETE FROM requests WHERE id = ?", [id])?;
        Ok(())
    }

    pub fn get_request_by_id(&self, id: &str) -> SqliteResult<Option<HttpRequest>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, method, url, params, headers, body, auth, pre_script, post_script,
                    collection_id, folder_id, created_at, updated_at
             FROM requests WHERE id = ?"
        )?;

        let mut rows = stmt.query_map([id], |row| {
            Ok(HttpRequest {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                method: row.get(3)?,
                url: row.get(4)?,
                params: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                headers: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
                body: serde_json::from_str(&row.get::<_, Option<String>>(7)?.unwrap_or_default())
                    .ok(),
                auth: serde_json::from_str(&row.get::<_, Option<String>>(8)?.unwrap_or_default())
                    .ok(),
                pre_script: row.get(9)?,
                post_script: row.get(10)?,
                return_bytes: None,
                timeout: None,
                collection_id: row.get(11)?,
                folder_id: row.get(12)?,
                proxy: None,
                follow_redirects: None,
                verify_ssl: None,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        })?;

        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    // ==================== Temporary Request 操作 ====================

    pub fn save_temporary_request(&self, id: &str, tab_data: &str) -> SqliteResult<()> {
        let conn = self.get_conn()?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT OR REPLACE INTO open_tabs (id, tab_data, active_tab_id, updated_at)
             VALUES (?1, ?2, ?3, ?4)",
            (id, tab_data, id, &now),
        )?;
        Ok(())
    }

    pub fn get_temporary_request(&self, id: &str) -> SqliteResult<Option<String>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT tab_data FROM open_tabs WHERE id = ?")?;
        let result = stmt.query_row([id], |row| row.get(0))?;
        Ok(Some(result))
    }

    pub fn clear_temporary_request(&self, id: &str) -> SqliteResult<()> {
        let conn = self.get_conn()?;
        conn.execute("DELETE FROM open_tabs WHERE id = ?", [id])?;
        Ok(())
    }

    // ==================== Collection 操作 ====================

    pub fn save_collection(&self, collection: &Collection) -> SqliteResult<()> {
        let conn = self.get_conn()?;
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

    pub fn get_all_collections(&self) -> SqliteResult<Vec<Collection>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, created_at, updated_at FROM collections ORDER BY updated_at DESC"
        )?;

        let collections = stmt.query_map([], |row| {
            Ok(Collection {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?;

        let mut result = Vec::new();
        for collection in collections {
            result.push(collection?);
        }
        Ok(result)
    }

    pub fn delete_collection(&self, id: &str) -> SqliteResult<()> {
        let conn = self.get_conn()?;
        let tx = conn.unchecked_transaction()?;

        // 先删除关联的 requests
        tx.execute("DELETE FROM requests WHERE collection_id = ?", [id])?;
        // 再删除关联的 folders
        tx.execute("DELETE FROM folders WHERE collection_id = ?", [id])?;
        // 最后删除 collection
        tx.execute("DELETE FROM collections WHERE id = ?", [id])?;

        tx.commit()
    }

    // ==================== Folder 操作 ====================

    pub fn save_folder(&self, folder: &Folder) -> SqliteResult<()> {
        let conn = self.get_conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO folders 
             (id, name, description, collection_id, parent_folder_id, created_at, updated_at)
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

    pub fn get_all_folders(&self) -> SqliteResult<Vec<Folder>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, collection_id, parent_folder_id, created_at, updated_at
             FROM folders ORDER BY updated_at DESC",
        )?;

        let folders = stmt.query_map([], |row| {
            Ok(Folder {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                collection_id: row.get(3)?,
                parent_folder_id: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;

        let mut result = Vec::new();
        for folder in folders {
            result.push(folder?);
        }
        Ok(result)
    }

    pub fn delete_folder(&self, id: &str) -> SqliteResult<()> {
        let conn = self.get_conn()?;
        conn.execute("DELETE FROM folders WHERE id = ?", [id])?;
        Ok(())
    }

    // ==================== Environment 操作 ====================

    pub fn save_environment(&self, env: &Environment) -> SqliteResult<()> {
        let conn = self.get_conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO environments (id, name, variables, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                &env.id,
                &env.name,
                &serde_json::to_string(&env.variables).unwrap_or_default(),
                &env.created_at,
                &env.updated_at,
            ),
        )?;
        Ok(())
    }

    pub fn get_all_environments(&self) -> SqliteResult<Vec<Environment>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, variables, created_at, updated_at FROM environments ORDER BY updated_at DESC"
        )?;

        let envs = stmt.query_map([], |row| {
            Ok(Environment {
                id: row.get(0)?,
                name: row.get(1)?,
                variables: serde_json::from_str(&row.get::<_, String>(2)?).unwrap_or_default(),
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?;

        let mut result = Vec::new();
        for env in envs {
            result.push(env?);
        }
        Ok(result)
    }

    pub fn delete_environment(&self, id: &str) -> SqliteResult<()> {
        let conn = self.get_conn()?;
        conn.execute("DELETE FROM environments WHERE id = ?", [id])?;
        Ok(())
    }

    // ==================== Settings 操作 ====================

    pub fn save_setting(&self, key: &str, value: &str) -> SqliteResult<()> {
        let conn = self.get_conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            (key, value),
        )?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> SqliteResult<Option<String>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?")?;
        let result = stmt.query_row([key], |row| row.get(0));
        match result {
            Ok(val) => Ok(Some(val)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    // ==================== AppState 操作 ====================

    pub fn save_app_state(&self, state: &AppState) -> SqliteResult<()> {
        let conn = self.get_conn()?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT OR REPLACE INTO app_state 
             (id, current_request_id, current_environment_id, sidebar_collapsed, request_panel_height, 
              response_panel_height, updated_at)
             VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6)",
            (
                &state.current_request_id,
                &state.current_environment_id,
                &state.sidebar_collapsed,
                &state.request_panel_height,
                &state.response_panel_height,
                &now,
            ),
        )?;
        Ok(())
    }

    pub fn get_app_state(&self) -> SqliteResult<Option<AppState>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT current_request_id, current_environment_id, sidebar_collapsed, request_panel_height,
                    response_panel_height
             FROM app_state WHERE id = 1"
        )?;

        let mut rows = stmt.query_map([], |row| {
            Ok(AppState {
                current_request_id: row.get(0)?,
                current_environment_id: row.get(1)?,
                sidebar_collapsed: row.get(2)?,
                request_panel_height: row.get(3)?,
                response_panel_height: row.get(4)?,
            })
        })?;

        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    // ==================== 新增方法 ====================

    pub fn rename_collection(&self, id: &str, name: &str, updated_at: &str) -> SqliteResult<()> {
        let conn = self.get_conn()?;
        conn.execute(
            "UPDATE collections SET name = ?1, updated_at = ?2 WHERE id = ?3",
            (name, updated_at, id),
        )?;
        Ok(())
    }

    pub fn rename_request(&self, id: &str, name: &str, updated_at: &str) -> SqliteResult<()> {
        let conn = self.get_conn()?;
        conn.execute(
            "UPDATE requests SET name = ?1, updated_at = ?2 WHERE id = ?3",
            (name, updated_at, id),
        )?;
        Ok(())
    }

    pub fn get_requests_by_collection(
        &self,
        collection_id: &str,
    ) -> SqliteResult<Vec<HttpRequest>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, method, url, params, headers, body, auth, pre_script, post_script,
                    collection_id, folder_id, created_at, updated_at
             FROM requests WHERE collection_id = ? ORDER BY updated_at DESC"
        )?;

        let requests = stmt.query_map([collection_id], |row| {
            Ok(HttpRequest {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                method: row.get(3)?,
                url: row.get(4)?,
                params: serde_json::from_str(&row.get::<_, String>(5)?).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        5,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
                    )
                })?,
                headers: serde_json::from_str(&row.get::<_, String>(6)?).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
                    )
                })?,
                body: serde_json::from_str(&row.get::<_, String>(7)?).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        7,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
                    )
                })?,
                auth: serde_json::from_str(&row.get::<_, String>(8)?).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        8,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
                    )
                })?,
                pre_script: row.get(9)?,
                post_script: row.get(10)?,
                return_bytes: None,
                timeout: None,
                collection_id: row.get(11)?,
                folder_id: row.get(12)?,
                proxy: None,
                follow_redirects: None,
                verify_ssl: None,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        })?;

        let mut result = Vec::new();
        for request in requests {
            result.push(request?);
        }
        Ok(result)
    }

    pub fn save_open_tabs(
        &self,
        id: &str,
        tab_data: &str,
        active_tab_id: &Option<String>,
        updated_at: &str,
    ) -> SqliteResult<()> {
        let conn = self.get_conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO open_tabs (id, tab_data, active_tab_id, updated_at)
             VALUES (?1, ?2, ?3, ?4)",
            (id, tab_data, active_tab_id, updated_at),
        )?;
        Ok(())
    }

    pub fn get_open_tabs(&self, id: &str) -> SqliteResult<Option<(String, Option<String>)>> {
        let conn = self.get_conn()?;
        let mut stmt =
            conn.prepare("SELECT tab_data, active_tab_id FROM open_tabs WHERE id = ?")?;
        let mut rows = stmt.query_map([id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
        })?;
        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    pub fn delete_open_tab(&self, id: &str) -> SqliteResult<()> {
        let conn = self.get_conn()?;
        conn.execute("DELETE FROM open_tabs WHERE id = ?", [id])?;
        Ok(())
    }

    pub fn batch_delete_requests(&self, request_ids: &[String]) -> SqliteResult<()> {
        if request_ids.is_empty() {
            return Ok(());
        }
        let conn = self.get_conn()?;
        let tx = conn.unchecked_transaction()?;
        for id in request_ids {
            tx.execute("DELETE FROM requests WHERE id = ?", [id])?;
        }
        tx.commit()
    }
}
