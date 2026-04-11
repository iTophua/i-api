#![allow(clippy::all)]

mod curl;
mod database;
mod error;
mod http;
mod models;
mod openapi;
mod script;
mod secure_storage;

use database::Database;
use models::{AppState, Collection, Environment, History, HttpRequest, HttpResponse};
use script::{execute_post_request_script, execute_pre_request_script, ScriptContext};
use secure_storage::SecureStorage;
use std::sync::Arc;
use tauri::{Emitter, Manager};

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
            send_http_request_stream,
            cancel_http_request,
            save_request,
            get_all_requests,
            delete_request,
            get_recent_history,
            delete_history,
            clear_history,
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
            batch_delete_requests,
            save_open_tabs,
            get_open_tabs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn send_http_request(
    request: HttpRequest,
    history_limit: Option<i64>,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<HttpResponse, String> {
    let response = http::send_request(request.clone()).await?;

    let history = History::with_request_data(
        request.method,
        request.url,
        response.status,
        response.response_time,
        response.response_size as usize,
        Some(request.params),
        Some(request.headers),
        request.body,
        request.auth,
    );
    let limit = history_limit.unwrap_or(100);
    let _ = db.repository.add_history(&history, limit);

    Ok(response)
}

#[tauri::command]
async fn send_http_request_stream(
    request: HttpRequest,
    window: tauri::Window,
    event_id: String,
    history_limit: Option<i64>,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<(), String> {
    let start = std::time::Instant::now();
    let request_clone = request.clone();
    let window_clone = window.clone();
    let event_id_clone = event_id.clone();

    // 发送响应头事件
    let _ = window.emit(
        &format!("{}-headers", event_id),
        serde_json::json!({
            "status": "started",
            "timestamp": start.elapsed().as_millis() as u64
        }),
    );

    match http::send_request_stream(request, move |chunk, headers, status, elapsed| {
        let win = window_clone.clone();
        let eid = event_id_clone.clone();

        // 发送响应块事件（同步）
        let _ = win.emit(
            &format!("{}-chunk", eid),
            serde_json::json!({
                "chunk": chunk,
                "headers": headers,
                "status": status,
                "responseTime": elapsed.elapsed().as_millis() as u64,
                "isFinal": false
            }),
        );
    })
    .await
    {
        Ok((full_body, headers, status, response_time, response_size)) => {
            // 发送完成事件
            let _ = window.emit(
                &format!("{}-complete", event_id),
                serde_json::json!({
                    "body": full_body,
                    "headers": headers,
                    "status": status,
                    "responseTime": response_time,
                    "responseSize": response_size,
                    "isFinal": true
                }),
            );

            // 保存历史
            let history = History::with_request_data(
                request_clone.method,
                request_clone.url,
                status,
                response_time,
                response_size,
                Some(request_clone.params),
                Some(request_clone.headers),
                request_clone.body,
                request_clone.auth,
            );
            let limit = history_limit.unwrap_or(100);
            let _ = db.repository.add_history(&history, limit);
        }
        Err(e) => {
            let _ = window.emit(
                &format!("{}-error", event_id),
                serde_json::json!({
                    "error": e,
                    "isFinal": true
                }),
            );
        }
    }

    Ok(())
}

#[tauri::command]
async fn cancel_http_request(request_id: String) -> Result<bool, String> {
    Ok(http::cancel_request(&request_id).await)
}

#[tauri::command]
fn save_request(request: HttpRequest, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    db.repository
        .save_request(&request)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_all_requests(db: tauri::State<'_, Arc<Database>>) -> Result<Vec<HttpRequest>, String> {
    db.repository.get_all_requests().map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_request(id: String, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    db.repository.delete_request(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_recent_history(
    limit: usize,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<Vec<History>, String> {
    db.repository
        .get_recent_history(limit as i64)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_history(id: String, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    db.repository.delete_history(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn clear_history(db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    db.repository.clear_history().map_err(|e| e.to_string())
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

    let method = request
        .get("method")
        .and_then(|m| m.as_str())
        .unwrap_or("GET")
        .to_uppercase();

    let url = request
        .get("url")
        .and_then(|u| {
            if u.is_string() {
                u.as_str().map(|s| s.to_string())
            } else {
                u.get("raw").and_then(|r| r.as_str()).map(|s| s.to_string())
            }
        })
        .unwrap_or_default();

    let headers = request
        .get("header")
        .and_then(|h| h.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|h| {
                    Some(crate::models::KeyValuePair {
                        key: h.get("key")?.as_str()?.to_string(),
                        value: h.get("value")?.as_str()?.to_string(),
                        description: None,
                        enabled: !h.get("disabled").and_then(|d| d.as_bool()).unwrap_or(false),
                    })
                })
                .collect()
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

    Some(HttpRequest::new_with_data(name, method, url, headers, body))
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
fn save_temporary_request(
    request: HttpRequest,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<(), String> {
    let tab_data = serde_json::to_string(&request).map_err(|e| e.to_string())?;
    db.repository
        .save_temporary_request(&request.id, &tab_data)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_temporary_request(
    db: tauri::State<'_, Arc<Database>>,
) -> Result<Option<HttpRequest>, String> {
    let tab_data = db
        .repository
        .get_open_tabs("temporary")
        .map_err(|e| e.to_string())?;

    match tab_data {
        Some((data, _)) => {
            let request: HttpRequest =
                serde_json::from_str(&data).map_err(|e| format!("解析临时请求失败: {}", e))?;
            Ok(Some(request))
        }
        None => Ok(None),
    }
}

#[tauri::command]
fn clear_temporary_request(db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    db.repository
        .delete_open_tab("temporary")
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn save_app_state(state: AppState, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    db.repository
        .save_app_state(&state)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_app_state(db: tauri::State<'_, Arc<Database>>) -> Result<AppState, String> {
    match db.repository.get_app_state() {
        Ok(Some(state)) => Ok(state),
        Ok(None) => Ok(AppState {
            current_request_id: None,
            current_environment_id: None,
            sidebar_collapsed: false,
            request_panel_height: None,
            response_panel_height: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn save_environment(
    environment: Environment,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<(), String> {
    db.repository
        .save_environment(&environment)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_all_environments(db: tauri::State<'_, Arc<Database>>) -> Result<Vec<Environment>, String> {
    db.repository
        .get_all_environments()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_environment(id: String, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    db.repository
        .delete_environment(&id)
        .map_err(|e| e.to_string())
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
    SecureStorage::retrieve_credential(&resource_id, &secret_type, &key).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_secret(resource_id: String, secret_type: String, key: String) -> Result<(), String> {
    SecureStorage::delete_credential(&resource_id, &secret_type, &key).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_collection(
    collection: Collection,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<(), String> {
    db.repository
        .save_collection(&collection)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_all_collections(db: tauri::State<'_, Arc<Database>>) -> Result<Vec<Collection>, String> {
    db.repository
        .get_all_collections()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_collection(id: String, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    db.repository
        .delete_collection(&id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn rename_collection(
    id: String,
    name: String,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    db.repository
        .rename_collection(&id, &name, &now)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn save_request_to_collection(
    request: HttpRequest,
    collection_id: String,
    folder_id: Option<String>,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<(), String> {
    // 直接保存请求，关联 collection_id 和 folder_id
    let mut req = request;
    req.collection_id = Some(collection_id);
    req.folder_id = folder_id;
    db.repository.save_request(&req).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_requests_by_collection(
    collection_id: String,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<Vec<HttpRequest>, String> {
    db.repository
        .get_requests_by_collection(&collection_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn rename_request(
    id: String,
    name: String,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    db.repository
        .rename_request(&id, &name, &now)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_request_from_collection(
    id: String,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<(), String> {
    db.repository.delete_request(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn batch_delete_requests(
    request_ids: Vec<String>,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<(), String> {
    db.repository
        .batch_delete_requests(&request_ids)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn save_open_tabs(
    tabs_data: String,
    active_tab_id: Option<String>,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    db.repository
        .save_open_tabs("current_tabs", &tabs_data, &active_tab_id, &now)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_open_tabs(
    db: tauri::State<'_, Arc<Database>>,
) -> Result<Option<(String, Option<String>)>, String> {
    db.repository
        .get_open_tabs("current_tabs")
        .map_err(|e| e.to_string())
}
