// src/infrastructure/repository/mod.rs

//! Repository interfaces and implementations for storage and retrieval of entities.

mod document_repository;
mod corpus_repository;

pub use document_repository::{DocumentRepository, InMemoryDocumentRepository};
pub use corpus_repository::{CorpusRepository, InMemoryCorpusRepository};

/// Common error type for repository operations
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Entity not found: {0}")]
    NotFound(String),
    
    #[error("Persistence error: {0}")]
    PersistenceError(String),
    
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Other repository error: {0}")]
    Other(String),
}

/// Result type for repository operations
pub type RepositoryResult<T> = Result<T, RepositoryError>;