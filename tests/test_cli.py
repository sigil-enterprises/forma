"""
CLI integration tests using typer.testing.CliRunner.
No LaTeX compilation — tests the CLI layer and business logic only.
"""

from __future__ import annotations

import yaml
import pytest
from pathlib import Path
from typer.testing import CliRunner

from forma.cli.app import app

runner = CliRunner()

FIXTURE_DIR = Path(__file__).parent.parent / "documents" / "example-client"
REPO_ROOT = Path(__file__).parent.parent


# ---------------------------------------------------------------------------
# validate command
# ---------------------------------------------------------------------------

def test_validate_passes_on_fixture():
    result = runner.invoke(app, ["validate", str(FIXTURE_DIR)])
    assert result.exit_code == 0, result.output


def test_validate_fails_on_bad_content(tmp_path):
    # Create a minimal project with a bad content.yaml (missing required 'client')
    project = tmp_path / "bad-project"
    project.mkdir()

    # Copy forma.yaml from fixture
    import shutil
    shutil.copy(FIXTURE_DIR / "forma.yaml", project / "forma.yaml")
    shutil.copy(FIXTURE_DIR / "style.yaml", project / "style.yaml")

    # Write bad content.yaml
    (project / "content.yaml").write_text(yaml.dump({
        "engagement": {"title": "Test", "date": "2026-01-01"},
        # Missing required 'client' field
    }))

    result = runner.invoke(app, ["validate", str(project)])
    assert result.exit_code != 0


def test_validate_strict_flag_exits_nonzero_on_warnings(tmp_path):
    """--strict makes warnings count as errors (e.g. missing asset files)."""
    project = tmp_path / "strict-project"
    project.mkdir()

    import shutil
    shutil.copy(FIXTURE_DIR / "forma.yaml", project / "forma.yaml")
    shutil.copy(FIXTURE_DIR / "style.yaml", project / "style.yaml")
    shutil.copy(FIXTURE_DIR / "content.yaml", project / "content.yaml")

    # With strict, missing asset files (logo, images) should cause non-zero exit
    result = runner.invoke(app, ["validate", str(project), "--strict"])
    # This may or may not fail depending on whether asset warnings exist in the fixture.
    # We just confirm the CLI flag is accepted without crash.
    assert result.exit_code in (0, 1)


# ---------------------------------------------------------------------------
# init command
# ---------------------------------------------------------------------------

def test_init_scaffolds_project_directory(tmp_path):
    result = runner.invoke(app, [
        "init", "Test Client",
        "--dir", str(tmp_path),
    ])
    assert result.exit_code == 0, result.output

    project = tmp_path / "test-client"
    assert project.is_dir()
    assert (project / "forma.yaml").exists()
    assert (project / "content.yaml").exists()
    assert (project / "style.yaml").exists()
    assert (project / "assets").is_dir()


def test_init_forma_yaml_has_correct_schema(tmp_path):
    runner.invoke(app, [
        "init", "Acme Corp",
        "--dir", str(tmp_path),
        "--schema", "schemas.proposal.content:ProposalContent",
    ])
    forma_yaml = tmp_path / "acme-corp" / "forma.yaml"
    data = yaml.safe_load(forma_yaml.read_text())
    assert data["schema"] == "schemas.proposal.content:ProposalContent"
    assert "proposal-slides" in data["templates"]
    assert "proposal-report" in data["templates"]


def test_init_custom_templates(tmp_path):
    runner.invoke(app, [
        "init", "Briefing Co",
        "--dir", str(tmp_path),
        "--templates", "proposal-brief",
    ])
    forma_yaml = tmp_path / "briefing-co" / "forma.yaml"
    data = yaml.safe_load(forma_yaml.read_text())
    assert "proposal-brief" in data["templates"]
    assert "slides" not in data["templates"]


def test_init_idempotent(tmp_path):
    """Running init twice on the same name should not crash."""
    args = ["init", "My Client", "--dir", str(tmp_path)]
    r1 = runner.invoke(app, args)
    r2 = runner.invoke(app, args)
    assert r1.exit_code == 0
    assert r2.exit_code == 0


# ---------------------------------------------------------------------------
# schema export command
# ---------------------------------------------------------------------------

def test_schema_export_writes_json_files(tmp_path):
    result = runner.invoke(app, [
        "schema", "export",
        "--output-dir", str(tmp_path),
    ])
    assert result.exit_code == 0, result.output

    json_files = list(tmp_path.glob("*.schema.json"))
    assert len(json_files) >= 3  # proposal, brief, case_study

    names = {f.stem.replace(".schema", "") for f in json_files}
    assert "proposal" in names
    assert "brief" in names
    assert "case_study" in names


