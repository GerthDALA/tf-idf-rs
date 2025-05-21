// src/domain/tf_idf.rs

use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

use super::{Document, Corpus, Term, DomainError, DomainResult};

/// Error type specific to TF-IDF operations
#[derive(Debug, thiserror::Error)]
pub enum TfIdfError {
    #[error("Corpus is not indexed")]
    CorpusNotIndexed,
    
    #[error("Invalid calculation: {0}")]
    InvalidCalculation(String),
    
    #[error("Document not found: {0}")]
    DocumentNotFound(String),
}

/// A TF-IDF score for a term in a document
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TfIdfScore {
    /// The term this score is for
    term: Term,
    
    /// The raw term frequency in the document
    tf: f64,
    
    /// The inverse document frequency in the corpus
    idf: f64,
    
    /// The combined TF-IDF score
    score: f64,
}

impl TfIdfScore {
    /// Create a new TF-IDF score
    pub fn new(term: Term, tf: f64, idf: f64) -> Self {
        let score = tf * idf;
        Self { term, tf, idf, score }
    }
    
    /// Get the term
    pub fn term(&self) -> &Term {
        &self.term
    }
    
    /// Get the term frequency component
    pub fn tf(&self) -> f64 {
        self.tf
    }
    
    /// Get the inverse document frequency component
    pub fn idf(&self) -> f64 {
        self.idf
    }
    
    /// Get the combined TF-IDF score
    pub fn score(&self) -> f64 {
        self.score
    }
}

impl PartialOrd for TfIdfScore {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

/// A document with its relevance score for a query
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoredDocument {
    /// The document
    document: Document,
    
    /// The overall relevance score
    score: f64,
    
    /// Individual term scores that contributed to the overall score
    term_scores: Vec<TfIdfScore>,
}

impl ScoredDocument {
    /// Create a new scored document
    pub fn new(document: Document, score: f64, term_scores: Vec<TfIdfScore>) -> Self {
        Self { document, score, term_scores }
    }
    
    /// Get the document
    pub fn document(&self) -> &Document {
        &self.document
    }
    
    /// Get the overall relevance score
    pub fn score(&self) -> f64 {
        self.score
    }
    
    /// Get the individual term scores
    pub fn term_scores(&self) -> &[TfIdfScore] {
        &self.term_scores
    }
    
    /// Get the most important terms (highest TF-IDF scores)
    pub fn top_terms(&self, limit: usize) -> Vec<&TfIdfScore> {
        let mut scores = self.term_scores.iter().collect::<Vec<_>>();
        scores.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(limit);
        scores
    }
}

impl PartialOrd for ScoredDocument {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

/// Options for TF-IDF calculation
/// Options for TF-IDF calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TfIdfOptions {
    /// Whether to apply smoothing to prevent zero IDF scores
    pub apply_smoothing: bool,
    
    /// Whether to normalize TF-IDF vectors
    pub normalize: bool,
    
    /// Whether to use logarithmic term frequency instead of raw counts
    pub use_log_tf: bool,
    
    /// Whether to filter out stopwords
    pub filter_stopwords: bool,
    
    /// Custom TF weighting function (None = use default)
    #[serde(skip)]
    pub tf_weighting: Option<fn(usize, usize) -> f64>,
    
    /// Custom IDF weighting function (None = use default)
    #[serde(skip)]
    pub idf_weighting: Option<fn(usize, usize) -> f64>,
}

impl Default for TfIdfOptions {
    fn default() -> Self {
        Self {
            apply_smoothing: true,
            normalize: true,
            use_log_tf: true,
            filter_stopwords: true,
            tf_weighting: None,
            idf_weighting: None,
        }
    }
}

/// The main TF-IDF calculator
#[derive(Debug, Clone)]
pub struct TfIdf {
    /// Options for TF-IDF calculation
    options: TfIdfOptions,
}

impl Default for TfIdf {
    fn default() -> Self {
        Self::new(TfIdfOptions::default())
    }
}

impl TfIdf {
    /// Create a new TF-IDF calculator with the given options
    pub fn new(options: TfIdfOptions) -> Self {
        Self { options }
    }
    
    /// Get the current options
    pub fn options(&self) -> &TfIdfOptions {
        &self.options
    }
    
    /// Update the options
    pub fn set_options(&mut self, options: TfIdfOptions) {
        self.options = options;
    }

