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
use models::{AppState, Collection, Environment, Folder, History, HttpRequest, HttpResponse};
use script::{execute_post_request_script, execute_pre_request_script, ScriptContext};
use secure_storage::SecureStorage;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("无法获取应用数据目录");
            let db_file = if cfg!(debug_assertions) {
                "iapi_dev.db"
            } else {
                "iapi.db"
            };
            let db = Database::new_with_path(app_data_dir, db_file).expect("无法初始化数据库");
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
            save_folder,
            get_all_folders,
            delete_folder,
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
    mut request: HttpRequest,
    environment: Option<std::collections::HashMap<String, String>>,
    history_limit: Option<i64>,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<HttpResponse, String> {
    let env = environment.unwrap_or_default();
    let mut all_script_variables = std::collections::HashMap::new();

    // 1. 执行前置脚本（可能修改 request.url / request.method，并产生变量）
    let pre_script = request.pre_script.clone();
    if let Some(pre_script) = &pre_script {
        if !pre_script.trim().is_empty() {
            match script::execute_pre_request_script(pre_script, &mut request, &env) {
                Ok(ctx) => {
                    all_script_variables.extend(ctx.variables);
                }
                Err(e) => {
                    eprintln!("前置脚本执行失败: {}", e);
                }
            }
        }
    }

    // 2. 发送 HTTP 请求（使用脚本可能修改过的 request）
    let mut response = http::send_request(request.clone()).await?;

    // 3. 执行后置脚本（断言 + 变量提取）
    let post_script = request.post_script.clone();
    if let Some(post_script) = &post_script {
        if !post_script.trim().is_empty() {
            match script::execute_post_request_script(post_script, &request, &response, &env) {
                Ok(ctx) => {
                    // 回传测试结果
                    if !ctx.test_results.is_empty() {
                        response.test_results = Some(ctx.test_results);
                    }
                    // 合并 post 脚本产生的变量
                    all_script_variables.extend(ctx.variables);
                }
                Err(e) => {
                    eprintln!("后置脚本执行失败: {}", e);
                }
            }
        }
    }

    // 4. 回传脚本产生的变量（供前端写入 runtimeVariables，实现跨请求传递）
    if !all_script_variables.is_empty() {
        response.script_variables = Some(all_script_variables);
    }

    // 5. 保存历史（用脚本修改后的 request，反映实际发送内容）
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

/// 导出范围树中的文件夹节点（前端已按勾选过滤，空的 folders/requests 直接省略）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportFolderNode {
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    folders: Vec<ExportFolderNode>,
    #[serde(default)]
    requests: Vec<HttpRequest>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportCollectionNode {
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    folders: Vec<ExportFolderNode>,
    #[serde(default)]
    requests: Vec<HttpRequest>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportOptions {
    #[serde(default = "export_default_true")]
    include_params: bool,
    #[serde(default = "export_default_true")]
    include_auth: bool,
    #[serde(default = "export_default_true")]
    include_scripts: bool,
    #[serde(default = "export_default_true")]
    include_description: bool,
}

fn export_default_true() -> bool {
    true
}

fn export_enabled_pairs(pairs: &[models::KeyValuePair]) -> Vec<serde_json::Value> {
    pairs
        .iter()
        .filter(|p| p.enabled)
        .map(|p| serde_json::json!({ "key": p.key, "value": p.value }))
        .collect()
}

/// URL 已自带 query 时原样返回，否则把 params 表格拼进 raw，保证 Postman 端可读
fn export_raw_url(url: &str, params: &[models::KeyValuePair]) -> String {
    if url.contains('?') {
        return url.to_string();
    }
    let query: Vec<String> = params
        .iter()
        .filter(|p| p.enabled && !p.key.is_empty())
        .map(|p| format!("{}={}", p.key, p.value))
        .collect();
    if query.is_empty() {
        url.to_string()
    } else {
        format!("{}?{}", url, query.join("&"))
    }
}

fn export_request_body(body: &models::RequestBody) -> serde_json::Value {
    match body.body_mode.as_str() {
        "raw" => {
            let lang = body.raw_type.as_deref().unwrap_or("text");
            serde_json::json!({
                "mode": "raw",
                "raw": body.raw,
                "options": { "raw": { "language": lang } }
            })
        }
        "form-data" => {
            let formdata: Vec<serde_json::Value> = body
                .form_data
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .filter(|f| f.enabled)
                .map(|f| {
                    let mut item = serde_json::json!({
                        "key": f.key,
                        "type": f.form_type,
                        "disabled": false
                    });
                    if f.form_type == "file" {
                        item["src"] = serde_json::json!(f.file_path);
                    } else {
                        item["value"] = serde_json::json!(f.value);
                    }
                    item
                })
                .collect();
            serde_json::json!({ "mode": "formdata", "formdata": formdata })
        }
        "urlencoded" => {
            let urlencoded: Vec<serde_json::Value> = body
                .urlencoded
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .filter(|p| p.enabled)
                .map(|p| serde_json::json!({ "key": p.key, "value": p.value, "disabled": false }))
                .collect();
            serde_json::json!({ "mode": "urlencoded", "urlencoded": urlencoded })
        }
        "binary" => serde_json::json!({
            "mode": "file",
            "file": { "src": body.binary }
        }),
        _ => serde_json::Value::Null,
    }
}

fn export_auth(auth: &models::AuthConfig) -> serde_json::Value {
    match auth.auth_type.as_str() {
        "basic" => auth.basic.as_ref().map(|a| {
            serde_json::json!({
                "type": "basic",
                "basic": [
                    { "key": "username", "value": a.username },
                    { "key": "password", "value": a.password }
                ]
            })
        }),
        "bearer" => auth.bearer.as_ref().map(|a| {
            serde_json::json!({
                "type": "bearer",
                "bearer": [{ "key": "token", "value": a.token }]
            })
        }),
        "apikey" => auth.apikey.as_ref().map(|a| {
            serde_json::json!({
                "type": "apikey",
                "apikey": [
                    { "key": "key", "value": a.key },
                    { "key": "value", "value": a.value },
                    { "key": "in", "value": a.add_to }
                ]
            })
        }),
        _ => None,
    }
    .unwrap_or(serde_json::Value::Null)
}

fn export_script_event(listen: &str, script: &str) -> serde_json::Value {
    let exec: Vec<&str> = script.lines().collect();
    serde_json::json!({
        "listen": listen,
        "script": { "type": "text/javascript", "exec": exec }
    })
}

fn export_request_item(req: &HttpRequest, options: &ExportOptions) -> serde_json::Value {
    let mut url = serde_json::json!(req.url);
    if options.include_params {
        let query = export_enabled_pairs(&req.params);
        if !query.is_empty() {
            url = serde_json::json!({
                "raw": export_raw_url(&req.url, &req.params),
                "query": query
            });
        }
    }

    let headers: Vec<serde_json::Value> = req
        .headers
        .iter()
        .filter(|h| h.enabled)
        .map(|h| serde_json::json!({ "key": h.key, "value": h.value, "disabled": false }))
        .collect();

    let mut request = serde_json::json!({
        "method": req.method,
        "url": url,
        "header": headers
    });

    if let Some(body) = &req.body {
        let exported = export_request_body(body);
        if !exported.is_null() {
            request["body"] = exported;
        }
    }
    if options.include_auth {
        if let Some(auth) = &req.auth {
            let exported = export_auth(auth);
            if !exported.is_null() {
                request["auth"] = exported;
            }
        }
    }
    if options.include_description {
        if let Some(desc) = &req.description {
            request["description"] = serde_json::json!(desc);
        }
    }

    let mut item = serde_json::json!({ "name": req.name, "request": request });
    if options.include_scripts {
        let mut events = Vec::new();
        if let Some(pre) = &req.pre_script {
            if !pre.trim().is_empty() {
                events.push(export_script_event("prerequest", pre));
            }
        }
        if let Some(post) = &req.post_script {
            if !post.trim().is_empty() {
                events.push(export_script_event("test", post));
            }
        }
        if !events.is_empty() {
            item["event"] = serde_json::json!(events);
        }
    }
    item
}

fn export_folder_items(
    name: &str,
    description: Option<&str>,
    folders: &[ExportFolderNode],
    requests: &[HttpRequest],
    options: &ExportOptions,
) -> serde_json::Value {
    let mut items: Vec<serde_json::Value> = folders
        .iter()
        .map(|f| {
            export_folder_items(&f.name, f.description.as_deref(), &f.folders, &f.requests, options)
        })
        .collect();
    items.extend(requests.iter().map(|r| export_request_item(r, options)));

    let mut group = serde_json::json!({ "name": name, "item": items });
    if let Some(desc) = description {
        group["description"] = serde_json::json!(desc);
    }
    group
}

#[tauri::command]
fn export_postman_collection(
    collections: Vec<ExportCollectionNode>,
    options: ExportOptions,
) -> Result<String, String> {
    if collections.is_empty() {
        return Err("未选择任何要导出的请求".to_string());
    }

    // 单集合：集合名作为 Collection 名，保持原层级；
    // 多集合：顶层合并为 iApi Export，每个集合是一个分组
    let (collection_name, items) = if collections.len() == 1 {
        let c = &collections[0];
        let mut items: Vec<serde_json::Value> = c
            .folders
            .iter()
            .map(|f| export_folder_items(&f.name, f.description.as_deref(), &f.folders, &f.requests, &options))
            .collect();
        items.extend(c.requests.iter().map(|r| export_request_item(r, &options)));
        (c.name.clone(), items)
    } else {
        (
            "iApi Export".to_string(),
            collections
                .iter()
                .map(|c| export_folder_items(&c.name, c.description.as_deref(), &c.folders, &c.requests, &options))
                .collect(),
        )
    };

    let mut info = serde_json::json!({
        "name": collection_name,
        "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
    });
    if let Some(desc) = collections[0].description.as_deref() {
        info["description"] = serde_json::json!(desc);
    }

    let result = serde_json::json!({ "info": info, "item": items });
    serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
}


/// 导入结果的文件夹节点（保持 Postman 集合内的层级）
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ImportedFolder {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    folders: Vec<ImportedFolder>,
    requests: Vec<HttpRequest>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ImportedCollection {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    folders: Vec<ImportedFolder>,
    requests: Vec<HttpRequest>,
}

#[tauri::command]
fn import_postman_collection(json: String) -> Result<ImportedCollection, String> {
    let value: serde_json::Value = serde_json::from_str(&json).map_err(|e| e.to_string())?;

    let name = value
        .pointer("/info/name")
        .and_then(|n| n.as_str())
        .unwrap_or("Postman Collection")
        .to_string();
    let description = value
        .pointer("/info/description")
        .and_then(|d| d.as_str())
        .map(|s| s.to_string());

    let mut folders = Vec::new();
    let mut requests = Vec::new();
    if let Some(items) = value.get("item").and_then(|i| i.as_array()) {
        for item in items {
            sort_postman_item(item, &mut folders, &mut requests);
        }
    }

    Ok(ImportedCollection {
        name,
        description,
        folders,
        requests,
    })
}

/// 区分 Postman item：带 item 数组的是文件夹分组，带 request 的是请求
fn sort_postman_item(
    item: &serde_json::Value,
    folders: &mut Vec<ImportedFolder>,
    requests: &mut Vec<HttpRequest>,
) {
    if item.get("request").is_some() {
        if let Some(req) = parse_postman_item(item) {
            requests.push(req);
        }
        return;
    }
    let Some(children) = item.get("item").and_then(|i| i.as_array()) else {
        return;
    };
    let mut folder = ImportedFolder {
        name: item
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("文件夹")
            .to_string(),
        description: item
            .get("description")
            .and_then(|d| d.as_str())
            .map(|s| s.to_string()),
        folders: Vec::new(),
        requests: Vec::new(),
    };
    for child in children {
        sort_postman_item(child, &mut folder.folders, &mut folder.requests);
    }
    folders.push(folder);
}

/// Postman 的 event 挂在 item 层级（与 request 平级），不在 request 内
fn parse_postman_script(item: &serde_json::Value, listen: &str) -> Option<String> {
    let script = item.get("event")?.as_array()?.iter().find_map(|e| {
        let matches_listen = e.get("listen").and_then(|l| l.as_str()) == Some(listen);
        matches_listen.then(|| e.get("script").cloned()).flatten()
    })?;
    let exec = script.get("exec").and_then(|x| x.as_array())?;
    let lines: Vec<&str> = exec.iter().filter_map(|l| l.as_str()).collect();
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

fn parse_postman_item(item: &serde_json::Value) -> Option<HttpRequest> {
    let name = item.get("name")?.as_str()?.to_string();
    let request = item.get("request")?;

    let method = request
        .get("method")
        .and_then(|m| m.as_str())
        .unwrap_or("GET")
        .to_uppercase();

    let mut url = request
        .get("url")
        .and_then(|u| {
            if u.is_string() {
                u.as_str().map(|s| s.to_string())
            } else {
                u.get("raw").and_then(|r| r.as_str()).map(|s| s.to_string())
            }
        })
        .unwrap_or_default();

    let params: Vec<models::KeyValuePair> = request
        .pointer("/url/query")
        .and_then(|q| q.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|p| {
                    let key = p.get("key")?.as_str()?;
                    if key.is_empty() {
                        return None;
                    }
                    Some(models::KeyValuePair {
                        key: key.to_string(),
                        value: p
                            .get("value")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        description: None,
                        enabled: !p.get("disabled").and_then(|d| d.as_bool()).unwrap_or(false),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    // params 与 raw URL 里的 query 二选一保留：有 query 数组时把 URL 截到 '?' 前，避免参数重复
    if !params.is_empty() && url.contains('?') {
        url = url.split('?').next().unwrap_or(&url).to_string();
    }

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
        match mode {
            "raw" => {
                let raw_type = b
                    .pointer("/options/raw/language")
                    .and_then(|l| l.as_str())
                    .unwrap_or("json")
                    .to_string();
                Some(models::RequestBody {
                    body_mode: "raw".to_string(),
                    raw: b.get("raw").and_then(|r| r.as_str()).map(|s| s.to_string()),
                    raw_type: Some(raw_type),
                    form_data: None,
                    urlencoded: None,
                    binary: None,
                })
            }
            "formdata" => {
                let form_data = b.get("formdata").and_then(|f| f.as_array()).map(|arr| {
                    arr.iter()
                        .filter_map(|f| {
                            let key = f.get("key")?.as_str()?;
                            let form_type = f
                                .get("type")
                                .and_then(|t| t.as_str())
                                .unwrap_or("text")
                                .to_string();
                            Some(models::FormData {
                                key: key.to_string(),
                                value: f
                                    .get("value")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or_default()
                                    .to_string(),
                                description: None,
                                enabled: !f.get("disabled").and_then(|d| d.as_bool()).unwrap_or(false),
                                form_type,
                                file_path: f
                                    .get("src")
                                    .and_then(|s| s.as_str())
                                    .map(|s| s.to_string()),
                            })
                        })
                        .collect::<Vec<_>>()
                });
                Some(models::RequestBody {
                    body_mode: "form-data".to_string(),
                    raw: None,
                    raw_type: None,
                    form_data,
                    urlencoded: None,
                    binary: None,
                })
            }
            "urlencoded" => {
                let urlencoded = b.get("urlencoded").and_then(|u| u.as_array()).map(|arr| {
                    arr.iter()
                        .filter_map(|p| {
                            Some(models::KeyValuePair {
                                key: p.get("key")?.as_str()?.to_string(),
                                value: p
                                    .get("value")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or_default()
                                    .to_string(),
                                description: None,
                                enabled: !p.get("disabled").and_then(|d| d.as_bool()).unwrap_or(false),
                            })
                        })
                        .collect::<Vec<_>>()
                });
                Some(models::RequestBody {
                    body_mode: "urlencoded".to_string(),
                    raw: None,
                    raw_type: None,
                    form_data: None,
                    urlencoded,
                    binary: None,
                })
            }
            "file" => Some(models::RequestBody {
                body_mode: "binary".to_string(),
                raw: None,
                raw_type: None,
                form_data: None,
                urlencoded: None,
                binary: b.pointer("/file/src").and_then(|s| s.as_str()).map(|s| s.to_string()),
            }),
            _ => None,
        }
    });

    let auth = request.get("auth").and_then(|a| {
        let auth_type = a.get("type").and_then(|t| t.as_str()).unwrap_or("none");
        let pair = |key: &str| -> String {
            a.get(auth_type)
                .and_then(|arr| arr.as_array())
                .and_then(|arr| {
                    arr.iter()
                        .find(|p| p.get("key").and_then(|k| k.as_str()) == Some(key))
                        .and_then(|p| p.get("value"))
                        .and_then(|v| v.as_str())
                })
                .unwrap_or_default()
                .to_string()
        };
        match auth_type {
            "basic" => Some(models::AuthConfig {
                auth_type: "basic".to_string(),
                basic: Some(models::BasicAuth {
                    username: pair("username"),
                    password: pair("password"),
                }),
                bearer: None,
                apikey: None,
            }),
            "bearer" => Some(models::AuthConfig {
                auth_type: "bearer".to_string(),
                basic: None,
                bearer: Some(models::BearerAuth {
                    token: pair("token"),
                }),
                apikey: None,
            }),
            "apikey" => Some(models::AuthConfig {
                auth_type: "apikey".to_string(),
                basic: None,
                bearer: None,
                apikey: Some(models::ApiKeyAuth {
                    key: pair("key"),
                    value: pair("value"),
                    add_to: {
                        let v = pair("in");
                        if v == "query" { "query".to_string() } else { "header".to_string() }
                    },
                }),
            }),
            _ => None,
        }
    });

    let pre_script = parse_postman_script(item, "prerequest");
    let post_script = parse_postman_script(item, "test");
    let description = request
        .get("description")
        .and_then(|d| d.as_str())
        .map(|s| s.to_string());

    let mut req = HttpRequest::new_with_data(name, method, url, headers, body);
    req.params = params;
    req.auth = auth;
    req.pre_script = pre_script;
    req.post_script = post_script;
    req.description = description;
    Some(req)
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
fn save_folder(folder: Folder, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    db.repository
        .save_folder(&folder)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_all_folders(db: tauri::State<'_, Arc<Database>>) -> Result<Vec<Folder>, String> {
    db.repository
        .get_all_folders()
        .map_err(|e| e.to_string())
}

/// 递归删除文件夹：先删该 folder 下所有 requests，再递归删子 folders，最后删自身
/// 避免 parent_folder_id 自引用外键约束报错
#[tauri::command]
fn delete_folder(id: String, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    let all_folders = db.repository.get_all_folders().map_err(|e| e.to_string())?;

    // 收集要删除的 folder id（自身 + 所有后代）
    let mut to_delete: Vec<String> = vec![id.clone()];
    let mut changed = true;
    while changed {
        changed = false;
        for f in &all_folders {
            if let Some(pid) = &f.parent_folder_id {
                if to_delete.contains(pid) && !to_delete.contains(&f.id) {
                    to_delete.push(f.id.clone());
                    changed = true;
                }
            }
        }
    }

    // 先删除这些 folder 下所有 requests（folder_id 匹配）
    for fid in &to_delete {
        let _ = db.repository.delete_requests_by_folder(fid);
    }
    // 再删除 folder 记录自身（子 folder 先删，父 folder 后删，满足自引用外键约束）
    for fid in to_delete.iter().rev() {
        db.repository.delete_folder(fid).map_err(|e| e.to_string())?;
    }
    Ok(())
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

#[cfg(test)]
mod export_tests {
    use super::*;

    fn sample_request(name: &str, method: &str, url: &str) -> HttpRequest {
        HttpRequest::new_with_data(
            name.to_string(),
            method.to_string(),
            url.to_string(),
            vec![models::KeyValuePair {
                key: "Authorization".to_string(),
                value: "Bearer x".to_string(),
                description: None,
                enabled: true,
            }],
            Some(models::RequestBody {
                body_mode: "urlencoded".to_string(),
                raw: None,
                raw_type: None,
                form_data: None,
                urlencoded: Some(vec![models::KeyValuePair {
                    key: "k".to_string(),
                    value: "v".to_string(),
                    description: None,
                    enabled: true,
                }]),
                binary: None,
            }),
        )
    }

    #[test]
    fn export_produces_nested_structure_and_full_fields() {
        let req = sample_request("登录", "POST", "https://api.example.com/login");
        let collections = vec![ExportCollectionNode {
            name: "项目A".to_string(),
            description: None,
            folders: vec![ExportFolderNode {
                name: "用户模块".to_string(),
                description: None,
                folders: vec![],
                requests: vec![req],
            }],
            requests: vec![],
        }];

        let json = export_postman_collection(
            collections,
            ExportOptions {
                include_params: true,
                include_auth: true,
                include_scripts: true,
                include_description: true,
            },
        )
        .unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();

        // 单集合：集合名提升为 Collection 名，文件夹保留层级
        assert_eq!(v["info"]["name"], "项目A");
        assert_eq!(v["item"][0]["name"], "用户模块");
        assert_eq!(v["item"][0]["item"][0]["name"], "登录");

        let request = &v["item"][0]["item"][0]["request"];
        assert_eq!(request["method"], "POST");
        assert_eq!(request["header"][0]["key"], "Authorization");
        assert_eq!(request["body"]["mode"], "urlencoded");
        assert_eq!(request["body"]["urlencoded"][0]["key"], "k");
    }

    #[test]
    fn export_multiple_collections_merge_under_iapi_export() {
        let collections = vec![
            ExportCollectionNode {
                name: "A".to_string(),
                description: None,
                folders: vec![],
                requests: vec![sample_request("a1", "GET", "https://x.com/a1")],
            },
            ExportCollectionNode {
                name: "B".to_string(),
                description: None,
                folders: vec![],
                requests: vec![sample_request("b1", "GET", "https://x.com/b1")],
            },
        ];
        let json = export_postman_collection(
            collections,
            ExportOptions {
                include_params: true,
                include_auth: true,
                include_scripts: true,
                include_description: true,
            },
        )
        .unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(v["info"]["name"], "iApi Export");
        assert_eq!(v["item"].as_array().unwrap().len(), 2);
        assert_eq!(v["item"][0]["name"], "A");
    }

    #[test]
    fn export_input_from_frontend_camelcase_json() {
        // 模拟前端 invoke 传参：camelCase 字段必须完整映射（preScript/auth/formData 等）
        let input = r#"{
          "collections": [{
            "name": "前端集合",
            "folders": [],
            "requests": [{
              "id": "r1", "name": "登录", "method": "POST",
              "url": "https://x.com/login",
              "params": [{"key": "page", "value": "1", "enabled": true}],
              "headers": [],
              "body": {"mode": "form-data", "formData": [
                {"key": "f", "value": "v", "enabled": true, "type": "text"}
              ]},
              "auth": {"type": "basic", "basic": {"username": "u", "password": "p"}},
              "preScript": "pm.info()",
              "createdAt": "2026-01-01T00:00:00Z",
              "updatedAt": "2026-01-01T00:00:00Z"
            }]
          }],
          "options": {"includeParams": true, "includeAuth": true, "includeScripts": true, "includeDescription": true}
        }"#;
        let payload: serde_json::Value = serde_json::from_str(input).unwrap();
        let collections: Vec<ExportCollectionNode> =
            serde_json::from_value(payload["collections"].clone()).unwrap();
        let options: ExportOptions = serde_json::from_value(payload["options"].clone()).unwrap();

        let json = export_postman_collection(collections, options).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();

        let request = &v["item"][0]["request"];
        // camelCase 边界字段全部存活
        assert_eq!(request["url"]["query"][0]["key"], "page");
        assert_eq!(request["body"]["mode"], "formdata");
        assert_eq!(request["body"]["formdata"][0]["key"], "f");
        assert_eq!(request["auth"]["type"], "basic");
        assert_eq!(v["item"][0]["event"][0]["listen"], "prerequest");
    }

    #[test]
    fn export_excludes_auth_when_disabled() {
        let mut req = sample_request("r", "GET", "https://x.com/r");
        req.auth = Some(models::AuthConfig {
            auth_type: "bearer".to_string(),
            basic: None,
            bearer: Some(models::BearerAuth {
                token: "secret".to_string(),
            }),
            apikey: None,
        });
        let collections = vec![ExportCollectionNode {
            name: "C".to_string(),
            description: None,
            folders: vec![],
            requests: vec![req],
        }];

        let json = export_postman_collection(
            collections,
            ExportOptions {
                include_params: true,
                include_auth: false,
                include_scripts: true,
                include_description: true,
            },
        )
        .unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(v["item"][0]["request"].get("auth").is_none());
    }

    #[test]
    fn export_import_roundtrip_preserves_structure() {
        // 导出：文件夹内一个带 urlencoded body + 认证 + 脚本的请求
        let mut req = sample_request("登录", "POST", "https://api.example.com/login");
        req.auth = Some(models::AuthConfig {
            auth_type: "bearer".to_string(),
            basic: None,
            bearer: Some(models::BearerAuth {
                token: "tk".to_string(),
            }),
            apikey: None,
        });
        req.pre_script = Some("console.log('pre')".to_string());
        let collections = vec![ExportCollectionNode {
            name: "往返".to_string(),
            description: None,
            folders: vec![ExportFolderNode {
                name: "模块".to_string(),
                description: None,
                folders: vec![],
                requests: vec![req],
            }],
            requests: vec![],
        }];

        let json = export_postman_collection(
            collections,
            ExportOptions {
                include_params: true,
                include_auth: true,
                include_scripts: true,
                include_description: true,
            },
        )
        .unwrap();

        // 导入同一份 JSON：结构与关键字段应还原
        let imported = import_postman_collection(json).unwrap();
        assert_eq!(imported.name, "往返");
        assert_eq!(imported.folders.len(), 1);
        assert_eq!(imported.folders[0].name, "模块");

        let r = &imported.folders[0].requests[0];
        assert_eq!(r.name, "登录");
        assert_eq!(r.method, "POST");
        assert_eq!(
            r.auth.as_ref().unwrap().bearer.as_ref().unwrap().token,
            "tk"
        );
        assert_eq!(
            r.body.as_ref().unwrap().urlencoded.as_ref().unwrap()[0].key,
            "k"
        );
        assert_eq!(r.pre_script.as_deref(), Some("console.log('pre')"));
    }
}
