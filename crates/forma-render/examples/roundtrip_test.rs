use serde_json::json;
use tera::{Tera, Context};

fn json_roundtrip(v: &serde_json::Value) -> serde_json::Value {
    let s = serde_json::to_string(v).unwrap();
    serde_json::from_str(&s).unwrap()
}

fn main() {
    // Test 1: json!() directly
    let slides_json = json!([
        {"title": "Slide1", "type": "title"},
        {"title": "Slide2", "type": "problem"},
    ]);
    let ctx1 = json!({"content": {"slides": slides_json}});
    let ctx_tera1 = Context::from_value(ctx1).unwrap();
    let mut tera1 = Tera::default();
    tera1.add_raw_template("t1", "{% for s in content.slides %}{{ s.title }}{% endfor %}").unwrap();
    println!("json!() direct: {:?}", tera1.render("t1", &ctx_tera1));

    // Test 2: JSON round-trip of json!() values
    let slides_rt = json_roundtrip(&slides_json);
    let ctx2 = json!({"content": {"slides": slides_rt}});
    let ctx_tera2 = Context::from_value(ctx2).unwrap();
    let mut tera2 = Tera::default();
    tera2.add_raw_template("t2", "{% for s in content.slides %}{{ s.title }}{% endfor %}").unwrap();
    println!("json!() round-trip: {:?}", tera2.render("t2", &ctx_tera2));

    // Test 3: serde_yaml → serde_json → JSON round-trip
    let yaml_str = r#"
slides:
  - title: Slide1
    type: title
  - title: Slide2
    type: problem
"#;
    let yaml_val: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
    let json_val: serde_json::Value = serde_json::to_value(&yaml_val).unwrap();
    let ctx3 = json!({"content": {"slides": &json_val}});
    let ctx_tera3 = Context::from_value(ctx3).unwrap();
    let mut tera3 = Tera::default();
    tera3.add_raw_template("t3", "{% for s in content.slides %}{{ s.title }}{% endfor %}").unwrap();
    println!("yaml→json direct: {:?}", tera3.render("t3", &ctx_tera3));

    // Test 4: serde_yaml → serde_json → JSON round-trip
    let json_rt = json_roundtrip(&json_val);
    let content4 = json!({"slides": &json_rt});
    let ctx4 = json!({"content": content4});
    let ctx_tera4 = Context::from_value(ctx4).unwrap();
    let mut tera4 = Tera::default();
    tera4.add_raw_template("t4", "{% for s in content.slides %}{{ s.title }}{% endfor %}").unwrap();
    println!("yaml→json→roundtrip: {:?}", tera4.render("t4", &ctx_tera4));

    // Test 5: Raw document.title access with json!() direct
    let doc1 = json!({"title": "MyTitle", "resourceType": "TestDoc"});
    let ctx5 = json!({"document": doc1});
    let ctx_tera5 = Context::from_value(ctx5).unwrap();
    let mut tera5 = Tera::default();
    tera5.add_raw_template("t5", "{{ document.title }}").unwrap();
    println!("doc.title json!() direct: {:?}", tera5.render("t5", &ctx_tera5));

    // Test 6: Raw document.title access with YAML-sourced
    let yaml_doc: serde_yaml::Value = serde_yaml::from_str("title: MyTitle\nresourceType: TestDoc").unwrap();
    let json_doc: serde_json::Value = serde_json::to_value(&yaml_doc).unwrap();
    let ctx6 = json!({"document": &json_doc});
    let ctx_tera6 = Context::from_value(ctx6).unwrap();
    let mut tera6 = Tera::default();
    tera6.add_raw_template("t6", "{{ document.title }}").unwrap();
    println!("doc.title yaml→json: {:?}", tera6.render("t6", &ctx_tera6));
}
