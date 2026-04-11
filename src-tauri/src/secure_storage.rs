#![allow(dead_code)]

use crate::error::{IApiError, Result};
use keyring::Entry;
use serde::{Deserialize, Serialize};

const SERVICE_NAME: &str = "iApi";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureCredential {
    pub request_id: String,
    pub auth_type: String,
    pub key: String,
    pub value: String,
}

pub struct SecureStorage;

impl SecureStorage {
    fn build_key(request_id: &str, auth_type: &str, key: &str) -> String {
        format!("{}_{}_{}", request_id, auth_type, key)
    }

    pub fn store_credential(
        request_id: &str,
        auth_type: &str,
        key: &str,
        value: &str,
    ) -> Result<()> {
        let entry_key = Self::build_key(request_id, auth_type, key);
        let entry = Entry::new(SERVICE_NAME, &entry_key)
            .map_err(|e| IApiError::Unknown(format!("创建密钥条目失败: {}", e)))?;

        entry
            .set_password(value)
            .map_err(|e| IApiError::Unknown(format!("存储凭证失败: {}", e)))?;

        log::info!("凭证已安全存储: {}", entry_key);
        Ok(())
    }

    pub fn retrieve_credential(
        request_id: &str,
        auth_type: &str,
        key: &str,
    ) -> Result<Option<String>> {
        let entry_key = Self::build_key(request_id, auth_type, key);
        let entry = Entry::new(SERVICE_NAME, &entry_key)
            .map_err(|e| IApiError::Unknown(format!("创建密钥条目失败: {}", e)))?;

        match entry.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(IApiError::Unknown(format!("检索凭证失败: {}", e))),
        }
    }

    pub fn delete_credential(request_id: &str, auth_type: &str, key: &str) -> Result<()> {
        let entry_key = Self::build_key(request_id, auth_type, key);
        let entry = Entry::new(SERVICE_NAME, &entry_key)
            .map_err(|e| IApiError::Unknown(format!("创建密钥条目失败: {}", e)))?;

        entry
            .delete_credential()
            .map_err(|e| IApiError::Unknown(format!("删除凭证失败: {}", e)))?;

        log::info!("凭证已删除: {}", entry_key);
        Ok(())
    }

    pub fn store_bearer_token(request_id: &str, token: &str) -> Result<()> {
        Self::store_credential(request_id, "bearer", "token", token)
    }

    pub fn retrieve_bearer_token(request_id: &str) -> Result<Option<String>> {
        Self::retrieve_credential(request_id, "bearer", "token")
    }

    pub fn delete_bearer_token(request_id: &str) -> Result<()> {
        Self::delete_credential(request_id, "bearer", "token")
    }

    pub fn store_basic_auth(request_id: &str, username: &str, password: &str) -> Result<()> {
        Self::store_credential(request_id, "basic", "username", username)?;
        Self::store_credential(request_id, "basic", "password", password)
    }

    pub fn retrieve_basic_auth(request_id: &str) -> Result<Option<(String, String)>> {
        let username = Self::retrieve_credential(request_id, "basic", "username")?;
        let password = Self::retrieve_credential(request_id, "basic", "password")?;

        match (username, password) {
            (Some(u), Some(p)) => Ok(Some((u, p))),
            _ => Ok(None),
        }
    }

    pub fn delete_basic_auth(request_id: &str) -> Result<()> {
        Self::delete_credential(request_id, "basic", "username")?;
        Self::delete_credential(request_id, "basic", "password")
    }

    pub fn store_api_key(request_id: &str, key: &str, value: &str) -> Result<()> {
        Self::store_credential(request_id, "apikey", key, value)
    }

    pub fn retrieve_api_key(request_id: &str, key: &str) -> Result<Option<String>> {
        Self::retrieve_credential(request_id, "apikey", key)
    }

    pub fn delete_api_key(request_id: &str, key: &str) -> Result<()> {
        Self::delete_credential(request_id, "apikey", key)
    }

    pub fn store_environment_secret(env_id: &str, key: &str, value: &str) -> Result<()> {
        Self::store_credential(env_id, "env", key, value)
    }

    pub fn retrieve_environment_secret(env_id: &str, key: &str) -> Result<Option<String>> {
        Self::retrieve_credential(env_id, "env", key)
    }

    pub fn delete_environment_secret(env_id: &str, key: &str) -> Result<()> {
        Self::delete_credential(env_id, "env", key)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSecrets {
    pub bearer_token: Option<String>,
    pub basic_username: Option<String>,
    pub basic_password: Option<String>,
    pub api_key_value: Option<String>,
}

impl AuthSecrets {
    pub fn new() -> Self {
        Self {
            bearer_token: None,
            basic_username: None,
            basic_password: None,
            api_key_value: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.bearer_token.is_none()
            && self.basic_username.is_none()
            && self.basic_password.is_none()
            && self.api_key_value.is_none()
    }
}

pub fn is_sensitive_field(auth_type: &str, field: &str) -> bool {
    match auth_type {
        "bearer" => field == "token",
        "basic" => field == "password",
        "apikey" => field == "value",
        _ => false,
    }
}

pub fn mask_sensitive_value(value: &str) -> String {
    if value.len() <= 4 {
        return "*".repeat(value.len());
    }
    let visible = &value[..2];
    let masked = "*".repeat(value.len() - 4);
    let end = &value[value.len() - 2..];
    format!("{}{}{}", visible, masked, end)
}
