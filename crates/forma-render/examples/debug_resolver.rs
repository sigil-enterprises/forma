use std::path::PathBuf;

fn main() {
    // Use the actual include_loader from forma-core
    let doc_path = PathBuf::from("/Users/tiagotaveira/dev/proj/sigil-enterprises/forma/tests/fixtures/example-client/slides.yaml");
    let doc_path_canonical = doc_path.canonicalize().unwrap();
    let doc_base_dir = doc_path_canonical.parent().unwrap();
    
    // We can't directly call include_loader::load_mapping from example because it's not re-exported
    // Let me check the forma_core lib.rs
    println!("Trying to use forma_core...");
    
    // Check if load_mapping is public
    match forma_core::include_loader::load_mapping(&doc_path_canonical, doc_base_dir) {
        Ok(val) => {
            let json_val: serde_json::Value = serde_json::to_value(&val).unwrap();
            println!("=== RESOLVED YAML (via include_loader) ===");
            println!("{}", serde_yaml::to_string(&val).unwrap());
            println!("\n=== AS JSON ===");
            println!("{}", serde_json::to_string_pretty(&json_val).unwrap());
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}
