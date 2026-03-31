use crate::models::{HttpRequest, HttpResponse, KeyValuePair};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use once_cell::sync::Lazy;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

fn base64_encode(data: &[u8]) -> String {
    STANDARD.encode(data)
}

fn is_binary_response(headers: &HashMap<String, String>) -> bool {
    let content_type = headers
        .get("content-type")
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    let text_types = [
        "text/",
        "application/json",
        "application/xml",
        "application/javascript",
        "application/vnd.api+json",
    ];

    !text_types.iter().any(|t| content_type.starts_with(t))
        && !content_type.is_empty()
}

static HTTP_CLIENT: Lazy<Arc<Client>> = Lazy::new(|| {
    Arc::new(
        Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .user_agent("iApi/0.1.0")
            .build()
            .expect("Failed to create HTTP client"),
    )
});

static PENDING_REQUESTS: Lazy<Arc<RwLock<HashMap<String, tokio_util::sync::CancellationToken>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

pub async fn send_request(request: HttpRequest) -> Result<HttpResponse, String> {
    let client = if let Some(timeout_ms) = request.timeout {
        let timeout_secs = timeout_ms / 1000;
        let timeout_nanos = (timeout_ms % 1000) * 1_000_000;
        Arc::new(
            Client::builder()
                .timeout(Duration::new(timeout_secs, timeout_nanos as u32))
                .connect_timeout(Duration::from_secs(10))
                .pool_max_idle_per_host(10)
                .pool_idle_timeout(Duration::from_secs(90))
                .user_agent("iApi/0.1.0")
                .build()
                .map_err(|e| format!("创建HTTP客户端失败: {}", e))?,
        )
    } else {
        HTTP_CLIENT.clone()
    };
    let request_id = request.id.clone();

    let cancellation_token = tokio_util::sync::CancellationToken::new();
    {
        let mut pending = PENDING_REQUESTS.write().await;
        pending.insert(request_id.clone(), cancellation_token.clone());
    }

    let result = send_request_internal(client, request, cancellation_token.clone()).await;

    {
        let mut pending = PENDING_REQUESTS.write().await;
        pending.remove(&request_id);
    }

    result
}

async fn send_request_internal(
    client: Arc<Client>,
    request: HttpRequest,
    cancellation_token: tokio_util::sync::CancellationToken,
) -> Result<HttpResponse, String> {
    let url = build_url_with_auth(&request.url, &request.params, &request.auth);
    let method = parse_method(&request.method);

    let mut req_builder = client.request(method, &url);

    // 判断是否为 form-data，如果是则跳过 Content-Type header（让 reqwest 自动设置）
    let is_form_data = request.body.as_ref()
        .map(|b| b.body_mode == "form-data")
        .unwrap_or(false);

    req_builder = add_headers(req_builder, &request.headers, &request.auth, is_form_data);

    req_builder = add_body(req_builder, &request.body).await;

    let start = Instant::now();

    tokio::select! {
        result = req_builder.send() => {
            let response = result.map_err(|e| e.to_string())?;
            let response_time = start.elapsed().as_millis() as u64;

            let status = response.status().as_u16();
            let status_text = response.status().canonical_reason().unwrap_or("").to_string();

            let headers: HashMap<String, String> = response
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect();

            let cookies = extract_cookies(&headers);
            let return_bytes = request.return_bytes.unwrap_or(false);

            let (body, body_bytes, response_size) = if return_bytes {
                let bytes = response.bytes().await.map_err(|e| e.to_string())?;
                let size = bytes.len();
                (String::new(), Some(bytes.to_vec()), size)
            } else {
                let text = response.text().await.map_err(|e| e.to_string())?;
                let size = text.len();
                (text, None, size)
            };

            Ok(HttpResponse {
                status,
                status_text,
                headers,
                cookies,
                body,
                body_bytes,
                response_time,
                response_size,
            })
        }
        _ = cancellation_token.cancelled() => {
            Err("请求已取消".to_string())
        }
    }
}

pub async fn cancel_request(request_id: &str) -> bool {
    let pending = PENDING_REQUESTS.read().await;
    if let Some(token) = pending.get(request_id) {
        token.cancel();
        true
    } else {
        false
    }
}

fn build_url_with_auth(
    base_url: &str,
    params: &[KeyValuePair],
    auth: &Option<crate::models::AuthConfig>,
) -> String {
    let mut all_params: Vec<KeyValuePair> = params.iter().filter(|p| p.enabled).cloned().collect();

    if let Some(auth_config) = auth {
        if auth_config.auth_type == "apikey" {
            if let Some(apikey) = &auth_config.apikey {
                if apikey.add_to == "query" {
                    all_params.push(KeyValuePair {
                        key: apikey.key.clone(),
                        value: apikey.value.clone(),
                        description: None,
                        enabled: true,
                    });
                }
            }
        }
    }

    if all_params.is_empty() {
        return base_url.to_string();
    }

    let query = all_params
        .iter()
        .map(|p| format!("{}={}", urlencoding::encode(&p.key), urlencoding::encode(&p.value)))
        .collect::<Vec<_>>()
        .join("&");

    if base_url.contains('?') {
        format!("{}&{}", base_url, query)
    } else {
        format!("{}?{}", base_url, query)
    }
}