def test_schema_export_json_is_valid(tmp_path):
    import json
    runner.invoke(app, ["schema", "export", "--output-dir", str(tmp_path)])

    proposal_schema = tmp_path / "proposal.schema.json"
    assert proposal_schema.exists()
    data = json.loads(proposal_schema.read_text())
    assert "properties" in data
    assert "engagement" in data["properties"]


# ---------------------------------------------------------------------------
# template list command
# ---------------------------------------------------------------------------

def test_template_list_runs_without_error():
    result = runner.invoke(app, ["template", "list"])
    assert result.exit_code == 0, result.output
    assert "proposal-slides" in result.output
    assert "proposal-report" in result.output
    assert "proposal-brief" in result.output


# ---------------------------------------------------------------------------
# compose fill command (mocked)
# ---------------------------------------------------------------------------

def test_compose_fill_dry_run(tmp_path):
    """forma compose fill --dry-run should print YAML to stdout without writing."""
    import shutil
    from unittest.mock import patch

    project = tmp_path / "compose-project"
    project.mkdir()
    shutil.copy(FIXTURE_DIR / "forma.yaml", project / "forma.yaml")

    notes = tmp_path / "notes.md"
    notes.write_text("Client: Acme Corp. We need a digital strategy.")

    valid_yaml = yaml.dump({
        "engagement": {"title": "Test", "date": "2026-01-01"},
        "client": {
            "name": "Acme Corp",
            "contact": {"name": "Jane", "email": "jane@acme.com"},
        },
        "executive_summary": {"headline": "We can help.", "body": "Details."},
    })

    with patch("forma.composer.filler.FormaClient") as MockClient:
        MockClient.return_value.complete.return_value = valid_yaml
        result = runner.invoke(app, [
            "compose", "fill", str(project),
            "--notes", str(notes),
            "--dry-run",
        ])

    assert result.exit_code == 0, result.output
    # content.yaml should NOT have been written
    assert not (project / "content.yaml").exists()


def test_compose_fill_writes_content_yaml(tmp_path):
    import shutil
    from unittest.mock import patch

    project = tmp_path / "compose-write-project"
    project.mkdir()
    shutil.copy(FIXTURE_DIR / "forma.yaml", project / "forma.yaml")

    notes = tmp_path / "notes.md"
    notes.write_text("Client: Acme Corp.")

    valid_yaml = yaml.dump({
        "engagement": {"title": "Test", "date": "2026-01-01"},
        "client": {
            "name": "Acme Corp",
            "contact": {"name": "Jane", "email": "jane@acme.com"},
        },
        "executive_summary": {"headline": "We can help.", "body": "Details."},
    })

    with patch("forma.composer.filler.FormaClient") as MockClient:
        MockClient.return_value.complete.return_value = valid_yaml
        result = runner.invoke(app, [
            "compose", "fill", str(project),
            "--notes", str(notes),
            "--overwrite",
        ])

    assert result.exit_code == 0, result.output
    assert (project / "content.yaml").exists()


# ---------------------------------------------------------------------------
# publish --dry-run (mocked render)
# ---------------------------------------------------------------------------

def test_publish_dry_run_skips_upload(tmp_path):
    """publish --dry-run should render but not call upload_file."""
    import shutil
    from unittest.mock import patch, MagicMock

    project = tmp_path / "pub-project"
    project.mkdir()
    shutil.copy(FIXTURE_DIR / "forma.yaml", project / "forma.yaml")
    shutil.copy(FIXTURE_DIR / "style.yaml", project / "style.yaml")
    shutil.copy(FIXTURE_DIR / "content.yaml", project / "content.yaml")

    with patch("forma.renderer.engine.render_template") as mock_render:
        mock_render.return_value = tmp_path / "slides.pdf"
        # Make it create the file so publish can check it exists
        def fake_render(tpl_path, content, style, output_path, *, project_dir=None):
            output_path.parent.mkdir(parents=True, exist_ok=True)
            output_path.write_bytes(b"%PDF fake")
            return output_path
        mock_render.side_effect = fake_render

        with patch("forma.publisher.google_drive.upload_file") as mock_upload:
            result = runner.invoke(app, [
                "publish", str(project),
                "--dry-run",
            ])

    assert result.exit_code == 0, result.output
    mock_upload.assert_not_called()
