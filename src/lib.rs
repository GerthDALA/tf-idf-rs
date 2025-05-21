pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod interfaces;

/// Re-export commonly used types for convenience
//pub use domain::{Document, Corpus, Term, TfIdf};
//pub use application::{DocumentService, CorpusService, TfIdfService};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");