    pub fn calculate_term_tfidf(
        &self,
        term: &Term,
        document: &Document,
        corpus: &Corpus
    ) -> DomainResult<TfIdfScore> {
        println!("[DEBUG] At start of calculate_term_tfidf for term '{}': corpus.is_indexed() = {}", term.text(), corpus.is_indexed());
        if !corpus.is_indexed() {
            return Err(DomainError::TfIdfError(TfIdfError::CorpusNotIndexed));
        }

        //Skip stopwords if configured to do so
        if self.options.filter_stopwords && term.is_stopword() {
            return Err(DomainError::TfIdfError(TfIdfError::InvalidCalculation("Term is a stopword".to_string())));
        }

        let tf = if let Some(tf_fn) = self.options.tf_weighting {
            //Use custom weighting function
            let term_count = document.term_frequency(term).0;
            let total_terms = document.term_count();
            tf_fn(term_count, total_terms)

        } else if self.options.use_log_tf {
            let tf_raw = document.term_frequency(term).0 as f64;
            if tf_raw > 0.0 {
                1.0 + tf_raw.ln()
            } else {
                0.0
            }
        } else {
            document.normalized_term_frequency(term)
        };

        let idf = if let Some(idf_fn) = self.options.idf_weighting {
            let doc_freq = corpus.document_frequency(term);
            let total_docs = corpus.document_count();
            idf_fn(doc_freq, total_docs)
        } else {
            let mut idf = corpus.inverse_document_frequency(term);

            if self.options.apply_smoothing {
                // Add 1 to document frequency to prevent division by zero
                let doc_count = corpus.document_count() as f64;
                let doc_freq = corpus.document_frequency(term) as f64 + 1.0;
                idf = (doc_count / doc_freq).ln();
            }

            idf
        };

        Ok(TfIdfScore::new(term.clone(), tf, idf))
    }

    pub fn calculate_document_tfidf(
        &self,
        document: &Document,
        corpus: &Corpus
    ) -> DomainResult<Vec<TfIdfScore>> {
        if !corpus.is_indexed() {
            return Err(DomainError::TfIdfError(TfIdfError::CorpusNotIndexed))
        }

        let mut scores = Vec::new();

        for (term, _) in document.term_frequencies() {
            if self.options.filter_stopwords && term.is_stopword() {
                continue;
            }

            match self.calculate_term_tfidf(term, document, corpus) {
                Ok(score) => scores.push(score),
                Err(DomainError::TfIdfError(TfIdfError::InvalidCalculation(_))) => {
                    continue
                },
                Err(e) => return Err(e) 
            }
        }

        if self.options.normalize {
            self.normalize_scores(&mut scores);
        }

       // Sort by score (highest first)
        scores.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(scores)
    }

    pub fn search(
        &self,
        query_terms: &[Term],
        corpus: &Corpus
    ) -> DomainResult<Vec<ScoredDocument>> {

        if !corpus.is_indexed() {
             return Err(DomainError::TfIdfError(TfIdfError::CorpusNotIndexed))
        }
        let mut results = Vec::new();

        for document in corpus.documents() {
            let mut doc_score = 0.0;
            let mut term_scores = Vec::new();

            for term in query_terms {
                if self.options.filter_stopwords && term.is_stopword() {
                    continue;
                }

                match  self.calculate_term_tfidf(term, document, corpus) {
                    Ok(score) => {
                        doc_score += score.score();
                        term_scores.push(score);
                    },
                    Err(DomainError::TfIdfError(TfIdfError::InvalidCalculation(_))) => {
                        continue
                    },
                    Err(e) => return Err(e) 
                }
            }

            if doc_score > 0.0 {
                results.push(ScoredDocument::new(
                    document.clone(),
                    doc_score,
                    term_scores
                ));
            }
        }

         // Sort by score (highest first)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(results)
    }

      /// Generate document vectors for all documents in a corpus
    pub fn generate_document_vectors(
        &self,
        corpus: &Corpus,
    ) -> DomainResult<HashMap<String, HashMap<String, f64>>> {
        if !corpus.is_indexed() {
            return Err(DomainError::TfIdfError(TfIdfError::CorpusNotIndexed))
        }

        let mut documents_vector = HashMap::new();
        for document in corpus.documents() {
            let mut vector = HashMap::new();
            let scores = self.calculate_document_tfidf(document, corpus)?;

            for score in scores {
                vector.insert(score.term().text().to_string(), score.score());
            }

            documents_vector.insert(document.id().value().to_string(), vector);
        }
        
        Ok(documents_vector)
    }

