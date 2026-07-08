#![allow(dead_code)]

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptContext {
    pub variables: HashMap<String, String>,
    pub test_results: Vec<TestResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub error: Option<String>,
    pub duration_ms: Option<u64>,
}

impl Default for ScriptContext {
    fn default() -> Self {
        Self {
            variables: HashMap::new(),
            test_results: Vec::new(),
        }
    }
}

/// 脚本编译缓存
static SCRIPT_CACHE: Lazy<Arc<dashmap::DashMap<String, CompiledScript>>> =
    Lazy::new(|| Arc::new(dashmap::DashMap::new()));

#[derive(Clone)]
struct CompiledScript {
    source: String,
    tokens: Vec<Token>,
}

#[derive(Clone, Debug)]
enum Token {
    VariableSet {
        key: String,
        value: String,
    },
    EnvironmentSet {
        key: String,
        value: String,
    },
    Test {
        name: String,
        assertion: Assertion,
    },
    RequestUrl {
        url: String,
    },
    RequestMethod {
        method: String,
    },
    ResponseGet {
        var_name: String,
        expression: ResponseExpression,
    },
}

#[derive(Clone, Debug)]
enum ResponseExpression {
    Json,
    Text,
    Header(String),
    Status,
    Time,
    Size,
}

#[derive(Clone, Debug)]
enum Assertion {
    StatusEquals(u16),
    TimeBelow(u64),
    TimeAbove(u64),
    HasBody,
    HasHeader(String),
    JsonPath { path: String, expected: String },
}

pub fn execute_pre_request_script(
    script: &str,
    request: &mut crate::models::HttpRequest,
    environment: &HashMap<String, String>,
) -> Result<ScriptContext, String> {
    let mut context = ScriptContext::default();

    if script.trim().is_empty() {
        return Ok(context);
    }

    // 环境变量替换
    let mut processed_script = script.to_string();
    for (key, value) in environment {
        processed_script = processed_script.replace(&format!("{{{{{}}}}}", key), value);
    }

    // 编译（带缓存）并执行替换后的脚本
    let compiled = compile_script(&processed_script);
    execute_tokens(&compiled.tokens, request, None, &mut context);

    Ok(context)
}

pub fn execute_post_request_script(
    script: &str,
    _request: &crate::models::HttpRequest,
    response: &crate::models::HttpResponse,
    environment: &HashMap<String, String>,
) -> Result<ScriptContext, String> {
    let mut context = ScriptContext::default();

    if script.trim().is_empty() {
        return Ok(context);
    }

    // 环境变量替换
    let mut processed_script = script.to_string();
    for (key, value) in environment {
        processed_script = processed_script.replace(&format!("{{{{{}}}}}", key), value);
    }

    // 编译并执行脚本
    let compiled = compile_script(&processed_script);
    execute_tokens(
        &compiled.tokens,
        std::ptr::null_mut(),
        Some(response),
        &mut context,
    );

    Ok(context)
}

/// 编译脚本（带缓存）
fn compile_script(source: &str) -> CompiledScript {
    let cache_key = format!("{:x}", md5::compute(source.as_bytes()));

    if let Some(cached) = SCRIPT_CACHE.get(&cache_key) {
        return cached.clone();
    }

    let tokens = parse_script(source);
    let compiled = CompiledScript {
        source: source.to_string(),
        tokens,
    };

    SCRIPT_CACHE.insert(cache_key, compiled.clone());
    compiled
}

