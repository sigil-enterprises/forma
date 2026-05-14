//! forma-schema: embedded JSON schemas + serde types for content and documents.

pub mod embedded;
pub mod content;
pub mod document;

// Re-export commonly used types
pub use content::*;
pub use document::*;
