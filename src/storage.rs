//! In-memory storage with expiration support for ephemeral files

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::encryption::EncryptedFile;

/// Storage manager for ephemeral files
pub struct Storage {
    files: Arc<RwLock<HashMap<String, EncryptedFile>>>,
    cleanup_interval_secs: u64,
}

impl Storage {
    /// Create a new storage instance
    pub async fn new() -> Result<Self, anyhow::Error> {
        let storage = Self {
            files: Arc::new(RwLock::new(HashMap::new())),
            cleanup_interval_secs: 60, // Check for expired files every minute
        };
        
        // Start background cleanup task
        let files_clone = storage.files.clone();
        tokio::spawn(async move {
            Self::cleanup_loop(files_clone, storage.cleanup_interval_secs).await;
        });
        
        info!("Storage initialized");
        Ok(storage)
    }

    /// Store an encrypted file
    pub async fn store(&self, file: EncryptedFile) -> Result<(), anyhow::Error> {
        let mut files = self.files.write().await;
        debug!("Storing file with ID: {}", file.id);
        files.insert(file.id.clone(), file);
        Ok(())
    }

    /// Retrieve an encrypted file by ID
    pub async fn get(&self, id: &str) -> Option<EncryptedFile> {
        let files = self.files.read().await;
        
        if let Some(file) = files.get(id) {
            // Check if expired
            if let Some(expires_at) = file.expires_at {
                if Utc::now() > expires_at {
                    warn!("File {} has expired", id);
                    return None;
                }
            }
            Some(file.clone())
        } else {
            None
        }
    }

    /// Delete a file by ID
    pub async fn delete(&self, id: &str) -> bool {
        let mut files = self.files.write().await;
        debug!("Deleting file with ID: {}", id);
        files.remove(id).is_some()
    }

    /// Check if a file exists and is not expired
    pub async fn exists(&self, id: &str) -> bool {
        self.get(id).await.is_some()
    }

    /// Get the number of stored files
    pub async fn count(&self) -> usize {
        let files = self.files.read().await;
        files.len()
    }

    /// Background cleanup loop for expired files
    async fn cleanup_loop(files: Arc<RwLock<HashMap<String, EncryptedFile>>>, interval_secs: u64) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval_secs));
        
        loop {
            interval.tick().await;
            
            let now = Utc::now();
            let mut to_remove = Vec::new();
            
            {
                let files_guard = files.read().await;
                for (id, file) in files_guard.iter() {
                    if let Some(expires_at) = file.expires_at {
                        if now > expires_at {
                            to_remove.push(id.clone());
                        }
                    }
                }
            }
            
            if !to_remove.is_empty() {
                let mut files_guard = files.write().await;
                for id in to_remove {
                    debug!("Cleaning up expired file: {}", id);
                    files_guard.remove(&id);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encryption::{generate_key, encrypt};

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let storage = Storage::new().await.unwrap();
        
        let key = generate_key().unwrap();
        let (encrypted_data, nonce) = encrypt(b"test data", &key).unwrap();
        
        let file = EncryptedFile {
            id: "test-123".to_string(),
            filename: "test.txt".to_string(),
            size: 8,
            encrypted_data,
            nonce,
            created_at: Utc::now(),
            expires_at: None,
        };
        
        storage.store(file).await.unwrap();
        
        let retrieved = storage.get("test-123").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().filename, "test.txt");
    }

    #[tokio::test]
    async fn test_expired_file() {
        let storage = Storage::new().await.unwrap();
        
        let key = generate_key().unwrap();
        let (encrypted_data, nonce) = encrypt(b"test data", &key).unwrap();
        
        let file = EncryptedFile {
            id: "expired-456".to_string(),
            filename: "expired.txt".to_string(),
            size: 8,
            encrypted_data,
            nonce,
            created_at: Utc::now(),
            expires_at: Some(Utc::now() - chrono::Duration::minutes(1)), // Already expired
        };
        
        storage.store(file).await.unwrap();
        
        let retrieved = storage.get("expired-456").await;
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_delete() {
        let storage = Storage::new().await.unwrap();
        
        let key = generate_key().unwrap();
        let (encrypted_data, nonce) = encrypt(b"test data", &key).unwrap();
        
        let file = EncryptedFile {
            id: "delete-789".to_string(),
            filename: "delete.txt".to_string(),
            size: 8,
            encrypted_data,
            nonce,
            created_at: Utc::now(),
            expires_at: None,
        };
        
        storage.store(file).await.unwrap();
        assert!(storage.exists("delete-789").await);
        
        let deleted = storage.delete("delete-789").await;
        assert!(deleted);
        assert!(!storage.exists("delete-789").await);
    }
}
