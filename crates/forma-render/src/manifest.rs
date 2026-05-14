use std::path::Path;

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("No manifest.yaml found in {0}")]
    NotFound(std::path::PathBuf),
    #[error("Failed to parse manifest.yaml: {0}")]
    Parse(#[from] serde_yaml::Error),
    #[error("Failed to read manifest.yaml: {0}")]
    IO(#[from] std::io::Error),
}

#[derive(Debug, Clone, Deserialize)]
pub struct TemplateManifest {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(default = "default_format")]
    pub format: String,
    #[serde(default = "default_engine")]
    pub engine: String,
    #[serde(default = "default_entry")]
    pub entry: String,
    #[serde(rename = "compatible_schemas", default)]
    pub compatible_schemas: Vec<String>,
}

fn default_format() -> String { "document".into() }
fn default_engine() -> String { "xelatex".into() }
fn default_entry() -> String { "main.tex.j2".into() }

impl TemplateManifest {
    pub fn from_path(template_dir: &Path) -> Result<Self, ManifestError> {
        let manifest_path = template_dir.join("manifest.yaml");

        if !manifest_path.exists() {
            return Err(ManifestError::NotFound(manifest_path));
        }

        let contents = std::fs::read_to_string(&manifest_path)?;
        let mut manifest: Self = serde_yaml::from_str(&contents)?;

        if manifest.name.is_none() {
            manifest.name = Some(template_dir.display().to_string());
        }

        Ok(manifest)
    }
}
