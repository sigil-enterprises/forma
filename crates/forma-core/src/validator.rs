use std::path::Path;

use jsonschema::Validator;

use crate::loader::SchemaRegistry;

#[derive(Debug)]
pub struct ValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn ok(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn add_error(&mut self, msg: String) {
        self.errors.push(msg);
    }

    pub fn add_warning(&mut self, msg: String) {
        self.warnings.push(msg);
    }

    pub fn merge(&mut self, other: Self) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

fn load_schema_content(content: &str) -> serde_json::Value {
    let yaml_val: serde_yaml::Value = serde_yaml::from_str(content).unwrap_or(serde_yaml::Value::Null);
    let mut json = yaml_to_json(&yaml_val);
    // The YAML schemas use "type" for schema type name (e.g. "ProposalContent@1"),
    // not JSON Schema type. Replace top-level non-standard "type" with "object"
    // so jsonschema can parse it properly.
    if let Some(obj) = json.as_object_mut() {
        if let Some(type_val) = obj.get("type") {
            if let Some(type_str) = type_val.as_str() {
                // If type is not a valid JSON Schema type, it's our custom type name
                if !matches!(type_str, "object" | "array" | "string" | "number" | "integer" | "boolean" | "null") {
                    obj.insert("type".to_string(), serde_json::json!("object"));
                }
            }
        }
    }
    json
}

fn yaml_to_json(val: &serde_yaml::Value) -> serde_json::Value {
    match val {
        serde_yaml::Value::Null => serde_json::Value::Null,
        serde_yaml::Value::Bool(b) => serde_json::Value::Bool(*b),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                serde_json::Value::Number(i.into())
            } else if let Some(f) = n.as_f64() {
                serde_json::Value::Number(serde_json::Number::from_f64(f).unwrap_or_else(|| serde_json::Number::from(0)))
            } else {
                serde_json::Value::Number(serde_json::Number::from(0))
            }
        }
        serde_yaml::Value::String(s) => serde_json::Value::String(s.clone()),
        serde_yaml::Value::Sequence(seq) => {
            serde_json::Value::Array(seq.iter().map(yaml_to_json).collect())
        }
        serde_yaml::Value::Mapping(map) => {
            let obj: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .filter_map(|(k, v)| {
                    k.as_str().map(|s| (s.to_string(), yaml_to_json(v)))
                })
                .collect();
            serde_json::Value::Object(obj)
        }
        serde_yaml::Value::Tagged(tagged) => yaml_to_json(&tagged.value),
    }
}

pub fn validate_document(
    doc: &serde_yaml::Value,
    schema_content: &str,
    label: &str,
) -> ValidationResult {
    let mut result = ValidationResult::new();

    let schema_json = load_schema_content(schema_content);
    let doc_json = yaml_to_json(doc);

    let validator = match Validator::new(&schema_json) {
        Ok(v) => v,
        Err(e) => {
            result.add_error(format!("Failed to parse schema for {label}: {e}"));
            return result;
        }
    };

    for error in validator.iter_errors(&doc_json) {
        let path = error.instance_path.as_str();
        result.add_error(format!("{label}: [{path}] {error}"));
    }

    result
}

pub fn validate_file(
    path: &Path,
    base_dir: Option<&Path>,
) -> ValidationResult {
    let mut result = ValidationResult::new();

    if !path.exists() {
        result.add_error(format!("File not found: {}", path.display()));
        return result;
    }

    let doc = match base_dir {
        Some(base) => {
            match crate::include_loader::load_mapping(path, base) {
                Ok(v) => v,
                Err(e) => {
                    result.add_error(format!("Failed to load {}: {e}", path.display()));
                    return result;
                }
            }
        }
        None => {
            match crate::loader::load_content(path) {
                Ok(v) => v,
                Err(e) => {
                    result.add_error(format!("Failed to load {}: {e}", path.display()));
                    return result;
                }
            }
        }
    };

    let doc_map = match &doc {
        serde_yaml::Value::Mapping(m) => m,
        _ => {
            result.add_error(format!("{}: expected a YAML mapping (dict) at root", path.display()));
            return result;
        }
    };

    let resource_type = doc_map.get("resourceType")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let schema_content = match resource_type {
        Some(rt) => SchemaRegistry::schema_content(&rt),
        None => None,
    };

    match schema_content {
        Some(content) => {
            let label = path.file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "document".to_string());
            return validate_document(&doc, content, &label);
        }
        None => {
            result.add_warning(format!("{}: no resourceType or schema found — skipping validation", path.display()));
            return result;
        }
    }
}

pub fn validate_project(project_dir: &Path) -> ValidationResult {
    let mut combined = ValidationResult::new();

    let project_path = project_dir.to_path_buf();

    let content_path = project_path.join("content.yaml");
    if content_path.exists() {
        let result = validate_file(&content_path, None);
        combined.merge(result);
    } else {
        combined.add_warning("No content.yaml found in project directory".into());
    }

    for mapping_name in &["slides.yaml", "report.yaml", "brief.yaml"] {
        let mapping_path = project_path.join(mapping_name);
        if mapping_path.exists() {
            let result = validate_file(&mapping_path, Some(&project_path));
            combined.merge(result);
        }
    }

    let forma_path = project_path.join("forma.yaml");
    if forma_path.exists() {
        let result = validate_file(&forma_path, None);
        combined.merge(result);
    }

    combined
}
