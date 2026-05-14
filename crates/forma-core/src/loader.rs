use std::path::Path;

use forma_schema::embedded;

use crate::include_loader::load_mapping;

/// Registry mapping resourceType values to schema YAML content.
pub struct SchemaRegistry;

impl SchemaRegistry {
    /// Get the schema content for a given resourceType.
    pub fn schema_content(resource_type: &str) -> Option<&'static str> {
        for (_name, content) in embedded::all() {
            if let Some(schema_type) = Self::extract_schema_type(content) {
                if schema_type == resource_type {
                    return Some(content);
                }
            }
        }
        None
    }

    fn extract_schema_type(content: &str) -> Option<String> {
        // Collect both $id and top-level title, prefer title (matches resourceType)
        let mut id: Option<String> = None;
        let mut title: Option<String> = None;
        for line in content.lines() {
            let line = line.trim();
            // Only match top-level $id (no leading whitespace)
            if line.starts_with('$') || line.starts_with("'$id'") || line.starts_with("\"$id\"") {
                if id.is_none() {
                    let val = line.splitn(2, ':').nth(1)?.trim().trim_matches(|c| c == '\'' || c == '"').to_string();
                    id = Some(val);
                }
            }
            // Only match top-level title (no leading whitespace)
            if line.starts_with("title:") && title.is_none() {
                let val = line.splitn(2, ':').nth(1)?.trim().to_string();
                if !val.is_empty() {
                    title = Some(val);
                }
            }
        }
        // Prefer title over $id — documents use PascalCase resourceType
        title.or(id)
    }
}

/// Load a YAML mapping file (slides.yaml / report.yaml), resolving !include tags.
pub fn load_document(path: &Path, base_dir: &Path) -> Result<serde_yaml::Value, crate::include_loader::IncludeError> {
    load_mapping(path, base_dir)
}

/// Load a plain content.yaml file (no !include resolution needed).
pub fn load_content(path: &Path) -> Result<serde_yaml::Value, std::io::Error> {
    if !path.exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Content file not found: {path:?}")));
    }
    let contents = std::fs::read_to_string(path)?;
    Ok(serde_yaml::from_str(&contents).unwrap_or(serde_yaml::Value::Mapping(Default::default())))
}

/// Load a style.yaml file into a plain Value.
pub fn load_style(path: &Path) -> serde_yaml::Value {
    if !path.exists() {
        return serde_yaml::Value::Mapping(Default::default());
    }
    let contents = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return serde_yaml::Value::Mapping(Default::default()),
    };
    serde_yaml::from_str(&contents).unwrap_or(serde_yaml::Value::Mapping(Default::default()))
}

/// Load the project config (forma.yaml).
pub fn load_config(path: &Path) -> Result<crate::config::FormaConfig, Box<dyn std::error::Error>> {
    crate::config::FormaConfig::from_yaml(path)
}
