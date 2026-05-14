//! Tests for forma-core: include_loader, config, loader, validator.

use std::path::PathBuf;
use std::fs;

use serde_yaml;
use tempfile::TempDir;
use forma_core::include_loader::{self, IncludeError};
use forma_core::validate_file;
use forma_core::validate_project;
use forma_core::load_document;

// ── helpers ──────────────────────────────────────────────────────────

fn tmp_dir() -> TempDir {
    tempfile::tempdir().unwrap()
}

fn write_file(dir: &TempDir, name: &str, content: &str) {
    let path = dir.path().join(name);
    fs::create_dir_all(path.parent().unwrap()).ok();
    fs::write(&path, content).unwrap();
}

// ── include_loader tests ─────────────────────────────────────────────

#[test]
fn test_include_scalar_value() {
    let dir = tmp_dir();
    write_file(&dir, "data.yaml", "name: hello\nage: 42\n");
    write_file(&dir, "main.yaml", "greeting: !include \"@data.yaml:name\"\n");

    let result = include_loader::load_mapping(&dir.path().join("main.yaml"), dir.path());
    assert!(result.is_ok(), "expected Ok, got {:?}", result);
    let val = result.unwrap();
    assert_eq!(val["greeting"].as_str(), Some("hello"));
}

#[test]
fn test_include_entire_file() {
    let dir = tmp_dir();
    write_file(&dir, "colors.yaml", "primary: blue\nsecondary: green\n");
    write_file(&dir, "main.yaml", "style: !include \"@colors.yaml\"\n");

    let result = include_loader::load_mapping(&dir.path().join("main.yaml"), dir.path());
    let val = result.unwrap();
    assert_eq!(val["style"]["primary"].as_str(), Some("blue"));
    assert_eq!(val["style"]["secondary"].as_str(), Some("green"));
}

#[test]
fn test_include_nested_dot_path() {
    let dir = tmp_dir();
    write_file(&dir, "content.yaml", "client:\n  contact:\n    email: a@b.com\n");
    write_file(&dir, "main.yaml", "email: !include \"@content.yaml:client.contact.email\"\n");

    let result = include_loader::load_mapping(&dir.path().join("main.yaml"), dir.path());
    let val = result.unwrap();
    assert_eq!(val["email"].as_str(), Some("a@b.com"));
}

#[test]
fn test_include_list_value() {
    let dir = tmp_dir();
    write_file(&dir, "content.yaml", "items:\n  - foo\n  - bar\n");
    write_file(&dir, "main.yaml", "list: !include \"@content.yaml:items\"\n");

    let result = include_loader::load_mapping(&dir.path().join("main.yaml"), dir.path());
    let val = result.unwrap();
    assert_eq!(val["list"][0].as_str(), Some("foo"));
    assert_eq!(val["list"][1].as_str(), Some("bar"));
}

#[test]
fn test_include_list_index_traversal() {
    let dir = tmp_dir();
    write_file(&dir, "content.yaml", "items:\n  - name: first\n  - name: second\n");
    write_file(&dir, "main.yaml", "name: !include \"@content.yaml:items.1.name\"\n");

    let result = include_loader::load_mapping(&dir.path().join("main.yaml"), dir.path());
    let val = result.unwrap();
    assert_eq!(val["name"].as_str(), Some("second"));
}

#[test]
fn test_include_file_caching() {
    let dir = tmp_dir();
    write_file(&dir, "content.yaml", "title: Hello\n\n");

    let result1 = include_loader::load_mapping(&dir.path().join("content.yaml"), dir.path());
    assert!(result1.is_ok());
    let result2 = include_loader::load_mapping(&dir.path().join("content.yaml"), dir.path());
    assert!(result2.is_ok());
    assert_eq!(result1.unwrap(), result2.unwrap());
}

