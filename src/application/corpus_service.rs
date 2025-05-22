// src/application/corpus_service.rs

use std::sync::Arc;

use crate::domain::{Corpus, CorpusId, Document, DocumentId};
use crate::infrastructure::repository::{CorpusRepository, DocumentRepository};

use super::{ApplicationError, ApplicationResult, DocumentService};

/// Service interface for managing Corpora
pub trait CorpusService {
    /// Create a new corpus
    fn create_corpus(&self, id: &str, name: &str) -> ApplicationResult<Corpus>;
    
    /// Create a corpus with description
    fn create_corpus_with_description(
        &self, 
        id: &str, 
        name: &str, 
        description: &str
    ) -> ApplicationResult<Corpus>;
    
    /// Get a corpus by ID
    fn get_corpus(&self, id: &str) -> ApplicationResult<Corpus>;
    
    /// Update a corpus's name
    fn update_name(&self, id: &str, new_name: &str) -> ApplicationResult<Corpus>;
    
    /// Update a corpus's description
    fn update_description(&self, id: &str, new_description: &str) -> ApplicationResult<Corpus>;
    
    /// Delete a corpus
    fn delete_corpus(&self, id: &str) -> ApplicationResult<()>;
    
    /// Add a document to a corpus
    fn add_document(&self, corpus_id: &str, document_id: &str) -> ApplicationResult<Corpus>;
    
    /// Remove a document from a corpus
    fn remove_document(&self, corpus_id: &str, document_id: &str) -> ApplicationResult<Corpus>;
    
    /// Add a stopword to a corpus
    fn add_stopword(&self, corpus_id: &str, word: &str) -> ApplicationResult<Corpus>;
    
    /// Remove a stopword from a corpus
    fn remove_stopword(&self, corpus_id: &str, word: &str) -> ApplicationResult<Corpus>;
    
    /// Build the document frequency index for a corpus
    fn build_index(&self, corpus_id: &str) -> ApplicationResult<Corpus>;
    
    /// List all corpora
    fn list_corpora(&self) -> ApplicationResult<Vec<Corpus>>;
    
    /// Count all corpora
    fn count_corpora(&self) -> ApplicationResult<usize>;
    
    /// Get documents in a corpus
    fn get_corpus_documents(&self, corpus_id: &str) -> ApplicationResult<Vec<Document>>;
    
    /// Count documents in a corpus
    fn count_corpus_documents(&self, corpus_id: &str) -> ApplicationResult<usize>;
}

/// Implementation of the CorpusService
pub struct CorpusServiceImpl<CR, DR, DS>
where
    CR: CorpusRepository,
    DR: DocumentRepository,
    DS: DocumentService,
{
    corpus_repository: Arc<CR>,
    document_repository: Arc<DR>,
    document_service: Arc<DS>,
}

impl<CR, DR, DS> CorpusServiceImpl<CR, DR, DS>
where
    CR: CorpusRepository,
    DR: DocumentRepository,
    DS: DocumentService,
{
    /// Create a new CorpusServiceImpl
    pub fn new(
        corpus_repository: Arc<CR>,
        document_repository: Arc<DR>,
        document_service: Arc<DS>,
    ) -> Self {
        Self {
            corpus_repository,
            document_repository,
            document_service,
        }
    }
}

