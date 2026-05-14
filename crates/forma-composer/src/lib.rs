//! forma-composer: Anthropic API client, prompt builders, and fill orchestrator.

pub mod client;
pub mod prompts;
pub mod filler;

pub use client::{ClientError, FormaClient};
pub use filler::{ComposerError, FillResult, SchemaType, fill_from_notes};
pub use prompts::{build_system_prompt, build_user_prompt};