fn parse_method(method: &str) -> reqwest::Method {
    match method.to_uppercase().as_str() {
        "GET" => reqwest::Method::GET,
        "POST" => reqwest::Method::POST,
        "PUT" => reqwest::Method::PUT,
        "DELETE" => reqwest::Method::DELETE,
        "PATCH" => reqwest::Method::PATCH,
        "OPTIONS" => reqwest::Method::OPTIONS,
        "HEAD" => reqwest::Method::HEAD,
        _ => reqwest::Method::GET,
    }
}

fn add_headers(
    builder: reqwest::RequestBuilder,
    headers: &[KeyValuePair],
    auth: &Option<crate::models::AuthConfig>,
    skip_content_type: bool,
) -> reqwest::RequestBuilder {
    let mut builder = builder;

    for header in headers.iter().filter(|h| h.enabled) {
        // 跳过 Content-Type，让 reqwest 的 multipart 自动设置
        if skip_content_type && header.key.eq_ignore_ascii_case("Content-Type") {
            continue;
        }
        builder = builder.header(&header.key, &header.value);
    }

    if let Some(auth_config) = auth {
        builder = match auth_config.auth_type.as_str() {
            "bearer" => {
                if let Some(bearer) = &auth_config.bearer {
                    builder.bearer_auth(&bearer.token)
                } else {
                    builder
                }
            }
            "basic" => {
                if let Some(basic) = &auth_config.basic {
                    builder.basic_auth(&basic.username, Some(&basic.password))
                } else {
                    builder
                }
            }
            "apikey" => {
                if let Some(apikey) = &auth_config.apikey {
                    if apikey.add_to == "header" {
                        builder.header(&apikey.key, &apikey.value)
                    } else {
                        builder
                    }
                } else {
                    builder
                }
            }
            _ => builder,
        };
    }

    builder
}

async fn add_body(
    builder: reqwest::RequestBuilder,
    body: &Option<crate::models::RequestBody>,
) -> reqwest::RequestBuilder {
    let Some(body) = body else {
        return builder;
    };

    match body.body_mode.as_str() {
        "raw" => {
            if let Some(raw) = &body.raw {
                let mut builder = builder;
                if let Some(raw_type) = &body.raw_type {
                    let content_type = match raw_type.as_str() {
                        "json" => "application/json",
                        "xml" => "application/xml",
                        "html" => "text/html",
                        _ => "",
                    };
                    if !content_type.is_empty() {
                        builder = builder.header("Content-Type", content_type);
                    }
                }
                builder.body(raw.clone())
            } else {
                builder
            }
        }
        "urlencoded" => {
            if let Some(form_data) = &body.urlencoded {
                let form: Vec<_> = form_data
                    .iter()
                    .filter(|f| f.enabled)
                    .map(|f| (&f.key, &f.value))
                    .collect();
                builder.form(&form)
            } else {
                builder
            }
        }
        "form-data" => {
            if let Some(form_data) = &body.form_data {
                let mut form = reqwest::multipart::Form::new();
                for field in form_data.iter().filter(|f| f.enabled) {
                    if field.form_type == "file" {
                        if let Some(file_path) = &field.file_path {
                            if let Ok(part) = reqwest::multipart::Part::file(file_path).await {
                                form = form.part(field.key.clone(), part);
                            }
                        }
                    } else {
                        form = form.text(field.key.clone(), field.value.clone());
                    }
                }
                builder.multipart(form)
            } else {
                builder
            }
        }
        "binary" => {
            if let Some(binary) = &body.binary {
                builder.body(binary.clone())
            } else {
                builder
            }
        }
        _ => builder,
    }
}

fn extract_cookies(headers: &HashMap<String, String>) -> Vec<crate::models::Cookie> {
    let mut cookies = Vec::new();

    for (key, value) in headers {
        if key.to_lowercase() == "set-cookie" {
            if let Some(cookie) = parse_single_cookie(value) {
                cookies.push(cookie);
            }
        }
    }

    cookies
}

fn parse_single_cookie(cookie_str: &str) -> Option<crate::models::Cookie> {
    let parts: Vec<&str> = cookie_str.split(';').collect();
    let first = parts.first()?;
    let (name, value) = first.split_once('=')?;

    let mut cookie = crate::models::Cookie {
        name: name.trim().to_string(),
        value: value.trim().to_string(),
        domain: None,
        path: None,
        expires: None,
        http_only: None,
        secure: None,
    };

    for part in parts.iter().skip(1) {
        let part = part.trim();
        if part.eq_ignore_ascii_case("HttpOnly") {
            cookie.http_only = Some(true);
        } else if part.eq_ignore_ascii_case("Secure") {
            cookie.secure = Some(true);
        } else if let Some((key, val)) = part.split_once('=') {
            match key.trim().to_lowercase().as_str() {
                "domain" => cookie.domain = Some(val.trim().to_string()),
                "path" => cookie.path = Some(val.trim().to_string()),
                "expires" => cookie.expires = Some(val.trim().to_string()),
                _ => {}
            }
        }
    }

    Some(cookie)
}
