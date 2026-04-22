//! AES-256-GCM 加密模块
//! 用于在 Keyring 不可用时安全存储 API Key

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use secrecy::{ExposeSecret, Secret, SecretString};
use std::sync::Arc;

use crate::db::Database;
use crate::utils::error::{AppError, Result};

/// 从机器指纹和盐派生加密 key
fn derive_key(machine_id: &str, salt: &[u8]) -> [u8; 32] {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // 简单的 key 派生（生产环境应使用 PBKDF2 或 Argon2）
    let mut hasher = DefaultHasher::new();
    machine_id.hash(&mut hasher);
    salt.hash(&mut hasher);
    let hash1 = hasher.finish();

    let mut hasher = DefaultHasher::new();
    hash1.hash(&mut hasher);
    salt.hash(&mut hasher);
    let hash2 = hasher.finish();

    let mut hasher = DefaultHasher::new();
    hash2.hash(&mut hasher);
    machine_id.hash(&mut hasher);
    let hash3 = hasher.finish();

    let mut hasher = DefaultHasher::new();
    hash3.hash(&mut hasher);
    hash1.hash(&mut hasher);
    let hash4 = hasher.finish();

    // 组合四个 hash 值生成 32 字节 key
    let mut key = [0u8; 32];
    key[..8].copy_from_slice(&hash1.to_le_bytes());
    key[8..16].copy_from_slice(&hash2.to_le_bytes());
    key[16..24].copy_from_slice(&hash3.to_le_bytes());
    key[24..32].copy_from_slice(&hash4.to_le_bytes());
    key
}

/// 获取机器唯一标识
fn get_machine_id() -> String {
    // 使用机器名、用户名、目录组合作为机器标识
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let username = std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "user".to_string());

    format!("ceo-claw-{}-{}", hostname, username)
}

/// 使用 AES-256-GCM 加密数据
fn encrypt(plaintext: &str, key: &[u8; 32]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| AppError::Encryption(format!("Failed to create cipher: {}", e)))?;

    // 使用随机 nonce
    let mut nonce_bytes = [0u8; 12];
    getrandom::getrandom(&mut nonce_bytes)
        .map_err(|e| AppError::Encryption(format!("Failed to generate nonce: {}", e)))?;

    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| AppError::Encryption(format!("Encryption failed: {}", e)))?;

    // 拼接 nonce + ciphertext
    let mut result = nonce_bytes.to_vec();
    result.extend(ciphertext);
    Ok(result)
}

/// 使用 AES-256-GCM 解密数据
fn decrypt(ciphertext: &[u8], key: &[u8; 32]) -> Result<String> {
    if ciphertext.len() < 12 {
        return Err(AppError::Encryption("Invalid ciphertext".to_string()));
    }

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| AppError::Encryption(format!("Failed to create cipher: {}", e)))?;

    let nonce = Nonce::from_slice(&ciphertext[..12]);
    let plaintext = cipher
        .decrypt(nonce, &ciphertext[12..])
        .map_err(|e| AppError::Encryption(format!("Decryption failed: {}", e)))?;

    String::from_utf8(plaintext)
        .map_err(|e| AppError::Encryption(format!("Invalid UTF-8: {}", e)))
}

/// 加密并存储 API Key 到数据库
pub fn encrypt_and_store(db: &Arc<Database>, provider: &str, api_key: SecretString) -> Result<()> {
    let machine_id = get_machine_id();
    let salt = b"ceo-claw-api-key-v1"; // 固定盐，与 keyring 区分
    let key = derive_key(&machine_id, salt);

    let encrypted = encrypt(api_key.expose_secret(), &key)?;
    let encrypted_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &encrypted);

    db.set_secret(provider, SecretString::new(encrypted_b64))?;
    tracing::debug!("API key for {} encrypted and stored", provider);
    Ok(())
}

/// 从数据库获取并解密 API Key
pub fn retrieve_and_decrypt(db: &Arc<Database>, provider: &str) -> Result<Option<SecretString>> {
    let encrypted_b64 = match db.get_secret(provider)? {
        Some(s) => s,
        None => return Ok(None),
    };

    let machine_id = get_machine_id();
    let salt = b"ceo-claw-api-key-v1";
    let key = derive_key(&machine_id, salt);

    let encrypted = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encrypted_b64.expose_secret())
        .map_err(|e| AppError::Encryption(format!("Base64 decode failed: {}", e)))?;

    let decrypted = decrypt(&encrypted, &key)?;
    Ok(Some(SecretString::new(decrypted)))
}

/// 从加密存储删除 API Key
pub fn delete_encrypted(db: &Arc<Database>, provider: &str) -> Result<()> {
    db.delete_config(provider)?;
    tracing::debug!("Encrypted API key for {} deleted", provider);
    Ok(())
}
