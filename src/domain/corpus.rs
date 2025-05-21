// src/domain/corpus.rs

use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

use super::{Document, DocumentId, Term, DomainError, DomainResult};

/// Unique identifier for a corpus
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CorpusId(pub String);

impl CorpusId {
    /// Create a new corpus ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    /// Get the string representation of the ID
    pub fn value(&self) -> &str {
        &self.0
    }
}

/// Corpus represents a collection of documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Corpus {
    /// Unique identifier for the corpus
    id: CorpusId,
    
    /// Name of the corpus
    name: String,
    
    /// Description of the corpus
    description: Option<String>,
    
    /// Collection of documents in this corpus
    documents: HashMap<DocumentId, Document>,
    
    /// Document frequency for each term (how many documents contain the term)
    document_frequencies: HashMap<Term, usize>,
    
    /// Stopwords specific to this corpus
    stopwords: HashSet<String>,
    
    /// Whether the corpus has been indexed for TF-IDF calculations
    indexed: bool,
    
    /// Metadata associated with the corpus
    metadata: HashMap<String, String>,
}

impl Corpus {
    /// Create a new corpus with the given ID and name
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            id: CorpusId::new(id.into()),
            name: name.into(),
            description: None,
            documents: HashMap::new(),
            document_frequencies: HashMap::new(),
            stopwords: HashSet::new(),
            indexed: false,
            metadata: HashMap::new(),
        }
    }
    
    /// Create a new corpus with a description
    pub fn with_description(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let mut corpus = Self::new(id, name);
        corpus.description = Some(description.into());
        corpus
    }
    
    /// Get the corpus ID
    pub fn id(&self) -> &CorpusId {
        &self.id
    }
    
    /// Get the name of the corpus
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Set the name of the corpus
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }
    
    /// Get the description of the corpus, if available
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    
    /// Set the description for this corpus
    pub fn set_description(&mut self, description: impl Into<String>) {
        self.description = Some(description.into());
    }
    
    /// Clear the description
    pub fn clear_description(&mut self) {
        self.description = None;
    }
    
    /// Get the number of documents in the corpus
    pub fn document_count(&self) -> usize {
        self.documents.len()
    }
    
    /// Check if the corpus contains a document with the given ID
    pub fn contains_document(&self, document_id: &DocumentId) -> bool {
        self.documents.contains_key(document_id)
    }
    
    /// Get a document by ID
    pub fn get_document(&self, document_id: &DocumentId) -> Option<&Document> {
        self.documents.get(document_id)
    }
    
    /// Get a mutable reference to a document by ID
    pub fn get_document_mut(&mut self, document_id: &DocumentId) -> Option<&mut Document> {
        self.documents.get_mut(document_id)
    }

    pub fn add_document(&mut self, document: Document) -> DomainResult<()> {
        let document_id = document.id().clone();


        if self.contains_document(&document_id) {
            return Err(DomainError::InvalidOperation(
                format!("Document with ID '{}' already exists in corpus", document_id.value())
            ))
        }

        // If the corpus is already indexed, we need to update document frequencies
        if self.indexed {

            let unique_terms: HashSet<_> = document.term_frequencies().keys().collect();
            for term in unique_terms {
                let count = self.document_frequencies.entry(term.clone()).or_insert(0);
                *count += 1;
            }

        }

        self.documents.insert(document_id, document);
        Ok(())
    }

     /// Remove a document from the corpus
    pub fn remove_document(&mut self, document_id: &DocumentId) -> DomainResult<Document> {
        if !self.contains_document(document_id) {
            return Err(DomainError::NotFound(
                format!("Document with ID '{}' not found in corpus", document_id.value())
            ));
        }
        
        let document = self.documents.remove(document_id).unwrap();
        
        // If the corpus is indexed, update document frequencies
        if self.indexed {
            let unique_terms: HashSet<_> = document.term_frequencies().keys().collect();
            for term in unique_terms {
                if let Some(count) = self.document_frequencies.get_mut(term) {
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        self.document_frequencies.remove(term);
                    }
                }
            }
        }
        
        Ok(document)
    }
    /// Get all documents in the corpus
    pub fn documents(&self) -> impl Iterator<Item = &Document> {
        self.documents.values()
    }
    
    /// Get document IDs in the corpus
    pub fn document_ids(&self) -> impl Iterator<Item = &DocumentId> {
        self.documents.keys()
    }
    
    /// Add a stopword to the corpus
    pub fn add_stopword(&mut self, word: impl Into<String>) {
        self.stopwords.insert(word.into());
    }
    
    /// Add multiple stopwords
    pub fn add_stopwords(&mut self, words: impl IntoIterator<Item = impl Into<String>>) {
        for word in words {
            self.add_stopword(word);
        }
    }
    
    /// Remove a stopword from the corpus
    pub fn remove_stopword(&mut self, word: &str) -> bool {
        self.stopwords.remove(word)
    }
    
    /// Check if a word is a stopword in this corpus
    pub fn is_stopword(&self, word: &str) -> bool {
        self.stopwords.contains(word)
    }
    
    /// Get all stopwords
    pub fn stopwords(&self) -> impl Iterator<Item = &String> {
        self.stopwords.iter()
    }
    
    /// Get the number of documents containing a specific term
    pub fn document_frequency(&self, term: &Term) -> usize {
        self.document_frequencies.get(term).copied().unwrap_or(0)
    }

    pub fn inverse_document_frequency(&self, term: &Term) -> f64 {
        let doc_count = self.document_count() as f64;
        if doc_count == 0.0 {
            return 0.0
        }

        let doc_freq = self.document_frequency(term) as f64;

        if doc_freq == 0.0 {
            return 0.0
        }

        (doc_count / doc_freq).ln()

    }

     /// Build or rebuild the document frequency index
    pub fn build_index(&mut self) {
        self.document_frequencies.clear();
        
        for document in self.documents.values() {
            let unique_terms: HashSet<_> = document.term_frequencies().keys().collect();
            for term in unique_terms {
                let count = self.document_frequencies.entry(term.clone()).or_insert(0);
                *count += 1;
            }
        }

        self.indexed = true;
    }

     /// Check if the corpus is indexed
    pub fn is_indexed(&self) -> bool {
        self.indexed
    }
    
    /// Get corpus metadata
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
    
    /// Get mutable reference to metadata
    pub fn metadata_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.metadata
    }
    
    /// Set a metadata field
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }

   
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Term;
    
    #[test]
    fn test_corpus_creation() {
        let corpus = Corpus::new("corpus1", "Test Corpus");
        assert_eq!(corpus.id().value(), "corpus1");
        assert_eq!(corpus.name(), "Test Corpus");
        assert_eq!(corpus.description(), None);
        assert_eq!(corpus.document_count(), 0);
        assert_eq!(corpus.is_indexed(), false);
    }
    
    #[test]
    fn test_add_document() {
        let mut corpus = Corpus::new("corpus1", "Test Corpus");
        let doc = Document::new("doc1", "This is a test");
        
        corpus.add_document(doc).unwrap();
        assert_eq!(corpus.document_count(), 1);
        assert!(corpus.contains_document(&DocumentId::new("doc1")));
    }
    
    #[test]
    fn test_document_frequencies() {
        let mut corpus = Corpus::new("corpus1", "Test Corpus");
        
        // Create first document with terms
        let mut doc1 = Document::new("doc1", "This is a test");
        doc1.add_term(Term::new("this"));
        doc1.add_term(Term::new("is"));
        doc1.add_term(Term::new("a"));
        doc1.add_term(Term::new("test"));
        
        // Create second document with some overlapping terms
        let mut doc2 = Document::new("doc2", "This is another example");
        doc2.add_term(Term::new("this"));
        doc2.add_term(Term::new("is"));
        doc2.add_term(Term::new("another"));
        doc2.add_term(Term::new("example"));
        
        // Add documents to corpus
        corpus.add_document(doc1).unwrap();
        corpus.add_document(doc2).unwrap();
        
        // Build index
        corpus.build_index();
        assert!(corpus.is_indexed());
        
        // Check document frequencies
        assert_eq!(corpus.document_frequency(&Term::new("this")), 2);
        assert_eq!(corpus.document_frequency(&Term::new("is")), 2);
        assert_eq!(corpus.document_frequency(&Term::new("a")), 1);
        assert_eq!(corpus.document_frequency(&Term::new("test")), 1);
        assert_eq!(corpus.document_frequency(&Term::new("another")), 1);
        assert_eq!(corpus.document_frequency(&Term::new("example")), 1);
        assert_eq!(corpus.document_frequency(&Term::new("unknown")), 0);
        
        // Check IDF calculations
        let idf_this = corpus.inverse_document_frequency(&Term::new("this"));
        let idf_a = corpus.inverse_document_frequency(&Term::new("a"));
        
        // IDF of "this" should be ln(2/2) = 0
        assert!((idf_this - 0.0).abs() < f64::EPSILON);
        
        // IDF of "a" should be ln(2/1) = ln(2)
        assert!((idf_a - 2.0_f64.ln()).abs() < f64::EPSILON);
    }
    
    #[test]
    fn test_remove_document() {
        let mut corpus = Corpus::new("corpus1", "Test Corpus");
        
        // Create and add a document
        let mut doc = Document::new("doc1", "This is a test");
        doc.add_term(Term::new("this"));
        doc.add_term(Term::new("is"));
        
        corpus.add_document(doc).unwrap();
        corpus.build_index();
        
        // Verify document frequency
        assert_eq!(corpus.document_frequency(&Term::new("this")), 1);
        
        // Remove the document
        let removed_doc = corpus.remove_document(&DocumentId::new("doc1")).unwrap();
        assert_eq!(removed_doc.id().value(), "doc1");
        assert_eq!(corpus.document_count(), 0);
        
        // Document frequency should be updated
        assert_eq!(corpus.document_frequency(&Term::new("this")), 0);
    }
}