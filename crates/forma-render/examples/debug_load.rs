use regex::Regex;
use std::path::PathBuf;

fn main() {
    let doc_path = PathBuf::from("/Users/tiagotaveira/dev/proj/sigil-enterprises/forma/tests/fixtures/example-client/content.yaml");
    let doc_path_canonical = doc_path.canonicalize().unwrap();
    let fallback = PathBuf::from("/");
    let doc_base_dir = doc_path_canonical.parent().unwrap_or(fallback.as_path());

    println!("doc_path: {:?}", doc_path_canonical);
    println!("doc_base_dir: {:?}", doc_base_dir);

    let raw = std::fs::read_to_string(&doc_path_canonical).unwrap();
    println!("\n=== RAW YAML ===");
    println!("{}", raw);

    let yaml_no_include: serde_yaml::Value = serde_yaml::from_str(&raw).unwrap();
    println!("\n=== YAML PARSED (no !include) ===");
    println!("{}", serde_yaml::to_string(&yaml_no_include).unwrap());

    let yaml_includes = include_mapping(&doc_path_canonical, doc_base_dir);
    println!("\n=== YAML WITH !INCLUDE ===");
    println!("{}", serde_yaml::to_string(&yaml_includes).unwrap());

    let json_val: serde_json::Value = serde_json::to_value(&yaml_includes).unwrap();
    println!("\n=== JSON ===");
    println!("{}", serde_json::to_string_pretty(&json_val).unwrap());
}

fn include_mapping(path: &std::path::Path, base: &std::path::Path) -> serde_yaml::Value {
    let raw = std::fs::read_to_string(path).unwrap();
    let value: serde_yaml::Value = serde_yaml::from_str(&raw).unwrap();
    resolve_includes(value, path, base)
}

fn resolve_includes(value: serde_yaml::Value, _path: &std::path::Path, base: &std::path::Path) -> serde_yaml::Value {
    let re = Regex::new(r"!include\s+@([^:]+):(.+)").unwrap();

    match value {
        serde_yaml::Value::String(s) => {
            if let Some(caps) = re.captures(&s) {
                let file = caps.get(1).unwrap().as_str();
                let path_str = caps.get(2).unwrap().as_str();
                let file_path = base.join(file);
                let resolved = include_mapping(&file_path, base);
                select_path(resolved, path_str)
            } else {
                serde_yaml::Value::String(s)
            }
        }
        serde_yaml::Value::Mapping(m) => {
            let mut new = serde_yaml::Mapping::new();
            for (k, v) in m {
                new.insert(k, resolve_includes(v, _path, base));
            }
            serde_yaml::Value::Mapping(new)
        }
        serde_yaml::Value::Sequence(s) => {
            serde_yaml::Value::Sequence(s.into_iter().map(|v| resolve_includes(v, _path, base)).collect())
        }
        other => other,
    }
}

fn select_path(value: serde_yaml::Value, path: &str) -> serde_yaml::Value {
    let keys: Vec<&str> = path.split('.').collect();
    let mut current = value;
    for key in keys {
        current = match current {
            serde_yaml::Value::Mapping(m) => m.get(&serde_yaml::Value::String(key.to_string())).cloned().unwrap_or(serde_yaml::Value::Null),
            _ => serde_yaml::Value::Null,
        };
    }
    current
}
