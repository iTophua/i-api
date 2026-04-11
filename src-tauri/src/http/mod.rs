use crate::models::{HttpRequest, HttpResponse, KeyValuePair, ProxyConfig};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use futures_util::StreamExt;
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

    !text_types.iter().any(|t| content_type.starts_with(t)) && !content_type.is_empty()
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

fn build_client(request: &HttpRequest) -> Result<Client, String> {
    let mut builder = Client::builder();

    if let Some(timeout_ms) = request.timeout {
        builder = builder.timeout(Duration::from_millis(timeout_ms));
    } else {
        builder = builder.timeout(Duration::from_secs(30));
    }

    builder = builder
        .connect_timeout(Duration::from_secs(10))
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(Duration::from_secs(90))
        .user_agent("iApi/0.1.0");

    if let Some(follow_redirects) = request.follow_redirects {
        if follow_redirects {
            builder = builder.redirect(reqwest::redirect::Policy::limited(10));
        } else {
            builder = builder.redirect(reqwest::redirect::Policy::none());
        }
    } else {
        builder = builder.redirect(reqwest::redirect::Policy::limited(10));
    }

    if let Some(verify_ssl) = request.verify_ssl {
        if !verify_ssl {
            builder = builder.danger_accept_invalid_certs(true);
        }
    }

    builder
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))
}

pub async fn send_request(request: HttpRequest) -> Result<HttpResponse, String> {
    let client = build_client(&request).map(Arc::new)?;
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
    let mut url = build_url_with_auth(&request.url, &request.params, &request.auth);

    if let Some(proxy) = &request.proxy {
        if proxy.enabled {
            url = apply_proxy(&url, proxy);
        }
    }

    let method = parse_method(&request.method);

    let mut req_builder = client.request(method, &url);

    let is_form_data = request
        .body
        .as_ref()
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

pub async fn send_request_stream<F>(
    request: HttpRequest,
    mut on_chunk: F,
) -> Result<(String, HashMap<String, String>, u16, u64, usize), String>
where
    F: FnMut(Vec<u8>, HashMap<String, String>, u16, std::time::Instant) + Send + 'static,
{
    let client = build_client(&request).map(Arc::new)?;
    let request_id = request.id.clone();

    let cancellation_token = tokio_util::sync::CancellationToken::new();
    {
        let mut pending = PENDING_REQUESTS.write().await;
        pending.insert(request_id.clone(), cancellation_token.clone());
    }

    let result = send_request_stream_internal(client, request, cancellation_token, on_chunk).await;

    {
        let mut pending = PENDING_REQUESTS.write().await;
        pending.remove(&request_id);
    }

    result
}

async fn send_request_stream_internal<F>(
    client: Arc<Client>,
    request: HttpRequest,
    cancellation_token: tokio_util::sync::CancellationToken,
    mut on_chunk: F,
) -> Result<(String, HashMap<String, String>, u16, u64, usize), String>
where
    F: FnMut(Vec<u8>, HashMap<String, String>, u16, std::time::Instant) + Send + 'static,
{
    let mut url = build_url_with_auth(&request.url, &request.params, &request.auth);

    if let Some(proxy) = &request.proxy {
        if proxy.enabled {
            url = apply_proxy(&url, proxy);
        }
    }

    let method = parse_method(&request.method);

    let mut req_builder = client.request(method, &url);
    let is_form_data = request
        .body
        .as_ref()
        .map(|b| b.body_mode == "form-data")
        .unwrap_or(false);

    req_builder = add_headers(req_builder, &request.headers, &request.auth, is_form_data);
    req_builder = add_body(req_builder, &request.body).await;

    let start = Instant::now();

    tokio::select! {
        result = req_builder.send() => {
            let response = result.map_err(|e| e.to_string())?;

            let status = response.status().as_u16();
            let status_text = response.status().canonical_reason().unwrap_or("").to_string();

            let headers: HashMap<String, String> = response
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect();

            on_chunk(Vec::new(), headers.clone(), status, start);

            let cookies = extract_cookies(&headers);
            let return_bytes = request.return_bytes.unwrap_or(false);

            let mut full_body = String::new();
            let mut total_size = 0usize;
            let mut stream = response.bytes_stream();

            while let Some(chunk_result) = stream.next().await {
                let chunk = chunk_result.map_err(|e| e.to_string())?;
                let chunk_vec = chunk.to_vec();
                total_size += chunk_vec.len();

                if !return_bytes {
                    if let Ok(text) = std::str::from_utf8(&chunk_vec) {
                        full_body.push_str(text);
                    }
                }

                if total_size % 8192 < chunk_vec.len() || chunk_vec.len() >= 8192 {
                    on_chunk(chunk_vec, headers.clone(), status, start);
                }
            }

            on_chunk(Vec::new(), headers.clone(), status, start);

            let response_time = start.elapsed().as_millis() as u64;

            Ok((full_body, headers, status, response_time, total_size))
        }
        _ = cancellation_token.cancelled() => {
            Err("请求已取消".to_string())
        }
    }
}

fn apply_proxy(url: &str, proxy: &ProxyConfig) -> String {
    let parsed_url = match reqwest::Url::parse(url) {
        Ok(u) => u,
        Err(_) => return url.to_string(),
    };
    let scheme = parsed_url.scheme();

    match scheme {
        "http" => format!(
            "http://{}@{}:{}",
            proxy.host,
            proxy.port,
            url.trim_start_matches("http://")
        ),
        "https" => format!(
            "https://{}@{}:{}",
            proxy.host,
            proxy.port,
            url.trim_start_matches("https://")
        ),
        _ => url.to_string(),
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
        .map(|p| {
            format!(
                "{}={}",
                urlencoding::encode(&p.key),
                urlencoding::encode(&p.value)
            )
        })
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
