"""
Tests for the Claude composer (v0.4): prompts and filler logic.
All calls to the Anthropic API are mocked — no live API key required.
"""

from __future__ import annotations

import yaml
import pytest
from pathlib import Path
from unittest.mock import MagicMock, patch

FIXTURE_DIR = Path(__file__).parent.parent / "documents" / "example-client"


# ---------------------------------------------------------------------------
# build_system_prompt
# ---------------------------------------------------------------------------

def test_build_system_prompt_contains_schema():
    from schemas.proposal.content import ProposalContent
    from forma.composer.prompts import build_system_prompt

    prompt = build_system_prompt(ProposalContent)
    assert "JSON Schema" in prompt
    assert "engagement" in prompt  # field from ProposalContent
    assert "YAML" in prompt


def test_build_system_prompt_embeds_full_json_schema():
    from schemas.proposal.content import ProposalContent
    from forma.composer.prompts import build_system_prompt
    import json

    prompt = build_system_prompt(ProposalContent)
    # The embedded JSON schema should be parseable
    schema_start = prompt.index("JSON Schema:\n") + len("JSON Schema:\n")
    embedded = prompt[schema_start:]
    parsed = json.loads(embedded)
    assert "properties" in parsed


# ---------------------------------------------------------------------------
# build_user_prompt
# ---------------------------------------------------------------------------

def test_build_user_prompt_wraps_notes():
    from forma.composer.prompts import build_user_prompt

    prompt = build_user_prompt("My meeting notes here.")
    assert "My meeting notes here." in prompt
    assert "YAML" in prompt


def test_build_user_prompt_includes_existing_yaml():
    from forma.composer.prompts import build_user_prompt

    prompt = build_user_prompt("Notes.", existing_yaml="existing: true")
    assert "existing: true" in prompt
    assert "build on" in prompt.lower() or "preserve" in prompt.lower()


def test_build_user_prompt_no_existing_yaml():
    from forma.composer.prompts import build_user_prompt

    prompt = build_user_prompt("Notes.")
    assert "existing" not in prompt.lower() or "build on" not in prompt.lower()


# ---------------------------------------------------------------------------
# fill_from_notes — mocked client
# ---------------------------------------------------------------------------

def _minimal_proposal_yaml() -> str:
    """Minimal valid ProposalContent YAML for tests."""
    return yaml.dump({
        "engagement": {
            "title": "Test Engagement",
            "date": "2026-01-01",
        },
        "client": {
            "name": "Test Client",
            "contact": {
                "name": "Jane Doe",
                "email": "jane@example.com",
            },
        },
        "executive_summary": {
            "headline": "We can help.",
            "body": "Lorem ipsum.",
        },
    })


def test_filler_returns_fill_result_on_valid_output():
    from schemas.proposal.content import ProposalContent
    from forma.composer.filler import fill_from_notes, FillResult

    valid_yaml = _minimal_proposal_yaml()

    with patch("forma.composer.filler.FormaClient") as MockClient:
        instance = MockClient.return_value
        instance.complete.return_value = valid_yaml

        result = fill_from_notes(
            notes="Some meeting notes.",
            schema_cls=ProposalContent,
        )

    assert isinstance(result, FillResult)
    assert result.content.client.name == "Test Client"
    assert result.raw_yaml == valid_yaml.strip()


def test_filler_strips_markdown_fences():
    from schemas.proposal.content import ProposalContent
    from forma.composer.filler import fill_from_notes

    valid_yaml = _minimal_proposal_yaml()
    fenced = f"```yaml\n{valid_yaml}\n```"

    with patch("forma.composer.filler.FormaClient") as MockClient:
        instance = MockClient.return_value
        instance.complete.return_value = fenced

        result = fill_from_notes(
            notes="Notes.",
            schema_cls=ProposalContent,
        )

    assert result.content.client.name == "Test Client"


def test_filler_strips_markdown_fences_without_lang_tag():
    from schemas.proposal.content import ProposalContent
    from forma.composer.filler import fill_from_notes

    valid_yaml = _minimal_proposal_yaml()
    fenced = f"```\n{valid_yaml}\n```"

    with patch("forma.composer.filler.FormaClient") as MockClient:
        instance = MockClient.return_value
        instance.complete.return_value = fenced

        result = fill_from_notes(
            notes="Notes.",
            schema_cls=ProposalContent,
        )

    assert result.content.client.name == "Test Client"


def test_filler_raises_on_bad_yaml_output():
    from schemas.proposal.content import ProposalContent
    from forma.composer.filler import fill_from_notes

    with patch("forma.composer.filler.FormaClient") as MockClient:
        instance = MockClient.return_value
        instance.complete.return_value = "this: is: not: valid: yaml: ["

        with pytest.raises(Exception):
            fill_from_notes(notes="Notes.", schema_cls=ProposalContent)


def test_filler_raises_on_schema_violation():
    """Claude returns valid YAML but it doesn't match the schema."""
    from schemas.proposal.content import ProposalContent
    from forma.composer.filler import fill_from_notes
    from pydantic import ValidationError

    # Missing required 'client' field
    bad_yaml = yaml.dump({
        "engagement": {"title": "Test", "date": "2026-01-01"},
    })

    with patch("forma.composer.filler.FormaClient") as MockClient:
        instance = MockClient.return_value
        instance.complete.return_value = bad_yaml

        with pytest.raises(ValidationError):
            fill_from_notes(notes="Notes.", schema_cls=ProposalContent)


def test_filler_loads_existing_yaml(tmp_path):
    """When existing_yaml_path exists, its content is passed to the user prompt."""
    from schemas.proposal.content import ProposalContent
    from forma.composer.filler import fill_from_notes
    from forma.composer.prompts import build_user_prompt

    valid_yaml = _minimal_proposal_yaml()
    existing = tmp_path / "content.yaml"
    existing.write_text("existing_field: preserved")

    captured_user_prompt = []

    def fake_complete(system, user):
        captured_user_prompt.append(user)
        return valid_yaml

    with patch("forma.composer.filler.FormaClient") as MockClient:
        instance = MockClient.return_value
        instance.complete.side_effect = fake_complete

        fill_from_notes(
            notes="Notes.",
            schema_cls=ProposalContent,
            existing_yaml_path=existing,
        )

    assert "existing_field: preserved" in captured_user_prompt[0]
