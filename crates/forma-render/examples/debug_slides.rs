use std::path::PathBuf;

fn main() {
    let doc_path = PathBuf::from("/Users/tiagotaveira/dev/proj/sigil-enterprises/forma/tests/fixtures/example-client/slides.yaml");
    let doc_path_canonical = doc_path.canonicalize().unwrap();
    let doc_base_dir = doc_path_canonical.parent().unwrap();
    
    // Load and resolve !include
    let contents = std::fs::read_to_string(&doc_path_canonical).unwrap();
    println!("=== RAW slides.yaml ===");
    println!("{}", contents);
    
    // Use the actual include_loader
    // But we can't call it from example... let me just parse YAML and resolve manually
    let yaml_val: serde_yaml::Value = serde_yaml::from_str(&contents).unwrap();
    println!("\n=== YAML (raw, no includes resolved) ===");
    println!("{}", serde_yaml::to_string(&yaml_val).unwrap());
    
    // Convert to JSON
    let json_val: serde_json::Value = serde_json::to_value(&yaml_val).unwrap();
    println!("\n=== JSON (raw) ===");
    println!("{}", serde_json::to_string_pretty(&json_val).unwrap());
    
    // Now resolve !include manually
    let content_path = doc_base_dir.join("content.yaml");
    let content_raw = std::fs::read_to_string(&content_path).unwrap();
    let content_yaml: serde_yaml::Value = serde_yaml::from_str(&content_raw).unwrap();
    println!("\n=== content.yaml raw ===");
    println!("{}", serde_yaml::to_string(&content_yaml).unwrap());
    
    // Check what !include values would resolve to
    println!("\n=== Would resolve ===");
    println!("@content.yaml:engagement.title = {:?}", content_yaml.get("engagement").and_then(|e| e.get("title")));
    println!("@content.yaml:client.name = {:?}", content_yaml.get("client").and_then(|c| c.get("name")));
    println!("@content.yaml:problem.title = {:?}", content_yaml.get("problem").and_then(|p| p.get("title")));
    println!("@content.yaml:problem.description = {:?}", content_yaml.get("problem").and_then(|p| p.get("description")));
    println!("@content.yaml:solution.title = {:?}", content_yaml.get("solution").and_then(|s| s.get("title")));
    println!("@content.yaml:solution.description = {:?}", content_yaml.get("solution").and_then(|s| s.get("description")));
    println!("@content.yaml:closing.logo = {:?}", content_yaml.get("closing").and_then(|c| c.get("logo")));
}
