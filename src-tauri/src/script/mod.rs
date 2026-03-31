use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptContext {
    pub variables: std::collections::HashMap<String, String>,
    pub test_results: Vec<TestResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub error: Option<String>,
}

impl Default for ScriptContext {
    fn default() -> Self {
        Self {
            variables: std::collections::HashMap::new(),
            test_results: Vec::new(),
        }
    }
}

pub fn execute_pre_request_script(
    script: &str,
    request: &mut crate::models::HttpRequest,
    environment: &std::collections::HashMap<String, String>,
) -> Result<ScriptContext, String> {
    let mut context = ScriptContext::default();

    if script.trim().is_empty() {
        return Ok(context);
    }

    let mut processed_script = script.to_string();

    for (key, value) in environment {
        processed_script = processed_script.replace(&format!("{{{{{}}}}}", key), value);
    }

    if processed_script.contains("pm.request.url") {
        if let Some(url_start) = processed_script.find("pm.request.url = ") {
            let start = url_start + "pm.request.url = ".len();
            if let Some(end) = processed_script[start..].find(';') {
                let url_value = processed_script[start..start + end].trim();
                let url = url_value.trim_matches('"').trim_matches('\'');
                request.url = url.to_string();
            }
        }
    }

    if processed_script.contains("pm.request.method") {
        if let Some(method_start) = processed_script.find("pm.request.method = ") {
            let start = method_start + "pm.request.method = ".len();
            if let Some(end) = processed_script[start..].find(';') {
                let method_value = processed_script[start..start + end].trim();
                let method = method_value.trim_matches('"').trim_matches('\'');
                request.method = method.to_uppercase();
            }
        }
    }

    extract_variables(&processed_script, &mut context);

    Ok(context)
}

pub fn execute_post_request_script(
    script: &str,
    _request: &crate::models::HttpRequest,
    response: &crate::models::HttpResponse,
    environment: &std::collections::HashMap<String, String>,
) -> Result<ScriptContext, String> {
    let mut context = ScriptContext::default();

    if script.trim().is_empty() {
        return Ok(context);
    }

    let mut processed_script = script.to_string();

    for (key, value) in environment {
        processed_script = processed_script.replace(&format!("{{{{{}}}}}", key), value);
    }

    if processed_script.contains("pm.test(") {
        parse_tests(&processed_script, response, &mut context);
    }

    if processed_script.contains("pm.environment.set(") {
        extract_environment_sets(&processed_script, &mut context);
    }

    extract_variables(&processed_script, &mut context);

    Ok(context)
}

fn parse_tests(script: &str, response: &crate::models::HttpResponse, context: &mut ScriptContext) {
    let test_pattern = "pm.test(";
    let mut pos = 0;

    while let Some(start) = script[pos..].find(test_pattern) {
        let test_start = pos + start + test_pattern.len();

        if let Some(name_end) = script[test_start..].find(',') {
            let name = script[test_start..test_start + name_end]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');

            let passed = check_test_condition(&script[test_start + name_end..], response);

            context.test_results.push(TestResult {
                name: name.to_string(),
                passed,
                error: if passed {
                    None
                } else {
                    Some("断言失败".to_string())
                },
            });

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
}

fn check_test_condition(script: &str, response: &crate::models::HttpResponse) -> bool {
    if script.contains("pm.response.status") || script.contains("pm.response.code") {
        if script.contains(&format!("{}.equals(", response.status))
            || script.contains(&format!("== {}", response.status))
            || script.contains(&format!("=== {}", response.status))
        {
            return true;
        }

        if script.contains("to.equal(200)") && response.status == 200 {
            return true;
        }

        if script.contains("to.be.below(") {
            if let Some(below_start) = script.find("to.be.below(") {
                let start = below_start + "to.be.below(".len();
                if let Some(end) = script[start..].find(')') {
                    if let Ok(threshold) = script[start..start + end].parse::<u64>() {
                        return response.response_time < threshold;
                    }
                }
            }
        }
    }

    true
}

fn extract_environment_sets(script: &str, context: &mut ScriptContext) {
    let set_pattern = "pm.environment.set(";
    let mut pos = 0;

    while let Some(start) = script[pos..].find(set_pattern) {
        let set_start = pos + start + set_pattern.len();

        if let Some(args_end) = script[set_start..].find(')') {
            let args = &script[set_start..set_start + args_end];
            let parts: Vec<&str> = args.split(',').collect();

            if parts.len() == 2 {
                let key = parts[0].trim().trim_matches('"').trim_matches('\'');
                let value = parts[1].trim().trim_matches('"').trim_matches('\'');
                context.variables.insert(key.to_string(), value.to_string());
            }

            pos = set_start + args_end + 1;
        } else {
            break;
        }
    }
}

fn extract_variables(script: &str, context: &mut ScriptContext) {
    let var_pattern = "pm.variables.set(";
    let mut pos = 0;

    while let Some(start) = script[pos..].find(var_pattern) {
        let set_start = pos + start + var_pattern.len();

        if let Some(args_end) = script[set_start..].find(')') {
            let args = &script[set_start..set_start + args_end];
            let parts: Vec<&str> = args.split(',').collect();

            if parts.len() == 2 {
                let key = parts[0].trim().trim_matches('"').trim_matches('\'');
                let value = parts[1].trim().trim_matches('"').trim_matches('\'');
                context.variables.insert(key.to_string(), value.to_string());
            }

            pos = set_start + args_end + 1;
        } else {
            break;
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
