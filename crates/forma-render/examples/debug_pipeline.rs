fn main() {
    // Load document via include_loader (exact same path as failing test)
    let slides_path = std::path::PathBuf::from(
        "/Users/tiagotaveira/dev/proj/sigil-enterprises/forma/tests/fixtures/example-client/slides.yaml",
    );
    let base_dir = slides_path.parent().unwrap();
    let doc_yaml = forma_core::include_loader::load_mapping(&slides_path, base_dir).unwrap();
    let doc: serde_json::Value = serde_json::to_value(&doc_yaml).unwrap();
    let style: serde_json::Value = serde_json::json!({});

    // Build context via build_context
    let ctx_val = forma_render::context::build_context(&doc, &style);

    println!("=== ctx keys ===");
    println!("{:?}", ctx_val.as_object().map(|o| o.keys().collect::<Vec<_>>()));

    println!("\n=== content.slides[0] ===");
    let slides = ctx_val.get("content").and_then(|c| c.get("slides")).and_then(|s| s.get(0));
    println!("{:?}", slides);

    // Test 1: Direct property access
    let mut tera1 = tera::Tera::default();
    tera1.add_raw_template("t1", "{{ document.title }}").unwrap();
    let ctx1 = tera::Context::from_value(ctx_val.clone()).unwrap();
    println!("\n=== T1: document.title ===");
    println!("{:?}", tera1.render("t1", &ctx1));

    // Test 2: Direct slides[0].title access (index)
    tera1.add_raw_template("t2", "{{ content.slides[0].title }}").unwrap();
    println!("=== T2: content.slides[0].title ===");
    println!("{:?}", tera1.render("t2", &ctx1));

    // Test 3: for-loop variable
    tera1.add_raw_template("t3", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("=== T3: for-loop slide.title ===");
    println!("{:?}", tera1.render("t3", &ctx1));

    // Test 4: for-loop with json_encode filter on loop var
    forma_render::register_filters(&mut tera1);
    tera1.add_raw_template("t4", "{% for slide in content.slides %}{{ slide | json_encode }}|{% endfor %}").unwrap();
    println!("=== T4: for-loop slide | json_encode ===");
    match tera1.render("t4", &ctx1) {
        Ok(r) => println!("OK: {}", r),
        Err(e) => println!("ERR: {:?}", e),
    }

    // Test 5: for-loop with json_encode then as_str
    tera1.add_raw_template("t5", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("=== T5: for-loop (after filters registered) ===");
    println!("{:?}", tera1.render("t5", &ctx1));

    // Test 6: Compare with json!() constructed data
    println!("\n=== CONTROL: json!() constructed data ===");
    let doc2 = serde_json::json!({
        "resourceType": "SlideDocument",
        "title": "Test Title",
        "slides": [
            {"type": "title", "title": "Slide 1", "subtitle": "Sub 1"},
            {"type": "problem", "title": "Slide 2", "content": "Problem"},
        ],
    });
    let ctx2_val = forma_render::context::build_context(&doc2, &serde_json::json!({}));
    let ctx2 = tera::Context::from_value(ctx2_val).unwrap();
    let mut tera2 = tera::Tera::default();
    tera2.add_raw_template("t", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("{:?}", tera2.render("t", &ctx2));

    // Test 7: Compare with YAML→json direct
    println!("\n=== CONTROL: serde_yaml → serde_json → build_context ===");
    let yaml_str = r#"
resourceType: SlideDocument
title: YAML Test
slides:
  - type: title
    title: YAML Slide 1
  - type: problem
    title: YAML Slide 2
"#;
    let yaml_val: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
    let doc3: serde_json::Value = serde_json::to_value(&yaml_val).unwrap();
    let ctx3_val = forma_render::context::build_context(&doc3, &serde_json::json!({}));
    let ctx3 = tera::Context::from_value(ctx3_val).unwrap();
    let mut tera3 = tera::Tera::default();
    tera3.add_raw_template("t", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("{:?}", tera3.render("t", &ctx3));

    // Test 8: Direct serde_yaml to_value, wrapped in json!() (no build_context)
    println!("\n=== CONTROL: YAML to_value + json!() wrapper ===");
    let doc4 = serde_json::json!({
        "document": &doc3,
        "content": &doc3,
        "style": {},
    });
    let ctx4 = tera::Context::from_value(doc4).unwrap();
    let mut tera4 = tera::Tera::default();
    tera4.add_raw_template("t", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("{:?}", tera4.render("t", &ctx4));
}
