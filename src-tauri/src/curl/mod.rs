use crate::models::{AuthConfig, BasicAuth, BearerAuth, HttpRequest, KeyValuePair, RequestBody};

pub fn parse_curl(curl_command: &str) -> Result<HttpRequest, String> {
    let curl_command = curl_command.trim();

    if !curl_command.starts_with("curl") {
        return Err("不是有效的 cURL 命令".to_string());
    }

    let tokens = tokenize_curl(curl_command);

    let mut request = HttpRequest::new("导入的请求".to_string(), "GET".to_string(), String::new());
    let mut raw_body = None;
    let mut content_type = None;

    let mut i = 0;
    while i < tokens.len() {
        let token = &tokens[i];

        match token.as_str() {
            "-X" | "--request" => {
                if i + 1 < tokens.len() {
                    request.method = tokens[i + 1].to_uppercase();
                    i += 2;
                    continue;
                }
            }
            "-H" | "--header" => {
                if i + 1 < tokens.len() {
                    let header = &tokens[i + 1];
                    if let Some((key, value)) = header.split_once(':') {
                        let key = key.trim().to_string();
                        let value = value.trim().to_string();

                        if key.eq_ignore_ascii_case("Content-Type") {
                            content_type = Some(value.clone());
                            request.headers.push(KeyValuePair {
                                key,
                                value,
                                description: None,
                                enabled: true,
                            });
                        } else if key.eq_ignore_ascii_case("Authorization") {
                            if value.starts_with("Bearer ") {
                                request.auth = Some(AuthConfig {
                                    auth_type: "bearer".to_string(),
                                    bearer: Some(BearerAuth {
                                        token: value[7..].to_string(),
                                    }),
                                    basic: None,
                                    apikey: None,
                                });
                            } else if value.starts_with("Basic ") {
                                let encoded = &value[6..];
                                if let Ok(decoded) = base64_decode(encoded) {
                                    if let Some((username, password)) = decoded.split_once(':') {
                                        request.auth = Some(AuthConfig {
                                            auth_type: "basic".to_string(),
                                            basic: Some(BasicAuth {
                                                username: username.to_string(),
                                                password: password.to_string(),
                                            }),
                                            bearer: None,
                                            apikey: None,
                                        });
                                    }
                                }
                            } else {
                                request.headers.push(KeyValuePair {
                                    key,
                                    value,
                                    description: None,
                                    enabled: true,
                                });
                            }
                        } else {
                            request.headers.push(KeyValuePair {
                                key,
                                value,
                                description: None,
                                enabled: true,
                            });
                        }
                    } else if let Some((key, _)) = header.split_once(';') {
                        let key = key.trim().to_string();
                        if !key.is_empty() {
                            request.headers.push(KeyValuePair {
                                key,
                                value: String::new(),
                                description: None,
                                enabled: true,
                            });
                        }
                    } else {
                        let trimmed = header.trim();
                        if !trimmed.is_empty() {
                            request.headers.push(KeyValuePair {
                                key: trimmed.to_string(),
                                value: String::new(),
                                description: None,
                                enabled: true,
                            });
                        }
                    }
                    i += 2;
                    continue;
                }
            }
            "-d" | "--data" | "--data-raw" | "--data-binary" => {
                if i + 1 < tokens.len() {
                    raw_body = Some(tokens[i + 1].clone());
                    if request.method == "GET" {
                        request.method = "POST".to_string();
                    }
                    i += 2;
                    continue;
                }
            }
            "--data-urlencode" => {
                if i + 1 < tokens.len() {
                    if request.body.is_none() {
                        request.body = Some(RequestBody {
                            body_mode: "urlencoded".to_string(),
                            raw: None,
                            raw_type: None,
                            form_data: None,
                            urlencoded: Some(Vec::new()),
                            binary: None,
                        });
                    }

                    let data = &tokens[i + 1];
                    if let Some(body) = &mut request.body {
                        if let Some(urlencoded) = &mut body.urlencoded {
                            if let Some((key, value)) = data.split_once('=') {
                                urlencoded.push(KeyValuePair {
                                    key: key.to_string(),
                                    value: value.to_string(),
                                    description: None,
                                    enabled: true,
                                });
                            }
                        }
                    }
                    if request.method == "GET" {
                        request.method = "POST".to_string();
                    }
                    i += 2;
                    continue;
                }
            }
            "-F" | "--form" => {
                if i + 1 < tokens.len() {
                    if request.body.is_none() {
                        request.body = Some(RequestBody {
                            body_mode: "form-data".to_string(),
                            raw: None,
                            raw_type: None,
                            form_data: Some(Vec::new()),
                            urlencoded: None,
                            binary: None,
                        });
                    }

                    let data = &tokens[i + 1];
                    if let Some(body) = &mut request.body {
                        if let Some(form_data) = &mut body.form_data {
                            if let Some((key, value)) = data.split_once('=') {
                                form_data.push(crate::models::FormData {
                                    key: key.to_string(),
                                    value: value.to_string(),
                                    description: None,
                                    enabled: true,
                                    form_type: "text".to_string(),
                                    file_path: None,
                                });
                            }
                        }
                    }
                    if request.method == "GET" {
                        request.method = "POST".to_string();
                    }
                    i += 2;
                    continue;
                }
            }
            "-u" | "--user" => {
                if i + 1 < tokens.len() {
                    let creds = &tokens[i + 1];
                    if let Some((username, password)) = creds.split_once(':') {
                        request.auth = Some(AuthConfig {
                            auth_type: "basic".to_string(),
                            basic: Some(BasicAuth {
                                username: username.to_string(),
                                password: password.to_string(),
                            }),
                            bearer: None,
                            apikey: None,
                        });
                    }
                    i += 2;
                    continue;
                }
            }
            "-A" | "--user-agent" => {
                if i + 1 < tokens.len() {
                    request.headers.push(KeyValuePair {
                        key: "User-Agent".to_string(),
                        value: tokens[i + 1].clone(),
                        description: None,
                        enabled: true,
                    });
                    i += 2;
                    continue;
                }
            }
            "-b" | "--cookie" => {
                if i + 1 < tokens.len() {
                    request.headers.push(KeyValuePair {
                        key: "Cookie".to_string(),
                        value: tokens[i + 1].clone(),
                        description: None,
                        enabled: true,
                    });
                    i += 2;
                    continue;
                }
            }
            "-k" | "--insecure" => {
                request.verify_ssl = Some(false);
                i += 1;
                continue;
            }
            "-L" | "--location" => {
                request.follow_redirects = Some(true);
                i += 1;
                continue;
            }
            "--compressed" | "-s" | "--silent" | "-S" | "--show-error" | "-v" | "--verbose"
            | "-#" | "--progress-bar" | "-i" | "--include" | "-I" | "--head" => {
                i += 1;
                continue;
            }
            "--max-time" | "-m" | "--connect-timeout" => {
                i += 2;
                continue;
            }
            "-e" | "--referer" => {
                if i + 1 < tokens.len() {
                    request.headers.push(KeyValuePair {
                        key: "Referer".to_string(),
                        value: tokens[i + 1].clone(),
                        description: None,
                        enabled: true,
                    });
                    i += 2;
                    continue;
                }
            }
            _ => {
                if token.starts_with("http://") || token.starts_with("https://") {
                    // 保持 URL 完整
                    request.url = token.clone();
                    // 解析 URL 中的查询参数到 params
                    if let Some((_, query_string)) = token.split_once('?') {
                        for param_pair in query_string.split('&') {
                            if let Some((key, value)) = param_pair.split_once('=') {
                                let decoded_key = urlencoding_decode(key);
                                let decoded_value = urlencoding_decode(value);
                                request.params.push(crate::models::KeyValuePair {
                                    key: decoded_key,
                                    value: decoded_value,
                                    description: None,
                                    enabled: true,
                                });
                            } else if !param_pair.is_empty() {
                                let decoded_key = urlencoding_decode(param_pair);
                                request.params.push(crate::models::KeyValuePair {
                                    key: decoded_key,
                                    value: String::new(),
                                    description: None,
                                    enabled: true,
                                });
                            }
                        }
                    }
                }
            }
        }

        i += 1;
    }

    if let Some(body) = raw_body {
        // 检查是否为 multipart/form-data，如果是则解析各个字段
        let is_multipart = content_type
            .as_ref()
            .map(|ct| ct.to_lowercase().contains("multipart/form-data"))
            .unwrap_or(false);

        if is_multipart {
            // 从 Content-Type 提取 boundary
            let boundary = content_type.as_ref().and_then(|ct| extract_boundary(ct));

            if let Some(boundary) = boundary {
                let form_data = parse_multipart_body(&body, &boundary);
                if !form_data.is_empty() {
                    request.body = Some(RequestBody {
                        body_mode: "form-data".to_string(),
                        raw: None,
                        raw_type: None,
                        form_data: Some(form_data),
                        urlencoded: None,
                        binary: None,
                    });
                } else {
                    // 解析失败，回退到 raw 模式
                    request.body = Some(RequestBody {
                        body_mode: "raw".to_string(),
                        raw: Some(body),
                        raw_type: Some("text".to_string()),
                        form_data: None,
                        urlencoded: None,
                        binary: None,
                    });
                }
            } else {
                // 没有 boundary，回退到 raw 模式
                request.body = Some(RequestBody {
                    body_mode: "raw".to_string(),
                    raw: Some(body),
                    raw_type: Some("text".to_string()),
                    form_data: None,
                    urlencoded: None,
                    binary: None,
                });
            }
        } else {
            // 非 multipart，保持原有的 raw 处理逻辑
            let raw_type = content_type
                .map(|ct| {
                    if ct.contains("json") {
                        "json".to_string()
                    } else if ct.contains("xml") {
                        "xml".to_string()
                    } else if ct.contains("html") {
                        "html".to_string()
                    } else {
                        "text".to_string()
                    }
                })
                .unwrap_or_else(|| {
                    if body.trim().starts_with('{') || body.trim().starts_with('[') {
                        "json".to_string()
                    } else if body.trim().starts_with('<') {
                        "xml".to_string()
                    } else {
                        "text".to_string()
                    }
                });

            request.body = Some(RequestBody {
                body_mode: "raw".to_string(),
                raw: Some(body),
                raw_type: Some(raw_type),
                form_data: None,
                urlencoded: None,
                binary: None,
            });
        }
    }

    if request.url.is_empty() {
        return Err("未找到 URL".to_string());
    }

    Ok(request)
}

