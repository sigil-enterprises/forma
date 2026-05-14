use std::path::PathBuf;
use serde_json::json;
use tera::Context;

// Minimal include_loader logic
fn include_mapping(path: &std::path::Path, base_dir: &std::path::Path) -> serde_yaml::Value {
    let raw = std::fs::read_to_string(path).unwrap();
    let val: serde_yaml::Value = serde_yaml::from_str(&raw).unwrap();
    // No !include in content.yaml, so just return it
    val
}

fn build_context(document: &serde_json::Value, style: &serde_json::Value) -> serde_json::Value {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let forma_version = env!("CARGO_PKG_VERSION");
    let slides = document
        .as_object()
        .and_then(|o| o.get("slides"))
        .unwrap_or(&serde_json::Value::Null);
    let slide_count = slides.as_array().map_or(0, |a| a.len());
    let ctx = json!({
        "document": document,
        "content": document,
        "style": style,
        "meta": {
            "rendered_date": today,
            "forma_version": forma_version,
            "project_dir": "",
            "presskit_root": "",
        },
        "page": {
            "default_currency": "USD",
            "slide_count": slide_count,
        },
        "page_accessor": [document],
    });
    ctx
}

fn main() {
    let doc_path = PathBuf::from("/Users/tiagotaveira/dev/proj/sigil-enterprises/forma/tests/fixtures/example-client/content.yaml");
    let doc_path_canonical = doc_path.canonicalize().unwrap();
    let doc_base_dir = doc_path_canonical.parent().unwrap();
    
    let doc_yaml = include_mapping(&doc_path_canonical, doc_base_dir);
    println!("=== YAML from load_mapping ===");
    println!("resourceType: {:?}", doc_yaml.get("resourceType"));
    println!("has slides: {}", doc_yaml.get("slides").is_some());
    println!("has engagement: {}", doc_yaml.get("engagement").is_some());
    
    let doc: serde_json::Value = serde_json::to_value(&doc_yaml).unwrap();
    println!("\n=== JSON doc keys ===");
    println!("{:?}", doc.as_object().map(|o| o.keys().collect::<Vec<_>>()));
    println!("doc has slides: {}", doc.get("slides").is_some());
    println!("doc has engagement: {}", doc.get("engagement").is_some());
    println!("doc.engagement.title: {:?}", doc.get("engagement").and_then(|e| e.get("title")));
    
    let style: serde_json::Value = json!({"primary_color": "#333333"});
    
    let ctx_val = build_context(&doc, &style);
    println!("\n=== CONTEXT keys ===");
    println!("{:?}", ctx_val.as_object().map(|o| o.keys().collect::<Vec<_>>()));
    println!("\n=== content ===");
    println!("{:?}", ctx_val.get("content").and_then(|c| c.get("resourceType")));
    println!("=== document ===");
    println!("{:?}", ctx_val.get("document").and_then(|d| d.get("resourceType")));
    println!("\n=== document.slides ===");
    println!("{:?}", ctx_val.get("document").and_then(|d| d.get("slides")));
    
    // Try rendering with Tera
    let ctx = Context::from_value(ctx_val.clone()).expect("build context");
    let mut tera = tera::Tera::default();
    
    // Test simple property access
    tera.add_raw_template("t1", "{{ document.resourceType }}").unwrap();
    println!("\n=== TERA: document.resourceType ===");
    println!("{:?}", tera.render("t1", &ctx));
    
    tera.add_raw_template("t2", "{{ document.engagement.title }}").unwrap();
    println!("=== TERA: document.engagement.title ===");
    println!("{:?}", tera.render("t2", &ctx));
    
    tera.add_raw_template("t3", "{{ content.engagement.title }}").unwrap();
    println!("=== TERA: content.engagement.title ===");
    println!("{:?}", tera.render("t3", &ctx));
}
