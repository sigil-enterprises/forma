"""
Tests for schema loading, validation, and Pydantic models.
"""

import pytest
from pathlib import Path

FIXTURE_DIR = Path(__file__).parent.parent / "documents" / "example-client"


def test_proposal_content_loads_from_fixture():
    from schemas.proposal.content import ProposalContent
    content = ProposalContent.from_yaml(FIXTURE_DIR / "content.yaml")
    assert content.engagement.title == "Digital Transformation Strategy"
    assert content.client.name == "Acme Corp"


def test_proposal_content_investment_totals():
    from schemas.proposal.content import ProposalContent
    content = ProposalContent.from_yaml(FIXTURE_DIR / "content.yaml")
    assert content.investment is not None
    assert content.investment.total_usd > 0
    # Phase subtotals should sum to total
    manual_total = sum(p.subtotal_usd for p in content.investment.phases)
    assert manual_total == pytest.approx(content.investment.total_usd)


def test_validate_content_passes_for_fixture():
    from schemas.proposal.content import ProposalContent
    from forma.core.validator import validate_content
    result = validate_content(
        FIXTURE_DIR / "content.yaml",
        ProposalContent,
        FIXTURE_DIR,
    )
    # Warnings about missing asset files are OK; errors are not
    assert result.ok, f"Validation errors: {result.errors}"


def test_validate_content_catches_missing_required_field(tmp_path):
    from schemas.proposal.content import ProposalContent
    from forma.core.validator import validate_content
    import yaml
    # Missing required 'client' field
    bad = {"engagement": {"title": "Test", "date": "2026-01-01"}}
    p = tmp_path / "content.yaml"
    p.write_text(yaml.dump(bad))
    result = validate_content(p, ProposalContent, tmp_path)
    assert not result.ok


def test_brief_content_model():
    from schemas.brief.content import BriefContent
    data = {
        "meta": {"title": "One-Pager", "date": "2026-01-01", "prepared_for": "Acme"},
        "sections": [{"heading": "Overview", "body": "Text here."}],
    }
    content = BriefContent.model_validate(data)
    assert content.meta.title == "One-Pager"
    assert len(content.sections) == 1


def test_case_study_content_model():
    from schemas.case_study.content import CaseStudyContent
    data = {
        "meta": {"title": "Case Study", "client_name": "Acme", "date": "2026-01-01"},
        "challenge": {"statement": "They had a problem."},
        "approach": {"overview": "We did this."},
        "outcomes": {"headline": "It worked."},
    }
    content = CaseStudyContent.model_validate(data)
    assert content.meta.client_name == "Acme"


def test_schema_export(tmp_path):
    from schemas.proposal.content import ProposalContent
    schema = ProposalContent.model_json_schema()
    assert "properties" in schema
    assert "engagement" in schema["properties"]


def test_config_loads_forma_yaml():
    from forma.core.config import FormaConfig
    config = FormaConfig.from_yaml(FIXTURE_DIR / "forma.yaml")
    assert config.schema_path == "schemas.proposal.content:ProposalContent"
    assert "slides" in config.templates
    assert "report" in config.templates


def test_loader_resolves_schema_class():
    from forma.core.loader import load_content_class
    from forma.core.base import BaseContent
    repo_root = Path(__file__).parents[1]
    cls = load_content_class("schemas.proposal.content:ProposalContent", repo_root)
    assert issubclass(cls, BaseContent)