#[test]
fn test_include_missing_file_raises() {
    let dir = tmp_dir();
    write_file(&dir, "main.yaml", "x: !include \"@missing.yaml\"\n");

    let result = include_loader::load_mapping(&dir.path().join("main.yaml"), dir.path());
    assert!(result.is_err());
    match result.unwrap_err() {
        IncludeError::FileNotFound(_) => {},
        other => panic!("expected FileNotFound, got {:?}", other),
    }
}

#[test]
fn test_include_missing_key_raises() {
    let dir = tmp_dir();
    write_file(&dir, "data.yaml", "a: 1\n");
    write_file(&dir, "main.yaml", "x: !include \"@data.yaml:b\"\n");

    let result = include_loader::load_mapping(&dir.path().join("main.yaml"), dir.path());
    assert!(result.is_err());
    match result.unwrap_err() {
        IncludeError::KeyNotFound { key, .. } => assert_eq!(key, "b"),
        other => panic!("expected KeyNotFound, got {:?}", other),
    }
}

#[test]
fn test_include_missing_list_index_raises() {
    let dir = tmp_dir();
    write_file(&dir, "data.yaml", "items:\n  - a\n  - b\n");
    write_file(&dir, "main.yaml", "x: !include \"@data.yaml:items.5\"\n");

    let result = include_loader::load_mapping(&dir.path().join("main.yaml"), dir.path());
    assert!(result.is_err());
    match result.unwrap_err() {
        IncludeError::IndexInvalid { .. } => {},
        other => panic!("expected IndexInvalid, got {:?}", other),
    }
}

#[test]
fn test_include_invalid_ref_value_raises() {
    let dir = tmp_dir();
    write_file(&dir, "main.yaml", "x: !include \"data.yaml:key\"\n");

    let result = include_loader::load_mapping(&dir.path().join("main.yaml"), dir.path());
    assert!(result.is_err());
    match result.unwrap_err() {
        IncludeError::InvalidRef(_) => {},
        other => panic!("expected InvalidRef, got {:?}", other),
    }
}

#[test]
fn test_include_traverse_none_raises() {
    let dir = tmp_dir();
    write_file(&dir, "data.yaml", "a: ~\n");
    write_file(&dir, "main.yaml", "x: !include \"@data.yaml:a.b\"\n");

    let result = include_loader::load_mapping(&dir.path().join("main.yaml"), dir.path());
    assert!(result.is_err());
    match result.unwrap_err() {
        IncludeError::NoneTraversal { .. } => {},
        other => panic!("expected NoneTraversal, got {:?}", other),
    }
}

#[test]
fn test_include_traverse_scalar_raises() {
    let dir = tmp_dir();
    write_file(&dir, "data.yaml", "a: 42\n");
    write_file(&dir, "main.yaml", "x: !include \"@data.yaml:a.b\"\n");

    let result = include_loader::load_mapping(&dir.path().join("main.yaml"), dir.path());
    assert!(result.is_err());
    match result.unwrap_err() {
        IncludeError::NoneTraversal { .. } => {},
        other => panic!("expected NoneTraversal (scalar traversal), got {:?}", other),
    }
}

#[test]
fn test_include_fresh_cache_per_call() {
    let dir = tmp_dir();
    write_file(&dir, "data.yaml", "x: one\n");
    write_file(&dir, "main.yaml", "a: !include \"@data.yaml:x\"\n");

    let result1 = include_loader::load_mapping(&dir.path().join("main.yaml"), dir.path()).unwrap();
    assert_eq!(result1["a"].as_str(), Some("one"));

    write_file(&dir, "data.yaml", "x: two\n");

    let result2 = include_loader::load_mapping(&dir.path().join("main.yaml"), dir.path()).unwrap();
    assert_eq!(result2["a"].as_str(), Some("two"));
}

#[test]
fn test_include_mapping_not_found() {
    let result = include_loader::load_mapping(
        &PathBuf::from("/nonexistent/path/main.yaml"),
        &PathBuf::from("/"),
    );
    assert!(result.is_err());
    match result.unwrap_err() {
        IncludeError::FileNotFound(_) => {},
        other => panic!("expected FileNotFound, got {:?}", other),
    }
}

