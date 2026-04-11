#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IApiError {
    #[error("数据库错误: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("HTTP请求错误: {0}")]
    Http(#[from] reqwest::Error),

    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("脚本执行错误: {0}")]
    Script(String),

    #[error("解析错误: {0}")]
    Parse(String),

    #[error("验证错误: {0}")]
    Validation(String),

    #[error("未找到资源: {0}")]
    NotFound(String),

    #[error("请求已取消")]
    Cancelled,

    #[error("超时错误")]
    Timeout,

    #[error("未知错误: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, IApiError>;

impl Serialize for IApiError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<IApiError> for String {
    fn from(error: IApiError) -> Self {
        error.to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
}

impl ErrorResponse {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

impl From<&IApiError> for ErrorResponse {
    fn from(error: &IApiError) -> Self {
        let (code, message) = match error {
            IApiError::Database(_) => ("DATABASE_ERROR", "数据库操作失败"),
            IApiError::Http(_) => ("HTTP_ERROR", "HTTP请求失败"),
            IApiError::Io(_) => ("IO_ERROR", "文件操作失败"),
            IApiError::Serialization(_) => ("SERIALIZATION_ERROR", "数据序列化失败"),
            IApiError::Script(_) => ("SCRIPT_ERROR", "脚本执行失败"),
            IApiError::Parse(_) => ("PARSE_ERROR", "解析失败"),
            IApiError::Validation(_) => ("VALIDATION_ERROR", "数据验证失败"),
            IApiError::NotFound(_) => ("NOT_FOUND", "资源未找到"),
            IApiError::Cancelled => ("CANCELLED", "请求已取消"),
            IApiError::Timeout => ("TIMEOUT", "请求超时"),
            IApiError::Unknown(_) => ("UNKNOWN_ERROR", "未知错误"),
        };

        ErrorResponse::new(code, message).with_details(error.to_string())
    }
}

pub fn error_to_string(error: &IApiError) -> String {
    match error {
        IApiError::Database(e) => format!("数据库错误: {}", e),
        IApiError::Http(e) => {
            if e.is_timeout() {
                "请求超时，请检查网络连接或增加超时时间".to_string()
            } else if e.is_connect() {
                "无法连接到服务器，请检查网络连接".to_string()
            } else if e.is_status() {
                format!("HTTP错误: {}", e)
            } else {
                format!("请求失败: {}", e)
            }
        }
        IApiError::Io(e) => format!("文件操作失败: {}", e),
        IApiError::Serialization(e) => format!("数据格式错误: {}", e),
        IApiError::Script(e) => format!("脚本执行错误: {}", e),
        IApiError::Parse(e) => format!("解析错误: {}", e),
        IApiError::Validation(e) => format!("验证失败: {}", e),
        IApiError::NotFound(e) => format!("未找到: {}", e),
        IApiError::Cancelled => "请求已取消".to_string(),
        IApiError::Timeout => "请求超时".to_string(),
        IApiError::Unknown(e) => format!("未知错误: {}", e),
    }
}
