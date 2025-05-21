// src/domain/mod.rs

//! Domain module contains the core business entities and logic.
//! 
//! This module implements the essential domain concepts of the TF-IDF algorithm
//! following Domain-Driven Design principles.

mod document;
mod corpus;
mod term;
mod tf_idf;

pub use document::{Document, DocumentId};
pub use corpus::{Corpus, CorpusId};
pub use term::{Term, TermId, TermFrequency};
pub use tf_idf::{TfIdf, TfIdfScore, TfIdfError};

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
     #[error("Entity not found: {0}")]
    NotFound(String),
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    #[error("TF-IDF calculation error: {0}")]
    TfIdfError(#[from] TfIdfError),
    
    #[error("Other domain error: {0}")]
    Other(String),
}

pub type DomainResult<T> = Result<T, DomainError>;