impl<CR, DR, DS> CorpusService for CorpusServiceImpl<CR, DR, DS>
where
    CR: CorpusRepository,
    DR: DocumentRepository,
    DS: DocumentService,
{
    fn create_corpus(&self, id: &str, name: &str) -> ApplicationResult<Corpus> {
        // Check if corpus already exists
        if self.corpus_repository.exists(&CorpusId::new(id)).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error checking existence: {}", e))
        })? {
            return Err(ApplicationError::InvalidInput(
                format!("Corpus with ID '{}' already exists", id)
            ));
        }
        
        // Create new corpus
        let corpus = Corpus::new(id, name);
        
        // Save corpus
        self.corpus_repository.save(&corpus).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error saving corpus: {}", e))
        })?;
        
        Ok(corpus)
    }
    
    fn create_corpus_with_description(
        &self, 
        id: &str, 
        name: &str, 
        description: &str
    ) -> ApplicationResult<Corpus> {
        // Check if corpus already exists
        if self.corpus_repository.exists(&CorpusId::new(id)).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error checking existence: {}", e))
        })? {
            return Err(ApplicationError::InvalidInput(
                format!("Corpus with ID '{}' already exists", id)
            ));
        }
        
        // Create new corpus with description
        let corpus = Corpus::with_description(id, name, description);
        
        // Save corpus
        self.corpus_repository.save(&corpus).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error saving corpus: {}", e))
        })?;
        
        Ok(corpus)
    }
    
    fn get_corpus(&self, id: &str) -> ApplicationResult<Corpus> {
        let corpus_id = CorpusId::new(id);
        
        self.corpus_repository.find(&corpus_id).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error retrieving corpus: {}", e))
        })?.ok_or_else(|| {
            ApplicationError::NotFound(format!("Corpus with ID '{}' not found", id))
        })
    }
    
    fn update_name(&self, id: &str, new_name: &str) -> ApplicationResult<Corpus> {
        let corpus_id = CorpusId::new(id);
        
        // Get existing corpus
        let mut corpus = self.corpus_repository.find(&corpus_id).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error retrieving corpus: {}", e))
        })?.ok_or_else(|| {
            ApplicationError::NotFound(format!("Corpus with ID '{}' not found", id))
        })?;
        
        // Update name
        corpus.set_name(new_name);
        
        // Save updated corpus
        self.corpus_repository.save(&corpus).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error saving corpus: {}", e))
        })?;
        
        Ok(corpus)
    }
    
    fn update_description(&self, id: &str, new_description: &str) -> ApplicationResult<Corpus> {
        let corpus_id = CorpusId::new(id);
        
        // Get existing corpus
        let mut corpus = self.corpus_repository.find(&corpus_id).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error retrieving corpus: {}", e))
        })?.ok_or_else(|| {
            ApplicationError::NotFound(format!("Corpus with ID '{}' not found", id))
        })?;
        
        // Update description
        corpus.set_description(new_description);
        
        // Save updated corpus
        self.corpus_repository.save(&corpus).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error saving corpus: {}", e))
        })?;
        
        Ok(corpus)
    }
    
    fn delete_corpus(&self, id: &str) -> ApplicationResult<()> {
        let corpus_id = CorpusId::new(id);
        
        // Check if corpus exists
        if !self.corpus_repository.exists(&corpus_id).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error checking existence: {}", e))
        })? {
            return Err(ApplicationError::NotFound(
                format!("Corpus with ID '{}' not found", id)
            ));
        }
        
        // Delete corpus
        self.corpus_repository.delete(&corpus_id).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error deleting corpus: {}", e))
        })?;
        
        Ok(())
    }
    
    fn add_document(&self, corpus_id: &str, document_id: &str) -> ApplicationResult<Corpus> {
        let corpus_id = CorpusId::new(corpus_id);
        let document_id_obj = DocumentId::new(document_id);
        
        // Get existing corpus
        let mut corpus = self.corpus_repository.find(&corpus_id).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error retrieving corpus: {}", e))
        })?.ok_or_else(|| {
            ApplicationError::NotFound(format!("Corpus with ID '{}' not found", corpus_id.value()))
        })?;
        
        // Check if document exists
        let document = self.document_repository.find(&document_id_obj).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error retrieving document: {}", e))
        })?.ok_or_else(|| {
            ApplicationError::NotFound(format!("Document with ID '{}' not found", document_id))
        })?;
        
        // Check if document is already in corpus
        if corpus.contains_document(&document_id_obj) {
            return Err(ApplicationError::InvalidInput(
                format!("Document '{}' is already in corpus '{}'", document_id, corpus_id.value())
            ));
        }
        
        // Add document to corpus
        corpus.add_document(document).map_err(|e| {
            ApplicationError::DomainError(e)
        })?;
        
        // Rebuild index if corpus was already indexed
        if corpus.is_indexed() {
            corpus.build_index();
        }
        
        // Save updated corpus
        self.corpus_repository.save(&corpus).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error saving corpus: {}", e))
        })?;
        
        Ok(corpus)
    }
    
    fn remove_document(&self, corpus_id: &str, document_id: &str) -> ApplicationResult<Corpus> {
        let corpus_id = CorpusId::new(corpus_id);
        let document_id_obj = DocumentId::new(document_id);
        
        // Get existing corpus
        let mut corpus = self.corpus_repository.find(&corpus_id).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error retrieving corpus: {}", e))
        })?.ok_or_else(|| {
            ApplicationError::NotFound(format!("Corpus with ID '{}' not found", corpus_id.value()))
        })?;
        
        // Remove document from corpus
        corpus.remove_document(&document_id_obj).map_err(|e| {
            ApplicationError::DomainError(e)
        })?;
        
        // Save updated corpus
        self.corpus_repository.save(&corpus).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error saving corpus: {}", e))
        })?;
        
        Ok(corpus)
    }
    
    fn add_stopword(&self, corpus_id: &str, word: &str) -> ApplicationResult<Corpus> {
        let corpus_id = CorpusId::new(corpus_id);
        
        // Get existing corpus
        let mut corpus = self.corpus_repository.find(&corpus_id).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error retrieving corpus: {}", e))
        })?.ok_or_else(|| {
            ApplicationError::NotFound(format!("Corpus with ID '{}' not found", corpus_id.value()))
        })?;
        
        // Add stopword
        corpus.add_stopword(word.to_lowercase());
        
        // Save updated corpus
        self.corpus_repository.save(&corpus).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error saving corpus: {}", e))
        })?;
        
        Ok(corpus)
    }
    
    fn remove_stopword(&self, corpus_id: &str, word: &str) -> ApplicationResult<Corpus> {
        let corpus_id = CorpusId::new(corpus_id);
        
        // Get existing corpus
        let mut corpus = self.corpus_repository.find(&corpus_id).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error retrieving corpus: {}", e))
        })?.ok_or_else(|| {
            ApplicationError::NotFound(format!("Corpus with ID '{}' not found", corpus_id.value()))
        })?;
        
        // Remove stopword
        corpus.remove_stopword(&word.to_lowercase());
        
        // Save updated corpus
        self.corpus_repository.save(&corpus).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error saving corpus: {}", e))
        })?;
        
        Ok(corpus)
    }
    
    fn build_index(&self, corpus_id: &str) -> ApplicationResult<Corpus> {
        let corpus_id = CorpusId::new(corpus_id);
        
        // Get existing corpus
        let mut corpus = self.corpus_repository.find(&corpus_id).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error retrieving corpus: {}", e))
        })?.ok_or_else(|| {
            ApplicationError::NotFound(format!("Corpus with ID '{}' not found", corpus_id.value()))
        })?;
        
        // Build index
        corpus.build_index();
        
        // Save updated corpus
        self.corpus_repository.save(&corpus).map_err(|e| {
            ApplicationError::RepositoryError(format!("Error saving corpus: {}", e))
        })?;
        
        Ok(corpus)
    }
    
    fn list_corpora(&self) -> ApplicationResult<Vec<Corpus>> {
        self.corpus_repository.find_all().map_err(|e| {
            ApplicationError::RepositoryError(format!("Error listing corpora: {}", e))
        })
    }
    
    fn count_corpora(&self) -> ApplicationResult<usize> {
        self.corpus_repository.count().map_err(|e| {
            ApplicationError::RepositoryError(format!("Error counting corpora: {}", e))
        })
    }
    
    fn get_corpus_documents(&self, corpus_id: &str) -> ApplicationResult<Vec<Document>> {
        let corpus = self.get_corpus(corpus_id)?;
        
        let document_ids: Vec<_> = corpus.document_ids()
            .map(|id| id.value().to_string())
            .collect();
            
        let mut documents = Vec::new();
        
        for id in document_ids {
            let document = self.document_service.get_document(&id)?;
            documents.push(document);
        }
        
        Ok(documents)
    }
    
    fn count_corpus_documents(&self, corpus_id: &str) -> ApplicationResult<usize> {
        let corpus = self.get_corpus(corpus_id)?;
        Ok(corpus.document_count())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::repository::{InMemoryCorpusRepository, InMemoryDocumentRepository};
    use crate::infrastructure::tokenizer::SimpleTokenizer;
    use crate::application::document_service::DocumentServiceImpl;
    
    fn create_service() -> (impl DocumentService, impl CorpusService) {
    // Dependencies for DocumentService
    let doc_repo_for_ds = Arc::new(InMemoryDocumentRepository::new());
    let tokenizer_for_ds = Arc::new(SimpleTokenizer::new());

    // Create an owned DocumentServiceImpl to be returned by this function
    let returned_doc_service = DocumentServiceImpl::new(doc_repo_for_ds.clone(), tokenizer_for_ds.clone());

    // Create an Arc<DocumentServiceImpl> for CorpusServiceImpl's dependency
    // This can share the same underlying repo/tokenizer if they are Arc'd, or use new ones if needed.
    // For simplicity here, let's assume it can share the same Arc'd repo and tokenizer.
    let arc_doc_service_for_corpus_dep = Arc::new(DocumentServiceImpl::new(doc_repo_for_ds.clone(), tokenizer_for_ds.clone()));
    // Or if you want it to be the *exact same instance* as returned_doc_service was based on,
    // and if DocumentServiceImpl was Clone (which it typically isn't):
    // let arc_doc_service_for_corpus_dep = Arc::new(returned_doc_service.clone());
    // More commonly, you'd just create the Arc first, then give a clone of the Arc to CorpusService,
    // and figure out how to return an owned version or if Arc can be made to work.
    // Given the compiler error, returning an owned one is the direct fix:

    // Dependencies for CorpusService
    let corpus_repo = Arc::new(InMemoryCorpusRepository::new());
    // CorpusServiceImpl needs a DocumentRepository and a DocumentService
    // Using doc_repo_for_ds for the DocumentRepository dependency of CorpusService

    let corpus_service = CorpusServiceImpl::new(
        corpus_repo,
        doc_repo_for_ds, // DocumentRepository dependency for CorpusService
        arc_doc_service_for_corpus_dep, // DocumentService dependency for CorpusService
    );

    (returned_doc_service, corpus_service)
}
    
    #[test]
    fn test_create_and_get_corpus() {
        let (_, corpus_service) = create_service();
        
        // Create a corpus
        let corpus = corpus_service.create_corpus("corpus1", "Test Corpus").unwrap();
        assert_eq!(corpus.id().value(), "corpus1");
        assert_eq!(corpus.name(), "Test Corpus");
        
        // Get the corpus
        let retrieved = corpus_service.get_corpus("corpus1").unwrap();
        assert_eq!(retrieved.id().value(), "corpus1");
    }
    
    #[test]
    fn test_add_and_remove_document() {
        let (doc_service, corpus_service) = create_service();
        
        // Create a document
        doc_service.create_document("doc1", "Test document").unwrap();
        
        // Create a corpus
        corpus_service.create_corpus("corpus1", "Test Corpus").unwrap();
        
        // Add document to corpus
        let corpus = corpus_service.add_document("corpus1", "doc1").unwrap();
        assert_eq!(corpus.document_count(), 1);
        
        // Remove document from corpus
        let corpus = corpus_service.remove_document("corpus1", "doc1").unwrap();
        assert_eq!(corpus.document_count(), 0);
    }
    
    #[test]
    fn test_build_index() {
        let (doc_service, corpus_service) = create_service();
        
        // Create documents
        doc_service.create_document("doc1", "This is document one").unwrap();
        doc_service.create_document("doc2", "This is document two").unwrap();
        
        // Create a corpus
        corpus_service.create_corpus("corpus1", "Test Corpus").unwrap();
        
        // Add documents to corpus
        corpus_service.add_document("corpus1", "doc1").unwrap();
        corpus_service.add_document("corpus1", "doc2").unwrap();
        
        // Build index
        let corpus = corpus_service.build_index("corpus1").unwrap();
        assert!(corpus.is_indexed());
        
        // Check document frequencies
        assert_eq!(corpus.document_frequency(&crate::domain::Term::new("this")), 2);
    }
    
    #[test]
    fn test_stopwords() {
        let (_, corpus_service) = create_service();
        
        // Create a corpus
        corpus_service.create_corpus("corpus1", "Test Corpus").unwrap();
        
        // Add stopwords
        let corpus = corpus_service.add_stopword("corpus1", "the").unwrap();
        assert!(corpus.is_stopword("the"));
        
        // Remove stopword
        let corpus = corpus_service.remove_stopword("corpus1", "the").unwrap();
        assert!(!corpus.is_stopword("the"));
    }
    
    #[test]
    fn test_get_corpus_documents() {
        let (doc_service, corpus_service) = create_service();
        
        // Create documents
        doc_service.create_document("doc1", "Document one").unwrap();
        doc_service.create_document("doc2", "Document two").unwrap();
        
        // Create a corpus
        corpus_service.create_corpus("corpus1", "Test Corpus").unwrap();
        
        // Add documents to corpus
        corpus_service.add_document("corpus1", "doc1").unwrap();
        corpus_service.add_document("corpus1", "doc2").unwrap();
        
        // Get corpus documents
        let documents = corpus_service.get_corpus_documents("corpus1").unwrap();
        assert_eq!(documents.len(), 2);
        
        // Check document count
        let count = corpus_service.count_corpus_documents("corpus1").unwrap();
        assert_eq!(count, 2);
    }
}