/// 解析脚本为 token 流
fn parse_script(script: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    // 解析 pm.variables.set - 支持值中包含逗号的情况
    let var_pattern = "pm.variables.set(";
    let mut pos = 0;
    while let Some(start) = script[pos..].find(var_pattern) {
        let set_start = pos + start + var_pattern.len();
        // 查找匹配的括号，处理嵌套和引号
        if let Some(args_end) = find_matching_paren(&script[set_start..]) {
            let args = &script[set_start..set_start + args_end];
            // 使用更智能的方式分割参数，考虑引号内的内容
            if let Some((key, value)) = parse_function_args(args) {
                tokens.push(Token::VariableSet { key, value });
            }
            pos = set_start + args_end + 1;
        } else {
            break;
        }
    }

    // 解析 pm.environment.set
    let env_pattern = "pm.environment.set(";
    pos = 0;
    while let Some(start) = script[pos..].find(env_pattern) {
        let set_start = pos + start + env_pattern.len();
        if let Some(args_end) = find_matching_paren(&script[set_start..]) {
            let args = &script[set_start..set_start + args_end];
            if let Some((key, value)) = parse_function_args(args) {
                tokens.push(Token::EnvironmentSet { key, value });
            }
            pos = set_start + args_end + 1;
        } else {
            break;
        }
    }

    // 解析 pm.test
    let test_pattern = "pm.test(";
    pos = 0;
    while let Some(start) = script[pos..].find(test_pattern) {
        let test_start = pos + start + test_pattern.len();
        if let Some(name_end) = find_closing_quote_and_comma(&script[test_start..]) {
            let name = script[test_start..test_start + name_end]
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();

            let assertion = parse_assertion(&script[test_start + name_end..]);
            tokens.push(Token::Test { name, assertion });

            pos = test_start + name_end;
            if let Some(func_end) = script[pos..].find("});") {
                pos += func_end + 3;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // 解析 pm.request.url
    if let Some(url_start) = script.find("pm.request.url = ") {
        let start = url_start + "pm.request.url = ".len();
        if let Some(end) = script[start..].find(';') {
            let url = script[start..start + end]
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            tokens.push(Token::RequestUrl { url });
        }
    }

    // 解析 pm.request.method
    if let Some(method_start) = script.find("pm.request.method = ") {
        let start = method_start + "pm.request.method = ".len();
        if let Some(end) = script[start..].find(';') {
            let method = script[start..start + end]
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_uppercase();
            tokens.push(Token::RequestMethod { method });
        }
    }

    // 解析 pm.response.json()
    if let Some(json_pos) = script.find("pm.response.json()") {
        if let Some(var_name) = extract_var_name_before(script, json_pos) {
            tokens.push(Token::ResponseGet {
                var_name,
                expression: ResponseExpression::Json,
            });
        }
    }

    // 解析 pm.response.text()
    if let Some(text_pos) = script.find("pm.response.text()") {
        if let Some(var_name) = extract_var_name_before(script, text_pos) {
            tokens.push(Token::ResponseGet {
                var_name,
                expression: ResponseExpression::Text,
            });
        }
    }

    // 解析 pm.response.headers.get("header-name")
    let header_pattern = "pm.response.headers.get(";
    let mut header_pos = 0;
    while let Some(start) = script[header_pos..].find(header_pattern) {
        let get_start = header_pos + start + header_pattern.len();
        if let Some(end) = script[get_start..].find(')') {
            let header_name = script[get_start..get_start + end]
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();

            let full_pos = header_pos + start;
            if let Some(var_name) = extract_var_name_before(script, full_pos) {
                tokens.push(Token::ResponseGet {
                    var_name,
                    expression: ResponseExpression::Header(header_name),
                });
            }

            header_pos = get_start + end + 1;
        } else {
            break;
        }
    }

    // 解析 pm.response.status
    if script.contains("pm.response.status") {
        let var_match = script.find("pm.response.status");
        if let Some(status_pos) = var_match {
            let before = &script[..status_pos];
            if let Some(eq_pos) = before.rfind('=') {
                let var_name = script[eq_pos + 1..status_pos].trim().to_string();
                if !var_name.is_empty() && !var_name.contains('.') {
                    tokens.push(Token::ResponseGet {
                        var_name,
                        expression: ResponseExpression::Status,
                    });
                }
            }
        }
    }

    // 解析 pm.response.time
    if script.contains("pm.response.time") {
        let var_match = script.find("pm.response.time");
        if let Some(time_pos) = var_match {
            let before = &script[..time_pos];
            if let Some(eq_pos) = before.rfind('=') {
                let var_name = script[eq_pos + 1..time_pos].trim().to_string();
                if !var_name.is_empty() && !var_name.contains('.') {
                    tokens.push(Token::ResponseGet {
                        var_name,
                        expression: ResponseExpression::Time,
                    });
                }
            }
        }
    }

    tokens
}

/// 从表达式的起始位置往前找赋值语句，提取变量名
/// 例如 "const token = pm.response.json()" 中 expr_pos 指向 "pm"，返回 Some("token")
fn extract_var_name_before(script: &str, expr_pos: usize) -> Option<String> {
    let before = &script[..expr_pos];
    let eq_pos = before.rfind('=')?;
    // 确认是赋值（排除 ==、>=、<=、!= 等比较运算符）
    let before_eq = &script[..eq_pos];
    if before_eq.ends_with('=') || before_eq.ends_with('>') || before_eq.ends_with('<') || before_eq.ends_with('!') {
        return None;
    }
    let var_name = script[eq_pos + 1..expr_pos].trim().to_string();
    // 变量名必须合法：不含空格、运算符、点
    if var_name.is_empty() || var_name.contains('.') || var_name.contains(char::is_whitespace) {
        return None;
    }
    Some(var_name)
}

/// 查找匹配的右括号
fn find_matching_paren(s: &str) -> Option<usize> {
    let mut depth = 1;
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escape_next = false;

    for (i, ch) in s.chars().enumerate() {
        if escape_next {
            escape_next = false;
            continue;
        }

        if ch == '\\' && (in_single_quote || in_double_quote) {
            escape_next = true;
            continue;
        }

        if !in_single_quote && !in_double_quote {
            if ch == '"' {
                in_double_quote = true;
            } else if ch == '\'' {
                in_single_quote = true;
            } else if ch == '(' {
                depth += 1;
            } else if ch == ')' {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
        } else if in_single_quote && ch == '\'' {
            in_single_quote = false;
        } else if in_double_quote && ch == '"' {
            in_double_quote = false;
        }
    }

    None
}

/// 解析函数参数，返回 (key, value)
fn parse_function_args(args: &str) -> Option<(String, String)> {
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escape_next = false;
    let mut comma_pos = None;

    // 找到第一个不在引号内的逗号
    for (i, ch) in args.chars().enumerate() {
        if escape_next {
            escape_next = false;
            continue;
        }

        if ch == '\\' && (in_single_quote || in_double_quote) {
            escape_next = true;
            continue;
        }

        if !in_single_quote && !in_double_quote {
            if ch == '"' {
                in_double_quote = true;
            } else if ch == '\'' {
                in_single_quote = true;
            } else if ch == ',' {
                comma_pos = Some(i);
                break;
            }
        } else if in_single_quote && ch == '\'' {
            in_single_quote = false;
        } else if in_double_quote && ch == '"' {
            in_double_quote = false;
        }
    }

    let comma_pos = comma_pos?;
    let key = args[..comma_pos]
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string();
    let value = args[comma_pos + 1..]
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string();

    Some((key, value))
}

/// 查找闭合引号和逗号的位置
fn find_closing_quote_and_comma(s: &str) -> Option<usize> {
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escape_next = false;

    for (i, ch) in s.chars().enumerate() {
        if escape_next {
            escape_next = false;
            continue;
        }

        if ch == '\\' && (in_single_quote || in_double_quote) {
            escape_next = true;
            continue;
        }

        if !in_single_quote && !in_double_quote {
            if ch == '"' {
                in_double_quote = true;
            } else if ch == '\'' {
                in_single_quote = true;
            }
        } else if in_single_quote && ch == '\'' {
            in_single_quote = false;
        } else if in_double_quote && ch == '"' {
            in_double_quote = false;
        } else if (in_single_quote || in_double_quote) && ch == ',' {
            // 在引号内的逗号，继续
            continue;
        }

        // 如果刚关闭引号且下一个字符是逗号
        if !in_single_quote && !in_double_quote && i + 1 < s.len() && s[i + 1..].starts_with(',') {
            return Some(i + 1);
        }
    }

    None
}

/// 解析断言
fn parse_assertion(script: &str) -> Assertion {
    if script.contains("to.equal(") || script.contains(".equals(") {
        if let Some(eq_start) = script.find("to.equal(").or_else(|| script.find(".equals(")) {
            let start = eq_start + "to.equal(".len();
            if let Some(end) = script[start..].find(')') {
                if let Ok(status) = script[start..start + end].parse::<u16>() {
                    return Assertion::StatusEquals(status);
                }
            }
        }
    }

    if script.contains("to.be.below(") {
        if let Some(below_start) = script.find("to.be.below(") {
            let start = below_start + "to.be.below(".len();
            if let Some(end) = script[start..].find(')') {
                if let Ok(threshold) = script[start..start + end].parse::<u64>() {
                    return Assertion::TimeBelow(threshold);
                }
            }
        }
    }

    if script.contains("to.be.above(") {
        if let Some(above_start) = script.find("to.be.above(") {
            let start = above_start + "to.be.above(".len();
            if let Some(end) = script[start..].find(')') {
                if let Ok(threshold) = script[start..start + end].parse::<u64>() {
                    return Assertion::TimeAbove(threshold);
                }
            }
        }
    }

    if script.contains("to.have.body()") {
        return Assertion::HasBody;
    }

    if script.contains("pm.response.to.have.header(") {
        if let Some(header_start) = script.find("pm.response.to.have.header(") {
            let start = header_start + "pm.response.to.have.header(".len();
            if let Some(end) = script[start..].find(')') {
                let header = script[start..start + end]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                return Assertion::HasHeader(header);
            }
        }
    }

    Assertion::StatusEquals(200) // 默认断言
}

/// 执行 token
fn execute_tokens(
    tokens: &[Token],
    request: *mut crate::models::HttpRequest,
    response: Option<&crate::models::HttpResponse>,
    context: &mut ScriptContext,
) {
    let mut request_ref = if request.is_null() {
        None
    } else {
        Some(unsafe { &mut *request })
    };

    for token in tokens {
        match token {
            Token::VariableSet { key, value } => {
                context.variables.insert(key.clone(), value.clone());
            }
            Token::EnvironmentSet { key, value } => {
                context.variables.insert(key.clone(), value.clone());
            }
            Token::Test { name, assertion } => {
                let start = std::time::Instant::now();
                let passed = evaluate_assertion(assertion, response);
                let duration = start.elapsed().as_millis() as u64;

                context.test_results.push(TestResult {
                    name: name.clone(),
                    passed,
                    error: if passed {
                        None
                    } else {
                        Some("断言失败".to_string())
                    },
                    duration_ms: Some(duration),
                });
            }
            Token::RequestUrl { url } => {
                if let Some(req) = &mut request_ref {
                    req.url = url.clone();
                }
            }
            Token::RequestMethod { method } => {
                if let Some(req) = &mut request_ref {
                    req.method = method.clone();
                }
            }
            Token::ResponseGet {
                var_name,
                expression,
            } => {
                if let Some(resp) = response {
                    let value = match expression {
                        ResponseExpression::Json => serde_json::from_str::<Value>(&resp.body)
                            .map(|v| serde_json::to_string_pretty(&v).unwrap_or_default())
                            .unwrap_or_default(),
                        ResponseExpression::Text => resp.body.clone(),
                        ResponseExpression::Header(header_name) => {
                            resp.headers.get(header_name).cloned().unwrap_or_default()
                        }
                        ResponseExpression::Status => resp.status.to_string(),
                        ResponseExpression::Time => resp.response_time.to_string(),
                        ResponseExpression::Size => resp.response_size.to_string(),
                    };
                    context.variables.insert(var_name.clone(), value);
                }
            }
        }
    }
}

/// 评估断言
fn evaluate_assertion(
    assertion: &Assertion,
    response: Option<&crate::models::HttpResponse>,
) -> bool {
    let Some(response) = response else {
        return false;
    };

    match assertion {
        Assertion::StatusEquals(expected) => response.status == *expected,
        Assertion::TimeBelow(threshold) => response.response_time < *threshold,
        Assertion::TimeAbove(threshold) => response.response_time > *threshold,
        Assertion::HasBody => !response.body.is_empty(),
        Assertion::HasHeader(header) => response.headers.contains_key(header),
        Assertion::JsonPath { path, expected } => {
            extract_json_value(&response.body, path).map_or(false, |v| &v == expected)
        }
    }
}

pub fn extract_json_value(json: &str, path: &str) -> Option<String> {
    let value: Value = serde_json::from_str(json).ok()?;

    let parts: Vec<&str> = path.split('.').collect();
    let mut current = &value;

    for part in parts {
        if part.starts_with('$') {
            continue;
        }

        if let Some(idx) = part.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            if let Ok(index) = idx.parse::<usize>() {
                if let Value::Array(arr) = current {
                    current = arr.get(index)?;
                } else {
                    return None;
                }
            }
        } else if let Value::Object(map) = current {
            current = map.get(part)?;
        } else {
            return None;
        }
    }

    match current {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Null => Some("null".to_string()),
        _ => Some(current.to_string()),
    }
}
