mod curl;
mod database;
mod error;
mod http;
mod models;
mod openapi;
mod script;
mod secure_storage;

use database::Database;
use error::IApiError;
use models::{AppState, Collection, HttpRequest, HttpResponse, History, Environment, Folder};
use script::{ScriptContext, execute_pre_request_script, execute_post_request_script};
use secure_storage::SecureStorage;
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("无法获取应用数据目录");
            let db = Database::new(app_data_dir).expect("无法初始化数据库");
            app.manage(Arc::new(db));

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            send_http_request,
            cancel_http_request,
            save_request,
            get_all_requests,
            delete_request,
            get_recent_history,
            execute_script,
            parse_curl_command,
            export_postman_collection,
            import_postman_collection,
            import_openapi,
            import_har,
            save_temporary_request,
            get_temporary_request,
            clear_temporary_request,
            save_app_state,
            get_app_state,
            save_environment,
            get_all_environments,
            delete_environment,
            store_secret,
            retrieve_secret,
            delete_secret,
            save_collection,
            get_all_collections,
            delete_collection,
            rename_collection,
            save_request_to_collection,
            get_requests_by_collection,
            rename_request,
            delete_request_from_collection,
            save_open_tabs,
            get_open_tabs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn send_http_request(request: HttpRequest, history_limit: Option<i64>, db: tauri::State<'_, Arc<Database>>) -> Result<HttpResponse, String> {
    let response = http::send_request(request.clone()).await?;

    let history = History::new(
        request.method,
        request.url,
        response.status,
        response.response_time,
        response.response_size as usize,
    );
    let limit = history_limit.unwrap_or(100);
    db.add_history(&history, limit).map_err(|e| e.to_string())?;

    Ok(response)
}

#[tauri::command]
async fn cancel_http_request(request_id: String) -> Result<bool, String> {
    Ok(http::cancel_request(&request_id).await)
}

