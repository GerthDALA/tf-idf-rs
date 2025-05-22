// src/infrastructure/mod.rs

//! Infrastructure layer module for TF-IDF functionality.
//!
//! This module contains infrastructure implementations for repositories,
//! persistence, and other technical concerns.

pub mod repository;
pub mod persistence;
pub mod tokenizer;

/// Common error type for infrastructure operations
#[derive(Debug, thiserror::Error)]
pub enum InfrastructureError {
    #[error("Repository error: {0}")]
    RepositoryError(String),
    
    #[error("Persistence error: {0}")]
    PersistenceError(String),
    
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Tokenization error: {0}")]
    TokenizationError(String),
    
    #[error("Other infrastructure error: {0}")]
    Other(String),
}

/// Result type for infrastructure operations
pub type InfrastructureResult<T> = Result<T, InfrastructureError>;