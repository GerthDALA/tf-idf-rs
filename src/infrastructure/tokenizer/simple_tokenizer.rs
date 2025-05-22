use std::{collections::HashSet, sync::RwLock};

use super::Tokenizer;

pub struct SimpleTokenizer {
    stopwords: RwLock<HashSet<String>>   
}

impl SimpleTokenizer {
    pub fn new() -> Self {
        let mut stopwords = HashSet::new();

        for word in DEFAULT_STOPWORDS {
            stopwords.insert(word.to_string());
        }

        Self {
            stopwords: RwLock::new(stopwords)
        }
    }

    pub fn with_stopwords(stopwords: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let stopwords_set: HashSet<String> = stopwords
            .into_iter()
            .map(|s| s.into().to_lowercase())
            .collect();
        Self {
            stopwords: RwLock::new(stopwords_set)
        }
    }
}

impl Default for SimpleTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for SimpleTokenizer {
    fn tokenize(&self, text: &str) -> Vec<String> {
        let text = text.to_lowercase();

        let tokens: Vec<String> = text
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .map(|t|t.to_string())
            .collect();

        tokens
    }
    
    fn is_stopword(&self, word: &str) -> bool {
        let stopwords = self.stopwords.read().expect("Failed acquire read lock");
        stopwords.contains(&word.to_lowercase())
    }
    
    fn stopwords(&self) -> Vec<String> {
        let stopwords = self.stopwords.read().expect("Failed acquire read lock");
        stopwords.iter().cloned().collect()
    }
    
    fn add_stopword(&mut self, word: &str) {
        let mut stopwords = self.stopwords.write().expect("FAILED to acquire write lock");
        stopwords.insert(word.to_lowercase());
    }
    
    fn remove_stopword(&mut self, word: &str) -> bool {
        let mut stopwords = self.stopwords.write().expect("FAILED to acquire write lock");
        stopwords.remove(&word.to_lowercase())
    }
}

static DEFAULT_STOPWORDS: &[&str] = &[
    "a", "about", "above", "after", "again", "against", "all", "am", "an", "and", "any", "are", "aren't", "as", "at",
    "be", "because", "been", "before", "being", "below", "between", "both", "but", "by",
    "can", "cannot", "can't", "could", "couldn't",
    "did", "didn't", "do", "does", "doesn't", "doing", "don't", "down", "during",
    "each",
    "few", "for", "from", "further",
    "had", "hadn't", "has", "hasn't", "have", "haven't", "having", "he", "he'd", "he'll", "he's", "her", "here", "here's", "hers", "herself", "him", "himself", "his", "how", "how's",
    "i", "i'd", "i'll", "i'm", "i've", "if", "in", "into", "is", "isn't", "it", "it's", "its", "itself",
    "let's",
    "me", "more", "most", "mustn't", "my", "myself",
    "no", "nor", "not",
    "of", "off", "on", "once", "only", "or", "other", "ought", "our", "ours", "ourselves", "out", "over", "own",
    "same", "shan't", "she", "she'd", "she'll", "she's", "should", "shouldn't", "so", "some", "such",
    "than", "that", "that's", "the", "their", "theirs", "them", "themselves", "then", "there", "there's", "these", "they", "they'd", "they'll", "they're", "they've", "this", "those", "through", "to", "too",
    "under", "until", "up",
    "very",
    "was", "wasn't", "we", "we'd", "we'll", "we're", "we've", "were", "weren't", "what", "what's", "when", "when's", "where", "where's", "which", "while", "who", "who's", "whom", "why", "why's", "with", "won't", "would", "wouldn't",
    "you", "you'd", "you'll", "you're", "you've", "your", "yours", "yourself", "yourselves"
];

#[cfg(test)]
mod tests {
   use super::*;
    
    #[test]
    fn test_tokenize() {
        let tokenizer = SimpleTokenizer::new();
        
        // Test basic tokenization
        let tokens = tokenizer.tokenize("Hello, world!");
        assert_eq!(tokens, vec!["hello", "world"]);
        
        // Test with multiple spaces and punctuation
        let tokens = tokenizer.tokenize("This is a   test, with some punctuation!");
        assert_eq!(tokens, vec!["this", "is", "a", "test", "with", "some", "punctuation"]);
        
        // Test with numbers
        let tokens = tokenizer.tokenize("TF-IDF is calculated as tf * idf for term t in doc d.");
        assert_eq!(
            tokens, 
            vec!["tf", "idf", "is", "calculated", "as", "tf", "idf", "for", "term", "t", "in", "doc", "d"]
        );
    }
    
    #[test]
    fn test_stopwords() {
        let mut tokenizer = SimpleTokenizer::new();
        
        // Test default stopwords
        assert!(tokenizer.is_stopword("the"));
        assert!(tokenizer.is_stopword("and"));
        assert!(!tokenizer.is_stopword("hello"));
        
        // Test case insensitivity
        assert!(tokenizer.is_stopword("The"));
        
        // Add custom stopword
        tokenizer.add_stopword("hello");
        assert!(tokenizer.is_stopword("hello"));
        
        // Remove stopword
        assert!(tokenizer.remove_stopword("hello"));
        assert!(!tokenizer.is_stopword("hello"));
        
        // Get all stopwords
        let stopwords = tokenizer.stopwords();
        assert!(stopwords.contains(&"the".to_string()));
        assert!(stopwords.len() > 0);
    }
    
    #[test]
    fn test_custom_stopwords() {
        let tokenizer = SimpleTokenizer::with_stopwords(vec!["custom", "words"]);
        
        // Should only have our custom stopwords
        assert!(tokenizer.is_stopword("custom"));
        assert!(tokenizer.is_stopword("words"));
        assert!(!tokenizer.is_stopword("the")); // Default stopword, not included
    } 
}