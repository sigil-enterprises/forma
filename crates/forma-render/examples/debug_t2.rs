use serde_json::json;
use tera::{Tera, Context};

fn json_roundtrip(v: &serde_json::Value) -> serde_json::Value {
    let s = serde_json::to_string(v).unwrap();
    serde_json::from_str(&s).unwrap()
}

fn main() {
    // Simulate full YAML-loaded document with slides (like what slides.yaml produces)
    let yaml_str = r#"
resourceType: SlideDocument
title: Digital Transformation Proposal
slides:
  - type: title
    title: Digital Transformation Proposal
    subtitle: Acme Corp
  - type: problem
    title: Legacy Infrastructure
    content: Outdated systems causing operational inefficiencies
  - type: solution
    title: Cloud-Native Migration
    content: Migrate to modern cloud architecture
  - type: closing
    logo: https://example.com/closing-logo.png
"#;
    let yaml_val: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
    let json_val: serde_json::Value = serde_json::to_value(&yaml_val).unwrap();

    // Build context like build_context does
    let doc = json_val;
    let ctx_val = json!({
        "document": &doc,
        "content": &doc,
        "style": {},
        "meta": {
            "rendered_date": "2026-05-12",
            "forma_version": "0.1.0",
        },
    });

    // Debug: show what context looks like
    println!("=== CONTEXT STRUCTURE ===");
    println!("document keys: {:?}", ctx_val.get("document").and_then(|d| d.as_object()).map(|o| o.keys().collect::<Vec<_>>()));
    println!("document.title: {:?}", ctx_val.get("document").and_then(|d| d.as_object()).and_then(|o| o.get("title")));
    println!("slides type: {:?}", ctx_val.get("document").and_then(|d| d.as_object()).and_then(|o| o.get("slides")).and_then(|s| s.get(0)).map(|s| s.get("title")));

    // Test 1: document.title direct
    let ctx_tera1 = Context::from_value(ctx_val.clone()).unwrap();
    let mut tera1 = Tera::default();
    tera1.add_raw_template("t1", "{{ document.title }}").unwrap();
    println!("\n=== TEST 1: document.title ===");
    println!("{:?}", tera1.render("t1", &ctx_tera1));

    // Test 2: for-loop over slides
    let ctx_tera2 = Context::from_value(ctx_val.clone()).unwrap();
    let mut tera2 = Tera::default();
    tera2.add_raw_template("t2", "{% for s in content.slides %}{{ s.title }}|{% endfor %}").unwrap();
    println!("\n=== TEST 2: for-loop ===");
    println!("{:?}", tera2.render("t2", &ctx_tera2));

    // Test 3: JSON round-trip first
    let doc_rt = json_roundtrip(&doc);
    let ctx_val_rt = json!({
        "document": &doc_rt,
        "content": &doc_rt,
        "style": {},
        "meta": {},
    });
    let ctx_tera3a = Context::from_value(ctx_val_rt.clone()).unwrap();
    let mut tera3a = Tera::default();
    tera3a.add_raw_template("t3a", "{{ document.title }}").unwrap();
    println!("\n=== TEST 3: document.title (round-trip) ===");
    println!("{:?}", tera3a.render("t3a", &ctx_tera3a));
    let ctx_tera3b = Context::from_value(ctx_val_rt).unwrap();
    let mut tera3b = Tera::default();
    tera3b.add_raw_template("t3b", "{% for s in content.slides %}{{ s.title }}|{% endfor %}").unwrap();
    println!("\n=== TEST 3b: for-loop (round-trip) ===");
    println!("{:?}", tera3b.render("t3b", &ctx_tera3b));
}
