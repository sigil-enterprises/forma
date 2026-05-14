use serde_json::json;

fn main() {
    // Approach A: Direct serde_yaml parse of slides.yaml (no include resolution)
    let slides_path = std::path::PathBuf::from(
        "/Users/tiagotaveira/dev/proj/sigil-enterprises/forma/tests/fixtures/example-client/slides.yaml",
    );
    let base_dir = slides_path.parent().unwrap();
    let raw = std::fs::read_to_string(&slides_path).unwrap();

    // A: Parse with serde_yaml directly (includes stay as Tagged values)
    let yaml_a: serde_yaml::Value = serde_yaml::from_str(&raw).unwrap();
    let json_a: serde_json::Value = serde_json::to_value(&yaml_a).unwrap();
    println!("=== A: direct serde_yaml parse of slides.yaml ===");
    println!("{}", serde_json::to_string_pretty(&json_a).unwrap());

    // B: Use include_loader (resolves !include tags)
    let yaml_b = forma_core::include_loader::load_mapping(&slides_path, base_dir).unwrap();
    let json_b: serde_json::Value = serde_json::to_value(&yaml_b).unwrap();
    println!("\n=== B: include_loader resolved ===");
    println!("{}", serde_json::to_string_pretty(&json_b).unwrap());

    // C: json!() wrapper around A
    let ctx_a = json!({
        "document": &json_a,
        "content": &json_a,
        "style": {},
    });
    // D: json!() wrapper around B
    let ctx_b = json!({
        "document": &json_b,
        "content": &json_b,
        "style": {},
    });

    println!("\n=== A: content.slides[0] ===");
    println!("{:?}", ctx_a.get("content").and_then(|c| c.get("slides")).and_then(|s| s.get(0)));
    println!("=== B: content.slides[0] ===");
    println!("{:?}", ctx_b.get("content").and_then(|c| c.get("slides")).and_then(|s| s.get(0)));

    // Test Tera with A
    let ctx_tera_a = tera::Context::from_value(ctx_a).unwrap();
    let mut t_a = tera::Tera::default();
    t_a.add_raw_template("t", "{% for s in content.slides %}{{ s.title }}|{% endfor %}").unwrap();
    println!("\n=== Tera with A (direct parse): {:?} ===", t_a.render("t", &ctx_tera_a));

    // Test Tera with B
    let ctx_tera_b = tera::Context::from_value(ctx_b).unwrap();
    let mut t_b = tera::Tera::default();
    t_b.add_raw_template("t", "{% for s in content.slides %}{{ s.title }}|{% endfor %}").unwrap();
    println!("=== Tera with B (include_loader): {:?} ===", t_b.render("t", &ctx_tera_b));

    // Deep check: compare internal structure of slides[0]
    if let (Some(a_slides), Some(b_slides)) = (
        json_a.get("slides").and_then(|s| s.get(0)),
        json_b.get("slides").and_then(|s| s.get(0)),
    ) {
        println!("\n=== A slides[0] is_object? {} ===", a_slides.is_object());
        println!("=== B slides[0] is_object? {} ===", b_slides.is_object());

        if let (Some(a_obj), Some(b_obj)) = (a_slides.as_object(), b_slides.as_object()) {
            println!("\n=== A slides[0] keys ===");
            for k in a_obj.keys() {
                println!("  {:?}", k);
                let v = a_obj.get(k).unwrap();
                println!("    is_string? {}, value: {:?}", v.is_string(), v);
            }
            println!("\n=== B slides[0] keys ===");
            for k in b_obj.keys() {
                println!("  {:?}", k);
                let v = b_obj.get(k).unwrap();
                println!("    is_string? {}, value: {:?}", v.is_string(), v);
            }

            // Check if keys are identical
            let a_keys: Vec<_> = a_obj.keys().collect();
            let b_keys: Vec<_> = b_obj.keys().collect();
            println!("\n=== Keys identical? {} ===", a_keys == b_keys);
        }
    }
}
