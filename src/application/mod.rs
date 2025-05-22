// src/application/mod.rs

//! Application layer module for TF-IDF functionality.
//!
//! This module contains service interfaces and implementations that coordinate
//! domain entities and provide core application functionality.

mod document_service;
mod corpus_service;
mod tf_idf_service;

pub use document_service::{DocumentService, DocumentServiceImpl};
pub use corpus_service::{CorpusService, CorpusServiceImpl};
//pub use tf_idf_service::{TfIdfService, TfIdfServiceImpl};

/// Common error type for application operations
#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error("Domain error: {0}")]
    DomainError(#[from] crate::domain::DomainError),
    
    #[error("Repository error: {0}")]
    RepositoryError(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Entity not found: {0}")]
    NotFound(String),
    
    #[error("Operation not permitted: {0}")]
    NotPermitted(String),
    
    #[error("Other application error: {0}")]
    Other(String),
}

/// Result type for application operations
pub type ApplicationResult<T> = Result<T, ApplicationError>;