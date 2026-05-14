/// Filler: takes notes text -> calls Claude -> validates -> returns content + raw YAML.

use std::path::Path;

use forma_schema::content::{BriefContent, CaseStudyContent, ContentType, ProposalContent, StatusReportContent};
use serde_yaml;

use crate::client::{ClientError, FormaClient};
use crate::prompts::{build_system_prompt, build_user_prompt};

#[derive(Debug)]
pub struct FillResult {
    pub raw_yaml: String,
}

/// Types of content that can be composed.
pub enum SchemaType {
    Proposal,
    Brief,
    CaseStudy,
    StatusReport,
}

impl SchemaType {
    fn content_type(&self) -> ContentType {
        match self {
            Self::Proposal => ContentType::Proposal,
            Self::Brief => ContentType::Brief,
            Self::CaseStudy => ContentType::CaseStudy,
            Self::StatusReport => ContentType::StatusReport,
        }
    }

    /// Parse and validate raw YAML against the content schema.
    pub fn validate(&self, raw: &str) -> Result<String, ComposerError> {
        let data: serde_yaml::Value = serde_yaml::from_str(raw).map_err(|e| ComposerError::Validation(e.to_string()))?;
        // Validate by deserializing to the concrete type (errors on schema mismatch)
        match self {
            Self::Proposal => {
                let _validated: ProposalContent = serde_yaml::from_value(data.clone())?;
            }
            Self::Brief => {
                let _validated: BriefContent = serde_yaml::from_value(data.clone())?;
            }
            Self::CaseStudy => {
                let _validated: CaseStudyContent = serde_yaml::from_value(data.clone())?;
            }
            Self::StatusReport => {
                let _validated: StatusReportContent = serde_yaml::from_value(data.clone())?;
            }
        }
        // Serialize back to canonical YAML for consistent output
        let canonical = serde_yaml::to_string(&data).map_err(|e| ComposerError::Validation(e.to_string()))?;
        Ok(canonical)
    }
}

/// Fill content from notes.
///
/// Sends notes to Claude and returns validated content YAML.
/// Raises an error if the response doesn't conform to the schema.
pub fn fill_from_notes(
    notes: &str,
    schema_type: SchemaType,
    model: &str,
    max_tokens: u32,
    existing_yaml_path: Option<&Path>,
) -> Result<FillResult, ComposerError> {
    let existing_yaml = if let Some(path) = existing_yaml_path {
        if path.exists() {
            Some(std::fs::read_to_string(path)?)
        } else {
            None
        }
    } else {
        None
    };

    let client = FormaClient::new(model, max_tokens)?;
    let system = build_system_prompt(schema_type.content_type());
    let user = build_user_prompt(notes, existing_yaml.as_deref());

    eprintln!("Calling {model}...");
    let raw = client.complete(&system, &user)?;

    // Strip accidental markdown fences
    let trimmed = raw.trim();
    let raw = if trimmed.starts_with("```") {
        let lines: Vec<&str> = trimmed.split('\n').collect();
        if lines.len() >= 2 && lines.last().map_or(false, |l| l.trim() == "```") {
            lines[1..lines.len() - 1].join("\n")
        } else {
            lines[1..].join("\n")
        }
    } else {
        trimmed.to_string()
    };

    // Validate
    schema_type.validate(&raw)?;

    Ok(FillResult { raw_yaml: raw.trim().to_string() })
}

#[derive(thiserror::Error, Debug)]
pub enum ComposerError {
    #[error("Client error: {0}")]
    Client(#[from] ClientError),
    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Schema validation failed: {0}")]
    Validation(String),
}
