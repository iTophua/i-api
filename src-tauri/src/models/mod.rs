use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyValuePair {
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestBody {
    #[serde(rename = "mode")]
    pub body_mode: String,
    #[serde(default)]
    pub raw: Option<String>,
    #[serde(default)]
    pub raw_type: Option<String>,
    #[serde(default)]
    pub form_data: Option<Vec<FormData>>,
    #[serde(default)]
    pub urlencoded: Option<Vec<KeyValuePair>>,
    #[serde(default)]
    pub binary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormData {
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(rename = "type")]
    pub form_type: String,
    #[serde(default)]
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    #[serde(rename = "type")]
    pub auth_type: String,
    #[serde(default)]
    pub basic: Option<BasicAuth>,
    #[serde(default)]
    pub bearer: Option<BearerAuth>,
    #[serde(default)]
    pub apikey: Option<ApiKeyAuth>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearerAuth {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyAuth {
    pub key: String,
    pub value: String,
    pub add_to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpRequest {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub method: String,
    pub url: String,
    #[serde(default)]
    pub params: Vec<KeyValuePair>,
    #[serde(default)]
    pub headers: Vec<KeyValuePair>,
    #[serde(default)]
    pub body: Option<RequestBody>,
    #[serde(default)]
    pub auth: Option<AuthConfig>,
    #[serde(default)]
    pub pre_script: Option<String>,
    #[serde(default)]
    pub post_script: Option<String>,
    #[serde(default)]
    pub return_bytes: Option<bool>,
    #[serde(default)]
    pub timeout: Option<u64>,
    #[serde(default)]
    pub collection_id: Option<String>,
    #[serde(default)]
    pub folder_id: Option<String>,
    #[serde(default)]
    pub proxy: Option<ProxyConfig>,
    #[serde(default)]
    pub follow_redirects: Option<bool>,
    #[serde(default)]
    pub verify_ssl: Option<bool>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub cookies: Vec<Cookie>,
    pub body: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body_bytes: Option<Vec<u8>>,
    pub response_time: u64,
    pub response_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cookie {
    pub name: String,
    pub value: String,
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub expires: Option<String>,
    #[serde(default)]
    pub http_only: Option<bool>,
    #[serde(default)]
    pub secure: Option<bool>,
}

impl HttpRequest {
    pub fn new(name: String, method: String, url: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: None,
            method,
            url,
            params: vec![],
            headers: vec![],
            body: None,
            auth: None,
            pre_script: None,
            post_script: None,
            return_bytes: None,
            timeout: None,
            collection_id: None,
            folder_id: None,
            proxy: None,
            follow_redirects: None,
            verify_ssl: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn new_with_data(
        name: String,
        method: String,
        url: String,
        headers: Vec<KeyValuePair>,
        body: Option<RequestBody>,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: None,
            method,
            url,
            params: vec![],
            headers,
            body,
            auth: None,
            pre_script: None,
            post_script: None,
            return_bytes: None,
            timeout: None,
            collection_id: None,
            folder_id: None,
            proxy: None,
            follow_redirects: None,
            verify_ssl: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

impl Default for HttpRequest {
    fn default() -> Self {
        Self::new("未命名请求".to_string(), "GET".to_string(), String::new())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Environment {
    pub id: String,
    pub name: String,
    pub variables: Vec<KeyValuePair>,
    pub created_at: String,
    pub updated_at: String,
}

impl Environment {
    pub fn new(name: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            variables: vec![],
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct History {
    pub id: String,
    pub request_id: Option<String>,
    pub method: String,
    pub url: String,
    pub status: u16,
    pub response_time: u64,
    pub response_size: usize,
    pub created_at: String,
}

impl History {
    pub fn new(
        method: String,
        url: String,
        status: u16,
        response_time: u64,
        response_size: usize,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            request_id: None,
            method,
            url,
            status,
            response_time,
            response_size,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    /// 与前端 `currentTabId` 兼容（历史命名）
    #[serde(alias = "currentTabId")]
    pub current_request_id: Option<String>,
    pub current_environment_id: Option<String>,
    pub sidebar_collapsed: bool,
    pub request_panel_height: Option<i32>,
    pub response_panel_height: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Collection {
    pub fn new(name: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub collection_id: String,
    #[serde(default)]
    pub parent_folder_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Folder {
    pub fn new(name: String, collection_id: String, parent_folder_id: Option<String>) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: None,
            collection_id,
            parent_folder_id,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestTab {
    pub id: String,
    pub request_id: String,
    pub collection_id: Option<String>,
    pub is_temporary: bool,
    pub is_dirty: bool,
}
