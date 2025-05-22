
// src/infrastructure/persistence/in_memory.rs

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::infrastructure::{InfrastructureError, InfrastructureResult};

use super::Storage;

pub struct InMemoryStorage {
    data: Arc<RwLock<HashMap<String, Vec<u8>>>>
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new()))
        }
    }
}
impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl Storage for InMemoryStorage {
    fn save(&self, key: &str, data: &[u8]) -> InfrastructureResult<()> {
        let mut storage = self.data.write().map_err(|e| {
            InfrastructureError::PersistenceError(format!("Lock error: {}", e))
        })?;
        
        storage.insert(key.to_string(), data.to_vec());
        Ok(())
    }
    
    fn load(&self, key: &str) -> InfrastructureResult<Option<Vec<u8>>> {
        let storage = self.data.read().map_err(|e| {
            InfrastructureError::PersistenceError(format!("Lock error: {}", e))
        })?;
        
        Ok(storage.get(key).cloned())
    }
    
    fn exists(&self, key: &str) -> InfrastructureResult<bool> {
        let storage = self.data.read().map_err(|e| {
            InfrastructureError::PersistenceError(format!("Lock error: {}", e))
        })?;
        
        Ok(storage.contains_key(key))
    }
    
    fn delete(&self, key: &str) -> InfrastructureResult<()> {
        let mut storage = self.data.write().map_err(|e| {
            InfrastructureError::PersistenceError(format!("Lock error: {}", e))
        })?;
        
        storage.remove(key);
        Ok(())
    }
    
    fn list_keys(&self) -> InfrastructureResult<Vec<String>> {
        let storage = self.data.read().map_err(|e| {
            InfrastructureError::PersistenceError(format!("Lock error: {}", e))
        })?;
        
        let keys: Vec<String> = storage.keys().cloned().collect();
        Ok(keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_save_and_load() {
        let storage = InMemoryStorage::new();
        let data = b"test data".to_vec();
        
        // Save data
        storage.save("key1", &data).unwrap();
        
        // Load data
        let loaded = storage.load("key1").unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap(), data);
        
        // Non-existent key
        let nonexistent = storage.load("nonexistent").unwrap();
        assert!(nonexistent.is_none());
    }
    
    #[test]
    fn test_exists() {
        let storage = InMemoryStorage::new();
        
        // Save data
        storage.save("key1", b"test data").unwrap();
        
        // Check existence
        assert!(storage.exists("key1").unwrap());
        assert!(!storage.exists("key2").unwrap());
    }
    
    #[test]
    fn test_delete() {
        let storage = InMemoryStorage::new();
        
        // Save data
        storage.save("key1", b"test data").unwrap();
        
        // Delete data
        storage.delete("key1").unwrap();
        
        // Check it's gone
        assert!(!storage.exists("key1").unwrap());
    }
    
    #[test]
    fn test_list_keys() {
        let storage = InMemoryStorage::new();
        
        // Save data
        storage.save("key1", b"data1").unwrap();
        storage.save("key2", b"data2").unwrap();
        
        // List keys
        let keys = storage.list_keys().unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
    }
}