// src/infrastructure/repository/document_repository.rs

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::domain::{Document, DocumentId, Term};
use super::{RepositoryError, RepositoryResult};

/// Repository interface for Document entities
pub trait DocumentRepository: Send + Sync {
    /// Find a document by ID
    fn find(&self, id: &DocumentId) -> RepositoryResult<Option<Document>>;
    
    /// Check if a document exists
    fn exists(&self, id: &DocumentId) -> RepositoryResult<bool>;
    
    /// Save a document
    fn save(&self, document: &Document) -> RepositoryResult<()>;
    
    /// Delete a document
    fn delete(&self, id: &DocumentId) -> RepositoryResult<()>;
    
    /// Find all documents
    fn find_all(&self) -> RepositoryResult<Vec<Document>>;
    
    /// Count all documents
    fn count(&self) -> RepositoryResult<usize>;
    
    /// Find documents containing a specific term
    fn find_by_term(&self, term: &Term) -> RepositoryResult<Vec<Document>>;
}

/// In-memory implementation of DocumentRepository
pub struct InMemoryDocumentRepository {
    documents: Arc<RwLock<HashMap<String, Document>>>,
}

impl InMemoryDocumentRepository {
    /// Create a new InMemoryDocumentRepository
    pub fn new() -> Self {
        Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryDocumentRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentRepository for InMemoryDocumentRepository {
    fn find(&self, id: &DocumentId) -> RepositoryResult<Option<Document>> {
        let documents = self.documents.read().map_err(
            |e|  RepositoryError::Other(format!("Lock error {}", e))
        )?;

        Ok(documents.get(id.value()).cloned())
    }

    fn exists(&self, id: &DocumentId) -> RepositoryResult<bool> {
        let documents = self.documents.read().map_err(
            |e|  RepositoryError::Other(format!("Lock error {}", e))
        )?;
        
        Ok(documents.contains_key(id.value()))
    }

    fn save(&self, document: &Document) -> RepositoryResult<()> {
        let mut documents = self.documents.write().map_err(
            |e|  RepositoryError::Other(format!("Lock error {}", e))
        )?;
        
        documents.insert(document.id().value().to_string(), document.clone());
        Ok(())
    }

    fn delete(&self, id: &DocumentId) -> RepositoryResult<()> {
         let mut documents = self.documents.write().map_err(
            |e|  RepositoryError::Other(format!("Lock error {}", e))
        )?;

        documents.remove(id.value());

        Ok(())
    }

    fn find_all(&self) -> RepositoryResult<Vec<Document>> {
        let documents = self.documents.read().map_err(
            |e|  RepositoryError::Other(format!("Lock error {}", e))
        )?;

        Ok(documents.values().cloned().collect())
    }

    fn count(&self) -> RepositoryResult<usize> {
        let documents = self.documents.read().map_err(
            |e|  RepositoryError::Other(format!("Lock error {}", e))
        )?;

        Ok(documents.len())

    }

    fn find_by_term(&self, term: &Term) -> RepositoryResult<Vec<Document>> {
        let documents = self.documents.read().map_err(
            |e|  RepositoryError::Other(format!("Lock error {}", e))
        )?;

        let doc_vec = documents.values()
            .filter(|doc| doc.term_frequencies().contains_key(term))
            .cloned()
            .collect();

        Ok(doc_vec)

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_save_and_find_document() {
        let repo = InMemoryDocumentRepository::new();
        let doc = Document::new("doc1", "Test document");
        
        // Save document
        repo.save(&doc).unwrap();
        
        // Find document
        let found = repo.find(&DocumentId::new("doc1")).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id().value(), "doc1");
    }
    
    #[test]
    fn test_exists() {
        let repo = InMemoryDocumentRepository::new();
        let doc = Document::new("doc1", "Test document");
        
        // Save document
        repo.save(&doc).unwrap();
        
        // Check existence
        assert!(repo.exists(&DocumentId::new("doc1")).unwrap());
        assert!(!repo.exists(&DocumentId::new("doc2")).unwrap());
    }
    
    #[test]
    fn test_delete() {
        let repo = InMemoryDocumentRepository::new();
        let doc = Document::new("doc1", "Test document");
        
        // Save document
        repo.save(&doc).unwrap();
        
        // Delete document
        repo.delete(&DocumentId::new("doc1")).unwrap();
        
        // Check it's gone
        let found = repo.find(&DocumentId::new("doc1")).unwrap();
        assert!(found.is_none());
    }
    
    #[test]
    fn test_find_all_and_count() {
        let repo = InMemoryDocumentRepository::new();
        
        // Save documents
        repo.save(&Document::new("doc1", "First document")).unwrap();
        repo.save(&Document::new("doc2", "Second document")).unwrap();
        
        // Find all
        let all = repo.find_all().unwrap();
        assert_eq!(all.len(), 2);
        
        // Count
        assert_eq!(repo.count().unwrap(), 2);
    }
    
    #[test]
    fn test_find_by_term() {
        let repo = InMemoryDocumentRepository::new();
        
        // Create documents with terms
        let mut doc1 = Document::new("doc1", "Document about apples");
        doc1.add_term(Term::new("apples"));
        
        let mut doc2 = Document::new("doc2", "Document about oranges");
        doc2.add_term(Term::new("oranges"));
        
        // Save documents
        repo.save(&doc1).unwrap();
        repo.save(&doc2).unwrap();
        
        // Find by term
        let apple_docs = repo.find_by_term(&Term::new("apples")).unwrap();
        assert_eq!(apple_docs.len(), 1);
        assert_eq!(apple_docs[0].id().value(), "doc1");
        
        let orange_docs = repo.find_by_term(&Term::new("oranges")).unwrap();
        assert_eq!(orange_docs.len(), 1);
        assert_eq!(orange_docs[0].id().value(), "doc2");
        
        // Non-existent term
        let banana_docs = repo.find_by_term(&Term::new("bananas")).unwrap();
        assert_eq!(banana_docs.len(), 0);
    }
}