fn tokenize_curl(curl: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = ' ';
    let mut escape_next = false;
    let mut is_ansi_c_quote = false;

    let mut chars = curl.chars().peekable();

    while let Some(ch) = chars.next() {
        if escape_next {
            current.push(ch);
            escape_next = false;
            continue;
        }

        if ch == '\\' && in_quotes && !is_ansi_c_quote {
            escape_next = true;
            continue;
        }

        if in_quotes {
            if ch == quote_char {
                in_quotes = false;
                if is_ansi_c_quote {
                    tokens.push(interpret_ansi_c_quoting(&current));
                } else {
                    tokens.push(current.clone());
                }
                current.clear();
                is_ansi_c_quote = false;
            } else {
                current.push(ch);
            }
        } else if ch == '$' && chars.peek() == Some(&'\'') {
            // ANSI-C quoting: $'...'
            chars.next(); // consume the quote
            in_quotes = true;
            quote_char = '\'';
            is_ansi_c_quote = true;
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
        } else if ch == '"' || ch == '\'' {
            in_quotes = true;
            quote_char = ch;
            is_ansi_c_quote = false;
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
        } else if ch == ' ' || ch == '\t' || ch == '\n' {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
        } else {
            current.push(ch);
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn base64_decode(encoded: &str) -> Result<String, String> {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    STANDARD
        .decode(encoded)
        .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
        .map_err(|e| e.to_string())
}

/// 从 Content-Type header 中提取 boundary
fn extract_boundary(content_type: &str) -> Option<String> {
    for part in content_type.split(';') {
        let trimmed = part.trim();
        if trimmed.to_lowercase().starts_with("boundary=") {
            let boundary = &trimmed[9..];
            // 移除可能的引号
            let boundary = boundary.trim_matches(|c| c == '"' || c == '\'');
            return Some(boundary.to_string());
        }
    }
    None
}

/// 解析 multipart/form-data body，提取各个字段
fn parse_multipart_body(body: &str, boundary: &str) -> Vec<crate::models::FormData> {
    let mut form_data = Vec::new();

    // 处理字面的 \r\n 字符串（转义为实际的 CRLF）
    let body = body.replace("\\r\\n", "\r\n").replace("\\n", "\n");

    // multipart 格式: --boundary\r\nContent-Disposition: form-data; name="xxx"\r\n\r\nvalue\r\n--boundary--
    // 按 boundary 分割
    let delimiter = format!("--{}", boundary.trim());
    let parts: Vec<&str> = body.split(&delimiter).collect();

    for part in parts {
        let part = part.trim();

        // 跳过空部分和结束标记 (--)
        if part.is_empty() || part == "--" || part.starts_with("--\r\n") || part.starts_with("--\n")
        {
            continue;
        }

        // 查找 Content-Disposition 行和值
        // 格式: Content-Disposition: form-data; name="fieldName"\r\n\r\nfieldValue\r\n
        let lines: Vec<&str> = part.splitn(3, "\r\n\r\n").collect();
        if lines.len() < 2 {
            // 尝试用 \n\n 分割（兼容性）
            let lines: Vec<&str> = part.splitn(3, "\n\n").collect();
            if lines.len() < 2 {
                continue;
            }

            let header_section = lines[0];
            let value_section = lines[1];

            // 从 header 提取 name
            if let Some(name) = extract_form_name(header_section) {
                // 移除末尾的 \r\n 或 \n
                let value = value_section
                    .trim_end_matches("\r\n")
                    .trim_end_matches('\n')
                    .to_string();

                // 检查是否是文件类型
                let (form_type, file_path) = if header_section.contains("filename=") {
                    ("file".to_string(), extract_filename(header_section))
                } else {
                    ("text".to_string(), None)
                };

                form_data.push(crate::models::FormData {
                    key: name,
                    value,
                    description: None,
                    enabled: true,
                    form_type,
                    file_path,
                });
            }
        } else {
            let header_section = lines[0];
            let value_section = lines[1];

            // 从 header 提取 name
            if let Some(name) = extract_form_name(header_section) {
                // 移除末尾的 \r\n 或 \n
                let value = value_section
                    .trim_end_matches("\r\n")
                    .trim_end_matches('\n')
                    .to_string();

                // 检查是否是文件类型
                let (form_type, file_path) = if header_section.contains("filename=") {
                    ("file".to_string(), extract_filename(header_section))
                } else {
                    ("text".to_string(), None)
                };

                form_data.push(crate::models::FormData {
                    key: name,
                    value,
                    description: None,
                    enabled: true,
                    form_type,
                    file_path,
                });
            }
        }
    }

    form_data
}

/// 从 Content-Disposition header 中提取 name 属性
fn extract_form_name(header: &str) -> Option<String> {
    // 查找 name="xxx" 或 name=xxx
    for line in header.lines() {
        let line = line.trim();
        if line.to_lowercase().contains("content-disposition:") {
            // 提取 name 属性
            if let Some(name_start) = line.find("name=\"") {
                let after_quote = &line[name_start + 6..];
                if let Some(name_end) = after_quote.find('"') {
                    return Some(after_quote[..name_end].to_string());
                }
            } else if let Some(name_start) = line.find("name=") {
                let after_equals = &line[name_start + 5..];
                // 找到分号或行尾
                let name = after_equals
                    .split(';')
                    .next()
                    .unwrap_or(after_equals)
                    .trim()
                    .trim_matches('"');
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_c_quoting() {
        let input = "hello\\r\\nworld\\ttab";
        let result = interpret_ansi_c_quoting(input);
        assert_eq!(result, "hello\r\nworld\ttab");
    }

    #[test]
    fn test_tokenize_ansi_c_quote() {
        let curl = "curl --data-raw $'line1\\r\\nline2'";
        let tokens = tokenize_curl(curl);
        assert_eq!(tokens, vec!["curl", "--data-raw", "line1\r\nline2"]);
    }

    #[test]
    fn test_parse_multipart_with_ansi_c_quoting() {
        let curl = r#"curl 'http://example.com/api' \
          -H 'Content-Type: multipart/form-data; boundary=----WebKitFormBoundary' \
          --data-raw $'------WebKitFormBoundary\r\nContent-Disposition: form-data; name="field1"\r\n\r\nvalue1\r\n------WebKitFormBoundary--\r\n'"#;

        let result = parse_curl(curl).unwrap();
        assert_eq!(result.method, "POST");
        assert!(result.body.is_some());

        let body = result.body.unwrap();
        assert_eq!(body.body_mode, "form-data");
        assert!(body.form_data.is_some());

        let form_data = body.form_data.unwrap();
        assert_eq!(form_data.len(), 1);
        assert_eq!(form_data[0].key, "field1");
        assert_eq!(form_data[0].value, "value1");
    }

    #[test]
    fn test_parse_multipart_real_world() {
        // 模拟真实世界的 curl 命令（浏览器复制的格式）
        let curl = r#"curl 'http://172.18.11.114:8081/api/test' \
          -H 'Content-Type: multipart/form-data; boundary=----WebKitFormBoundaryqcerWfRlcIrtxkrd' \
          --data-raw $'------WebKitFormBoundaryqcerWfRlcIrtxkrd\r\nContent-Disposition: form-data; name="emodelId"\r\n\r\n1961350648283152385\r\n------WebKitFormBoundaryqcerWfRlcIrtxkrd\r\nContent-Disposition: form-data; name="timeCategory"\r\n\r\nmonth\r\n------WebKitFormBoundaryqcerWfRlcIrtxkrd--\r\n'"#;

        let result = parse_curl(curl).unwrap();
        assert_eq!(result.method, "POST");
        assert!(result.body.is_some());

        let body = result.body.unwrap();
        assert_eq!(body.body_mode, "form-data");

        let form_data = body.form_data.unwrap();
        assert_eq!(form_data.len(), 2);
        assert_eq!(form_data[0].key, "emodelId");
        assert_eq!(form_data[0].value, "1961350648283152385");
        assert_eq!(form_data[1].key, "timeCategory");
        assert_eq!(form_data[1].value, "month");
    }

    #[test]
    fn test_parse_insecure_flag() {
        let curl = "curl 'http://example.com/api' --insecure";
        let result = parse_curl(curl).unwrap();
        assert_eq!(result.verify_ssl, Some(false));
    }

    #[test]
    fn test_parse_insecure_short() {
        let curl = "curl -k 'https://example.com/api'";
        let result = parse_curl(curl).unwrap();
        assert_eq!(result.verify_ssl, Some(false));
    }

    #[test]
    fn test_parse_header_with_semicolon_no_value() {
        let curl = "curl 'http://example.com/api' -H 'X-Service;'";
        let result = parse_curl(curl).unwrap();
        let service_headers: Vec<_> = result
            .headers
            .iter()
            .filter(|h| h.key == "X-Service")
            .collect();
        assert_eq!(service_headers.len(), 1);
        assert_eq!(service_headers[0].key, "X-Service");
        assert_eq!(service_headers[0].value, "");
    }

    #[test]
    fn test_parse_skip_unknown_flags() {
        let curl = "curl --compressed --silent -v 'http://example.com/api'";
        let result = parse_curl(curl).unwrap();
        assert_eq!(result.url, "http://example.com/api");
    }

    #[test]
    fn test_parse_complex_real_world_curl() {
        let curl = r#"curl 'http://172.1.70.14:18081/api/test?foo=bar' \
  -H 'Accept: application/json, text/plain, */*' \
  -H 'Authorization: 16aeb37cd9b7b77008cb53207a4a9b1b' \
  -H 'X-Access-Token: 16aeb37cd9b7b77008cb53207a4a9b1b' \
  -H 'X-Service;' \
  -H 'X-Tenant-Id: 8888' \
  -b '_webtracing_device_id=t_13526a67' \
  -H 'Referer: http://172.1.70.14:18081/neo/test' \
  --insecure"#;

        let result = parse_curl(curl).unwrap();
        assert_eq!(result.url, "http://172.1.70.14:18081/api/test?foo=bar");
        assert_eq!(result.verify_ssl, Some(false));

        let auth_header = result
            .headers
            .iter()
            .find(|h| h.key == "Authorization")
            .unwrap();
        assert_eq!(auth_header.value, "16aeb37cd9b7b77008cb53207a4a9b1b");

        let service_headers: Vec<_> = result
            .headers
            .iter()
            .filter(|h| h.key == "X-Service")
            .collect();
        assert_eq!(service_headers.len(), 1);
        assert_eq!(service_headers[0].value, "");

        let cookie_header = result
            .headers
            .iter()
            .find(|h| h.key == "Cookie")
            .unwrap();
        assert_eq!(cookie_header.value, "_webtracing_device_id=t_13526a67");

        assert_eq!(result.params.len(), 1);
        assert_eq!(result.params[0].key, "foo");
        assert_eq!(result.params[0].value, "bar");
    }
}

/// 从 Content-Disposition header 中提取 filename 属性
fn extract_filename(header: &str) -> Option<String> {
    for line in header.lines() {
        let line = line.trim();
        if line.to_lowercase().contains("content-disposition:") {
            if let Some(name_start) = line.find("filename=\"") {
                let after_quote = &line[name_start + 10..];
                if let Some(name_end) = after_quote.find('"') {
                    return Some(after_quote[..name_end].to_string());
                }
            } else if let Some(name_start) = line.find("filename=") {
                let after_equals = &line[name_start + 9..];
                let name = after_equals
                    .split(';')
                    .next()
                    .unwrap_or(after_equals)
                    .trim()
                    .trim_matches('"');
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }
    }
    None
}

/// URL 解码函数
fn urlencoding_decode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();

    while let Some(ch) = chars.next() {
        if ch == '%' {
            // 读取两个十六进制字符
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                } else {
                    result.push('%');
                    result.push_str(&hex);
                }
            } else {
                result.push('%');
                result.push_str(&hex);
            }
        } else if ch == '+' {
            result.push(' ');
        } else {
            result.push(ch);
        }
    }

    result
}

/// 解释 ANSI-C 引用中的转义序列（bash $'...' 格式）
fn interpret_ansi_c_quoting(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('\\') => result.push('\\'),
                Some('\'') => result.push('\''),
                Some('"') => result.push('"'),
                Some('a') => result.push('\x07'), // bell
                Some('b') => result.push('\x08'), // backspace
                Some('f') => result.push('\x0C'), // form feed
                Some('v') => result.push('\x0B'), // vertical tab
                Some('e') | Some('E') => result.push('\x1B'), // escape
                Some('x') => {
                    // \xHH - 十六进制
                    let hex: String = chars.by_ref().take(2).collect();
                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                        result.push(byte as char);
                    }
                }
                Some(other) => {
                    // 未知转义序列，保留原样
                    result.push('\\');
                    result.push(other);
                }
                None => {
                    result.push('\\');
                }
            }
        } else {
            result.push(ch);
        }
    }

    result
}
