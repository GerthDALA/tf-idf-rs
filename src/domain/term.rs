use std::hash::Hash;

use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TermId(pub String);

impl TermId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Term {
    text: String,

    is_stopword: bool,


     /// Optional stem of the term (for stemming algorithms)
    stem: Option<String>
}


impl Term {

    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_stopword: false,
            stem: None
        }
    }


    pub fn stopword(text: impl Into<String>) -> Self {
        let mut term = Self::new(text);

        term.is_stopword = true;

        term
    }

    pub fn with_stem(text: impl Into<String>, stem: impl Into<String>) -> Self {
        let text = text.into();
        let stem = stem.into();
        Self {
            text,
            is_stopword: false,
            stem: Some(stem)
        }
    }

    /// Get the term text
    pub fn text(&self) -> &str {
        &self.text
    }
    
    /// Check if the term is a stopword
    pub fn is_stopword(&self) -> bool {
        self.is_stopword
    }
    
    /// Mark or unmark this term as a stopword
    pub fn set_stopword(&mut self, is_stopword: bool) {
        self.is_stopword = is_stopword;
    }

    pub fn stem(&self) -> Option<&str> {
        self.stem.as_deref()
    }

    /// Set the stem for this term
    pub fn set_stem(&mut self, stem: impl Into<String>) {
        self.stem = Some(stem.into());
    }
    
    /// Clear the stem
    pub fn clear_stem(&mut self) {
        self.stem = None;
    }

    pub fn canonical(&self) -> &str {
        self.stem.as_deref().unwrap_or(&self.text)
    }
}

impl PartialEq for Term {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
    }
}

impl Eq for Term {
    
}

impl Hash for Term {

    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.text.hash(state);
    }
}

/// The frequency of a term in a document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TermFrequency(pub usize);

impl TermFrequency {
    pub fn new(count: usize) -> Self {
        Self(count)
    }

    pub fn value(&self) -> usize {
        self.0
    }

     /// Increment the frequency
    pub fn increment(&mut self) {
        self.0 += 1;
    }
    
    /// Add a specific count to the frequency
    pub fn add(&mut self, count: usize) {
        self.0 += count;
    }
}

impl From<usize> for TermFrequency {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_term_creation() {
        let term = Term::new("test");
        assert_eq!(term.text(), "test");
        assert_eq!(term.is_stopword(), false);
        assert_eq!(term.stem(), None);
        assert_eq!(term.canonical(), "test");
    }

    #[test]
    fn test_term_with_stem() {
        let term = Term::with_stem("running", "run");
        assert_eq!(term.text(), "running");
        assert_eq!(term.stem(), Some("run"));
        assert_eq!(term.canonical(), "run");
    }
    
    #[test]
    fn test_stopword() {
        let term = Term::stopword("the");
        assert_eq!(term.text(), "the");
        assert_eq!(term.is_stopword(), true);
    }
    
    #[test]
    fn test_term_frequency() {
        let mut freq = TermFrequency::new(2);
        assert_eq!(freq.value(), 2);
        
        freq.increment();
        assert_eq!(freq.value(), 3);
        
        freq.add(5);
        assert_eq!(freq.value(), 8);
    }
}