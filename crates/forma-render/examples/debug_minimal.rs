use serde_json::json;

fn main() {
    // This is the EXACT context structure from test_inline_for_loop (PASSING)
    println!("=== PASSING CONTEXT STRUCTURE ===");
    let slides = json!([
        {"title": "T1", "content": "C1"},
        {"title": "T2", "content": "C2"},
    ]);
    let ctx_val = json!({
        "document": {"title": "Test", "slides": &slides},
        "content": {"title": "Test", "slides": &slides},
        "style": null,
    });
    let ctx = tera::Context::from_value(ctx_val).unwrap();
    let mut t = tera::Tera::default();
    t.add_raw_template("test", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("{:?}", t.render("test", &ctx));

    // Same but with style: {} instead of null
    println!("\n=== WITH style: {{}} ===");
    let slides2 = json!([
        {"title": "T1", "content": "C1"},
        {"title": "T2", "content": "C2"},
    ]);
    let ctx_val2 = json!({
        "document": {"title": "Test", "slides": &slides2},
        "content": {"title": "Test", "slides": &slides2},
        "style": {},
    });
    let ctx2 = tera::Context::from_value(ctx_val2).unwrap();
    let mut t2 = tera::Tera::default();
    t2.add_raw_template("test", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("{:?}", t2.render("test", &ctx2));

    // Same but with extra keys (matching YAML context shape)
    println!("\n=== WITH extra keys (meta, page, page_accessor) ===");
    let slides3 = json!([
        {"title": "T1", "content": "C1"},
        {"title": "T2", "content": "C2"},
    ]);
    let ctx_val3 = json!({
        "document": {"title": "Test", "slides": &slides3},
        "content": {"title": "Test", "slides": &slides3},
        "style": {},
        "meta": {"rendered_date": "2026-05-12", "forma_version": "0.1.0"},
        "page": {"slide_count": 2},
        "page_accessor": [{"title": "Test"}],
    });
    let ctx3 = tera::Context::from_value(ctx_val3).unwrap();
    let mut t3 = tera::Tera::default();
    t3.add_raw_template("test", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("{:?}", t3.render("test", &ctx3));

    // Same with slides having 3 keys (matching YAML slide shape)
    println!("\n=== WITH 3-key slides + extra context keys ===");
    let slides4 = json!([
        {"subtitle": "Acme", "title": "T1", "type": "title"},
        {"content": "Desc", "title": "T2", "type": "problem"},
    ]);
    let ctx_val4 = json!({
        "document": {"title": "Test", "slides": &slides4},
        "content": {"title": "Test", "slides": &slides4},
        "style": {},
        "meta": {"rendered_date": "2026-05-12", "forma_version": "0.1.0"},
        "page": {"slide_count": 2},
        "page_accessor": [{"title": "Test"}],
    });
    let ctx4 = tera::Context::from_value(ctx_val4).unwrap();
    let mut t4 = tera::Tera::default();
    t4.add_raw_template("test", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("{:?}", t4.render("test", &ctx4));

    // Same with 4-key slides (including logo for closing)
    println!("\n=== WITH 3-4 key slides (4 items) + extra context keys ===");
    let slides5 = json!([
        {"subtitle": "Acme", "title": "T1", "type": "title"},
        {"content": "Desc1", "title": "T2", "type": "problem"},
        {"content": "Desc2", "title": "T3", "type": "solution"},
        {"logo": "L", "type": "closing"},
    ]);
    let ctx_val5 = json!({
        "document": {"title": "Test", "slides": &slides5},
        "content": {"title": "Test", "slides": &slides5},
        "style": {},
        "meta": {"rendered_date": "2026-05-12", "forma_version": "0.1.0"},
        "page": {"slide_count": 4},
        "page_accessor": [{"title": "Test"}],
    });
    let ctx5 = tera::Context::from_value(ctx_val5).unwrap();
    let mut t5 = tera::Tera::default();
    t5.add_raw_template("test", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("{:?}", t5.render("test", &ctx5));

    // Same but WITHOUT "title" key on slides (closing slide has no title)
    println!("\n=== WITHOUT title on closing slide ===");
    let slides6 = json!([
        {"subtitle": "Acme", "title": "T1", "type": "title"},
        {"content": "Desc1", "title": "T2", "type": "problem"},
        {"content": "Desc2", "title": "T3", "type": "solution"},
        {"logo": "L", "type": "closing"},
    ]);
    // Remove title from closing slide
    let closing = slides6.as_array().unwrap().last().unwrap().clone();
    let slides6_clean = json!([
        slides6[0],
        slides6[1],
        slides6[2],
        closing,
    ]);
    // Actually they're the same - the closing slide just doesn't have "title"
    let ctx_val6 = json!({
        "document": {"title": "Test", "slides": &slides6},
        "content": {"title": "Test", "slides": &slides6},
        "style": {},
        "meta": {"rendered_date": "2026-05-12", "forma_version": "0.1.0"},
        "page": {"slide_count": 4},
        "page_accessor": [{"title": "Test"}],
    });
    let ctx6 = tera::Context::from_value(ctx_val6).unwrap();
    let mut t6 = tera::Tera::default();
    t6.add_raw_template("test", "{% for slide in content.slides %}{{ slide.title | default(value='') }}|{% endfor %}").unwrap();
    println!("{:?}", t6.render("test", &ctx6));

    // Now: clone the slides array FIRST, THEN use in json!() with reference
    println!("\n=== CLONED slides + reference in json!() ===");
    let slides7 = json!([
        {"subtitle": "Acme", "title": "T1", "type": "title"},
        {"content": "Desc1", "title": "T2", "type": "problem"},
        {"content": "Desc2", "title": "T3", "type": "solution"},
        {"logo": "L", "type": "closing"},
    ]);
    let slides7_clone: serde_json::Value = slides7.clone();
    let ctx_val7 = json!({
        "document": {"title": "Test", "slides": &slides7_clone},
        "content": {"title": "Test", "slides": &slides7_clone},
        "style": {},
    });
    let ctx7 = tera::Context::from_value(ctx_val7).unwrap();
    let mut t7 = tera::Tera::default();
    t7.add_raw_template("test", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("{:?}", t7.render("test", &ctx7));

    // Same but with extra context keys
    println!("\n=== CLONED slides + ref + extra context keys ===");
    let slides8 = json!([
        {"subtitle": "Acme", "title": "T1", "type": "title"},
        {"content": "Desc1", "title": "T2", "type": "problem"},
        {"content": "Desc2", "title": "T3", "type": "solution"},
        {"logo": "L", "type": "closing"},
    ]);
    let slides8_clone: serde_json::Value = slides8.clone();
    let ctx_val8 = json!({
        "document": {"title": "Test", "slides": &slides8_clone},
        "content": {"title": "Test", "slides": &slides8_clone},
        "style": {},
        "meta": {"rendered_date": "2026-05-12", "forma_version": "0.1.0"},
        "page": {"slide_count": 4},
        "page_accessor": [{"title": "Test"}],
    });
    let ctx8 = tera::Context::from_value(ctx_val8).unwrap();
    let mut t8 = tera::Tera::default();
    t8.add_raw_template("test", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("{:?}", t8.render("test", &ctx8));

    // Now test: json!() with slides constructed inside json!() (no clone)
    println!("\n=== INLINE slides directly in json!() ===");
    let ctx_val9 = json!({
        "document": {
            "title": "Test",
            "slides": [
                {"subtitle": "Acme", "title": "T1", "type": "title"},
                {"content": "Desc1", "title": "T2", "type": "problem"},
                {"content": "Desc2", "title": "T3", "type": "solution"},
                {"logo": "L", "type": "closing"},
            ]
        },
        "content": {
            "title": "Test",
            "slides": [
                {"subtitle": "Acme", "title": "T1", "type": "title"},
                {"content": "Desc1", "title": "T2", "type": "problem"},
                {"content": "Desc2", "title": "T3", "type": "solution"},
                {"logo": "L", "type": "closing"},
            ]
        },
        "style": {},
        "meta": {"rendered_date": "2026-05-12", "forma_version": "0.1.0"},
        "page": {"slide_count": 4},
        "page_accessor": [{"title": "Test"}],
    });
    let ctx9 = tera::Context::from_value(ctx_val9).unwrap();
    let mut t9 = tera::Tera::default();
    t9.add_raw_template("test", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("{:?}", t9.render("test", &ctx9));
}
