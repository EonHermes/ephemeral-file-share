//! End-to-end encryption module using ChaCha20-Poly1305

use chacha20poly1305::{aead::Aead, ChaCha20Poly1305, KeyInit, Nonce};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Re-export Key type for convenience
pub use chacha20poly1305::Key;

#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    
    #[error("Invalid key length")]
    InvalidKeyLength,
}

/// Encrypted file metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedFile {
    pub id: String,
    pub filename: String,
    pub size: u64,
    pub encrypted_data: Vec<u8>,
    pub nonce: Vec<u8>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Generate a new encryption key
pub fn generate_key() -> Result<Key, EncryptionError> {
    let mut key_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut key_bytes);
    Ok(Key::from(key_bytes))
}

/// Encrypt data using ChaCha20-Poly1305
pub fn encrypt(data: &[u8], key: &Key) -> Result<(Vec<u8>, Vec<u8>), EncryptionError> {
    let cipher = ChaCha20Poly1305::new(key);
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from(nonce_bytes);
    
    let encrypted = cipher.encrypt(&nonce, data)
        .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;
    
    Ok((encrypted, nonce.to_vec()))
}

/// Decrypt data using ChaCha20-Poly1305
pub fn decrypt(encrypted_data: &[u8], nonce: &[u8], key: &Key) -> Result<Vec<u8>, EncryptionError> {
    let cipher = ChaCha20Poly1305::new(key);
    let nonce = Nonce::from_slice(nonce);
    
    let decrypted = cipher.decrypt(nonce, encrypted_data)
        .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;
    
    Ok(decrypted)
}

/// Generate a secure transfer token
pub fn generate_token() -> String {
    use base64::Engine;
    let mut bytes = vec![0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = generate_key().unwrap();
        let original_data = b"Hello, World! This is a secret message.";
        
        let (encrypted, nonce) = encrypt(original_data, &key).unwrap();
        let decrypted = decrypt(&encrypted, &nonce, &key).unwrap();
        
        assert_eq!(original_data.as_slice(), &decrypted);
    }

    #[test]
    fn test_generate_token() {
        let token1 = generate_token();
        let token2 = generate_token();
        
        assert_ne!(token1, token2);
        assert_eq!(token1.len(), 43); // URL_SAFE_NO_PAD encoding of 32 bytes
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = generate_key().unwrap();
        let key2 = generate_key().unwrap();
        let data = b"Secret data";
        
        let (encrypted, nonce) = encrypt(data, &key1).unwrap();
        let result = decrypt(&encrypted, &nonce, &key2);
        
        assert!(result.is_err());
    }
}
