use serde_json::{json, Value};
use tera::{Tera, Context};

fn rebuild_value(v: &Value) -> Value {
    match v {
        Value::Null => Value::Null,
        Value::Bool(b) => Value::Bool(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() { return Value::Number(i.into()); }
            if let Some(u) = n.as_u64() { return Value::Number(u.into()); }
            if let Some(f) = n.as_f64() {
                return serde_json::Number::from_f64(f).map(Value::Number).unwrap_or_else(|| Value::Number(n.clone()));
            }
            Value::Number(n.clone())
        },
        Value::String(s) => Value::String(s.clone()),
        Value::Array(arr) => Value::Array(arr.iter().map(rebuild_value).collect()),
        Value::Object(obj) => {
            let mut new = serde_json::Map::new();
            for (k, val) in obj {
                new.insert(k.clone(), rebuild_value(val));
            }
            Value::Object(new)
        },
    }
}

fn main() {
    // Test 1: YAML slides + json!() wrapper (no rebuild) - from debug_t3
    let yaml_str = r#"
resourceType: SlideDocument
title: Test
slides:
  - title: S1
  - title: S2
"#;
    let yaml_doc: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
    let json_doc: serde_json::Value = serde_json::to_value(&yaml_doc).unwrap();
    
    let ctx1 = json!({
        "document": &json_doc,
        "content": &json_doc,
        "style": {},
    });
    let ctx_tera1 = Context::from_value(ctx1).unwrap();
    let mut t1 = Tera::default();
    t1.add_raw_template("t1", "{% for s in content.slides %}{{ s.title }}|{% endfor %}").unwrap();
    println!("1. yaml doc (no rebuild, json wrapper): {:?}", t1.render("t1", &ctx_tera1));

    // Test 2: Same but with rebuild_value on the final context
    let ctx2_val = json!({
        "document": &json_doc,
        "content": &json_doc,
        "style": {},
        "meta": {},
    });
    let ctx2_rebuilt = rebuild_value(&ctx2_val);
    let ctx_tera2 = Context::from_value(ctx2_rebuilt).unwrap();
    let mut t2 = Tera::default();
    t2.add_raw_template("t2", "{% for s in content.slides %}{{ s.title }}|{% endfor %}").unwrap();
    println!("2. yaml doc (with rebuild, json wrapper): {:?}", t2.render("t2", &ctx_tera2));

    // Test 3: Same but with rebuild_value on json_doc BEFORE wrapping
    let json_doc_rb = rebuild_value(&json_doc);
    let ctx3 = json!({
        "document": &json_doc_rb,
        "content": &json_doc_rb,
        "style": {},
    });
    let ctx_tera3 = Context::from_value(ctx3).unwrap();
    let mut t3 = Tera::default();
    t3.add_raw_template("t3", "{% for s in content.slides %}{{ s.title }}|{% endfor %}").unwrap();
    println!("3. yaml doc (rebuild before wrap, json wrapper): {:?}", t3.render("t3", &ctx_tera3));

    // Test 4: Round-trip via JSON string
    let json_str = serde_json::to_string(&json_doc).unwrap();
    let json_doc_rt: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let ctx4 = json!({
        "document": &json_doc_rt,
        "content": &json_doc_rt,
        "style": {},
    });
    let ctx_tera4 = Context::from_value(ctx4).unwrap();
    let mut t4 = Tera::default();
    t4.add_raw_template("t4", "{% for s in content.slides %}{{ s.title }}|{% endfor %}").unwrap();
    println!("4. yaml doc (json round-trip, json wrapper): {:?}", t4.render("t4", &ctx_tera4));

    // Test 5: Same as 1 but using full build_context pattern (with page, page_accessor)
    let ctx5_val = json!({
        "document": &json_doc,
        "content": &json_doc,
        "style": {},
        "meta": { "rendered_date": "2026-05-12", "forma_version": "0.1.0" },
        "page": { "default_currency": "USD", "slide_count": 2 },
        "page_accessor": [&json_doc],
    });
    let ctx_tera5 = Context::from_value(ctx5_val.clone()).unwrap();
    let mut t5 = Tera::default();
    t5.add_raw_template("t5", "{% for s in content.slides %}{{ s.title }}|{% endfor %}").unwrap();
    println!("5. full build_context pattern (no rebuild): {:?}", t5.render("t5", &ctx_tera5));

    // Test 6: full build_context pattern + rebuild_value
    let ctx5_rebuilt = rebuild_value(&ctx5_val);
    let ctx_tera6 = Context::from_value(ctx5_rebuilt).unwrap();
    let mut t6 = Tera::default();
    t6.add_raw_template("t6", "{% for s in content.slides %}{{ s.title }}|{% endfor %}").unwrap();
    println!("6. full build_context pattern (with rebuild): {:?}", t6.render("t6", &ctx_tera6));

    // Test 7: Use include_loader (same as failing test)
    let doc_path = std::path::PathBuf::from("/Users/tiagotaveira/dev/proj/sigil-enterprises/forma/tests/fixtures/example-client/slides.yaml");
    let doc_path_canonical = doc_path.canonicalize().unwrap();
    let doc_base_dir = doc_path_canonical.parent().unwrap();
    let doc_yaml = forma_core::include_loader::load_mapping(&doc_path_canonical, doc_base_dir).unwrap();
    let doc: serde_json::Value = serde_json::to_value(&doc_yaml).unwrap();
    
    let ctx7_val = json!({
        "document": &doc,
        "content": &doc,
        "style": {},
        "meta": {},
    });
    let ctx_tera7 = Context::from_value(ctx7_val.clone()).unwrap();
    let mut t7 = Tera::default();
    t7.add_raw_template("t7", "{% for s in content.slides %}{{ s.title }}|{% endfor %}").unwrap();
    println!("7. include_loader doc (no rebuild): {:?}", t7.render("t7", &ctx_tera7));

    let ctx7_rebuilt = rebuild_value(&ctx7_val);
    let ctx_tera7b = Context::from_value(ctx7_rebuilt).unwrap();
    let mut t7b = Tera::default();
    t7b.add_raw_template("t7b", "{% for s in content.slides %}{{ s.title }}|{% endfor %}").unwrap();
    println!("8. include_loader doc (with rebuild): {:?}", t7b.render("t7b", &ctx_tera7b));
}