#[test]
fn test_include_empty_mapping_returns_empty() {
    let dir = tmp_dir();
    write_file(&dir, "main.yaml", "");

    let result = include_loader::load_mapping(&dir.path().join("main.yaml"), dir.path());
    let val = result.unwrap();
    assert!(val.is_mapping() || val.is_null());
}

#[test]
fn test_include_plain_yaml_no_includes() {
    let dir = tmp_dir();
    write_file(&dir, "main.yaml", "name: hello\nvalue: 42\n");

    let result = include_loader::load_mapping(&dir.path().join("main.yaml"), dir.path());
    let val = result.unwrap();
    assert_eq!(val["name"].as_str(), Some("hello"));
    assert_eq!(val["value"].as_i64(), Some(42));
}

#[test]
fn test_include_multiple_includes_from_same_file() {
    let dir = tmp_dir();
    write_file(&dir, "content.yaml", "title: Hello\nauthor: World\n");
    write_file(&dir, "main.yaml",
        "title: !include \"@content.yaml:title\"\n\
         author: !include \"@content.yaml:author\"\n",
    );

    let result = include_loader::load_mapping(&dir.path().join("main.yaml"), dir.path());
    let val = result.unwrap();
    assert_eq!(val["title"].as_str(), Some("Hello"));
    assert_eq!(val["author"].as_str(), Some("World"));
}

// ── loader / config tests ────────────────────────────────────────────

#[test]
fn test_load_document_resolves_includes() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .join("tests").join("fixtures").join("example-client");
    let doc_path = fixture.join("slides.yaml");
    let result = load_document(&doc_path, &fixture);
    assert!(result.is_ok());
    let doc = result.unwrap();
    assert!(doc["slides"][0]["title"].as_str().unwrap_or("").contains("Digital"));
}

// ── validator tests ──────────────────────────────────────────────────

#[test]
fn test_validate_file_passes_for_fixture() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .join("tests").join("fixtures").join("example-client");
    let content_path = fixture.join("content.yaml");
    let result = validate_file(&content_path, None);
    assert!(result.ok(), "expected validation to pass, got errors: {:?}", result.errors);
}

#[test]
fn test_validate_file_catches_missing_required_field() {
    let dir = tmp_dir();
    write_file(&dir, "content.yaml",
        "resourceType: ProposalContent@1\n\
         engagement:\n  title: Test\n",
    );
    let content_path = dir.path().join("content.yaml");
    let result = validate_file(&content_path, None);
    assert!(!result.ok(), "expected validation to fail without client");
    let has_client_error = result.errors.iter().any(|e| e.to_lowercase().contains("client"));
    assert!(has_client_error, "expected client-related error, got: {:?}", result.errors);
}

#[test]
fn test_validate_file_warns_without_resource_type() {
    let dir = tmp_dir();
    write_file(&dir, "doc.yaml", "name: hello\n");
    let doc_path = dir.path().join("doc.yaml");
    let result = validate_file(&doc_path, None);
    assert!(result.ok());
    assert!(!result.warnings.is_empty(), "expected a warning about missing resourceType");
}

#[test]
fn test_validate_project_passes_for_fixture() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .join("tests").join("fixtures").join("example-client");
    let result = validate_project(&fixture);
    assert!(result.ok(), "expected project validation to pass, got errors: {:?}", result.errors);
}

#[test]
fn test_validate_project_content_missing_field_fails() {
    let dir = tmp_dir();
    write_file(&dir, "content.yaml",
        "resourceType: ProposalContent@1\n\
         engagement:\n  title: Test\n",
    );
    write_file(&dir, "slides.yaml",
        "resourceType: SlideDocument\nslides:\n  - type: cover\n    title: Test\n",
    );
    let result = validate_project(dir.path());
    assert!(!result.ok(), "expected validation to fail without client");
}
