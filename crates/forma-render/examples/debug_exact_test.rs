use serde_json::json;
use std::path::PathBuf;

fn preprocess_delimiters(input: &str) -> String {
    let mut output = input.to_string();
    // (( var )) → {{ var }}
    output = output.replace("((","{{").replace("))","}}");
    // (% block %) → {% block %}
    output = output.replace("(%","{%").replace("%)","%}");
    // (# comment #) → {# comment #}
    output = output.replace("(#", "{#").replace("#)", "#}");
    output
}

fn main() {
    let slides_path = PathBuf::from(
        "/Users/tiagotaveira/dev/proj/sigil-enterprises/forma/tests/fixtures/example-client/slides.yaml",
    );
    let base_dir = slides_path.parent().unwrap();
    let doc_yaml = forma_core::include_loader::load_mapping(&slides_path, base_dir).unwrap();
    let doc: serde_json::Value = serde_json::to_value(&doc_yaml).unwrap();
    let style: serde_json::Value = json!({});

    // Build context EXACTLY as build_context does (inline, since we can't import easily)
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let forma_version = "0.1.0";
    let slides = doc.as_object().and_then(|o| o.get("slides")).unwrap_or(&serde_json::Value::Null);
    let slide_count = slides.as_array().map_or(0, |a| a.len());
    let ctx_val = json!({
        "document": &doc,
        "content": &doc,
        "style": &style,
        "meta": {
            "rendered_date": today,
            "forma_version": forma_version,
            "project_dir": "",
            "presskit_root": "",
        },
        "page": {
            "default_currency": "USD",
            "slide_count": slide_count,
        },
        "page_accessor": [&doc],
    });

    // Dump context slides[0] for inspection
    let slides_arr = ctx_val.get("content").and_then(|c| c.get("slides"));
    println!("=== content.slides type: {} ===", slides_arr.map(|s| s.to_string()).unwrap_or("None".into()));
    if let Some(arr) = slides_arr.and_then(|s| s.as_array()) {
        println!("=== slides array len: {} ===", arr.len());
        for (i, slide) in arr.iter().enumerate().take(2) {
            println!("=== slides[{}]: {} ===", i, serde_json::to_string(slide).unwrap());
            if let Some(obj) = slide.as_object() {
                for (k, v) in obj {
                    println!("  {}.{} = {:?} (is_str: {})", i, k, v, v.is_string());
                }
            }
        }
    }

    let mut ctx = tera::Context::from_value(ctx_val.clone()).expect("build context");
    ctx.insert("project_dir", "");
    ctx.insert("presskit_root", "");

    // Load template
    let template_dir = PathBuf::from(
        "/Users/tiagotaveira/dev/proj/sigil-enterprises/forma/tests/fixtures/templates/proposal-slides-html",
    );
    let main_path = template_dir.join("main.html.j2");
    let raw = std::fs::read_to_string(&main_path).unwrap();
    let processed = preprocess_delimiters(&raw);

    // Check if for-loop line is correct
    let for_lines: Vec<&str> = processed.lines()
        .filter(|l| l.contains("for") && l.contains("slide") && l.contains("content.slides"))
        .collect();
    println!("\n=== For-loop lines in template ===");
    for l in &for_lines {
        println!("  {:?}", l);
    }

    let mut tera = tera::Tera::default();
    tera.add_raw_template("main.html.j2", &processed).unwrap();

    // Try to render
    let result = tera.render("main.html.j2", &ctx);
    match &result {
        Ok(html) => {
            println!("\n=== RENDERED OK ({} chars) ===", html.len());
            // Check if title appears
            println!("Contains 'Digital Transformation Proposal': {}", html.contains("Digital Transformation Proposal"));
            println!("Contains 'Acme Corp': {}", html.contains("Acme Corp"));
        }
        Err(e) => {
            println!("\n=== RENDER ERROR ===");
            println!("{:?}", e);

            // Try minimal test: just the for-loop part
            println!("\n=== MINIMAL TEST ===");
            // Extract the for-loop template
            let minimal = "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}";
            let mut t_min = tera::Tera::default();
            t_min.add_raw_template("min", minimal).unwrap();
            let min_result = t_min.render("min", &ctx);
            println!("Minimal for-loop: {:?}", min_result);

            // Try with json!() constructed context instead
            println!("\n=== CONTROL TEST (json!() context) ===");
            let doc2 = json!({
                "resourceType": "SlideDocument",
                "title": "Test Title",
                "slides": [
                    {"type": "title", "title": "Slide 1", "subtitle": "Sub 1"},
                    {"type": "problem", "title": "Slide 2", "content": "Problem"},
                ],
            });
            let ctx2_val = json!({
                "document": &doc2,
                "content": &doc2,
                "style": {},
                "meta": {},
                "page": {"slide_count": 2},
                "page_accessor": [&doc2],
            });
            let ctx2 = tera::Context::from_value(ctx2_val).unwrap();
            let mut t2 = tera::Tera::default();
            t2.add_raw_template("min", minimal).unwrap();
            println!("json!() context for-loop: {:?}", t2.render("min", &ctx2));
        }
    }
}
