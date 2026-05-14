//! Tests for forma-composer: prompt builders, schema validation.

use forma_composer::{
    build_system_prompt, build_user_prompt,
    filler::{SchemaType, ComposerError},
};
use forma_schema::content::ContentType;

// ── build_system_prompt tests ────────────────────────────────────────

#[test]
fn test_system_prompt_contains_schema() {
    let prompt = build_system_prompt(ContentType::Proposal);
    assert!(prompt.contains("JSON Schema:"), "expected 'JSON Schema:' in prompt");
    assert!(prompt.contains("ProposalContent"), "expected 'ProposalContent' in prompt");
}

#[test]
fn test_system_prompt_different_types() {
    for (ctype, label) in [
        (ContentType::Brief, "BriefContent"),
        (ContentType::CaseStudy, "CaseStudyContent"),
        (ContentType::StatusReport, "StatusReportContent"),
    ] {
        let prompt = build_system_prompt(ctype);
        assert!(prompt.contains(label), "expected '{label}' in system prompt for {ctype:?}");
    }
}

#[test]
fn test_system_prompt_contains_rules() {
    let prompt = build_system_prompt(ContentType::Proposal);
    assert!(prompt.contains("Output ONLY valid YAML"));
    assert!(prompt.contains("Every required field in the schema must be present"));
}

// ── build_user_prompt tests ──────────────────────────────────────────

#[test]
fn test_user_prompt_wraps_notes() {
    let notes = "Client wants a digital transformation proposal.";
    let prompt = build_user_prompt(notes, None);
    assert!(prompt.contains(notes));
    assert!(prompt.contains("Output only the YAML document."));
}

#[test]
fn test_user_prompt_includes_existing_yaml() {
    let notes = "Expand the proposal.";
    let existing = "client:\n  name: Acme\n";
    let prompt = build_user_prompt(notes, Some(existing));
    assert!(prompt.contains(notes));
    assert!(prompt.contains(existing.trim()));
    assert!(prompt.contains("existing partial content.yaml"));
}

#[test]
fn test_user_prompt_no_existing_yaml() {
    let notes = "Simple notes.";
    let prompt = build_user_prompt(notes, None);
    assert!(prompt.contains(notes));
    assert!(!prompt.contains("existing partial"));
}

#[test]
fn test_user_prompt_empty_notes() {
    let prompt = build_user_prompt("", None);
    assert!(prompt.contains("Output only the YAML document."));
}

// ── SchemaType::validate tests ───────────────────────────────────────

#[test]
fn test_validate_proposal_valid() {
    let schema = SchemaType::Proposal;
    let yaml = r#"
resourceType: ProposalContent@1
engagement:
  title: Digital Transformation
  date: 2025-06-15
client:
  name: Acme Corp
  industry: Technology
executive_summary:
  headline: Turning Vision into Reality
"#;
    let result = schema.validate(yaml);
    assert!(result.is_ok(), "expected valid proposal YAML to pass, got: {:?}", result);
}

#[test]
fn test_validate_proposal_missing_required_field() {
    let schema = SchemaType::Proposal;
    // Missing executive_summary (required)
    let yaml = r#"
resourceType: ProposalContent@1
engagement:
  title: Test
  date: 2025-06-15
client:
  name: Acme
"#;
    let result = schema.validate(yaml);
    assert!(result.is_err(), "expected validation to fail without executive_summary");
}

#[test]
fn test_validate_proposal_invalid_yaml() {
    let schema = SchemaType::Proposal;
    let result = schema.validate("{{invalid: yaml: [");
    assert!(result.is_err(), "expected invalid YAML to fail");
    match result.unwrap_err() {
        ComposerError::Validation(_) => {},
        other => panic!("expected Validation error, got {:?}", other),
    }
}
