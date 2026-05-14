use serde_json::json;
use tera::{Tera, Context};

fn main() {
    // Test A: json!() constructed slides
    let ctx_a = json!({
        "content": {
            "slides": [
                {"title": "Slide1", "type": "problem"},
                {"title": "Slide2", "type": "solution"},
            ]
        },
        "document": {},
        "style": {},
    });
    let ctx_tera_a = Context::from_value(ctx_a).unwrap();
    let mut tera_a = Tera::default();
    tera_a.add_raw_template("ta", "{% for s in content.slides %}{{ s.title }}|{% endfor %}").unwrap();
    println!("json!() slides: {:?}", tera_a.render("ta", &ctx_tera_a));

    // Test B: serde_yaml-loaded, then to_json slides
    let yaml_str = r#"
slides:
  - title: Slide1
    type: problem
  - title: Slide2
    type: solution
"#;
    let yaml_val: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
    let json_val: serde_json::Value = serde_json::to_value(&yaml_val).unwrap();
    
    let ctx_b = json!({
        "content": {"slides": &json_val["slides"]},
        "document": {},
        "style": {},
    });
    let ctx_tera_b = Context::from_value(ctx_b).unwrap();
    let mut tera_b = Tera::default();
    tera_b.add_raw_template("tb", "{% for s in content.slides %}{{ s.title }}|{% endfor %}").unwrap();
    println!("yaml slides: {:?}", tera_b.render("tb", &ctx_tera_b));

    // Test C: json!() slides embedded in a larger object
    let ctx_c = json!({
        "resourceType": "SlideDocument",
        "title": "Test Title",
        "slides": [
            {"title": "Slide1", "type": "problem"},
            {"title": "Slide2", "type": "solution"},
        ],
    });
    let ctx_tera_c = Context::from_value(json!({
        "content": &ctx_c,
        "document": &ctx_c,
        "style": {},
    })).unwrap();
    let mut tera_c = Tera::default();
    tera_c.add_raw_template("tc", "{{ content.title }}|{% for s in content.slides %}{{ s.title }}|{% endfor %}").unwrap();
    println!("json!() embedded slides: {:?}", tera_c.render("tc", &ctx_tera_c));

    // Test D: YAML slides embedded in a larger object
    let yaml_full: serde_yaml::Value = serde_yaml::from_str(r#"
resourceType: SlideDocument
title: Test Title
slides:
  - title: Slide1
    type: problem
  - title: Slide2
    type: solution
"#).unwrap();
    let json_full: serde_json::Value = serde_json::to_value(&yaml_full).unwrap();
    let ctx_tera_d = Context::from_value(json!({
        "content": &json_full,
        "document": &json_full,
        "style": {},
    })).unwrap();
    let mut tera_d = Tera::default();
    tera_d.add_raw_template("td", "{{ content.title }}|{% for s in content.slides %}{{ s.title }}|{% endfor %}").unwrap();
    println!("yaml embedded slides: {:?}", tera_d.render("td", &ctx_tera_d));

    // Test E: json!() nested object at top level
    let doc_e = json!({
        "resourceType": "SlideDocument",
        "title": "Test",
        "slides": [
            {"title": "S1"},
            {"title": "S2"},
        ],
    });
    let ctx_tera_e = Context::from_value(json!({
        "document": &doc_e,
        "content": &doc_e,
        "style": {},
    })).unwrap();
    let mut tera_e = Tera::default();
    tera_e.add_raw_template("te", "{{ document.title }}|{% for s in document.slides %}{{ s.title }}|{% endfor %}").unwrap();
    println!("json!() doc slides: {:?}", tera_e.render("te", &ctx_tera_e));

    // Test F: YAML nested object at top level
    let yaml_doc: serde_yaml::Value = serde_yaml::from_str(r#"
resourceType: SlideDocument
title: Test
slides:
  - title: S1
  - title: S2
"#).unwrap();
    let json_doc: serde_json::Value = serde_json::to_value(&yaml_doc).unwrap();
    let ctx_tera_f = Context::from_value(json!({
        "document": &json_doc,
        "content": &json_doc,
        "style": {},
    })).unwrap();
    let mut tera_f = Tera::default();
    tera_f.add_raw_template("tf", "{{ document.title }}|{% for s in document.slides %}{{ s.title }}|{% endfor %}").unwrap();
    println!("yaml doc slides: {:?}", tera_f.render("tf", &ctx_tera_f));

    // Test G: Check iteration var type with json_encode
    let ctx_tera_g = Context::from_value(json!({
        "content": &json_doc,
        "document": &json_doc,
        "style": {},
    })).unwrap();
    let mut tera_g = Tera::default();
    tera_g.add_raw_template("tg", "{% for s in document.slides %}{{ s|json_encode }}{% endfor %}").unwrap();
    println!("\nyaml doc json_encode: {:?}", tera_g.render("tg", &ctx_tera_g));
}