#[tauri::command]
fn save_request(request: HttpRequest, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    db.save_request(&request).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_all_requests(db: tauri::State<'_, Arc<Database>>) -> Result<Vec<HttpRequest>, String> {
    db.get_all_requests().map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_request(id: String, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    db.delete_request(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_recent_history(limit: usize, db: tauri::State<'_, Arc<Database>>) -> Result<Vec<History>, String> {
    db.get_recent_history(limit).map_err(|e| e.to_string())
}

#[tauri::command]
fn execute_script(
    script: String,
    script_type: String,
    request: HttpRequest,
    response: Option<HttpResponse>,
    environment: std::collections::HashMap<String, String>,
) -> Result<ScriptContext, String> {
    if script_type == "pre" {
        let mut req = request;
        execute_pre_request_script(&script, &mut req, &environment)
    } else if script_type == "post" {
        let resp = response.ok_or("后置脚本需要响应数据")?;
        execute_post_request_script(&script, &request, &resp, &environment)
    } else {
        Err("无效的脚本类型".to_string())
    }
}

#[tauri::command]
fn parse_curl_command(curl: String) -> Result<HttpRequest, String> {
    curl::parse_curl(&curl)
}

#[tauri::command]
fn export_postman_collection(requests: Vec<HttpRequest>) -> Result<serde_json::Value, String> {
    let items: Vec<serde_json::Value> = requests
        .iter()
        .map(|req| {
            serde_json::json!({
                "name": req.name,
                "request": {
                    "method": req.method,
                    "url": req.url,
                    "header": req.headers.iter().filter(|h| h.enabled).map(|h| {
                        serde_json::json!({
                            "key": h.key,
                            "value": h.value
                        })
                    }).collect::<Vec<_>>(),
                    "body": req.body.as_ref().map(|b| {
                        serde_json::json!({
                            "mode": b.body_mode,
                            "raw": b.raw
                        })
                    })
                }
            })
        })
        .collect();

    Ok(serde_json::json!({
        "info": {
            "name": "iApi Export",
            "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
        },
        "item": items
    }))
}

#[tauri::command]
fn import_postman_collection(json: String) -> Result<Vec<HttpRequest>, String> {
    let value: serde_json::Value = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    
    let mut requests = Vec::new();
    
    if let Some(items) = value.get("item").and_then(|i| i.as_array()) {
        for item in items {
            if let Some(req) = parse_postman_item(item) {
                requests.push(req);
            }
        }
    }
    
    Ok(requests)
}

fn parse_postman_item(item: &serde_json::Value) -> Option<HttpRequest> {
    let name = item.get("name")?.as_str()?.to_string();
    let request = item.get("request")?;
    
    let method = request.get("method")
        .and_then(|m| m.as_str())
        .unwrap_or("GET")
        .to_uppercase();
    
    let url = request.get("url")
        .and_then(|u| {
            if u.is_string() {
                u.as_str().map(|s| s.to_string())
            } else {
                u.get("raw").and_then(|r| r.as_str()).map(|s| s.to_string())
            }
        })
        .unwrap_or_default();
    
    let headers = request.get("header")
        .and_then(|h| h.as_array())
        .map(|arr| {
            arr.iter().filter_map(|h| {
                Some(crate::models::KeyValuePair {
                    key: h.get("key")?.as_str()?.to_string(),
                    value: h.get("value")?.as_str()?.to_string(),
                    description: None,
                    enabled: !h.get("disabled").and_then(|d| d.as_bool()).unwrap_or(false),
                })
            }).collect()
        })
        .unwrap_or_default();
    
    let body = request.get("body").and_then(|b| {
        let mode = b.get("mode").and_then(|m| m.as_str()).unwrap_or("none");
        let raw = b.get("raw").and_then(|r| r.as_str()).map(|s| s.to_string());
        
        Some(crate::models::RequestBody {
            body_mode: mode.to_string(),
            raw,
            raw_type: Some("json".to_string()),
            form_data: None,
            urlencoded: None,
            binary: None,
        })
    });
    
    Some(HttpRequest::new_with_data(
        name,
        method,
        url,
        headers,
        body,
    ))
}

#[tauri::command]
fn import_openapi(content: String) -> Result<Vec<HttpRequest>, String> {
    let parsed = openapi::parse_openapi(&content)?;
    Ok(parsed.requests)
}

#[tauri::command]
fn import_har(content: String) -> Result<Vec<HttpRequest>, String> {
    openapi::parse_har(&content)
}

#[tauri::command]
fn save_temporary_request(request: HttpRequest, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    db.save_temporary_request(&request).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_temporary_request(db: tauri::State<'_, Arc<Database>>) -> Result<Option<HttpRequest>, String> {
    db.get_temporary_request().map_err(|e| e.to_string())
}

#[tauri::command]
fn clear_temporary_request(db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    db.clear_temporary_request().map_err(|e| e.to_string())
}

#[tauri::command]
fn save_app_state(state: AppState, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    db.save_app_state(&state).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_app_state(db: tauri::State<'_, Arc<Database>>) -> Result<AppState, String> {
    db.get_app_state().map_err(|e| e.to_string())
}

#[tauri::command]
fn save_environment(environment: Environment, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    db.save_environment(&environment).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_all_environments(db: tauri::State<'_, Arc<Database>>) -> Result<Vec<Environment>, String> {
    db.get_all_environments().map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_environment(id: String, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    db.delete_environment(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn store_secret(
    resource_id: String,
    secret_type: String,
    key: String,
    value: String,
) -> Result<(), String> {
    SecureStorage::store_credential(&resource_id, &secret_type, &key, &value)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn retrieve_secret(
    resource_id: String,
    secret_type: String,
    key: String,
) -> Result<Option<String>, String> {
    SecureStorage::retrieve_credential(&resource_id, &secret_type, &key)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_secret(
    resource_id: String,
    secret_type: String,
    key: String,
) -> Result<(), String> {
    SecureStorage::delete_credential(&resource_id, &secret_type, &key)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn save_collection(collection: Collection, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    db.save_collection(&collection).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_all_collections(db: tauri::State<'_, Arc<Database>>) -> Result<Vec<Collection>, String> {
    db.get_all_collections().map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_collection(id: String, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    db.delete_collection(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn rename_collection(id: String, name: String, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    db.rename_collection(&id, &name).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_request_to_collection(
    request: HttpRequest,
    collection_id: String,
    folder_id: Option<String>,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<(), String> {
    db.save_request_to_collection(&request, &collection_id, folder_id.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_requests_by_collection(
    collection_id: String,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<Vec<HttpRequest>, String> {
    db.get_requests_by_collection(&collection_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn rename_request(id: String, name: String, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    db.rename_request(&id, &name).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_request_from_collection(id: String, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    db.delete_request_from_collection(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_open_tabs(tabs_data: String, active_tab_id: Option<String>, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    db.save_open_tabs(&tabs_data, active_tab_id.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_open_tabs(db: tauri::State<'_, Arc<Database>>) -> Result<Option<(String, Option<String>)>, String> {
    db.get_open_tabs().map_err(|e| e.to_string())
}
