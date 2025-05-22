// src/infrastructure/persistence/mod.rs

//! Persistence implementations for storing TF-IDF data.

mod in_memory;

//pub use in_memory::InMemoryStorage;

use crate::infrastructure::InfrastructureResult;

/// Generic persistence interface
pub trait Storage: Send + Sync {
    /// Save data under a key
    fn save(&self, key: &str, data: &[u8]) -> InfrastructureResult<()>;
    
    /// Load data by key
    fn load(&self, key: &str) -> InfrastructureResult<Option<Vec<u8>>>;
    
    /// Check if data exists for a key
    fn exists(&self, key: &str) -> InfrastructureResult<bool>;
    
    /// Delete data by key
    fn delete(&self, key: &str) -> InfrastructureResult<()>;
    
    /// List all keys
    fn list_keys(&self) -> InfrastructureResult<Vec<String>>;
}