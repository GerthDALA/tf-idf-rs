/// src/application/document_service.rs

use std::sync::Arc;

use crate::domain::{Document, DocumentId, Term};
use crate::infrastructure::repository::DocumentRepository;
use crate::infrastructure::tokenizer::Tokenizer;

use super::{ApplicationError, ApplicationResult};

/// Service interface for managing Documents
pub trait DocumentService {
    /// Create a new document
    fn create_document(&self, id: &str, content: &str) -> ApplicationResult<Document>;
    
    /// Create a document with title
    fn create_document_with_title(&self, id: &str, title: &str, content: &str) -> ApplicationResult<Document>;
    
    /// Get a document by ID
    fn get_document(&self, id: &str) -> ApplicationResult<Document>;
    
    /// Update a document's content
    fn update_content(&self, id: &str, new_content: &str) -> ApplicationResult<Document>;
    
    /// Update a document's title
    fn update_title(&self, id: &str, new_title: &str) -> ApplicationResult<Document>;
    
    /// Delete a document
    fn delete_document(&self, id: &str) -> ApplicationResult<()>;
    
    /// Process a document's content, tokenizing and analyzing it
    fn process_document(&self, id: &str) -> ApplicationResult<Document>;
    
    /// List all documents
    fn list_documents(&self) -> ApplicationResult<Vec<Document>>;
    
    /// Count all documents
    fn count_documents(&self) -> ApplicationResult<usize>;
    
    /// Search for documents by term
    fn search_by_term(&self, term: &str) -> ApplicationResult<Vec<Document>>;
}

pub struct DocumentServiceImpl<R, T>
where 
    R: DocumentRepository,
    T: Tokenizer
{
    repository: Arc<R>,
    tokenizer: Arc<T>
}

impl <R, T> DocumentServiceImpl<R, T> 
where
    R: DocumentRepository,
    T: Tokenizer
{
    pub fn new(repository: Arc<R>, tokenizer: Arc<T>) -> Self {
        Self {
            repository,
            tokenizer
        }
    }

    /// Tokenize and analyze document content
    fn analyze_content(&self, document: &mut Document) -> ApplicationResult<()> {
        document.clear_terms();

        let tokens = self.tokenizer.tokenize(document.content());

        for token in tokens {
            let term = Term::new(token);
            document.add_term(term);
        }

        Ok(())
    }
}

