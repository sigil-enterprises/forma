use serde_json::json;
use std::path::PathBuf;

// Inline rebuild_value since it's private in forma_render::context
fn rebuild_value(v: &serde_json::Value) -> serde_json::Value {
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
        serde_json::Value::Array(arr) => serde_json::Value::Array(arr.iter().map(rebuild_value).collect()),
        serde_json::Value::Object(obj) => {
            let mut new = serde_json::Map::new();
            for (k, val) in obj {
                new.insert(k.clone(), rebuild_value(val));
            }
            serde_json::Value::Object(new)
        },
    }
}

fn main() {
    let slides_path = PathBuf::from(
        "/Users/tiagotaveira/dev/proj/sigil-enterprises/forma/tests/fixtures/example-client/slides.yaml",
    );
    let base_dir = slides_path.parent().unwrap();
    let doc_yaml = forma_core::include_loader::load_mapping(&slides_path, base_dir).unwrap();
    let doc: serde_json::Value = serde_json::to_value(&doc_yaml).unwrap();
    let style = serde_json::json!({});

    // YAML-sourced context via build_context
    let ctx_val = forma_render::context::build_context(&doc, &style);
    let ctx = tera::Context::from_value(ctx_val.clone()).unwrap();

    // Inline context (matching json!() construction)
    let inline_slides = json!([
        {"subtitle": "Acme Corp", "title": "Digital Transformation Proposal", "type": "title"},
        {"content": "Outdated systems causing operational inefficiencies", "title": "Legacy Infrastructure", "type": "problem"},
        {"content": "Migrate to modern cloud architecture", "title": "Cloud-Native Migration", "type": "solution"},
        {"logo": "https://example.com/closing-logo.png", "type": "closing"},
    ]);
    let inline_doc = json!({
        "resourceType": "SlideDocument",
        "title": "Digital Transformation Proposal",
        "slides": inline_slides,
    });
    let inline_ctx_val = json!({
        "document": &inline_doc,
        "content": &inline_doc,
        "style": &style,
        "meta": {
            "rendered_date": "2026-05-12",
            "forma_version": "0.1.0",
            "project_dir": "",
            "presskit_root": "",
        },
        "page": {
            "cover_client": "",
            "cover_slides": [],
            "cover_title": "",
            "cover_titles": [],
            "slide_count": 4,
            "default_currency": "USD",
        },
        "page_accessor": [&inline_doc],
    });
    let inline_ctx = tera::Context::from_value(inline_ctx_val.clone()).unwrap();

    // Compare JSON dumps
    let yaml_json_str = serde_json::to_string_pretty(&ctx_val).unwrap();
    let inline_json_str = serde_json::to_string_pretty(&inline_ctx_val).unwrap();
    println!("\n=== YAML/Inline JSON length comparison ===");
    println!("YAML ctx JSON: {} bytes", yaml_json_str.len());
    println!("Inline ctx JSON: {} bytes", inline_json_str.len());

    println!("\n=== YAML ctx slides[0] title ===");
    let y_slides = ctx_val.get("content").and_then(|c| c.get("slides")).unwrap();
    let y_title = y_slides[0].get("title").unwrap();
    println!("Value: {:?}, is_string: {}, is_number: {}, is_object: {}, is_array: {}",
        y_title, y_title.is_string(), y_title.is_number(), y_title.is_object(), y_title.is_array());

    println!("\n=== Inline ctx slides[0] title ===");
    let i_slides = inline_ctx_val.get("content").and_then(|c| c.get("slides")).unwrap();
    let i_title = i_slides[0].get("title").unwrap();
    println!("Value: {:?}, is_string: {}, is_number: {}, is_object: {}, is_array: {}",
        i_title, i_title.is_string(), i_title.is_number(), i_title.is_object(), i_title.is_array());

    // Compare the raw JSON of slides[0]
    println!("\n=== YAML slides[0] JSON ===");
    println!("{}", serde_json::to_string_pretty(&y_slides[0]).unwrap());
    println!("=== Inline slides[0] JSON ===");
    println!("{}", serde_json::to_string_pretty(&i_slides[0]).unwrap());

    // Test Tera with both contexts
    println!("\n=== TERA: YAML context ===");
    let mut t_y = tera::Tera::default();
    t_y.add_raw_template("t", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("{:?}", t_y.render("t", &ctx));

    println!("\n=== TERA: Inline context ===");
    let mut t_i = tera::Tera::default();
    t_i.add_raw_template("t", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("{:?}", t_i.render("t", &inline_ctx));

    // Test: Take the inline ctx_val, rebuild it through serde_json round-trip
    println!("\n=== TERA: Inline via JSON RT ===");
    let inline_json_str = serde_json::to_string(&inline_ctx_val).unwrap();
    let inline_rt: serde_json::Value = serde_json::from_str(&inline_json_str).unwrap();
    let inline_rt_ctx = tera::Context::from_value(inline_rt).unwrap();
    let mut t_ir = tera::Tera::default();
    t_ir.add_raw_template("t", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("{:?}", t_ir.render("t", &inline_rt_ctx));

    // Test: Take the YAML ctx_val, apply rebuild_value, then test
    println!("\n=== TERA: YAML context rebuild ===");
    let ctx_rebuilt = rebuild_value(&ctx_val);
    let ctx_rebuilt_t = tera::Context::from_value(ctx_rebuilt).unwrap();
    let mut t_rb = tera::Tera::default();
    t_rb.add_raw_template("t", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("{:?}", t_rb.render("t", &ctx_rebuilt_t));

    // Test: json!() with reference to rebuilt YAML array
    println!("\n=== TERA: json!() with &ctx_val built ===");
    let ctx_built = json!({
        "document": &ctx_val["document"],
        "content": &ctx_val["content"],
        "style": &ctx_val["style"],
    });
    let ctx_built_t = tera::Context::from_value(ctx_built).unwrap();
    let mut t_bl = tera::Tera::default();
    t_bl.add_raw_template("t", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("{:?}", t_bl.render("t", &ctx_built_t));

    // Test: json!() with clone of YAML array
    println!("\n=== TERA: json!() with cloned YAML array ===");
    let slides_clone = ctx_val["content"]["slides"].clone();
    let ctx_cloned = json!({
        "document": {"resourceType": "SlideDocument", "title": &ctx_val["document"]["title"], "slides": slides_clone},
        "content": {"resourceType": "SlideDocument", "title": &ctx_val["content"]["title"], "slides": slides_clone.clone()},
        "style": &ctx_val["style"],
    });
    let ctx_cloned_t = tera::Context::from_value(ctx_cloned).unwrap();
    let mut t_cl = tera::Tera::default();
    t_cl.add_raw_template("t", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("{:?}", t_cl.render("t", &ctx_cloned_t));

    // Test: is the issue with the raw context_val being borrowed?
    println!("\n=== TERA: clone ctx_val then use ===");
    let ctx_cloned_full = ctx_val.clone();
    let ctx_cloned_full_t = tera::Context::from_value(ctx_cloned_full).unwrap();
    let mut t_cf = tera::Tera::default();
    t_cf.add_raw_template("t", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("{:?}", t_cf.render("t", &ctx_cloned_full_t));

    // Test: parse YAML ctx JSON string fresh
    println!("\n=== TERA: YAML ctx via JSON parse ===");
    let yaml_json_str = serde_json::to_string(&ctx_val).unwrap();
    let yaml_parsed: serde_json::Value = serde_json::from_str(&yaml_json_str).unwrap();
    let yaml_parsed_t = tera::Context::from_value(yaml_parsed).unwrap();
    let mut t_wp = tera::Tera::default();
    t_wp.add_raw_template("t", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("{:?}", t_wp.render("t", &yaml_parsed_t));

    // Test: parse inline ctx JSON string fresh
    println!("\n=== TERA: Inline ctx via JSON parse ===");
    let inline_json_str2 = serde_json::to_string(&inline_ctx_val).unwrap();
    let inline_parsed: serde_json::Value = serde_json::from_str(&inline_json_str2).unwrap();
    let inline_parsed_t = tera::Context::from_value(inline_parsed).unwrap();
    let mut t_ip = tera::Tera::default();
    t_ip.add_raw_template("t", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("{:?}", t_ip.render("t", &inline_parsed_t));

    // Test: build context from scratch using the EXACT same slides array from ctx_val
    // but construct a FRESH context with json!()
    println!("\n=== TERA: fresh context with yaml slides reference ===");
    let y_slides_for_json: serde_json::Value = ctx_val["content"]["slides"].clone();
    let ctx_fresh = json!({
        "content": {"slides": &y_slides_for_json},
        "document": {},
    });
    let ctx_fresh_t = tera::Context::from_value(ctx_fresh).unwrap();
    let mut t_fr = tera::Tera::default();
    t_fr.add_raw_template("t", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("{:?}", t_fr.render("t", &ctx_fresh_t));

    // Test: inline with reference
    println!("\n=== TERA: inline slides via reference ===");
    let i_slides_for_json: serde_json::Value = inline_ctx_val["content"]["slides"].clone();
    let ctx_i_ref = json!({
        "content": {"slides": &i_slides_for_json},
        "document": {},
    });
    let ctx_i_ref_t = tera::Context::from_value(ctx_i_ref).unwrap();
    let mut t_ir2 = tera::Tera::default();
    t_ir2.add_raw_template("t", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("{:?}", t_ir2.render("t", &ctx_i_ref_t));

    // Test: what if we json!() the YAML slides[i].title directly?
    println!("\n=== TERA: direct slide titles ===");
    let t1_titles: Vec<serde_json::Value> = ctx_val["content"]["slides"]
        .as_array().unwrap()
        .iter()
        .map(|s| s.get("title").cloned().unwrap_or(json!(null)))
        .collect();
    let ctx_titles = json!({
        "titles": t1_titles,
    });
    let ctx_titles_t = tera::Context::from_value(ctx_titles).unwrap();
    let mut t_tl = tera::Tera::default();
    t_tl.add_raw_template("t", "{% for t in titles %}{{ t }}|{% endfor %}").unwrap();
    println!("{:?}", t_tl.render("t", &ctx_titles_t));

    // FINAL: inspect raw bytes of YAML slides[0] title vs inline
    println!("\n=== RAW BYTES ===");
    let y_title_bytes = serde_json::to_string(&y_title).unwrap();
    let i_title_bytes = serde_json::to_string(&i_title).unwrap();
    println!("YAML title JSON bytes: {:?}", y_title_bytes.as_bytes());
    println!("Inline title JSON bytes: {:?}", i_title_bytes.as_bytes());
    println!("YAML title as_str from serde_json: {:?}", y_title.as_str());
    println!("Inline title as_str from serde_json: {:?}", i_title.as_str());
}
