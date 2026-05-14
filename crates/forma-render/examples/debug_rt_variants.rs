fn main() {
    let slides_path = std::path::PathBuf::from(
        "/Users/tiagotaveira/dev/proj/sigil-enterprises/forma/tests/fixtures/example-client/slides.yaml",
    );
    let base_dir = slides_path.parent().unwrap();
    let doc_yaml = forma_core::include_loader::load_mapping(&slides_path, base_dir).unwrap();
    let doc: serde_json::Value = serde_json::to_value(&doc_yaml).unwrap();

    // Test A: Standard path (serde_json::to_value, wrap in json!)
    println!("=== A: Standard (to_value + json!) ===");
    let ctx_a = serde_json::json!({
        "document": &doc,
        "content": &doc,
        "style": {},
        "meta": {},
    });
    let ctx_a = forma_render::context::build_context(&doc, &serde_json::json!({}));
    let c_a = tera::Context::from_value(ctx_a).unwrap();
    let mut t_a = tera::Tera::default();
    t_a.add_raw_template("t", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("{:?}", t_a.render("t", &c_a));

    // Test B: Serialize doc to YAML string, then parse back to serde_json
    println!("\n=== B: YAML round-trip ===");
    let yaml_s = serde_yaml::to_string(&doc_yaml).unwrap();
    let doc_yaml_rt: serde_yaml::Value = serde_yaml::from_str(&yaml_s).unwrap();
    let doc_rt: serde_json::Value = serde_json::to_value(&doc_yaml_rt).unwrap();
    let ctx_b = forma_render::context::build_context(&doc_rt, &serde_json::json!({}));
    let c_b = tera::Context::from_value(ctx_b).unwrap();
    let mut t_b = tera::Tera::default();
    t_b.add_raw_template("t", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("{:?}", t_b.render("t", &c_b));

    // Test C: Serialize doc to JSON string, parse back to serde_json
    println!("\n=== C: JSON round-trip ===");
    let json_s = serde_json::to_string(&doc).unwrap();
    let doc_rt2: serde_json::Value = serde_json::from_str(&json_s).unwrap();
    let ctx_c = forma_render::context::build_context(&doc_rt2, &serde_json::json!({}));
    let c_c = tera::Context::from_value(ctx_c).unwrap();
    let mut t_c = tera::Tera::default();
    t_c.add_raw_template("t", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("{:?}", t_c.render("t", &c_c));

    // Test D: Serialize doc_yaml to JSON bytes, re-parse
    println!("\n=== D: JSON bytes round-trip ===");
    let json_bytes = serde_json::to_vec(&doc).unwrap();
    let doc_rt3: serde_json::Value = serde_json::from_slice(&json_bytes).unwrap();
    let ctx_d = forma_render::context::build_context(&doc_rt3, &serde_json::json!({}));
    let c_d = tera::Context::from_value(ctx_d).unwrap();
    let mut t_d = tera::Tera::default();
    t_d.add_raw_template("t", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("{:?}", t_d.render("t", &c_d));

    // Test E: Deep clone the doc first, then build context
    println!("\n=== E: Deep clone doc before build_context ===");
    let doc_clone = doc.clone();
    let ctx_e = forma_render::context::build_context(&doc_clone, &serde_json::json!({}));
    let c_e = tera::Context::from_value(ctx_e).unwrap();
    let mut t_e = tera::Tera::default();
    t_e.add_raw_template("t", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("{:?}", t_e.render("t", &c_e));

    // Test F: Serialize context value to JSON string, re-parse
    println!("\n=== F: JSON round-trip on full context ===");
    let ctx_orig = forma_render::context::build_context(&doc, &serde_json::json!({}));
    let ctx_json = serde_json::to_string(&ctx_orig).unwrap();
    let ctx_rt: serde_json::Value = serde_json::from_str(&ctx_json).unwrap();
    let c_f = tera::Context::from_value(ctx_rt).unwrap();
    let mut t_f = tera::Tera::default();
    t_f.add_raw_template("t", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("{:?}", t_f.render("t", &c_f));

    // Test G: What if rebuild_value clones the doc first?
    println!("\n=== G: rebuild_value on doc first ===");
    // Inline rebuild_value
    fn rebuild(v: &serde_json::Value) -> serde_json::Value {
        match v {
            serde_json::Value::Null => serde_json::Value::Null,
            serde_json::Value::Bool(b) => serde_json::Value::Bool(*b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() { return serde_json::Value::Number(i.into()); }
                if let Some(u) = n.as_u64() { return serde_json::Value::Number(u.into()); }
                if let Some(f) = n.as_f64() {
                    return serde_json::Number::from_f64(f).map(serde_json::Value::Number).unwrap_or_else(|| serde_json::Value::Number(n.clone()));
                }
                n.clone().into()
            },
            serde_json::Value::String(s) => serde_json::Value::String(s.clone()),
            serde_json::Value::Array(arr) => serde_json::Value::Array(arr.iter().map(rebuild).collect()),
            serde_json::Value::Object(obj) => {
                let mut new = serde_json::Map::new();
                for (k, val) in obj {
                    new.insert(k.clone(), rebuild(val));
                }
                serde_json::Value::Object(new)
            },
        }
    }
    let doc_rebuilt = rebuild(&doc);
    let ctx_g = forma_render::context::build_context(&doc_rebuilt, &serde_json::json!({}));
    let c_g = tera::Context::from_value(ctx_g).unwrap();
    let mut t_g = tera::Tera::default();
    t_g.add_raw_template("t", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("{:?}", t_g.render("t", &c_g));
}
