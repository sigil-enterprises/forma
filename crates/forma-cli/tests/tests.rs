//! CLI integration tests via std::process::Command.
//! Uses the built binary; copies fixtures to temp dirs.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn forma_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_forma"))
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .join("tests").join("fixtures").join("example-client")
}

fn copy_fixture(tmp: &tempfile::TempDir) -> PathBuf {
    let src = fixture_dir();
    let dst = tmp.path().join("example-client");
    copy_dir_all(&src, &dst).unwrap();
    dst
}

fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let name = entry.file_name();
        let dst_path = dst.join(name);
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            fs::copy(entry.path(), &dst_path)?;
        }
    }
    Ok(())
}

fn run_forma(args: &[&str]) -> std::process::Output {
    Command::new(forma_bin())
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to run forma")
}

// ── validate command ─────────────────────────────────────────────────

#[test]
fn test_validate_passes_on_fixture() {
    let out = run_forma(&["validate", &fixture_dir().to_string_lossy()]);
    assert_eq!(out.status.code(), Some(0));
    let combined = String::from_utf8_lossy(&out.stdout).to_string()
        + String::from_utf8_lossy(&out.stderr).as_ref();
    assert!(combined.contains("OK: all documents valid") || combined.contains("valid"));
}

#[test]
fn test_validate_fails_on_bad_content() {
    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path().join("bad-project");
    fs::create_dir_all(&project).unwrap();

    // Copy forma.yaml from fixture
    let fixture = fixture_dir();
    fs::copy(fixture.join("forma.yaml"), project.join("forma.yaml")).unwrap();

    // Bad content: declares ProposalContent but missing required 'client'
    let bad_content = "resourceType: ProposalContent@1\n\
                      engagement:\n  title: Test\n";
    fs::write(project.join("content.yaml"), bad_content).unwrap();

    let out = run_forma(&["validate", project.to_str().unwrap()]);
    assert_ne!(out.status.code(), Some(0));
}

#[test]
fn test_validate_strict_flag_accepted() {
    let tmp = tempfile::tempdir().unwrap();
    let project = copy_fixture(&tmp);
    let out = run_forma(&["validate", project.to_str().unwrap(), "--strict"]);
    // Either pass (0) or fail on warnings (1) — CLI should not crash
    assert!(out.status.code() == Some(0) || out.status.code() == Some(1));
}

// ── mapping validate command ─────────────────────────────────────────

#[test]
fn test_mapping_validate_passes_on_fixture() {
    let out = run_forma(&["mapping", "validate", &fixture_dir().to_string_lossy()]);
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn test_mapping_validate_specific_file() {
    let out = run_forma(&[
        "mapping", "validate",
        &fixture_dir().to_string_lossy(),
        "--file", "slides.yaml",
    ]);
    assert_eq!(out.status.code(), Some(0));
}

// ── init command ─────────────────────────────────────────────────────

#[test]
fn test_init_scaffolds_project() {
    let tmp = tempfile::tempdir().unwrap();
    let out = run_forma(&["init", "Test Client", "--dir", tmp.path().to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(0), "init failed: {}", String::from_utf8_lossy(&out.stderr));

    let project = tmp.path().join("test-client");
    assert!(project.is_dir());
    assert!(project.join("forma.yaml").exists());
    assert!(project.join("content.yaml").exists());
    assert!(project.join("slides.yaml").exists());
    assert!(project.join("report.yaml").exists());
}

#[test]
fn test_init_forma_yaml_format() {
    let tmp = tempfile::tempdir().unwrap();
    run_forma(&["init", "Acme Corp", "--dir", tmp.path().to_str().unwrap()]);
    let forma = tmp.path().join("acme-corp/forma.yaml");
    let content = fs::read_to_string(&forma).unwrap();
    assert!(content.contains("resourceType"));
    assert!(content.contains("FormaConfig"));
    assert!(content.contains("slides"));
    assert!(content.contains("report"));
}

#[test]
fn test_init_content_has_resource_type() {
    let tmp = tempfile::tempdir().unwrap();
    run_forma(&["init", "Briefing Co", "--dir", tmp.path().to_str().unwrap()]);
    let content = fs::read_to_string(tmp.path().join("briefing-co/content.yaml")).unwrap();
    assert!(content.contains("resourceType"));
    assert!(content.contains("ProposalContent"));
}

#[test]
fn test_init_idempotent() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().to_str().unwrap();
    let out1 = run_forma(&["init", "My Client", "--dir", dir]);
    let out2 = run_forma(&["init", "My Client", "--dir", dir]);
    assert_eq!(out1.status.code(), Some(0));
    assert_eq!(out2.status.code(), Some(0));
}

// ── schema export command ────────────────────────────────────────────

#[test]
fn test_schema_export() {
    let tmp = tempfile::tempdir().unwrap();
    let out = run_forma(&["schema", "export", "--dir", tmp.path().to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(0), "schema export failed: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains(".yaml"));
}

// ── template list command ────────────────────────────────────────────

#[test]
fn test_template_list_runs() {
    let out = run_forma(&["template"]);
    assert_eq!(out.status.code(), Some(0), "template list failed: {}", String::from_utf8_lossy(&out.stderr));
}
