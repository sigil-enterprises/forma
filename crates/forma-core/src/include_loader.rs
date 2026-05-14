use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde_yaml::{Mapping, Value};

#[derive(Debug)]
pub enum IncludeError {
    InvalidRef(String),
    FileNotFound(PathBuf),
    KeyNotFound { ref_: String, key: String, available: Vec<String> },
    IndexInvalid { ref_: String, index: usize },
    NoneTraversal { ref_: String, part: String },
}

impl std::fmt::Display for IncludeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IncludeError::InvalidRef(v) => write!(f, "!include value must start with '@', got: {v:?}"),
            IncludeError::FileNotFound(p) => write!(f, "!include referenced file not found: {}", p.display()),
            IncludeError::KeyNotFound { ref_, key, available } => {
                write!(f, "!include: key '{}' not found in {:?}\n  available keys: {:?}", key, ref_, available)
            }
            IncludeError::IndexInvalid { ref_, index } => {
                write!(f, "!include: list index '{}' is invalid in {:?}", index, ref_)
            }
            IncludeError::NoneTraversal { ref_, part } => {
                write!(f, "!include: cannot traverse '{}' — parent is None\n  full ref: {:?}", part, ref_)
            }
        }
    }
}

type Cache = HashMap<PathBuf, Value>;

fn resolve_ref(ref_: &str, base_dir: &Path, cache: &mut Cache) -> Result<Value, IncludeError> {
    // Must start with @
    if !ref_.starts_with('@') {
        return Err(IncludeError::InvalidRef(ref_.into()));
    }

    let (file_ref, dot_path) = if let Some(pos) = ref_.find(':') {
        (&ref_[..pos], Some(&ref_[pos + 1..]))
    } else {
        (ref_, None)
    };

    let file_ref = &file_ref[1..]; // strip '@'
    let file_path = base_dir.join(file_ref);

    if !file_path.exists() {
        return Err(IncludeError::FileNotFound(file_path));
    }

    let data = match cache.get(&file_path) {
        Some(cached) => cached.clone(),
        None => {
            let contents = std::fs::read_to_string(&file_path)
                .map_err(|_| IncludeError::FileNotFound(file_path.clone()))?;
            let val: Value = serde_yaml::from_str(&contents)
                .map_err(|_| IncludeError::FileNotFound(file_path.clone()))?;
            cache.insert(file_path.clone(), val.clone());
            val
        }
    };

    match dot_path {
        Some(path) => traverse(&data, path),
        None => Ok(data),
    }
}

fn traverse(data: &Value, dot_path: &str) -> Result<Value, IncludeError> {
    if dot_path.is_empty() {
        return Ok(data.clone());
    }
    let mut current = data;
    for part in dot_path.split('.') {
        let next = match current {
            Value::Null => return Err(IncludeError::NoneTraversal {
                ref_: dot_path.into(), part: part.into(),
            }),
            Value::Mapping(map) => {
                let available: Vec<String> = map.keys()
                    .filter_map(|k| k.as_str().map(|s| s.to_string()))
                    .collect();
                map.get(part).ok_or_else(|| IncludeError::KeyNotFound {
                    ref_: dot_path.into(), key: part.into(), available,
                })?
            }
            Value::Sequence(seq) => {
                let idx: usize = part.parse().map_err(|_| IncludeError::IndexInvalid {
                    ref_: dot_path.into(), index: 0,
                })?;
                seq.get(idx).ok_or_else(|| IncludeError::IndexInvalid {
                    ref_: dot_path.into(), index: idx,
                })?
            }
            _ => return Err(IncludeError::NoneTraversal {
                ref_: dot_path.into(), part: part.into(),
            }),
        };
        current = next;
    }
    Ok(current.clone())
}

fn resolve_value(val: &Value, base_dir: &Path, cache: &mut Cache) -> Result<Value, IncludeError> {
    match val {
        Value::Tagged(tagged) => {
            let tag_str = tagged.tag.to_string();
            if tag_str == "!include" || tag_str == "include" {
                let inner = &tagged.value;
                if let Value::String(ref_) = inner {
                    return resolve_ref(ref_, base_dir, cache);
                }
                return Err(IncludeError::InvalidRef(
                    format!("{:?}", inner),
                ));
            }
            // Non-!include tags: unwrap and resolve inner recursively
            let inner = resolve_value(&tagged.value, base_dir, cache)?;
            Ok(inner)
        }
        Value::String(s) => {
            if s.trim().starts_with("!include @") {
                let ref_ = s.trim().trim_start_matches("!include").trim();
                let ref_ = ref_.trim_matches(|c| c == '"' || c == '\'');
                return resolve_ref(ref_, base_dir, cache);
            }
            Ok(val.clone())
        }
        Value::Sequence(seq) => {
            let resolved: Result<Vec<Value>, IncludeError> = seq
                .iter()
                .map(|v| resolve_value(v, base_dir, cache))
                .collect();
            Ok(Value::Sequence(resolved?))
        }
        Value::Mapping(map) => {
            let resolved: Result<Mapping, IncludeError> = map
                .iter()
                .map(|(k, v)| Ok((k.clone(), resolve_value(v, base_dir, cache)?)))
                .collect();
            Ok(Value::Mapping(resolved?))
        }
        Value::Null => Ok(val.clone()),
        Value::Bool(_) | Value::Number(_) => Ok(val.clone()),
    }
}

pub fn load_mapping(mapping_path: &Path, base_dir: &Path) -> Result<Value, IncludeError> {
    if !mapping_path.exists() {
        return Err(IncludeError::FileNotFound(mapping_path.to_path_buf()));
    }

    let contents = std::fs::read_to_string(mapping_path)
        .map_err(|_| IncludeError::FileNotFound(mapping_path.to_path_buf()))?;

    let mut cache: Cache = HashMap::new();
    let parsed: Value = serde_yaml::from_str(&contents)
        .map_err(|_| IncludeError::FileNotFound(mapping_path.to_path_buf()))?;

    // Use the directory of the mapping file as the base for resolving @refs
    let file_base = mapping_path.parent().unwrap_or(base_dir);
    resolve_value(&parsed, file_base, &mut cache)
}
