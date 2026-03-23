"""
Tests for schema loading, validation, and config.

Covers: FormaConfig, jsonschema-based validation, YAML schema files.
"""

import pytest
import yaml
from pathlib import Path

FIXTURE_DIR = Path(__file__).parent / "fixtures" / "example-client"
REPO_ROOT = Path(__file__).parent.parent


def test_proposal_content_loads_from_fixture():
    from forma.schemas.proposal.content import ProposalContent
    content = ProposalContent.from_yaml(FIXTURE_DIR / "content.yaml")
    assert content.engagement.title == "Digital Transformation Strategy"
    assert content.client.name == "Acme Corp"


def test_proposal_content_investment_totals():
    from forma.schemas.proposal.content import ProposalContent
    content = ProposalContent.from_yaml(FIXTURE_DIR / "content.yaml")
    assert content.investment is not None
    assert content.investment.total_usd > 0
    manual_total = sum(p.subtotal_usd for p in content.investment.phases)
    assert manual_total == pytest.approx(content.investment.total_usd)


def test_validate_file_passes_for_fixture():
    """Validate content.yaml against the ProposalContent JSON Schema."""
    from forma.core.validator import validate_file
    result = validate_file(FIXTURE_DIR / "content.yaml")
    # May have warnings about missing assets; errors are not OK
    assert result.ok, f"Validation errors: {result.errors}"


def test_validate_file_catches_missing_required_field(tmp_path):
    """validate_file catches documents missing required 'client' field."""
    from forma.core.validator import validate_file
    bad = {"resourceType": "ProposalContent", "engagement": {"title": "Test"}}
    p = tmp_path / "content.yaml"
    p.write_text(yaml.dump(bad))
    result = validate_file(p)
    assert not result.ok


def test_validate_file_warns_without_resource_type(tmp_path):
    """Documents without resourceType get a warning instead of an error."""
    from forma.core.validator import validate_file
    p = tmp_path / "content.yaml"
    p.write_text("title: hello\n")
    result = validate_file(p)
    assert result.ok  # not an error, just a warning
    assert any("resourceType" in w or "no resourceType" in w for w in result.warnings)


def test_brief_content_model():
    from forma.schemas.brief.content import BriefContent
    data = {
        "meta": {"title": "One-Pager", "date": "2026-01-01", "prepared_for": "Acme"},
        "sections": [{"heading": "Overview", "body": "Text here."}],
    }
    content = BriefContent.model_validate(data)
    assert content.meta.title == "One-Pager"
    assert len(content.sections) == 1


def test_case_study_content_model():
    from forma.schemas.case_study.content import CaseStudyContent
    data = {
        "meta": {"title": "Case Study", "client_name": "Acme", "date": "2026-01-01"},
        "challenge": {"statement": "They had a problem."},
        "approach": {"overview": "We did this."},
        "outcomes": {"headline": "It worked."},
    }
    content = CaseStudyContent.model_validate(data)
    assert content.meta.client_name == "Acme"


def test_schema_export():
    from forma.schemas.proposal.content import ProposalContent
    schema = ProposalContent.model_json_schema()
    assert "properties" in schema
    assert "engagement" in schema["properties"]


def test_config_loads_forma_yaml():
    from forma.core.config import FormaConfig
    config = FormaConfig.from_yaml(FIXTURE_DIR / "forma.yaml")
    assert "slides" in config.templates
    assert "report" in config.templates
    assert config.templates["slides"].mapping == "slides.yaml"
    assert config.templates["report"].mapping == "report.yaml"


def test_config_resolve_mapping_path():
    from forma.core.config import FormaConfig
    config = FormaConfig.from_yaml(FIXTURE_DIR / "forma.yaml")
    mapping = config.resolve_mapping_path("slides", FIXTURE_DIR)
    assert mapping == (FIXTURE_DIR / "slides.yaml").resolve()


def test_load_document_resolves_includes():
    """load_document resolves !include tags against the project root."""
    from forma.core.loader import load_document
    doc = load_document(FIXTURE_DIR / "slides.yaml", FIXTURE_DIR)
    assert doc["resourceType"] == "SlideDocument"
    assert doc["slides"][0]["type"] == "cover"
    assert doc["slides"][0]["title"] == "Digital Transformation Strategy"


def test_validate_project_passes_for_fixture():
    from forma.core.validator import validate_project
    result = validate_project(FIXTURE_DIR)
    assert result.ok, f"Validation errors: {result.errors}"
