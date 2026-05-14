fn main() {
    let base = std::path::PathBuf::from(
        "/Users/tiagotaveira/dev/proj/sigil-enterprises/forma/tests/fixtures/example-client/",
    );

    // Read and parse content.yaml via include_loader's internal mechanism
    let content_path = base.join("content.yaml");
    let raw = std::fs::read_to_string(&content_path).unwrap();
    let content_yaml: serde_yaml::Value = serde_yaml::from_str(&raw).unwrap();

    // Get engagement.title via traverse (same as include_loader)
    fn traverse(data: &serde_yaml::Value, dot_path: &str) -> serde_yaml::Value {
        let mut current = data.clone();
        for part in dot_path.split('.') {
            current = match &current {
                serde_yaml::Value::Mapping(m) => m.get(part).cloned().unwrap_or(serde_yaml::Value::Null),
                serde_yaml::Value::Sequence(s) => s.first().cloned().unwrap_or(serde_yaml::Value::Null),
                _ => serde_yaml::Value::Null,
            };
        }
        current
    }

    let eng_title = traverse(&content_yaml, "engagement.title");
    let slides = traverse(&content_yaml, "slides"); // This will be Null (no slides in content.yaml)

    println!("=== engagement.title (traverse) ===");
    println!("{:?}", eng_title);
    println!("is_string: {}", eng_title.is_string());
    println!("as_str: {:?}", eng_title.as_str());

    // Convert to JSON
    let eng_title_json: serde_json::Value = serde_json::to_value(&eng_title).unwrap();
    println!("\n=== engagement.title (json) ===");
    println!("{:?}", eng_title_json);

    // Now let's trace the full slides.yaml resolution
    println!("\n=== FULL FLOW ===");
    let slides_path = base.join("slides.yaml");
    let slides_raw = std::fs::read_to_string(&slides_path).unwrap();
    let slides_yaml: serde_yaml::Value = serde_yaml::from_str(&slides_raw).unwrap();

    println!("=== slides_yaml (unresolved) ===");
    println!("{}", serde_yaml::to_string(&slides_yaml).unwrap());

    // Now resolve
    let resolved = forma_core::include_loader::load_mapping(&slides_path, &base).unwrap();
    println!("\n=== slides_yaml (resolved) ===");
    println!("{}", serde_yaml::to_string(&resolved).unwrap());

    // Compare: manually resolve title vs include_loader resolve title
    // Manual: parse slides.yaml, find title field, resolve from content.yaml
    let manual_title = match &slides_yaml {
        serde_yaml::Value::Mapping(m) => {
            m.get("title")
                .and_then(|v| v.as_str())
                .map(|s| {
                    // Extract path from !include string
                    s.trim().trim_start_matches("!include").trim()
                        .trim_matches(|c| c == '"' || c == '\'')
                })
                .and_then(|path| {
                    let resolved = traverse(&content_yaml, path);
                    resolved.as_str().map(|s| s.to_string())
                })
        }
        _ => None,
    };
    println!("\n=== Manual resolved title: {:?} ===", manual_title);

    let loader_title: serde_yaml::Value = match &resolved {
        serde_yaml::Value::Mapping(m) => m.get("title").cloned().unwrap_or(serde_yaml::Value::Null),
        _ => serde_yaml::Value::Null,
    };
    println!("=== Loader resolved title: {:?} ===", loader_title);
    println!("Manual == Loader? {}", manual_title.as_deref() == loader_title.as_str());

    // Now convert both to JSON and test Tera
    println!("\n=== TERMINAL TEST ===");

    // Approach 1: Include loader
    let doc_loader: serde_json::Value = serde_json::to_value(&resolved).unwrap();
    let ctx_loader = serde_json::json!({
        "document": &doc_loader,
        "content": &doc_loader,
        "style": {},
    });
    let c_loader = tera::Context::from_value(ctx_loader).unwrap();
    let mut t_loader = tera::Tera::default();
    t_loader.add_raw_template("t", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    let loader_result = t_loader.render("t", &c_loader);

    // Approach 2: Build SAME structure manually (no include_loader)
    // Parse content.yaml, parse slides.yaml (with !include as plain strings), build merged doc
    let slides_unresolved: serde_yaml::Value = serde_yaml::from_str(&slides_raw).unwrap();
    // Convert directly - !include stays as Tagged
    let doc_unresolved: serde_json::Value = serde_json::to_value(&slides_unresolved).unwrap();

    // Check what slides[0].title looks like
    println!("\n=== doc_unresolved.slides[0].title ===");
    println!("{:?}", doc_unresolved.get("slides").and_then(|s| s.get(0)).and_then(|s| s.get("title")));

    // The unresolved has Tagged values which become objects like {"!include": "..."}
    // This is expected to fail. But let me build a manual resolved version
    // that matches the include_loader output exactly

    // Manually build what include_loader produces
    let manual_doc = serde_json::json!({
        "resourceType": "SlideDocument",
        "title": "Digital Transformation Proposal",
        "slides": [
            {
                "type": "title",
                "title": "Digital Transformation Proposal",
                "subtitle": "Acme Corp",
            },
            {
                "type": "problem",
                "title": "Legacy Infrastructure",
                "content": "Outdated systems causing operational inefficiencies",
            },
            {
                "type": "solution",
                "title": "Cloud-Native Migration",
                "content": "Migrate to modern cloud architecture",
            },
            {
                "type": "closing",
                "logo": "https://example.com/closing-logo.png",
            },
        ],
    });

    let ctx_manual = serde_json::json!({
        "document": &manual_doc,
        "content": &manual_doc,
        "style": {},
    });
    let c_manual = tera::Context::from_value(ctx_manual).unwrap();
    let mut t_manual = tera::Tera::default();
    t_manual.add_raw_template("t", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    let manual_result = t_manual.render("t", &c_manual);

    println!("Loader result: {:?}", loader_result);
    println!("Manual json!() result: {:?}", manual_result);

    // Now the CRITICAL test: use include_loader resolved YAML, convert to JSON
    // but wrap with json!() EXACTLY like manual_doc
    println!("\n=== CRITICAL TEST ===");
    // The resolved doc has the structure from include_loader
    // Convert to JSON and check what slides look like
    let loader_json_str = serde_json::to_string(&doc_loader).unwrap();
    let doc_from_str: serde_json::Value = serde_json::from_str(&loader_json_str).unwrap();
    let ctx_critical = serde_json::json!({
        "document": &doc_from_str,
        "content": &doc_from_str,
        "style": {},
    });
    let c_critical = tera::Context::from_value(ctx_critical).unwrap();
    let mut t_critical = tera::Tera::default();
    t_critical.add_raw_template("t", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("JSON round-trip loader: {:?}", t_critical.render("t", &c_critical));

    // FINAL test: take the JSON string from include_loader, re-parse, test directly
    // (NO json!() wrapper)
    let doc_from_str_2 = doc_from_str.clone();
    let ctx_direct = tera::Context::from_value(doc_from_str).unwrap();
    let mut t_direct = tera::Tera::default();
    t_direct.add_raw_template("t", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("Direct doc_from_str: {:?}", t_direct.render("t", &ctx_direct));

    // And with json!() wrapper
    let ctx_wrapped = tera::Context::from_value(serde_json::json!({
        "document": &doc_from_str_2,
        "content": &doc_from_str_2,
    })).unwrap();
    let mut t_wrapped = tera::Tera::default();
    t_wrapped.add_raw_template("t", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("Wrapped doc_from_str: {:?}", t_wrapped.render("t", &ctx_wrapped));

    // What if we test with EXACTLY the doc_loader directly (no wrapper)?
    let ctx_doc = tera::Context::from_value(doc_loader).unwrap();
    let mut t_doc = tera::Tera::default();
    t_doc.add_raw_template("t", "{% for slide in content.slides %}{{ slide.title }}|{% endfor %}").unwrap();
    println!("Direct doc_loader: {:?}", t_doc.render("t", &ctx_doc));
}
