// src/infrastructure/repository/corpus_repository.rs

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::domain::{Corpus, CorpusId};
use super::{RepositoryError, RepositoryResult};

/// Repository interface for Corpus entities
pub trait CorpusRepository: Send + Sync {
    /// Find a corpus by ID
    fn find(&self, id: &CorpusId) -> RepositoryResult<Option<Corpus>>;
    
    /// Check if a corpus exists
    fn exists(&self, id: &CorpusId) -> RepositoryResult<bool>;
    
    /// Save a corpus
    fn save(&self, corpus: &Corpus) -> RepositoryResult<()>;
    
    /// Delete a corpus
    fn delete(&self, id: &CorpusId) -> RepositoryResult<()>;
    
    /// Find all corpora
    fn find_all(&self) -> RepositoryResult<Vec<Corpus>>;
    
    /// Count all corpora
    fn count(&self) -> RepositoryResult<usize>;
    
    /// Find corpora by name (partial match)
    fn find_by_name(&self, name: &str) -> RepositoryResult<Vec<Corpus>>;
}

/// In-memory implementation of CorpusRepository
pub struct InMemoryCorpusRepository {
    corpora: Arc<RwLock<HashMap<String, Corpus>>>,
}

impl InMemoryCorpusRepository {
    /// Create a new InMemoryCorpusRepository
    pub fn new() -> Self {
        Self {
            corpora: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryCorpusRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl CorpusRepository for InMemoryCorpusRepository {
    fn find(&self, id: &CorpusId) -> RepositoryResult<Option<Corpus>> {
        let corpora = self.corpora.read().map_err(|e| {
            RepositoryError::Other(format!("Lock error: {}", e))
        })?;
        
        Ok(corpora.get(id.value()).cloned())
    }
    
    fn exists(&self, id: &CorpusId) -> RepositoryResult<bool> {
        let corpora = self.corpora.read().map_err(|e| {
            RepositoryError::Other(format!("Lock error: {}", e))
        })?;
        
        Ok(corpora.contains_key(id.value()))
    }
    
    fn save(&self, corpus: &Corpus) -> RepositoryResult<()> {
        let mut corpora = self.corpora.write().map_err(|e| {
            RepositoryError::Other(format!("Lock error: {}", e))
        })?;
        
        corpora.insert(corpus.id().value().to_string(), corpus.clone());
        Ok(())
    }
    
    fn delete(&self, id: &CorpusId) -> RepositoryResult<()> {
        let mut corpora = self.corpora.write().map_err(|e| {
            RepositoryError::Other(format!("Lock error: {}", e))
        })?;
        
        corpora.remove(id.value());
        Ok(())
    }
    
    fn find_all(&self) -> RepositoryResult<Vec<Corpus>> {
        let corpora = self.corpora.read().map_err(|e| {
            RepositoryError::Other(format!("Lock error: {}", e))
        })?;
        
        let results: Vec<Corpus> = corpora.values().cloned().collect();
        Ok(results)
    }
    
    fn count(&self) -> RepositoryResult<usize> {
        let corpora = self.corpora.read().map_err(|e| {
            RepositoryError::Other(format!("Lock error: {}", e))
        })?;
        
        Ok(corpora.len())
    }
    
    fn find_by_name(&self, name: &str) -> RepositoryResult<Vec<Corpus>> {
        let corpora = self.corpora.read().map_err(|e| {
            RepositoryError::Other(format!("Lock error: {}", e))
        })?;

        let match_corpora = corpora.values()
            .filter(|c| c.name().to_lowercase().contains(&name.to_lowercase()))
            .cloned()
            .collect();
        
        Ok(match_corpora)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_save_and_find_corpus() {
        let repo = InMemoryCorpusRepository::new();
        let corpus = Corpus::new("corpus1", "Test Corpus");
        
        // Save corpus
        repo.save(&corpus).unwrap();
        
        // Find corpus
        let found = repo.find(&CorpusId::new("corpus1")).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id().value(), "corpus1");
    }
    
    #[test]
    fn test_exists() {
        let repo = InMemoryCorpusRepository::new();
        let corpus = Corpus::new("corpus1", "Test Corpus");
        
        // Save corpus
        repo.save(&corpus).unwrap();
        
        // Check existence
        assert!(repo.exists(&CorpusId::new("corpus1")).unwrap());
        assert!(!repo.exists(&CorpusId::new("corpus2")).unwrap());
    }
    
    #[test]
    fn test_delete() {
        let repo = InMemoryCorpusRepository::new();
        let corpus = Corpus::new("corpus1", "Test Corpus");
        
        // Save corpus
        repo.save(&corpus).unwrap();
        
        // Delete corpus
        repo.delete(&CorpusId::new("corpus1")).unwrap();
        
        // Check it's gone
        let found = repo.find(&CorpusId::new("corpus1")).unwrap();
        assert!(found.is_none());
    }
    
    #[test]
    fn test_find_all_and_count() {
        let repo = InMemoryCorpusRepository::new();
        
        // Save corpora
        repo.save(&Corpus::new("corpus1", "First Corpus")).unwrap();
        repo.save(&Corpus::new("corpus2", "Second Corpus")).unwrap();
        
        // Find all
        let all = repo.find_all().unwrap();
        assert_eq!(all.len(), 2);
        
        // Count
        assert_eq!(repo.count().unwrap(), 2);
    }
    
    #[test]
    fn test_find_by_name() {
        let repo = InMemoryCorpusRepository::new();
        
        // Save corpora
        repo.save(&Corpus::new("corpus1", "News Articles")).unwrap();
        repo.save(&Corpus::new("corpus2", "Scientific Papers")).unwrap();
        repo.save(&Corpus::new("corpus3", "More News")).unwrap();
        
        // Find by name
        let news_corpora = repo.find_by_name("news").unwrap();
        assert_eq!(news_corpora.len(), 2);
        
        let scientific_corpora = repo.find_by_name("scientific").unwrap();
        assert_eq!(scientific_corpora.len(), 1);
        assert_eq!(scientific_corpora[0].id().value(), "corpus2");
        
        // Non-existent name
        let nonexistent = repo.find_by_name("nonexistent").unwrap();
        assert_eq!(nonexistent.len(), 0);
    }
}