     /// Calculate the cosine similarity between two documents
    pub fn cosine_similarity(
        &self,
        doc1_id: &str,
        doc2_id: &str,
        corpus: &Corpus,
    ) -> DomainResult<f64> {
        let vectors = self.generate_document_vectors(corpus)?;
        
        let vec1 = vectors.get(doc1_id).ok_or_else(|| {
            DomainError::TfIdfError(TfIdfError::DocumentNotFound(doc1_id.to_string()))
        })?;
        
        let vec2 = vectors.get(doc2_id).ok_or_else(|| {
            DomainError::TfIdfError(TfIdfError::DocumentNotFound(doc2_id.to_string()))
        })?;
        
        // Calculate dot product
        let mut dot_product = 0.0;
        let mut magnitude1 = 0.0;
        let mut magnitude2 = 0.0;
        
        // Get all unique terms from both vectors
        let mut all_terms = HashSet::new();
        all_terms.extend(vec1.keys().cloned());
        all_terms.extend(vec2.keys().cloned());
        
        // Calculate dot product and magnitudes
        for term in all_terms {
            let val1 = vec1.get(&term).copied().unwrap_or(0.0);
            let val2 = vec2.get(&term).copied().unwrap_or(0.0);
            
            dot_product += val1 * val2;
            magnitude1 += val1 * val1;
            magnitude2 += val2 * val2;
        }
        
        // Calculate cosine similarity
        let magnitude = magnitude1.sqrt() * magnitude2.sqrt();
        if magnitude == 0.0 {
            Ok(0.0)
        } else {
            Ok(dot_product / magnitude)
        }
    }