impl<R, T> DocumentService for DocumentServiceImpl<R, T> 
where
    R: DocumentRepository,
    T: Tokenizer,
{
    fn create_document(&self, id: &str, content: &str) -> ApplicationResult<Document> {
        if self.repository.exists(&DocumentId::new(id)).map_err(|e|{
            ApplicationError::RepositoryError(format!("Error checking existence: {}", e))
        })? {
            return Err(ApplicationError::InvalidInput(format!("Document wiht ID '{}' already existed", id)));
        }

        let mut document = Document::new(id, content);

        self.analyze_content(&mut document)?;

        self.repository.save(&document).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error saving document: {}", e))
        })?;

        Ok(document)
    }

    fn create_document_with_title(&self, id: &str, title: &str, content: &str) -> ApplicationResult<Document> {
        if self.repository.exists(&DocumentId::new(id)).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error checking existence: {}", e))
        })? {
            return Err(ApplicationError::InvalidInput(format!("Document wiht ID '{}' already existed", id)));
        }

        let mut document = Document::with_title(id, title, content);
        self.analyze_content(&mut document)?;

        self.repository.save(&document).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error saving document: {}", e))
        })?;

        Ok(document)
    }

    fn get_document(&self, id: &str) -> ApplicationResult<Document> {

        let doc_id = DocumentId::new(id);
    

        let document = self.repository.find(&doc_id).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error retriveing document: {}", e))
        })?.ok_or_else(|| ApplicationError::NotFound(format!("Document with ID '{}' not found", id)))?;

        Ok(document)
    }

    fn update_content(&self, id: &str, new_content: &str) -> ApplicationResult<Document> {
        let doc_id = DocumentId::new(id);

        let mut document = self.repository.find(&doc_id).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error retriveing document: {}", e))
        })?.ok_or_else(|| ApplicationError::NotFound(format!("Document with ID '{}' not found", id)))?;

        let new_content_bytes = new_content.as_bytes();
        let old_content_bytes = document.content().as_bytes();

        if new_content_bytes != old_content_bytes {
            let mut updated_doc = Document::new(id, new_content);

            if let Some(title) = document.title() {
                updated_doc.set_title(title);
            }

            for (k, v) in document.metadata().iter() {
                updated_doc.set_metadata(k, v);
            }

            self.analyze_content(&mut updated_doc)?;

            self.repository.save(&updated_doc).map_err(|e|{
                ApplicationError::RepositoryError(format!("Error saving doc: {}", e))
            })?;
            
            document = updated_doc;
        }

        Ok(document)

    }

    fn update_title(&self, id: &str, new_title: &str) -> ApplicationResult<Document> {
        let doc_id = DocumentId::new(id);

        let mut document = self.repository.find(&doc_id).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error retriveing document: {}", e))
        })?.ok_or_else(|| ApplicationError::NotFound(format!("Document with ID '{}' not found", id)))?;

        document.set_title(new_title);

        self.repository.save(&document).map_err(|e|{
                ApplicationError::RepositoryError(format!("Error saving doc: {}", e))
        })?;

        Ok(document)
    }

    fn delete_document(&self, id: &str) -> ApplicationResult<()> {

        let doc_id = DocumentId::new(id);

        // Check if document exists
        if !self.repository.exists(&doc_id).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error checking existence: {}", e))
        })? {
            return Err(ApplicationError::NotFound(
                format!("Document with ID '{}' not found", id)
            ));
        }

        self.repository.delete(&doc_id).map_err(|e|{
            ApplicationError::RepositoryError(format!("Error deleting document with ID: {}", e))
        })?;

        Ok(())
    }

    fn process_document(&self, id: &str) -> ApplicationResult<Document> {
         let document_id = DocumentId::new(id);
        
        // Get existing document
        let mut document = self.repository.find(&document_id).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error retrieving document: {}", e))
        })?.ok_or_else(|| {
            ApplicationError::NotFound(format!("Document with ID '{}' not found", id))
        })?;
        
        // Analyze content
        self.analyze_content(&mut document)?;
        
        // Save updated document
        self.repository.save(&document).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error saving document: {}", e))
        })?;
        
        Ok(document)
    }

    fn list_documents(&self) -> ApplicationResult<Vec<Document>> {

        let documents = self.repository.find_all().map_err(|e| {
            ApplicationError::RepositoryError(format!("Error listing  documents: {}", e))
        })?;

        Ok(documents)
    }

    fn count_documents(&self) -> ApplicationResult<usize> {
        let doc_count = self.repository.count().map_err(|e|{
            ApplicationError::RepositoryError(format!("Error counting documents {}", e))
        })?;

        Ok(doc_count)
    }

    fn search_by_term(&self, term: &str) -> ApplicationResult<Vec<Document>> {
        let term = Term::new(term.to_lowercase());

        self.repository.find_by_term(&term).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error searching documents: {}", e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::repository::InMemoryDocumentRepository;
    use crate::infrastructure::tokenizer::SimpleTokenizer;
    
    fn create_service() -> impl DocumentService {
        let repository = Arc::new(InMemoryDocumentRepository::new());
        let tokenizer = Arc::new(SimpleTokenizer::new());
        DocumentServiceImpl::new(repository, tokenizer)
    }
    
    #[test]
    fn test_create_and_get_document() {
        let service = create_service();
        
        // Create a document
        let doc = service.create_document("doc1", "This is a test document").unwrap();
        assert_eq!(doc.id().value(), "doc1");
        assert_eq!(doc.content(), "This is a test document");
        
        // Verify terms were processed
        assert!(doc.term_frequencies().len() > 0);
        
        // Get the document
        let retrieved = service.get_document("doc1").unwrap();
        assert_eq!(retrieved.id().value(), "doc1");
    }
    
    #[test]
    fn test_update_content() {
        let service = create_service();
        
        // Create a document
        service.create_document("doc1", "Initial content").unwrap();
        
        // Update content
        let updated = service.update_content("doc1", "Updated content").unwrap();
        assert_eq!(updated.content(), "Updated content");
        
        // Verify updated terms
        assert!(updated.term_frequencies().contains_key(&Term::new("updated")));
        assert!(!updated.term_frequencies().contains_key(&Term::new("initial")));
    }
    
    #[test]
    fn test_delete_document() {
        let service = create_service();
        
        // Create a document
        service.create_document("doc1", "Test document").unwrap();
        
        // Delete the document
        service.delete_document("doc1").unwrap();
        
        // Verify it's gone
        assert!(service.get_document("doc1").is_err());
    }
    
    #[test]
    fn test_search_by_term() {
        let service = create_service();
        
        // Create documents
        service.create_document("doc1", "This document mentions apples").unwrap();
        service.create_document("doc2", "This one talks about oranges").unwrap();
        service.create_document("doc3", "More about apples and fruits").unwrap();
        
        // Search for documents containing "apples"
        let results = service.search_by_term("apples").unwrap();
        assert_eq!(results.len(), 2);
        
        // Ensure correct documents were found
        let ids: Vec<_> = results.iter().map(|d| d.id().value()).collect();
        assert!(ids.contains(&"doc1"));
        assert!(ids.contains(&"doc3"));
        assert!(!ids.contains(&"doc2"));
    }
}