use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use serde::{Deserialize, Serialize};

use super::term::{Term, TermFrequency};

/// Unique identifier for a document
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocumentId(pub String);

impl DocumentId {
    /// Create a new document ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    /// Get the string representation of the ID
    pub fn value(&self) -> &str {
        &self.0
    }
}

/// Document represents a text document in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    id: DocumentId,

    content: String,

    title: Option<String>,

     /// Map of terms to their frequencies in this document
    term_frequencies: HashMap<Term, TermFrequency>,

     /// Total number of terms in the document (for normalization)
    term_count: usize,

    metadata: HashMap<String, String>
}

impl Document {
    pub fn new(
        id: impl Into<String>,
        content: impl Into<String>
    ) -> Self {
        Self {
            id: DocumentId(id.into()),
            content: content.into(),
            title: None,
            term_frequencies: HashMap::new(),
            term_count: 0,
            metadata: HashMap::new()
        }
    }

    pub fn with_title(
        id: impl Into<String>,
        title: impl Into<String>,
        content: impl Into<String>
    ) -> Self {
        let mut document = Self::new(id, content);
        document.title = Some(title.into());

        document
    }
    /// Get the document ID
    pub fn id(&self) -> &DocumentId {
        &self.id
    }
    
    /// Get the document content
    pub fn content(&self) -> &str {
        &self.content
    }
    
    /// Get the document title, if available
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }
    
    /// Set the document title
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }
    
    /// Get the term frequencies for this document
    pub fn term_frequencies(&self) -> &HashMap<Term, TermFrequency> {
        &self.term_frequencies
    }
    
    /// Get a mutable reference to term frequencies
    pub fn term_frequencies_mut(&mut self) -> &mut HashMap<Term, TermFrequency> {
        &mut self.term_frequencies
    }

    pub fn add_term(&mut self, term: Term) {
        let count = self.term_frequencies.entry(term).or_insert(TermFrequency(0));
        count.0 += 1;
        self.term_count += 1;
    }

    pub fn add_terms(&mut self, terms: impl IntoIterator<Item = Term>) {
        for term in terms {
            self.add_term(term);
        }
    }

    pub fn term_frequency(&self, term: &Term) -> TermFrequency {
        self.term_frequencies
        .get(term)
        .copied()
        .unwrap_or(TermFrequency(0))
    }

     /// Get the total number of terms in the document
    pub fn term_count(&self) -> usize {
        self.term_count
    }
    
    /// Get document metadata
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
    
    pub fn normalized_term_frequency(&self, term: &Term) -> f64 {
        if self.term_count == 0 {
            return 0.0
        }

        let term_freq = self.term_frequency(term).0 as f64;
        term_freq / self.term_count as f64
    }

     /// Clear all term frequencies (e.g., before reprocessing)
    pub fn clear_terms(&mut self) {
        self.term_frequencies.clear();
        self.term_count = 0;
    }
}

impl PartialEq for Document {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Document {}

impl Hash for Document {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_document_creation() {
        let doc = Document::new("doc1", "This is a test document");
        assert_eq!(doc.id().value(), "doc1");
        assert_eq!(doc.content(), "This is a test document");
        assert_eq!(doc.term_count(), 0);
    }

     #[test]
    fn test_add_terms() {
        let mut doc = Document::new("doc1", "This is a test");
        
        doc.add_term(Term::new("this"));
        doc.add_term(Term::new("is"));
        doc.add_term(Term::new("a"));
        doc.add_term(Term::new("test"));
        doc.add_term(Term::new("this")); // Duplicate term
        
        assert_eq!(doc.term_count(), 5);
        assert_eq!(doc.term_frequency(&Term::new("this")), TermFrequency(2));
        assert_eq!(doc.term_frequency(&Term::new("unknown")), TermFrequency(0));
        
        let normalized_freq = doc.normalized_term_frequency(&Term::new("this"));
        assert!((normalized_freq - 0.4).abs() < f64::EPSILON);
    }
}