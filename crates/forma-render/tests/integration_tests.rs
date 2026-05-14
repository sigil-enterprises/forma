use std::path::PathBuf;

use serde_json::json;

use forma_render::{tera_latex_escape, oxford_join, format_decimal, value_as_strings, preprocess_delimiters, render_template};
use forma_render::context::build_context;
use forma_core::include_loader;
use tera::Context;

#[test]
fn test_latex_escape_basics() {
    let cases: Vec<(&str, &str)> = vec![
        ("Hello & World", r"Hello \& World"),
        ("50% off", r"50\% off"),
        ("cost: $100", r"cost: \$100"),
        ("use _underscores_", r"use \_underscores\_"),
        ("#hash tag", r"\#hash tag"),
    ];
    for (input, expected) in cases {
        assert_eq!(tera_latex_escape(input), expected, "mismatch for input: {}", input);
    }
}

#[test]
fn test_latex_escape_unicode() {
    assert!(tera_latex_escape("\u{2192}").contains("rightarrow"));
    assert!(tera_latex_escape("\u{2190}").contains("leftarrow"));
    assert!(tera_latex_escape("\u{2013}").contains("--"));
    assert!(tera_latex_escape("\u{2014}").contains("---"));
    assert!(tera_latex_escape("\u{2026}").contains("ldots"));
}

#[test]
fn test_latex_escape_none() {
    assert_eq!(tera_latex_escape(""), "");
}

#[test]
fn test_format_decimal_basic() {
    assert_eq!(format_decimal(50000.0, 0), "50,000");
    assert_eq!(format_decimal(0.0, 0), "0");
    assert_eq!(format_decimal(1234.5, 2), "1,234.50");
    assert_eq!(format_decimal(1_000_000.0, 0), "1,000,000");
}

#[test]
fn test_format_decimal_no_commas_for_zero() {
    assert_eq!(format_decimal(0.0, 2), "0.00");
}

#[test]
fn test_join_oxford_single() {
    let items = vec![String::from("a")];
    assert_eq!(oxford_join(&items, "and"), "a");
}

#[test]
fn test_join_oxford_two() {
    let items = vec![String::from("a"), String::from("b")];
    assert_eq!(oxford_join(&items, "and"), "a and b");
}

#[test]
fn test_join_oxford_three() {
    let items = vec![String::from("a"), String::from("b"), String::from("c")];
    assert_eq!(oxford_join(&items, "and"), "a, b, and c");
}

#[test]
fn test_join_oxford_empty() {
    let items: Vec<String> = vec![];
    assert_eq!(oxford_join(&items, "and"), "");
}

#[test]
fn test_join_oxford_custom_conjunction() {
    let items = vec![String::from("a"), String::from("b"), String::from("c")];
    assert_eq!(oxford_join(&items, "or"), "a, b, or c");
}

#[test]
fn test_value_as_strings() {
    let arr = json!(["hello", "world", "test"]);
    let strings = value_as_strings(&arr);
    assert_eq!(strings, vec!["hello", "world", "test"]);
}

#[test]
fn test_value_as_strings_empty() {
    let arr = json!([]);
    let strings = value_as_strings(&arr);
    assert!(strings.is_empty());
}

#[test]
fn test_value_as_strings_filters_non_strings() {
    let arr = json!(["hello", 42, "world", null]);
    let strings = value_as_strings(&arr);
    assert_eq!(strings, vec!["hello", "world"]);
}

#[test]
fn test_build_context_structure() {
    let document = json!({
        "resourceType": "SlideDocument",
        "slides": [{"type": "cover", "title": "Test"}]
    });
    let style = json!({
        "colors": {"primary_dark": "#061E30"}
    });
    let ctx = build_context(&document, &style);

    assert!(ctx.as_object().unwrap().contains_key("document"));
    assert!(ctx.as_object().unwrap().contains_key("style"));
    assert!(ctx.as_object().unwrap().contains_key("meta"));
    assert!(ctx["meta"]["forma_version"].as_str().is_some());
    assert!(ctx["document"]["slides"][0]["title"] == "Test");
}

#[test]
fn test_build_context_meta_has_project_dir_blank() {
    let document = json!({"resourceType": "SlideDocument"});
    let style = json!({});
    let ctx = build_context(&document, &style);
    // project_dir and presskit_root are empty when no project_dir is passed
    assert_eq!(ctx["meta"]["project_dir"], "");
}

// --- Tera template rendering tests ---

