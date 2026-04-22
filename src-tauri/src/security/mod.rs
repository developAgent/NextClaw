//! 安全模块
//! 提供 API Key 加密存储和 Keyring 集成

pub mod encryption;

use crate::db::Database;
use crate::utils::error::{AppError, Result};
use keyring::Entry;
use secrecy::{ExposeSecret, SecretString};
use std::sync::Arc;

/// API Key 存储服务
/// 分层策略：
/// 1. 优先使用系统 Keyring (Windows Credential Manager / macOS Keychain)
/// 2. Fallback 使用 AES-256-GCM 加密存储到 SQLite
pub struct KeyStorage {
    db: Arc<Database>,
}

impl KeyStorage {
    /// 创建 KeyStorage 实例
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// 存储 API Key
    ///
    /// 策略：
    /// 1. 尝试使用 keyring 存储
    /// 2. 如果失败，使用 AES-GCM 加密存储到数据库
    pub fn store(&self, provider: &str, api_key: SecretString) -> Result<()> {
        // 1. 尝试 keyring
        let service = format!("ceo-claw-{}", provider);
        let entry = Entry::new(&service, "api_key");

        match entry {
            Ok(e) => {
                if e.set_password(api_key.expose_secret()).is_ok() {
                    tracing::info!("API key for {} stored in keyring", provider);
                    return Ok(());
                }
            }
            Err(e) => {
                tracing::debug!("Keyring not available: {}", e);
            }
        }

        // 2. Fallback: AES-GCM 加密存储
        tracing::info!("Using encrypted storage for {} API key", provider);
        encryption::encrypt_and_store(&self.db, provider, api_key)
    }

    /// 获取 API Key
    ///
    /// 策略：
    /// 1. 优先从 keyring 获取
    /// 2. 如果没有，从加密存储获取并解密
    pub fn get(&self, provider: &str) -> Result<Option<SecretString>> {
        // 1. 尝试从 keyring 获取
        let service = format!("ceo-claw-{}", provider);
        if let Ok(entry) = Entry::new(&service, "api_key") {
            if let Ok(password) = entry.get_password() {
                tracing::debug!("API key for {} retrieved from keyring", provider);
                return Ok(Some(SecretString::from(password)));
            }
        }

        // 2. Fallback: 从加密存储获取并解密
        tracing::debug!("Trying encrypted storage for {} API key", provider);
        encryption::retrieve_and_decrypt(&self.db, provider)
    }

    /// 删除 API Key
    pub fn delete(&self, provider: &str) -> Result<()> {
        let service = format!("ceo-claw-{}", provider);

        // 1. 尝试从 keyring 删除
        if let Ok(entry) = Entry::new(&service, "api_key") {
            let _ = entry.delete_credential();
        }

        // 2. 从加密存储删除
        encryption::delete_encrypted(&self.db, provider)?;

        tracing::info!("API key for {} deleted", provider);
        Ok(())
    }

    /// 检查 keyring 是否可用
    pub fn is_keyring_available() -> bool {
        let service = "ceo-claw-test";
        Entry::new(service, "test").is_ok()
    }
}