     /// Normalize a set of TF-IDF scores using L2 normalization
    fn normalize_scores(&self, scores: &mut [TfIdfScore]) {
        // Calculate the sum of squares

        let sum_of_squares: f64 = scores.iter()
            .map(|score| score.score * score.score)
            .sum();

        if sum_of_squares == 0.0 {
            return;
        }

        let normalization_factor = sum_of_squares.sqrt();
        for score in scores.iter_mut() {
            score.score /= normalization_factor;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Document, Term, DocumentId};
    
    fn create_test_corpus() -> Corpus {
        let mut corpus = Corpus::new("test", "Test Corpus");
        
        // Document 1: "this is a test"
        let mut doc1 = Document::new("doc1", "this is a test");
        doc1.add_term(Term::new("this"));
        doc1.add_term(Term::new("is"));
        doc1.add_term(Term::new("a"));
        doc1.add_term(Term::new("test"));
        
        // Document 2: "this is another test"
        let mut doc2 = Document::new("doc2", "this is another test");
        doc2.add_term(Term::new("this"));
        doc2.add_term(Term::new("is"));
        doc2.add_term(Term::new("another"));
        doc2.add_term(Term::new("test"));
        
        // Document 3: "yet another example"
        let mut doc3 = Document::new("doc3", "yet another example");
        doc3.add_term(Term::new("yet"));
        doc3.add_term(Term::new("another"));
        doc3.add_term(Term::new("example"));
        
        corpus.add_document(doc1).unwrap();
        corpus.add_document(doc2).unwrap();
        corpus.add_document(doc3).unwrap();
        
        corpus.build_index();
        println!("[DEBUG] In create_test_corpus, after build_index: corpus.is_indexed() = {}", corpus.is_indexed());
        corpus
    }
    
    #[test]
    fn test_tfidf_calculation() {
        let corpus = create_test_corpus();
        let tfidf = TfIdf::default();
        
        let doc1 = corpus.get_document(&DocumentId::new("doc1")).unwrap();
        
        // Calculate TF-IDF for the term "test" in doc1
        let term = Term::new("test");
        let score = tfidf.calculate_term_tfidf(&term, doc1, &corpus).unwrap();
        
        // With the default options (log TF), tf should be 1 + ln(1) = 1.0
        assert!((score.tf() - 1.0).abs() < f64::EPSILON);
        
        // IDF should be ln(3/(2+1)) = ln(1) = 0.0 with default smoothing
        let expected_idf = (3.0f64 / (corpus.document_frequency(&term) as f64 + 1.0)).ln(); 
        // Or simply: let expected_idf = 0.0; for this specific "test" term
        assert!((score.idf() - expected_idf).abs() < f64::EPSILON);
        
        // Score should be tf * idf
        assert!((score.score() - (1.0 * expected_idf)).abs() < f64::EPSILON);
    }
    
    #[test]
    fn test_document_tfidf() {
        let corpus = create_test_corpus();
        let tfidf = TfIdf::default();
        
        let doc1 = corpus.get_document(&DocumentId::new("doc1")).unwrap();
        
        // Calculate TF-IDF for all terms in doc1
        let scores = tfidf.calculate_document_tfidf(doc1, &corpus).unwrap();
        
        // There should be 4 terms
        assert_eq!(scores.len(), 4);
        
        let test_score = scores.iter().find(|s| s.term().text() == "test").unwrap();
        let this_score = scores.iter().find(|s| s.term().text() == "this").unwrap();

        // For "test" and "this", with default smoothing, the IDF is 0, so score is 0.
        assert!((test_score.score() - 0.0).abs() < f64::EPSILON);
        assert!((this_score.score() - 0.0).abs() < f64::EPSILON);

        // You might want to check a term that will have a non-zero score, e.g., "a":
        // DF("a") = 1. IDF("a") = ln(3/(1+1)) = ln(1.5) approx 0.405.
        // TF("a" in doc1) = 1.0. Score("a" in doc1) approx 0.405.
        let a_score = scores.iter().find(|s| s.term().text() == "a").unwrap();
        assert!(a_score.score() > 0.0); // This should pass
    }
    
    #[test]
    fn test_search() {
        let corpus = create_test_corpus(); // Uses your existing helper

        // --- Test with DEFAULT TfIdf options (apply_smoothing = true) ---
        let tfidf_default = TfIdf::default();

        // Search for "test"
        // With default smoothing, IDF("test") = ln(3/(2+1)) = ln(1) = 0.
        // So, TF-IDF score for "test" will be 0.
        // Thus, documents containing only "test" (or other terms that also get a zero score)
        // will have an overall document_score of 0 and won't be included in results.
        let query_terms_test_default = vec![Term::new("test")];
        let results_test_default = tfidf_default.search(&query_terms_test_default, &corpus).unwrap();
        
        println!("[DEBUG] test_search (default options) - Query 'test': Results len = {}", results_test_default.len());
        for (i, res) in results_test_default.iter().enumerate() {
            println!("[DEBUG] test_search (default options) - Result {}: Doc ID = {}, Score = {}", i, res.document().id().value(), res.score());
        }
        assert_eq!(results_test_default.len(), 0, "With default smoothing, 'test' should have a TF-IDF score of 0, leading to 0 search results for this query.");

        // Search for "another example"
        // IDF("another") with smoothing = ln(3/(2+1)) = 0.
        // IDF("example") with smoothing = ln(3/(1+1)) = ln(1.5) approx 0.405.
        // doc1 ("this is a test"): score = 0
        // doc2 ("this is another test"): score = 0
        // doc3 ("yet another example"): score for "example" will be > 0.
        let query_terms_another_example_default = vec![Term::new("another"), Term::new("example")];
        let results_another_example_default = tfidf_default.search(&query_terms_another_example_default, &corpus).unwrap();
        
        println!("[DEBUG] test_search (default options) - Query 'another example': Results len = {}", results_another_example_default.len());
        for (i, res) in results_another_example_default.iter().enumerate() {
            println!("[DEBUG] test_search (default options) - Result {}: Doc ID = {}, Score = {}", i, res.document().id().value(), res.score());
        }
        assert_eq!(results_another_example_default.len(), 1, "Only doc3 should have a non-zero score for 'another example' with default smoothing.");
        if !results_another_example_default.is_empty() {
            assert_eq!(results_another_example_default[0].document().id().value(), "doc3");
        }

        // --- Test with TfIdfOptions where apply_smoothing = false ---
        let options_no_smoothing = TfIdfOptions {
            apply_smoothing: false,
            ..TfIdfOptions::default() // use other defaults like use_log_tf = true
        };
        let tfidf_no_smoothing = TfIdf::new(options_no_smoothing);

        // Search for "test" (no smoothing)
        // IDF("test") without smoothing = ln(3/2) approx 0.405. Scores will be > 0.
        let query_terms_test_no_smoothing = vec![Term::new("test")];
        let results_test_no_smoothing = tfidf_no_smoothing.search(&query_terms_test_no_smoothing, &corpus).unwrap();
        
        println!("[DEBUG] test_search (no smoothing) - Query 'test': Results len = {}", results_test_no_smoothing.len());
        for (i, res) in results_test_no_smoothing.iter().enumerate() {
            println!("[DEBUG] test_search (no smoothing) - Result {}: Doc ID = {}, Score = {}", i, res.document().id().value(), res.score());
        }
        assert_eq!(results_test_no_smoothing.len(), 2, "Without smoothing, 'test' should match doc1 and doc2.");
        if results_test_no_smoothing.len() == 2 {
            // doc1: "this is a test" (4 terms)
            // doc2: "this is another test" (4 terms)
            // TF for "test" is 1 in both. IDF for "test" is the same for both.
            // So, their scores for the query "test" should be equal.
            // The order might not be strictly defined if scores are exactly equal,
            // so we check that both expected documents are present and scores are non-negative.
            assert!(results_test_no_smoothing.iter().any(|d| d.document().id().value() == "doc1"));
            assert!(results_test_no_smoothing.iter().any(|d| d.document().id().value() == "doc2"));
            assert!(results_test_no_smoothing[0].score() > 0.0);
            assert!(results_test_no_smoothing[1].score() > 0.0);
            // If scores are expected to be equal, their relative order is stable due to sort
            assert!((results_test_no_smoothing[0].score() - results_test_no_smoothing[1].score()).abs() < f64::EPSILON, "Scores for doc1 and doc2 for query 'test' (no smoothing) should be very close or equal");
        }

        // Search for "another example" (no smoothing)
        // IDF("another") without smoothing = ln(3/2) approx 0.405.
        // IDF("example") without smoothing = ln(3/1) = ln(3) approx 1.098.
        let query_terms_another_example_no_smoothing = vec![Term::new("another"), Term::new("example")];
        let results_another_example_no_smoothing = tfidf_no_smoothing.search(&query_terms_another_example_no_smoothing, &corpus).unwrap();
        
        println!("[DEBUG] test_search (no smoothing) - Query 'another example': Results len = {}", results_another_example_no_smoothing.len());
        for (i, res) in results_another_example_no_smoothing.iter().enumerate() {
            println!("[DEBUG] test_search (no smoothing) - Result {}: Doc ID = {}, Score = {}", i, res.document().id().value(), res.score());
        }

        // doc1 ("this is a test"): "another"=0, "example"=0. Score = 0.
        // doc2 ("this is another test"): TF-IDF("another") > 0, "example"=0. Score for "another" > 0.
        // doc3 ("yet another example"): TF-IDF("another") > 0, TF-IDF("example") > 0. Highest score.
        assert_eq!(results_another_example_no_smoothing.len(), 2, "doc2 and doc3 should match 'another example' without smoothing.");
        if results_another_example_no_smoothing.len() == 2 {
            assert_eq!(results_another_example_no_smoothing[0].document().id().value(), "doc3", "doc3 should be most relevant for 'another example' without smoothing");
            assert_eq!(results_another_example_no_smoothing[1].document().id().value(), "doc2");
        }
    }
    
    #[test]
    fn test_cosine_similarity() {
        let corpus = create_test_corpus();
        let tfidf = TfIdf::default();
        
        // In test_cosine_similarity for doc1 vs doc2
        let similarity = tfidf.cosine_similarity("doc1", "doc2", &corpus).unwrap();
        assert!((similarity - 0.0).abs() < f64::EPSILON); // Expect 0.0

        
        // Calculate similarity between doc1 and doc3
        let similarity = tfidf.cosine_similarity("doc1", "doc3", &corpus).unwrap();
        
        // They don't share any terms, so should be very dissimilar
        assert!(similarity < 0.1);
    }
    
    #[test]
    fn test_options() {
        let corpus = create_test_corpus();
        
        // Create TF-IDF with custom options
        let options = TfIdfOptions {
            apply_smoothing: false,
            normalize: false,
            use_log_tf: false,
            filter_stopwords: false,
            tf_weighting: None,
            idf_weighting: None,
        };
        
        let tfidf = TfIdf::new(options);
        
        let doc1 = corpus.get_document(&DocumentId::new("doc1")).unwrap();
        
        // Calculate TF-IDF for the term "test" in doc1
        let term = Term::new("test");
        let score = tfidf.calculate_term_tfidf(&term, doc1, &corpus).unwrap();
        
        // With raw TF (no log), tf should be 1/4 = 0.25
        assert!((score.tf() - 0.25).abs() < f64::EPSILON);
        
        // Without smoothing, IDF should be ln(3/2) = ln(1.5)
        let expected_idf = (3.0f64 / 2.0f64).ln();
        assert!((score.idf() - expected_idf).abs() < f64::EPSILON);
    }
}