fn make_tera(template_dir: &std::path::Path) -> tera::Tera {
    // Try loading main.html.j2 directly to get the parse error
    let main_path = template_dir.join("main.html.j2");
    if main_path.is_file() {
        let raw = std::fs::read_to_string(&main_path).ok();
        if let Some(contents) = raw {
            let processed = preprocess_delimiters(&contents);
            // Dump full preprocessed output for debugging
            std::fs::write("/tmp/main_preprocessed.txt", &processed).ok();
            let mut test_tera = tera::Tera::default();
            let err = test_tera.add_raw_template("test_main", &processed);
            use std::fmt::Write;
            let mut err_detail = String::new();
            write!(&mut err_detail, "{:?}", err).unwrap();
            eprintln!("DEBUG main.html.j2 full error: {}", err_detail);
            std::fs::write("/tmp/tera_error.txt", &err_detail).ok();
            if let Err(ref e) = err {
                eprintln!("DEBUG main.html.j2 chain: {e}");
            }
            // Also dump the problematic lines
            let lines: Vec<&str> = processed.split('\n').collect();
            if lines.len() >= 760 {
                for i in 748..=760.min(lines.len()-1) {
                    eprintln!("DEBUG line[{}]: {:?}", i, lines[i]);
                }
            }
        }
    }

    let mut builder = tera::Tera::default();
    let mut paths: Vec<std::path::PathBuf> = vec![template_dir.to_path_buf()];
    let partials_dir = template_dir.join("_partials");
    let slides_dir = template_dir.join("_slides");
    if partials_dir.is_dir() {
        paths.push(partials_dir);
    }
    if slides_dir.is_dir() {
        paths.push(slides_dir);
    }

    for search_path in &paths {
        if !search_path.is_dir() { continue; }
        eprintln!("DEBUG: search_path={:?}", search_path);
        let dir_name = search_path.file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        if let Ok(entries) = std::fs::read_dir(search_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                eprintln!("DEBUG: file entry {:?} is_file={}", path.file_name(), path.is_file());
                if path.is_file() {
                    let full_key = path.to_string_lossy().to_string();
                    let rel_key = if dir_name.is_empty() {
                        path.file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default()
                    } else {
                        format!("{}/{}", dir_name, path.file_name().map(|s| s.to_string_lossy()).unwrap_or_default())
                    };
                    let is_template = path.extension()
                        .map(|e| e == "j2" || e == "tera" || e == "html" || e == "tex")
                        .unwrap_or(false);
                    if is_template {
                        if let Ok(contents) = std::fs::read_to_string(&path) {
                            let processed = preprocess_delimiters(&contents);
                            std::fs::write(&format!("/tmp/raw_{}", path.file_name().unwrap().to_string_lossy()), &contents).ok();
                            // Write processed output for tex files to debug
                            if path.extension().map_or(false, |e| e == "tex" || e == "j2") {
                                std::fs::write("/tmp/main_tex_preprocessed.txt", &processed).ok();
                                eprintln!("DEBUG: preprocess {} : {} -> {} bytes",
                                    path.file_name().unwrap().to_string_lossy(), contents.len(), processed.len());
                                // Dump first 200 and last 200 chars for encoding check
                                let pre: Vec<char> = processed.chars().collect();
                                let first_ch = pre.first().map(|c| *c as u32);
                                let last_ch = pre.last().map(|c| *c as u32);
                                eprintln!("DEBUG: first_char codepoint={:?} last_char codepoint={:?}", first_ch, last_ch);
                            }
                            eprintln!("DEBUG: preprocess {} : {} -> {} bytes, first 100: {:?}",
                                path.file_name().unwrap().to_string_lossy(), contents.len(), processed.len(),
                                &processed[..processed.len().min(100)]);
                            let bare = path.file_name()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_default();
                            // Priority: bare > rel_key > full_key
                            if !bare.is_empty() && builder.get_template(&bare).is_err() {
                                let r = builder.add_raw_template(&bare, &processed);
                                if let Err(ref e) = r {
                                    eprintln!("DEBUG: add_raw_template(bare) ERR for {:?}: {}", path.file_name(), e);
                                    std::fs::write(&format!("/tmp/preprocessed_bare_{}", bare), &processed).ok();
                                    std::fs::write("/tmp/tera_bare_err_detail.txt", &format!("{:?}", e)).ok();
                                }
                            }
                            if !rel_key.is_empty() && builder.get_template(&rel_key).is_err() {
                                let r = builder.add_raw_template(&rel_key, &processed);
                                if let Err(ref e) = r {
                                    eprintln!("DEBUG: add_raw_template(rel) ERR for {:?}: {}", path.file_name(), e);
                                    std::fs::write("/tmp/tera_rel_err_detail.txt", &format!("{:?}", e)).ok();
                                }
                            }
                            if builder.get_template(&full_key).is_err() {
                                let r = builder.add_raw_template(&full_key, &processed);
                                if let Err(ref e) = r {
                                    eprintln!("DEBUG: add_raw_template(full) ERR for {:?}: {}", path.file_name(), e);
                                    std::fs::write("/tmp/tera_full_err_detail.txt", &format!("{:?}", e)).ok();
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    forma_render::register_filters(&mut builder);
    builder
}

#[test]
fn test_preprocess_pipeline_preserves_for() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .join("tests").join("fixtures")
        .join("templates").join("proposal-slides-html")
        .join("main.html.j2");
    let input = std::fs::read_to_string(&path).unwrap();
    let processed = preprocess_delimiters(&input);

    // Always write for debugging
    std::fs::write("/tmp/full_preprocessed.txt", &processed).ok();

    // These lines MUST be present
    eprintln!("Processed has {} chars, {} lines", processed.len(), processed.lines().count());
    for (i, line) in processed.lines().enumerate().take(40) {
        eprintln!("proc[{}]: {}", i, line);
    }
    assert!(processed.contains("set_global ns_count = 0"), "Missing: namespace init");
    assert!(processed.contains("for slide in content.slides"), "Missing: for loop");
    assert!(processed.contains("set_global ns_count = ns_count + 1"), "Missing: namespace increment");
    assert!(!processed.contains("ns.count"), "Should not contain ns. references after conversion");
}

#[test]
fn test_preprocess_debug() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .join("tests").join("fixtures")
        .join("templates").join("proposal-slides-html")
        .join("_slides").join("differentiators.html.j2");
    let contents = std::fs::read_to_string(&path).unwrap();
    let processed = preprocess_delimiters(&contents);
    // Write to file to bypass RTK filtering
    std::fs::write("/tmp/processed.txt", &processed).ok();
    // Try to create a Tera with just this template
    let mut builder = tera::Tera::default();
    let r = builder.add_raw_template("test", &processed);
    match r {
        Err(e) => { std::fs::write("/tmp/tera_error.txt", &e.to_string()).ok(); }
        Ok(()) => { std::fs::write("/tmp/tera_success.txt", "ok").ok(); }
    };
}

#[test]
fn test_html_template_renders() {
    let template_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests")
        .join("fixtures")
        .join("templates")
        .join("proposal-slides-html");
    if !template_dir.exists() {
        eprintln!("SKIP: proposal-slides-html template not found");
        return;
    }

    let doc_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests")
        .join("fixtures")
        .join("example-client")
        .join("slides.yaml");

    let doc_path_canonical = doc_path.canonicalize().unwrap_or(doc_path.clone());
    let fallback = PathBuf::from("/");
    let doc_base_dir = doc_path_canonical.parent().unwrap_or(fallback.as_path());
    let doc_yaml: serde_yaml::Value = include_loader::load_mapping(&doc_path_canonical, doc_base_dir)
        .map_err(|e| std::panic!("Failed to load document: {}", e))
        .unwrap_or_default();
    let doc: serde_json::Value = serde_json::to_value(&doc_yaml).unwrap_or(json!({}));
    let style_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests")
        .join("fixtures")
        .join("templates")
        .join("style.yaml");
    let style_yaml = std::fs::read_to_string(&style_path).unwrap_or_default();
    let style: serde_json::Value = serde_yaml::from_str(&style_yaml).unwrap_or(json!({}));

    let ctx_val = build_context(&doc, &style);
    std::fs::write("/tmp/html_ctx_debug.txt", &serde_json::to_string_pretty(&ctx_val).unwrap()).ok();
    let mut ctx = Context::from_value(ctx_val.clone()).expect("build context");
    ctx.insert("project_dir", "");
    ctx.insert("presskit_root", "");

    let tera = make_tera(&template_dir);
    std::fs::write("/tmp/html_templates.txt", &format!("{:?}", tera.get_template_names().collect::<Vec<_>>())).ok();
    let tmpl = tera.get_template("main.html.j2");
    std::fs::write("/tmp/html_template_ok.txt", &format!("{:?}", tmpl.is_ok())).ok();
    // Debug: try rendering the preprocessed template directly
    let main_path = template_dir.join("main.html.j2");
    if main_path.is_file() {
        let raw = std::fs::read_to_string(&main_path).ok();
        if let Some(contents) = raw {
            let preprocessed = preprocess_delimiters(&contents);
            std::fs::write("/tmp/direct_preprocessed.txt", &preprocessed).ok();
            let mut direct_tera = tera::Tera::default();
            let parse_r = direct_tera.add_raw_template("main.html.j2", &preprocessed);
            std::fs::write("/tmp/direct_parse.txt", &format!("{:?}", parse_r)).ok();
            if parse_r.is_ok() {
                // Write context for debugging
                let ctx_debug = serde_json::to_string_pretty(&ctx_val).ok();
                std::fs::write("/tmp/direct_ctx.txt", &ctx_debug.unwrap_or_default()).ok();
                let render_r = direct_tera.render("main.html.j2", &ctx);
                std::fs::write("/tmp/direct_render.txt", &format!("{:?}", render_r)).ok();
            }
        }
    }
    let html = tera.render("main.html.j2", &ctx);

    match html {
        Ok(rendered) => {
            assert!(rendered.contains("<!DOCTYPE html>"), "Expected DOCTYPE in rendered HTML");
            assert!(rendered.contains("Digital Transformation Proposal"), "Expected title in rendered HTML");
            assert!(rendered.contains("Acme Corp"), "Expected company name in rendered HTML");
        }
        Err(e) => {
            std::fs::write("/tmp/render_error.txt", &format!("{:?}", e)).ok();
            std::panic!("Template rendering failed: {:?}", e);
        }
    }
}

#[test]
fn test_report_template_renders() {
    let template_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests")
        .join("fixtures")
        .join("templates")
        .join("proposal-report");
    if !template_dir.exists() {
        eprintln!("SKIP: proposal-report template not found");
        return;
    }

    let doc_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests")
        .join("fixtures")
        .join("example-client")
        .join("report.yaml");

    let doc_path_canonical = doc_path.canonicalize().unwrap_or(doc_path.clone());
    let fallback = PathBuf::from("/");
    let doc_base_dir = doc_path_canonical.parent().unwrap_or(fallback.as_path());
    let doc_yaml: serde_yaml::Value = include_loader::load_mapping(&doc_path_canonical, doc_base_dir)
        .unwrap_or_default();
    let doc: serde_json::Value = serde_json::to_value(&doc_yaml).unwrap_or(json!({}));
    std::fs::write("/tmp/report_raw_doc.txt", &serde_json::to_string_pretty(&doc).unwrap_or_default()).ok();
    let style_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests")
        .join("fixtures")
        .join("templates")
        .join("style.yaml");
    let style_yaml = std::fs::read_to_string(&style_path).unwrap_or_default();
    let style_yaml_val: serde_yaml::Value = serde_yaml::from_str(&style_yaml).unwrap_or_default();
    let style: serde_json::Value = serde_json::to_value(&style_yaml_val).unwrap_or(json!({}));

    let ctx_val = build_context(&doc, &style);
    std::fs::write("/tmp/report_ctx_val.txt", &serde_json::to_string_pretty(&ctx_val).unwrap_or_default()).ok();
    let mut ctx = Context::from_value(ctx_val.clone()).expect("build context");
    ctx.insert("project_dir", "");
    ctx.insert("presskit_root", "");

    let tera = make_tera(&template_dir);
    std::fs::write("/tmp/report_available_templates.txt", &format!("{:?}", tera.get_template_names().collect::<Vec<_>>())).ok();

    // Direct test: parse preprocessed content with fresh Tera
    let raw = std::fs::read_to_string(template_dir.join("main.tex.j2")).ok();
    if let Some(r) = raw {
        let pre = preprocess_delimiters(&r);
        std::fs::write("/tmp/preprocessed_main_tex.txt", &pre).ok();
        let mut direct_tera = tera::Tera::default();
        let parse_err = direct_tera.add_raw_template("main.tex.j2", &pre);
        std::fs::write("/tmp/direct_parse_result.txt", &format!("{:?}", parse_err)).ok();
    }

    let tex = tera.render("main.tex.j2", &ctx);

    match tex {
        Ok(rendered) => {
            assert!(rendered.contains(r"\documentclass"), "Expected \\documentclass in rendered LaTeX");
            assert!(rendered.contains("Digital Transformation Proposal"), "Expected title in rendered LaTeX");
        }
        Err(e) => {
            std::fs::write("/tmp/report_full_error.txt", &format!("{:?}", e)).ok();
            panic!("Template rendering failed: {}", e);
        }
    }
}

// --- filter unit tests (direct — bypasses Tera arg parsing) ---

use std::collections::HashMap;

use forma_render::filters::{
    format_date_filter, currency_filter, hex_color_filter, bullet_list_filter,
    default_filter, selectattr_filter, map_filter, first_filter, escape_filter,
};

fn no_args() -> HashMap<String, tera::Value> {
    HashMap::new()
}

#[test]
fn test_format_date_filter_iso() {
    let val = serde_json::json!("2025-06-15").into();
    let result = format_date_filter(&val, &no_args()).unwrap();
    assert!(result.as_str().unwrap().contains("June 15, 2025"));
}

#[test]
fn test_format_date_filter_custom_fmt() {
    let val = serde_json::json!("2025-06-15").into();
    let mut args = no_args();
    args.insert("fmt".into(), "%m/%d/%Y".into());
    let result = format_date_filter(&val, &args).unwrap();
    assert_eq!(result.as_str().unwrap(), "06/15/2025");
}

#[test]
fn test_format_date_filter_slash_format() {
    let val = serde_json::json!("15/06/2025").into();
    let result = format_date_filter(&val, &no_args()).unwrap();
    assert!(result.as_str().unwrap().contains("June 15, 2025"));
}

#[test]
fn test_currency_filter_default() {
    let val = serde_json::json!(1234).into();
    let result = currency_filter(&val, &no_args()).unwrap();
    assert_eq!(result.as_str().unwrap(), "$1,234");
}

#[test]
fn test_currency_filter_brl() {
    let val = serde_json::json!(99).into();
    let mut args = no_args();
    args.insert("symbol".into(), "R$".into());
    let result = currency_filter(&val, &args).unwrap();
    assert_eq!(result.as_str().unwrap(), "R$99");
}

#[test]
fn test_currency_filter_decimals() {
    let val = serde_json::json!(99).into();
    let mut args = no_args();
    args.insert("decimals".into(), 2.into());
    let result = currency_filter(&val, &args).unwrap();
    assert_eq!(result.as_str().unwrap(), "$99.00");
}

#[test]
fn test_hex_color_filter_strips_hash() {
    let val = serde_json::json!("#061E30").into();
    let result = hex_color_filter(&val, &no_args()).unwrap();
    assert_eq!(result.as_str().unwrap(), "061E30");
}

#[test]
fn test_hex_color_filter_no_hash() {
    let val = serde_json::json!("FF00AA").into();
    let result = hex_color_filter(&val, &no_args()).unwrap();
    assert_eq!(result.as_str().unwrap(), "FF00AA");
}

#[test]
fn test_bullet_list_filter_basic() {
    let val = serde_json::json!(["alpha", "beta", "gamma"]).into();
    let result = bullet_list_filter(&val, &no_args()).unwrap();
    let s = result.as_str().unwrap();
    assert!(s.contains(r"\begin{itemize}"));
    assert!(s.contains(r"\item alpha"));
    assert!(s.contains(r"\item beta"));
    assert!(s.contains(r"\end{itemize}"));
}

#[test]
fn test_bullet_list_filter_empty() {
    let val = serde_json::json!([]).into();
    let result = bullet_list_filter(&val, &no_args()).unwrap();
    assert_eq!(result.as_str().unwrap(), "");
}

#[test]
fn test_default_filter_null() {
    let val = serde_json::json!(null).into();
    let mut args = no_args();
    args.insert("value".into(), "fallback".into());
    let result = default_filter(&val, &args).unwrap();
    assert_eq!(result.as_str().unwrap(), "fallback");
}

#[test]
fn test_default_filter_empty_string() {
    let val = serde_json::json!("").into();
    let mut args = no_args();
    args.insert("value".into(), "fallback".into());
    let result = default_filter(&val, &args).unwrap();
    assert_eq!(result.as_str().unwrap(), "fallback");
}

#[test]
fn test_default_filter_nonempty() {
    let val = serde_json::json!("real").into();
    let mut args = no_args();
    args.insert("value".into(), "fallback".into());
    let result = default_filter(&val, &args).unwrap();
    assert_eq!(result.as_str().unwrap(), "real");
}

#[test]
fn test_selectattr_filter_basic() {
    let val = serde_json::json!([
        {"name": "a", "active": true},
        {"name": "b", "active": false},
        {"name": "c", "active": true}
    ]).into();
    let mut args = no_args();
    args.insert("attribute".into(), "active".into());
    args.insert("value".into(), serde_json::Value::Bool(true));
    let result = selectattr_filter(&val, &args).unwrap();
    let arr = result.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["name"].as_str().unwrap(), "a");
    assert_eq!(arr[1]["name"].as_str().unwrap(), "c");
}

#[test]
fn test_map_filter_basic() {
    let val = serde_json::json!([
        {"title": "first"},
        {"title": "second"}
    ]).into();
    let mut args = no_args();
    args.insert("attribute".into(), "title".into());
    let result = map_filter(&val, &args).unwrap();
    let arr = result.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0].as_str().unwrap(), "first");
    assert_eq!(arr[1].as_str().unwrap(), "second");
}

#[test]
fn test_first_filter_basic() {
    let val = serde_json::json!(["x", "y", "z"]).into();
    let result = first_filter(&val, &no_args()).unwrap();
    assert_eq!(result.as_str().unwrap(), "x");
}

#[test]
fn test_first_filter_empty() {
    let val = serde_json::json!([]).into();
    let result = first_filter(&val, &no_args()).unwrap();
    assert!(result.is_null());
}

#[test]
fn test_escape_filter_basic() {
    let val = serde_json::json!("<script>alert('xss')</script>").into();
    let result = escape_filter(&val, &no_args()).unwrap();
    let s = result.as_str().unwrap();
    assert!(s.contains("&lt;script&gt;"));
    assert!(!s.contains("<script>"));
}

#[test]
fn test_escape_filter_alias() {
    let val = serde_json::json!("& &amp;").into();
    let result = escape_filter(&val, &no_args()).unwrap();
    assert!(result.as_str().unwrap().contains("&amp;amp;"));
}

#[test]
fn test_bullet_list_filter_none() {
    let val = serde_json::json!(null).into();
    let result = bullet_list_filter(&val, &no_args()).unwrap();
    assert_eq!(result.as_str().unwrap(), "");
}

#[test]
fn test_format_date_filter_none() {
    let val = serde_json::json!(null).into();
    let result = format_date_filter(&val, &no_args()).unwrap();
    assert_eq!(result.as_str().unwrap(), "");
}

#[test]
fn test_format_date_filter_bad_date() {
    let val = serde_json::json!("not-a-date").into();
    let result = format_date_filter(&val, &no_args()).unwrap();
    assert_eq!(result.as_str().unwrap(), "not-a-date");
}

#[test]
fn test_inline_for_loop() {
    use forma_render::filters::register_filters;
    let tmpl = r#"<!DOCTYPE html>
<html><body>
{% set ns_count = 0 %}
{% for slide in content.slides %}
{% set ns_count = ns_count + 1 %}
<div>{{ slide.title }}</div>
{% endfor %}
</body></html>"#;
    let mut t = tera::Tera::default();
    register_filters(&mut t);
    assert!(t.add_raw_template("test", tmpl).is_ok(), "should parse");
    let slides = json!([
        {"title": "T1", "content": "C1"},
        {"title": "T2", "content": "C2"},
    ]);
    let ctx_val = json!({
        "document": {"title": "Test", "slides": &slides},
        "content": {"title": "Test", "slides": &slides},
        "style": null,
    });
    let ctx = Context::from_value(ctx_val).unwrap();
    let result = t.render("test", &ctx);
    std::fs::write("/tmp/inline_for_result.txt", &format!("{:?}", result)).ok();
    assert!(result.is_ok(), "should render: {:?}", result);
}

#[test]
fn test_disk_template_exact_conditions() {
    // Exact conditions of test_html_template_renders minus the make_tera multi-template load
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .join("tests").join("fixtures")
        .join("templates").join("proposal-slides-html")
        .join("main.html.j2");
    let raw = std::fs::read_to_string(&path).unwrap();
    let preprocessed = preprocess_delimiters(&raw);

    let doc_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .join("tests").join("fixtures")
        .join("example-client")
        .join("slides.yaml");
    let doc_path_canonical = doc_path.canonicalize().unwrap_or(doc_path.clone());
    let fallback = PathBuf::from("/");
    let doc_base_dir = doc_path_canonical.parent().unwrap_or(fallback.as_path());
    let doc_yaml: serde_yaml::Value = include_loader::load_mapping(&doc_path_canonical, doc_base_dir)
        .unwrap_or_default();
    let doc: serde_json::Value = serde_json::to_value(&doc_yaml).unwrap_or(json!({}));
    let style_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .join("tests").join("fixtures")
        .join("templates")
        .join("style.yaml");
    let style_yaml = std::fs::read_to_string(&style_path).unwrap_or_default();
    let style: serde_json::Value = serde_yaml::from_str(&style_yaml).unwrap_or(json!({}));

    let ctx_val = build_context(&doc, &style);
    let ctx = Context::from_value(ctx_val.clone()).expect("build context");

    // Write both contexts for comparison
    let inline_ctx = serde_json::to_string_pretty(&json!({
        "document": {"resourceType": "SlideDocument", "slides": json!([{"title": "T1", "content": "C1"}, {"title": "T2", "content": null}])},
        "content": {"resourceType": "SlideDocument", "slides": json!([{"title": "T1", "content": "C1"}, {"title": "T2", "content": null}])},
        "style": null,
    })).unwrap_or_default();
    std::fs::write("/tmp/inline_ctx.txt", inline_ctx).ok();
    let disk_ctx = serde_json::to_string_pretty(&ctx_val).unwrap_or_default();
    std::fs::write("/tmp/disk_ctx.txt", disk_ctx).ok();

    // Test 1: single raw template (like inline test but with disk content)
    let mut t1 = tera::Tera::default();
    t1.add_raw_template("main.html.j2", &preprocessed).unwrap();
    let r1 = t1.render("main.html.j2", &ctx);
    std::fs::write("/tmp/disk_single.txt", &format!("{:?}", r1)).ok();
    assert!(r1.is_ok(), "disk single should render: {:?}", r1);

    // Test 2: with filters registered (like make_tera)
    let mut t2 = tera::Tera::default();
    forma_render::register_filters(&mut t2);
    t2.add_raw_template("main.html.j2", &preprocessed).unwrap();
    let r2 = t2.render("main.html.j2", &ctx);
    std::fs::write("/tmp/disk_with_filters.txt", &format!("{:?}", r2)).ok();
    assert!(r2.is_ok(), "disk with filters should render: {:?}", r2);
}

#[test]
fn test_multi_template_load_no_interference() {
    // Same as make_tera - load multiple templates then render
    let template_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .join("tests").join("fixtures")
        .join("templates").join("proposal-slides-html");

    let doc_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .join("tests").join("fixtures")
        .join("example-client")
        .join("slides.yaml");
    let doc_path_canonical = doc_path.canonicalize().unwrap_or(doc_path.clone());
    let fallback = PathBuf::from("/");
    let doc_base_dir = doc_path_canonical.parent().unwrap_or(fallback.as_path());
    let doc_yaml: serde_yaml::Value = include_loader::load_mapping(&doc_path_canonical, doc_base_dir)
        .unwrap_or_default();
    let doc: serde_json::Value = serde_json::to_value(&doc_yaml).unwrap_or(json!({}));
    let style_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .join("tests").join("fixtures")
        .join("templates")
        .join("style.yaml");
    let style_yaml = std::fs::read_to_string(&style_path).unwrap_or_default();
    let style: serde_json::Value = serde_yaml::from_str(&style_yaml).unwrap_or(json!({}));

    let ctx_val = build_context(&doc, &style);
    let ctx = Context::from_value(ctx_val).expect("build context");

    let mut builder = tera::Tera::default();
    let mut paths = vec![template_dir.clone()];
    let partials_dir = template_dir.join("_partials");
    let slides_dir = template_dir.join("_slides");
    if partials_dir.is_dir() { paths.push(partials_dir); }
    if slides_dir.is_dir() { paths.push(slides_dir); }

    for search_path in &paths {
        if !search_path.is_dir() { continue; }
        let dir_name = search_path.file_name()
            .map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        if let Ok(entries) = std::fs::read_dir(search_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let is_template = path.extension()
                        .map(|e| e == "j2" || e == "tera" || e == "html" || e == "tex")
                        .unwrap_or(false);
                    if is_template {
                        if let Ok(contents) = std::fs::read_to_string(&path) {
                            let processed = preprocess_delimiters(&contents);
                            let bare = path.file_name()
                                .map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                            if !bare.is_empty() && builder.get_template(&bare).is_err() {
                                let _ = builder.add_raw_template(&bare, &processed);
                            }
                        }
                    }
                }
            }
        }
    }
    forma_render::register_filters(&mut builder);

    let names: Vec<_> = builder.get_template_names().collect();
    std::fs::write("/tmp/multi_templates.txt", &format!("{:?}", names)).ok();

    let r = builder.render("main.html.j2", &ctx);
    std::fs::write("/tmp/multi_template_result.txt", &format!("{:?}", r)).ok();
    assert!(r.is_ok(), "multi-template should render: {:?}", r);
}

