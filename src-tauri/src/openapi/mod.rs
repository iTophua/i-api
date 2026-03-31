use crate::models::{HttpRequest, KeyValuePair, RequestBody};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct OpenApiInfo {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedOpenApi {
    pub info: OpenApiInfo,
    pub requests: Vec<HttpRequest>,
}

pub fn parse_openapi(content: &str) -> Result<ParsedOpenApi, String> {
    let json: Value = if content.trim().starts_with('{') || content.trim().starts_with('[') {
        serde_json::from_str(content).map_err(|e| format!("JSON 解析失败: {}", e))?
    } else {
        let yaml: serde_yaml::Value =
            serde_yaml::from_str(content).map_err(|e| format!("YAML 解析失败: {}", e))?;
        serde_json::to_value(yaml).map_err(|e| format!("YAML 转换失败: {}", e))?
    };

    let openapi_version = json.get("openapi").and_then(|v| v.as_str()).unwrap_or("");
    let swagger_version = json.get("swagger").and_then(|v| v.as_str()).unwrap_or("");

    if openapi_version.is_empty() && swagger_version.is_empty() {
        return Err("不是有效的 OpenAPI/Swagger 文档".to_string());
    }

    let is_openapi3 = !openapi_version.is_empty();

    let info = parse_info(&json);
    let requests = if is_openapi3 {
        parse_openapi3_paths(&json)
    } else {
        parse_swagger2_paths(&json)
    };

    Ok(ParsedOpenApi { info, requests })
}

fn parse_info(json: &Value) -> OpenApiInfo {
    let info_obj = json.get("info").unwrap_or(&Value::Null);

    OpenApiInfo {
        title: info_obj
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("未命名 API")
            .to_string(),
        version: info_obj
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("1.0.0")
            .to_string(),
        description: info_obj
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    }
}

fn parse_openapi3_paths(json: &Value) -> Vec<HttpRequest> {
    let mut requests = Vec::new();

    let paths = match json.get("paths").and_then(|p| p.as_object()) {
        Some(p) => p,
        None => return requests,
    };

    let base_url = extract_base_url(json);

    for (path, path_item) in paths {
        let path_item = match path_item.as_object() {
            Some(p) => p,
            None => continue,
        };

        for method in &["get", "post", "put", "delete", "patch", "options", "head"] {
            if let Some(operation) = path_item.get(*method).and_then(|o| o.as_object()) {
                if let Some(req) = parse_operation(path, method, operation, &base_url, json) {
                    requests.push(req);
                }
            }
        }
    }

    requests
}

fn parse_swagger2_paths(json: &Value) -> Vec<HttpRequest> {
    let mut requests = Vec::new();

    let paths = match json.get("paths").and_then(|p| p.as_object()) {
        Some(p) => p,
        None => return requests,
    };

    let base_url = extract_swagger2_base_url(json);

    for (path, path_item) in paths {
        let path_item = match path_item.as_object() {
            Some(p) => p,
            None => continue,
        };

        for method in &["get", "post", "put", "delete", "patch", "options", "head"] {
            if let Some(operation) = path_item.get(*method).and_then(|o| o.as_object()) {
                if let Some(req) = parse_swagger2_operation(path, method, operation, &base_url) {
                    requests.push(req);
                }
            }
        }
    }

    requests
}

fn parse_operation(
    path: &str,
    method: &str,
    operation: &serde_json::Map<String, Value>,
    base_url: &str,
    _root: &Value,
) -> Option<HttpRequest> {
    let name = operation
        .get("summary")
        .and_then(|s| s.as_str())
        .or_else(|| operation.get("operationId").and_then(|o| o.as_str()))
        .unwrap_or(path)
        .to_string();

    let url = format!("{}{}", base_url, path);

    let mut headers = Vec::new();

    if let Some(parameters) = operation.get("parameters").and_then(|p| p.as_array()) {
        for param in parameters {
            if let Some(param_in) = param.get("in").and_then(|i| i.as_str()) {
                if param_in == "header" {
                    if let (Some(key), Some(value)) = (
                        param.get("name").and_then(|n| n.as_str()),
                        param
                            .get("schema")
                            .and_then(|s| s.get("example"))
                            .or_else(|| param.get("example")),
                    ) {
                        headers.push(KeyValuePair {
                            key: key.to_string(),
                            value: value.as_str().unwrap_or("").to_string(),
                            description: param
                                .get("description")
                                .and_then(|d| d.as_str())
                                .map(|s| s.to_string()),
                            enabled: true,
                        });
                    }
                }
            }
        }
    }

    if let Some(request_body) = operation.get("requestBody") {
        if let Some(content) = request_body.get("content").and_then(|c| c.as_object()) {
            if let Some(json_content) = content.get("application/json") {
                if let Some(example) = json_content.get("example") {
                    let raw = serde_json::to_string_pretty(example).unwrap_or_default();
                    return Some(HttpRequest::new_with_data(
                        name,
                        method.to_uppercase(),
                        url,
                        headers,
                        Some(RequestBody {
                            body_mode: "raw".to_string(),
                            raw: Some(raw),
                            raw_type: Some("json".to_string()),
                            form_data: None,
                            urlencoded: None,
                            binary: None,
                        }),
                    ));
                }
            }
        }
    }

    Some(HttpRequest::new_with_data(
        name,
        method.to_uppercase(),
        url,
        headers,
        None,
    ))
}

fn parse_swagger2_operation(
    path: &str,
    method: &str,
    operation: &serde_json::Map<String, Value>,
    base_url: &str,
) -> Option<HttpRequest> {
    let name = operation
        .get("summary")
        .and_then(|s| s.as_str())
        .or_else(|| operation.get("operationId").and_then(|o| o.as_str()))
        .unwrap_or(path)
        .to_string();

    let url = format!("{}{}", base_url, path);

    let mut headers = Vec::new();
    let mut body: Option<RequestBody> = None;

    if let Some(parameters) = operation.get("parameters").and_then(|p| p.as_array()) {
        for param in parameters {
            if let Some(param_in) = param.get("in").and_then(|i| i.as_str()) {
                if param_in == "header" {
                    if let Some(key) = param.get("name").and_then(|n| n.as_str()) {
                        headers.push(KeyValuePair {
                            key: key.to_string(),
                            value: String::new(),
                            description: param
                                .get("description")
                                .and_then(|d| d.as_str())
                                .map(|s| s.to_string()),
                            enabled: true,
                        });
                    }
                } else if param_in == "body" {
                    if let Some(example) = param.get("example") {
                        let raw = if example.is_string() {
                            example.as_str().unwrap_or("").to_string()
                        } else {
                            serde_json::to_string_pretty(example).unwrap_or_default()
                        };
                        body = Some(RequestBody {
                            body_mode: "raw".to_string(),
                            raw: Some(raw),
                            raw_type: Some("json".to_string()),
                            form_data: None,
                            urlencoded: None,
                            binary: None,
                        });
                    }
                }
            }
        }
    }

    Some(HttpRequest::new_with_data(
        name,
        method.to_uppercase(),
        url,
        headers,
        body,
    ))
}

fn extract_base_url(json: &Value) -> String {
    if let Some(servers) = json.get("servers").and_then(|s| s.as_array()) {
        if let Some(server) = servers.first() {
            if let Some(url) = server.get("url").and_then(|u| u.as_str()) {
                return url.trim_end_matches('/').to_string();
            }
        }
    }
    String::new()
}

fn extract_swagger2_base_url(json: &Value) -> String {
    let scheme = json
        .get("schemes")
        .and_then(|s| s.as_array())
        .and_then(|arr| arr.first())
        .and_then(|s| s.as_str())
        .unwrap_or("https");

    let host = json
        .get("host")
        .and_then(|h| h.as_str())
        .unwrap_or("localhost");

    let base_path = json.get("basePath").and_then(|b| b.as_str()).unwrap_or("");

    format!("{}://{}{}", scheme, host, base_path.trim_end_matches('/'))
}

pub fn parse_har(content: &str) -> Result<Vec<HttpRequest>, String> {
    let json: Value = serde_json::from_str(content).map_err(|e| format!("JSON 解析失败: {}", e))?;

    let entries = json
        .get("log")
        .and_then(|log| log.get("entries"))
        .and_then(|e| e.as_array())
        .ok_or("无效的 HAR 文件格式")?;

    let mut requests = Vec::new();

    for entry in entries {
        if let Some(request) = entry.get("request") {
            if let Some(req) = parse_har_request(request) {
                requests.push(req);
            }
        }
    }

    Ok(requests)
}

fn parse_har_request(request: &Value) -> Option<HttpRequest> {
    let method = request
        .get("method")
        .and_then(|m| m.as_str())
        .unwrap_or("GET")
        .to_uppercase();

    let url = request
        .get("url")
        .and_then(|u| u.as_str())
        .unwrap_or("")
        .to_string();

    let name = url
        .split('?')
        .next()
        .unwrap_or(&url)
        .split('/')
        .last()
        .unwrap_or("未命名请求")
        .to_string();

    let mut headers = Vec::new();
    if let Some(header_list) = request.get("headers").and_then(|h| h.as_array()) {
        for header in header_list {
            if let (Some(key), Some(value)) = (
                header.get("name").and_then(|n| n.as_str()),
                header.get("value").and_then(|v| v.as_str()),
            ) {
                headers.push(KeyValuePair {
                    key: key.to_string(),
                    value: value.to_string(),
                    description: None,
                    enabled: true,
                });
            }
        }
    }

    let body = request
        .get("postData")
        .and_then(|p| p.get("text"))
        .and_then(|t| t.as_str())
        .map(|raw| RequestBody {
            body_mode: "raw".to_string(),
            raw: Some(raw.to_string()),
            raw_type: Some("json".to_string()),
            form_data: None,
            urlencoded: None,
            binary: None,
        });

    Some(HttpRequest::new_with_data(name, method, url, headers, body))
}
