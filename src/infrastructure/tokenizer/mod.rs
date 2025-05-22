mod simple_tokenizer;
pub use simple_tokenizer::SimpleTokenizer;

pub trait Tokenizer: Send + Sync {
    fn tokenize(&self, text: &str) -> Vec<String>;
    fn is_stopword(&self, word: &str) -> bool;
    fn stopwords(&self) -> Vec<String>;
    fn add_stopword(&mut self, word: &str);
    fn remove_stopword(&mut self, word: &str) -> bool;
}