#[test]
fn test_minimal_for_from_yaml() {
    // Load exact YAML context used by failing tests
    let doc_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .join("tests").join("fixtures")
        .join("example-client")
        .join("slides.yaml");
    let doc_path_canonical = doc_path.canonicalize().unwrap_or(doc_path.clone());
    let fallback = PathBuf::from("/");
    let doc_base_dir = doc_path_canonical.parent().unwrap_or(fallback.as_path());
    let doc_yaml: serde_yaml::Value = include_loader::load_mapping(&doc_path_canonical, doc_base_dir)
        .unwrap_or_default();
    let doc: serde_json::Value = serde_json::to_value(&doc_yaml).unwrap_or(json!({}));
    let ctx_val = build_context(&doc, &serde_json::json!({}));

    // Write context BEFORE rendering
    let ctx_dump = serde_json::to_string_pretty(&ctx_val).unwrap_or_default();
    std::fs::write("/tmp/min_yaml_ctx.txt", &ctx_dump).ok();

    // Inline comparison context (same as test_inline_for_loop)
    let slides = json!([
        {"title": "T1", "content": "C1"},
        {"title": "T2", "content": "C2"},
    ]);
    let inline_ctx_val = json!({
        "document": {"title": "Test", "slides": &slides},
        "content": {"title": "Test", "slides": &slides},
        "style": null,
    });
    let inline_dump = serde_json::to_string_pretty(&inline_ctx_val).unwrap_or_default();
    std::fs::write("/tmp/min_inline_ctx.txt", &inline_dump).ok();

    let ctx = Context::from_value(ctx_val.clone()).expect("build context");
    let inline_ctx = Context::from_value(inline_ctx_val).expect("inline build");

    // Test YAML context - simplest for loop
    let mut t = tera::Tera::default();
    t.add_raw_template("t1", "{% for slide in content.slides %}{{ slide.title }}{% endfor %}").unwrap();
    let r = t.render("t1", &ctx);
    std::fs::write("/tmp/min_t1.txt", &format!("{:?}", r)).ok();

    // Test inline context - simplest for loop (should pass)
    let mut t2 = tera::Tera::default();
    t2.add_raw_template("t2", "{% for slide in content.slides %}{{ slide.title }}{% endfor %}").unwrap();
    let r2 = t2.render("t2", &inline_ctx);
    std::fs::write("/tmp/min_t2.txt", &format!("{:?}", r2)).ok();

    // Test direct slide access on YAML context
    t.add_raw_template("t3", "{{ content.slides[0].title }}").unwrap();
    let r3 = t.render("t3", &ctx);
    std::fs::write("/tmp/min_t3.txt", &format!("{:?}", r3)).ok();

    // Test iteration count on YAML context
    t.add_raw_template("t4", "{% for slide in content.slides %}X{% endfor %}").unwrap();
    let r4 = t.render("t4", &ctx);
    std::fs::write("/tmp/min_t4.txt", &format!("{:?}", r4)).ok();

    eprintln!("YAML ctx: {}", ctx_dump.chars().take(200).collect::<String>());
    eprintln!("Inline ctx: {}", inline_dump.chars().take(200).collect::<String>());
    eprintln!("YAML render: {:?}", r);
    eprintln!("Inline render: {:?}", r2);
    eprintln!("YAML direct: {:?}", r3);
    eprintln!("YAML count: {:?}", r4);

    // Test: YAML context but with extra resourceType key on slides
    let slides_with_rt = json!([
        {"title": "T1", "content": "C1", "resourceType": "Slide"},
        {"title": "T2", "content": "C2", "resourceType": "Slide"},
    ]);
    let ctx_with_rt = json!({
        "document": {"title": "Test", "slides": &slides_with_rt},
        "content": {"title": "Test", "slides": &slides_with_rt},
        "style": null,
    });
    let ctx_rt = Context::from_value(ctx_with_rt).unwrap();
    let mut t_rt = tera::Tera::default();
    t_rt.add_raw_template("rt", "{% for slide in content.slides %}{{ slide.title }}{% endfor %}").unwrap();
    let r_rt = t_rt.render("rt", &ctx_rt);
    std::fs::write("/tmp/min_rt.txt", &format!("{:?}", r_rt)).ok();

    // Test: YAML context with resourceType on document and slides
    let yslides = json!([
        {"subtitle": "Acme", "title": "Prop", "type": "title"},
        {"content": "Legacy", "title": "Problem", "type": "problem"},
    ]);
    let ctx_yrt = json!({
        "content": {"resourceType": "SlideDocument", "slides": &yslides, "title": "Test"},
        "document": {"resourceType": "SlideDocument", "slides": &yslides, "title": "Test"},
        "style": null,
        "meta": {"forma_version": "0.1.0"},
        "page": {"slide_count": 2},
    });
    let ctx_yrt_c = Context::from_value(ctx_yrt).unwrap();
    let mut t_yrt = tera::Tera::default();
    t_yrt.add_raw_template("yrt", "{% for slide in content.slides %}{{ slide.title }}{% endfor %}").unwrap();
    let r_yrt = t_yrt.render("yrt", &ctx_yrt_c);
    std::fs::write("/tmp/min_yrt.txt", &format!("{:?}", r_yrt)).ok();

    // Test: exact YAML context but slides stripped of resourceType from document
    let ctx_plain = json!({
        "content": {"slides": &ctx_val["content"]["slides"], "title": &ctx_val["content"]["title"]},
        "document": {"slides": &ctx_val["document"]["slides"], "title": &ctx_val["document"]["title"]},
        "style": &ctx_val["style"],
    });
    let ctx_plain_c = Context::from_value(ctx_plain.clone()).unwrap();
    let mut t_plain = tera::Tera::default();
    t_plain.add_raw_template("plain", "{% for slide in content.slides %}{{ slide.title }}{% endfor %}").unwrap();
    let r_plain = t_plain.render("plain", &ctx_plain_c);
    std::fs::write("/tmp/min_plain.txt", &format!("{:?}", r_plain)).ok();

    // Test: rebuild context from scratch with exact same data values as YAML
    let slides_copy: serde_json::Value = ctx_val["content"]["slides"].clone();
    let ctx_rebuilt = json!({
        "content": {
            "resourceType": "SlideDocument",
            "slides": slides_copy,
            "title": &ctx_val["content"]["title"],
        },
        "document": {
            "resourceType": "SlideDocument",
            "slides": slides_copy,
            "title": &ctx_val["document"]["title"],
        },
        "style": &ctx_val["style"],
        "meta": &ctx_val["meta"],
        "page": &ctx_val["page"],
        "page_accessor": &ctx_val["page_accessor"],
    });
    let ctx_rebuilt_c = Context::from_value(ctx_rebuilt.clone()).unwrap();
    let mut t_rb = tera::Tera::default();
    t_rb.add_raw_template("rb", "{% for slide in content.slides %}{{ slide.title }}{% endfor %}").unwrap();
    let r_rb = t_rb.render("rb", &ctx_rebuilt_c);
    std::fs::write("/tmp/min_rebuilt.txt", &format!("{:?}", r_rb)).ok();

    // Test: clone slides explicitly as separate variable
    let slides_explicit: serde_json::Value = ctx_val["content"]["slides"].clone();
    let ctx_cloned = json!({
        "content": {"resourceType": "SlideDocument", "slides": slides_explicit, "title": "Test"},
        "document": {"resourceType": "SlideDocument", "slides": slides_explicit, "title": "Test"},
        "style": null,
    });
    let ctx_cloned_c = Context::from_value(ctx_cloned).unwrap();
    let mut t_cl = tera::Tera::default();
    t_cl.add_raw_template("cl", "{% for slide in content.slides %}{{ slide.title }}{% endfor %}").unwrap();
    let r_cl = t_cl.render("cl", &ctx_cloned_c);
    std::fs::write("/tmp/min_cloned.txt", &format!("{:?}", r_cl)).ok();

    // Test: identical inline context but built with .clone() instead of json!()
    let inline_slides: serde_json::Value = json!([
        {"title": "T1", "content": "C1"},
        {"title": "T2", "content": "C2"},
    ]);
    let inline_cloned = json!({
        "document": {"title": "Test", "slides": inline_slides.clone()},
        "content": {"title": "Test", "slides": inline_slides},
        "style": null,
    });
    let inline_cloned_c = Context::from_value(inline_cloned).unwrap();
    let mut t_ic = tera::Tera::default();
    t_ic.add_raw_template("ic", "{% for slide in content.slides %}{{ slide.title }}{% endfor %}").unwrap();
    let r_ic = t_ic.render("ic", &inline_cloned_c);
    std::fs::write("/tmp/min_inline_cloned.txt", &format!("{:?}", r_ic)).ok();

    // Test: context with ONLY content key (minimal possible)
    let slides_min: serde_json::Value = ctx_val["content"]["slides"].clone();
    let ctx_min = json!({
        "content": {"slides": slides_min},
    });
    let ctx_min_c = Context::from_value(ctx_min.clone()).unwrap();
    let mut t_min = tera::Tera::default();
    t_min.add_raw_template("min", "{% for slide in content.slides %}{{ slide.title }}{% endfor %}").unwrap();
    let r_min = t_min.render("min", &ctx_min_c);
    std::fs::write("/tmp/min_ctx_min.txt", &format!("{:?}", r_min)).ok();

    // Test: context with content + document, no extras
    let slides_min2: serde_json::Value = ctx_val["content"]["slides"].clone();
    let ctx_cd = json!({
        "content": {"slides": slides_min2, "title": &ctx_val["content"]["title"]},
        "document": {"slides": slides_min2, "title": &ctx_val["document"]["title"]},
    });
    let ctx_cd_c = Context::from_value(ctx_cd).unwrap();
    let mut t_cd = tera::Tera::default();
    t_cd.add_raw_template("cd", "{% for slide in content.slides %}{{ slide.title }}{% endfor %}").unwrap();
    let r_cd = t_cd.render("cd", &ctx_cd_c);
    std::fs::write("/tmp/min_cd.txt", &format!("{:?}", r_cd)).ok();

    // Test: inline context with a 7th extra key to match YAML context size
    let slides_inline: serde_json::Value = json!([
        {"title": "T1", "content": "C1"},
        {"title": "T2", "content": "C2"},
    ]);
    let ctx_7keys = json!({
        "content": {"title": "Test", "slides": &slides_inline},
        "document": {"title": "Test", "slides": &slides_inline},
        "style": null,
        "meta": {"forma_version": "0.1.0"},
        "page": {"slide_count": 2},
        "page_accessor": [{"title": "Test", "slides": &slides_inline}],
        "extra": "value",
    });
    let ctx_7keys_c = Context::from_value(ctx_7keys).unwrap();
    let mut t_7 = tera::Tera::default();
    t_7.add_raw_template("t7", "{% for slide in content.slides %}{{ slide.title }}{% endfor %}").unwrap();
    let r_7 = t_7.render("t7", &ctx_7keys_c);
    std::fs::write("/tmp/min_7keys.txt", &format!("{:?}", r_7)).ok();

    // Test: Take the raw JSON string from YAML ctx and parse fresh
    let raw_yaml_ctx = serde_json::to_string(&ctx_val).unwrap();
    let ctx_parsed: serde_json::Value = serde_json::from_str(&raw_yaml_ctx).unwrap();
    let ctx_parsed_c = Context::from_value(ctx_parsed).unwrap();
    let mut t_parsed = tera::Tera::default();
    t_parsed.add_raw_template("parsed", "{% for slide in content.slides %}{{ slide.title }}{% endfor %}").unwrap();
    let r_parsed = t_parsed.render("parsed", &ctx_parsed_c);
    std::fs::write("/tmp/min_parsed.txt", &format!("{:?}", r_parsed)).ok();

    // Test: single slide from YAML in a minimal context
    let slide_from_yaml = ctx_val["content"]["slides"][0].clone();
    let ctx_single_slide = json!({
        "slide": slide_from_yaml,
    });
    let ctx_ss_c = Context::from_value(ctx_single_slide).unwrap();
    let mut t_ss = tera::Tera::default();
    t_ss.add_raw_template("ss", "{{ slide.title }}").unwrap();
    let r_ss = t_ss.render("ss", &ctx_ss_c);
    std::fs::write("/tmp/min_single_slide.txt", &format!("{:?}", r_ss)).ok();

    // Test: for loop over cloned array from YAML context
    let slides_cloned = ctx_val["content"]["slides"].clone();
    let ctx_for_cloned = json!({
        "slides": slides_cloned,
    });
    let ctx_fc_c = Context::from_value(ctx_for_cloned).unwrap();
    let mut t_fc = tera::Tera::default();
    t_fc.add_raw_template("fc", "{% for slide in slides %}{{ slide.title }}{% endfor %}").unwrap();
    let r_fc = t_fc.render("fc", &ctx_fc_c);
    std::fs::write("/tmp/min_for_cloned.txt", &format!("{:?}", r_fc)).ok();

    // Test: exact same context as inline test but with YAML slide data nested
    let slides_from_yaml = ctx_val["content"]["slides"].clone();
    let ctx_yaml_nested = json!({
        "content": {"slides": slides_from_yaml},
    });
    let ctx_yn_c = Context::from_value(ctx_yaml_nested).unwrap();
    let mut t_yn = tera::Tera::default();
    t_yn.add_raw_template("yn", "{% for slide in content.slides %}{{ slide.title }}{% endfor %}").unwrap();
    let r_yn = t_yn.render("yn", &ctx_yn_c);
    std::fs::write("/tmp/min_yaml_nested.txt", &format!("{:?}", r_yn)).ok();

    // Test: from_value with a fresh struct
    let mut fresh_ctx = tera::Context::new();
    fresh_ctx.insert("content", &ctx_val["content"]);
    let mut t_fresh = tera::Tera::default();
    t_fresh.add_raw_template("fresh", "{% for slide in content.slides %}{{ slide.title }}{% endfor %}").unwrap();
    let r_fresh = t_fresh.render("fresh", &fresh_ctx);
    std::fs::write("/tmp/min_fresh_insert.txt", &format!("{:?}", r_fresh)).ok();

    // Test: different var name in for loop
    let mut t_varname = tera::Tera::default();
    t_varname.add_raw_template("varname", "{% for s in content.slides %}{{ s.title }}{% endfor %}").unwrap();
    let r_varname = t_varname.render("varname", &ctx);
    std::fs::write("/tmp/min_varname.txt", &format!("{:?}", r_varname)).ok();

    // Test: explicit let to extract array
    let mut t_let = tera::Tera::default();
    t_let.add_raw_template("let", "{% set slides_arr = content.slides %}{% for s in slides_arr %}{{ s.title }}{% endfor %}").unwrap();
    let r_let = t_let.render("let", &ctx);
    std::fs::write("/tmp/min_let.txt", &format!("{:?}", r_let)).ok();

    // Test: inline with ONLY content key (minimal, matching YAML shape)
    let slides_min_inline = json!([
        {"subtitle": "Acme", "title": "T1", "type": "title"},
        {"content": "C", "title": "T2", "type": "problem"},
        {"content": "S", "title": "T3", "type": "solution"},
        {"logo": "L", "type": "closing"},
    ]);
    let ctx_min_inline = json!({
        "content": {"slides": &slides_min_inline},
        "document": {"title": "Test"},
        "style": null,
    });
    let ctx_mi = Context::from_value(ctx_min_inline).unwrap();
    let mut t_minline = tera::Tera::default();
    t_minline.add_raw_template("mininline", "{% for s in content.slides %}{{ s.title }}{% endfor %}").unwrap();
    let r_minline = t_minline.render("mininline", &ctx_mi);
    std::fs::write("/tmp/min_inline_fullshape.txt", &format!("{:?}", r_minline)).ok();

    // Test: HTML template with direct index instead of for loop iteration
    let mut t_html_idx = tera::Tera::default();
    t_html_idx.add_raw_template("html_idx", "<h2>{{ content.slides[0].title }}</h2>").unwrap();
    let r_html_idx = t_html_idx.render("html_idx", &ctx);
    std::fs::write("/tmp/min_html_idx.txt", &format!("{:?}", r_html_idx)).ok();

    // Test: access iteration variable via first() filter
    let mut t_first = tera::Tera::default();
    t_first.add_raw_template("first", "{% for s in content.slides %}{{ s|first }}{% endfor %}").unwrap();
    let r_first = t_first.render("first", &ctx);
    std::fs::write("/tmp/min_first_filter.txt", &format!("{:?}", r_first)).ok();

    // Test: iteration with key + value
    let mut t_kv = tera::Tera::default();
    t_kv.add_raw_template("kv", "{% for k, v in content.slides %}{{ v.title }}{% endfor %}").unwrap();
    let r_kv = t_kv.render("kv", &ctx);
    std::fs::write("/tmp/min_kv.txt", &format!("{:?}", r_kv)).ok();

    // Test: simple primitive for loop to confirm basic Tera works
    let ctx_primitives = json!({
        "items": [1, 2, 3],
    });
    let ctx_prim = Context::from_value(ctx_primitives).unwrap();
    let mut t_prim = tera::Tera::default();
    t_prim.add_raw_template("prim", "{% for i in items %}{{ i }}{% endfor %}").unwrap();
    let r_prim = t_prim.render("prim", &ctx_prim);
    std::fs::write("/tmp/min_primitives.txt", &format!("{:?}", r_prim)).ok();

    // Test: array of primitive strings
    let ctx_strs = json!({
        "items": ["hello", "world"],
    });
    let ctx_str = Context::from_value(ctx_strs).unwrap();
    let mut t_strs = tera::Tera::default();
    t_strs.add_raw_template("strs", "{% for s in items %}{{ s }}{% endfor %}").unwrap();
    let r_strs = t_strs.render("strs", &ctx_str);
    std::fs::write("/tmp/min_primitive_strs.txt", &format!("{:?}", r_strs)).ok();

    // Test: full-shape inline with 7 keys per item (matching YAML structure exactly)
    let full_shape = json!([
        {"resourceType": "Slide", "subtitle": "Acme Corp", "title": "DTP", "type": "title"},
        {"resourceType": "Slide", "content": "Legacy", "title": "Problem", "type": "problem"},
        {"resourceType": "Slide", "content": "Cloud", "title": "Solution", "type": "solution"},
        {"resourceType": "Slide", "logo": "L", "title": "Closing", "type": "closing"},
    ]);
    let ctx_full = json!({
        "content": {"resourceType": "SlideDocument", "slides": &full_shape, "title": "Test"},
        "document": {"resourceType": "SlideDocument", "slides": &full_shape, "title": "Test"},
        "style": null, "meta": {"forma_version": "0.1.0"},
        "page": {"slide_count": 4}, "page_accessor": [{"title": "Test"}],
    });
    let ctx_full_c = Context::from_value(ctx_full).unwrap();
    let mut t_full = tera::Tera::default();
    t_full.add_raw_template("full", "{% for s in content.slides %}{{ s.title }}{% endfor %}").unwrap();
    let r_full = t_full.render("full", &ctx_full_c);
    std::fs::write("/tmp/min_full_inline_samekeys.txt", &format!("{:?}", r_full)).ok();

    // Test: JSON serialization of iteration variable
    let mut t_tojson = tera::Tera::default();
    t_tojson.add_raw_template("tojson", "{{ content.slides[0]|tojson }}").unwrap();
    let r_tojson = t_tojson.render("tojson", &ctx);
    std::fs::write("/tmp/min_tojson.txt", &format!("{:?}", r_tojson)).ok();

    // Test: JSON round-trip - serialize ctx_val to string then parse fresh
    let json_string = serde_json::to_string(&ctx_val).unwrap();
    let ctx_rt: serde_json::Value = serde_json::from_str(&json_string).unwrap();
    let ctx_rt_c = Context::from_value(ctx_rt).unwrap();
    let mut t_rt = tera::Tera::default();
    t_rt.add_raw_template("rt", "{% for s in content.slides %}{{ s.title }}{% endfor %}").unwrap();
    let r_rt = t_rt.render("rt", &ctx_rt_c);
    std::fs::write("/tmp/min_json_rt.txt", &format!("{:?}", r_rt)).ok();

    // Test: YAML → serde_yaml round-trip → serde_json (diagnostic)
    let raw_yaml = serde_yaml::to_string(&ctx_val).unwrap();
    let reloaded: serde_yaml::Value = serde_yaml::from_str(&raw_yaml).unwrap();
    let ctx_yml: serde_json::Value = serde_json::to_value(&reloaded).unwrap();
    let ctx_yml_c = Context::from_value(ctx_yml).unwrap();
    let mut t_yml = tera::Tera::default();
    t_yml.add_raw_template("yml", "{% for s in content.slides %}{{ s.title }}{% endfor %}").unwrap();
    let r_yml = t_yml.render("yml", &ctx_yml_c);
    std::fs::write("/tmp/min_yaml_rt_rt.txt", &format!("{:?}", r_yml)).ok();

    // Test: manual serde_json Value construction from YAML primitives
    let slides_yaml_obj = ctx_val["content"]["slides"].as_array().unwrap();
    let slides_manual: Vec<serde_json::Value> = slides_yaml_obj.iter().map(|item| {
        let obj = item.as_object().unwrap();
        let mut m = serde_json::Map::new();
        for (k, v) in obj {
            m.insert(k.clone(), v.clone());
        }
        serde_json::Value::Object(m)
    }).collect();
    let ctx_manual = json!({
        "content": {"slides": slides_manual, "title": &ctx_val["content"]["title"]},
        "document": {"title": &ctx_val["document"]["title"]},
        "style": &ctx_val["style"],
    });
    let ctx_manual_c = Context::from_value(ctx_manual).unwrap();
    let mut t_manual = tera::Tera::default();
    t_manual.add_raw_template("manual", "{% for s in content.slides %}{{ s.title }}{% endfor %}").unwrap();
    let r_manual = t_manual.render("manual", &ctx_manual_c);
    std::fs::write("/tmp/min_manual_construct.txt", &format!("{:?}", r_manual)).ok();

    // --- ADDITIONAL DIAGNOSTIC: appended to debug slide.title issue ---

    // Test A: Check if iteration var is string (false check on string)
    let mut t_a = tera::Tera::default();
    t_a.add_raw_template("da", "{% for x in content.slides %}{{ x is string }}{% endfor %}").unwrap();
    let r_a = t_a.render("da", &ctx);
    std::fs::write("/tmp/min_diag_string.txt", &format!("{:?}", r_a)).ok();

    // Test B: Different var name
    let mut t_b = tera::Tera::default();
    t_b.add_raw_template("db", "{% for it in content.slides %}{{ it.title }}{% endfor %}").unwrap();
    let r_b = t_b.render("db", &ctx);
    std::fs::write("/tmp/min_diag_varname.txt", &format!("{:?}", r_b)).ok();

    // Test C: access via loop.index0 directly
    let mut t_c = tera::Tera::default();
    t_c.add_raw_template("dc", "{{ content.slides[loop.index0].title }}").unwrap();
    let r_c = t_c.render("dc", &ctx);
    std::fs::write("/tmp/min_diag_idx0.txt", &format!("{:?}", r_c)).ok();

    // Test D: serialize iteration var to check type
    let mut t_d = tera::Tera::default();
    t_d.add_raw_template("dd", "{% for x in content.slides %}{{ x|json_encode }} {% endfor %}").unwrap();
    let r_d = t_d.render("dd", &ctx);
    std::fs::write("/tmp/min_diag_jsonenc.txt", &format!("{:?}", r_d)).ok();

    // Test E: json!() with reference to YAML array (mimicking rebuild_value scenario)
    let yaml_slides: serde_json::Value = ctx_val["content"]["slides"].clone();
    let ctx_ref_slides = json!({
        "content": {"slides": &yaml_slides},
    });
    let ctx_rs_c = Context::from_value(ctx_ref_slides).unwrap();
    let mut t_e = tera::Tera::default();
    t_e.add_raw_template("de", "{% for x in content.slides %}{{ x.title }}{% endfor %}").unwrap();
    let r_e = t_e.render("de", &ctx_rs_c);
    std::fs::write("/tmp/min_diag_ref_slides.txt", &format!("{:?}", r_e)).ok();

    // Test F: full rebuild via build_context (which now calls rebuild_value internally)
    let ctx_f_fresh = build_context(&doc, &serde_json::json!({}));
    let ctx_fb_c = Context::from_value(ctx_f_fresh).unwrap();
    let mut t_f = tera::Tera::default();
    t_f.add_raw_template("df", "{% for x in content.slides %}{{ x.title }}{% endfor %}").unwrap();
    let r_f = t_f.render("df", &ctx_fb_c);
    std::fs::write("/tmp/min_diag_full_rebuild.txt", &format!("{:?}", r_f)).ok();

    // Test G: test render_template directly (which also rebuilds)
    let tmp_dir = std::env::temp_dir();
    let out_path = tmp_dir.join("diag_slide.html");
    let template_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .join("tests").join("fixtures")
        .join("templates").join("proposal-slides-html");
    let r_gt = render_template(&template_dir, &doc, &serde_json::json!({}), &out_path, None);
    std::fs::write("/tmp/min_diag_render_template.txt", &format!("{:?}", r_gt)).ok();

    // Test H: Context::from_value with just the slides array as root
    let slides_only: serde_json::Value = json!({"slides": &ctx_val["content"]["slides"]});
    let ctx_ho = Context::from_value(slides_only).unwrap();
    let mut t_h = tera::Tera::default();
    t_h.add_raw_template("dh", "{% for x in slides %}{{ x.title }}{% endfor %}").unwrap();
    let r_h = t_h.render("dh", &ctx_ho);
    std::fs::write("/tmp/min_diag_slides_only.txt", &format!("{:?}", r_h)).ok();

    // Test I: range-based index iteration (bypasses for-loop var scoping)
    let mut t_i = tera::Tera::default();
    let tmpl_i = "{% set n = content.slides|length %}{% for i in range(0, n - 1) %}{{ content.slides[i].title }}{% endfor %}";
    let r_add = t_i.add_raw_template("di", tmpl_i);
    if let Err(ref e) = r_add {
        std::fs::write("/tmp/min_diag_range_err.txt", &format!("{:?}", e)).ok();
    }
    if r_add.is_ok() {
        let r_i = t_i.render("di", &ctx);
        std::fs::write("/tmp/min_diag_range.txt", &format!("{:?}", r_i)).ok();
    }

    // Test J: bracket access on iteration var
    let mut t_j = tera::Tera::default();
    let tmpl_j = "{% for x in content.slides %}{{ x[\"title\"] }}{% endfor %}";
    let r_add_j = t_j.add_raw_template("dj", tmpl_j);
    if let Err(ref e) = r_add_j {
        std::fs::write("/tmp/min_diag_bracket_err.txt", &format!("{:?}", e)).ok();
    }
    if r_add_j.is_ok() {
        let r_j = t_j.render("dj", &ctx);
        std::fs::write("/tmp/min_diag_bracket.txt", &format!("{:?}", r_j)).ok();
    }
}
