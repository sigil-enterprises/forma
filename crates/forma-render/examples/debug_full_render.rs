use std::path::PathBuf;
use serde_json::json;
use tera::Context;

fn main() {
    let doc_path = PathBuf::from("/Users/tiagotaveira/dev/proj/sigil-enterprises/forma/tests/fixtures/example-client/slides.yaml");
    let doc_path_canonical = doc_path.canonicalize().unwrap();
    let doc_base_dir = doc_path_canonical.parent().unwrap();
    
    // Resolve !include tags
    let doc_yaml = forma_core::include_loader::load_mapping(&doc_path_canonical, doc_base_dir).unwrap();
    let doc: serde_json::Value = serde_json::to_value(&doc_yaml).unwrap();
    
    println!("doc.title: {:?}", doc.get("title"));
    println!("doc.slides count: {:?}", doc.get("slides").and_then(|s| s.as_array()).map(|a| a.len()));
    println!("doc.slides[0].title: {:?}", doc.get("slides").and_then(|s| s.get(0)).and_then(|s| s.get("title")));
    
    // Build context
    let style: serde_json::Value = json!({});
    let ctx_val = forma_render::build_context(&doc, &style);
    
    println!("\nctx.keys: {:?}", ctx_val.as_object().map(|o| o.keys().collect::<Vec<_>>()));
    println!("ctx.content.title: {:?}", ctx_val.get("content").and_then(|c| c.get("title")));
    println!("ctx.content.slides[0].title: {:?}", ctx_val.get("content").and_then(|c| c.get("slides")).and_then(|s| s.get(0)).and_then(|s| s.get("title")));
    println!("ctx.document.title: {:?}", ctx_val.get("document").and_then(|d| d.get("title")));
    
    // Try Tera render
    let ctx = Context::from_value(ctx_val).unwrap();
    
    // Simple test
    let mut t = tera::Tera::default();
    t.add_raw_template("t1", "{{ content.title }}").unwrap();
    println!("\n=== {{ content.title }} ===");
    println!("{:?}", t.render("t1", &ctx));
    
    t.add_raw_template("t2", "{% for s in content.slides %}{{ s.title }}|{% endfor %}").unwrap();
    println!("=== for-loop ===");
    println!("{:?}", t.render("t2", &ctx));
    
    // Now test with the actual template using make_tera approach
    let template_dir = PathBuf::from("/Users/tiagotaveira/dev/proj/sigil-enterprises/forma/tests/fixtures/templates/proposal-slides-html");
    let main_path = template_dir.join("main.html.j2");
    let raw = std::fs::read_to_string(&main_path).unwrap();
    let preprocessed = forma_render::preprocess_delimiters(&raw);
    
    println!("\n=== PREPROCESSED TEMPLATE ===");
    println!("{}", preprocessed);
    
    let mut t2 = tera::Tera::default();
    t2.add_raw_template("main.html.j2", &preprocessed).unwrap();
    println!("\n=== RENDER ===");
    match t2.render("main.html.j2", &ctx) {
        Ok(html) => {
            println!("SUCCESS! Length: {}", html.len());
            println!("{}", &html[..html.len().min(500)]);
        }
        Err(e) => {
            println!("ERROR: {:?}", e);
        }
    }
}
