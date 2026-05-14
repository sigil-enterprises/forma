use serde::{Deserialize, Serialize};
use std::path::Path;

// -- FormaConfig --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateEntry {
    pub path: String,
    pub mapping: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublishOverride {
    #[serde(default)]
    pub google_drive_folder_id: String,
    #[serde(default)]
    pub filename_prefix: String,
}

impl Default for PublishOverride {
    fn default() -> Self {
        Self {
            google_drive_folder_id: String::new(),
            filename_prefix: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormaConfig {
    #[serde(rename = "resourceType")]
    #[serde(default = "default_resource_type")]
    pub resource_type: String,
    #[serde(default = "default_content")]
    pub content: String,
    #[serde(default = "default_style")]
    pub style: String,
    #[serde(default)]
    pub templates: std::collections::HashMap<String, TemplateEntry>,
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
    #[serde(default)]
    pub publishing: PublishOverride,
}

fn default_resource_type() -> String { "FormaConfig@1".into() }
fn default_content() -> String { "content.yaml".into() }
fn default_style() -> String { "style.yaml".into() }
fn default_output_dir() -> String { "../../var/builds".into() }

impl Default for FormaConfig {
    fn default() -> Self {
        Self {
            resource_type: default_resource_type(),
            content: default_content(),
            style: default_style(),
            templates: Default::default(),
            output_dir: default_output_dir(),
            publishing: Default::default(),
        }
    }
}

impl FormaConfig {
    pub fn from_yaml(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let config: Self = serde_yaml::from_str(&contents)?;
        Ok(config)
    }

    pub fn resolve_template_path(&self, name: &str, project_root: &Path) -> std::path::PathBuf {
        let entry = self.templates.get(name).expect("template entry not found");
        project_root.join(&entry.path).canonicalize().unwrap_or_else(|_| project_root.join(&entry.path))
    }

    pub fn resolve_mapping_path(&self, name: &str, project_root: &Path) -> std::path::PathBuf {
        let entry = self.templates.get(name).expect("template entry not found");
        project_root.join(&entry.mapping).canonicalize().unwrap_or_else(|_| project_root.join(&entry.mapping))
    }

    pub fn resolve_style_path(&self, project_root: &Path) -> std::path::PathBuf {
        project_root.join(&self.style).canonicalize().unwrap_or_else(|_| project_root.join(&self.style))
    }

    pub fn resolve_content_path(&self, project_root: &Path) -> std::path::PathBuf {
        project_root.join(&self.content).canonicalize().unwrap_or_else(|_| project_root.join(&self.content))
    }

    pub fn resolve_output_dir(&self, project_root: &Path) -> std::path::PathBuf {
        project_root.join(&self.output_dir).canonicalize().unwrap_or_else(|_| project_root.join(&self.output_dir))
    }
}
