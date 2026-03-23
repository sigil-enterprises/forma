"""
End-to-end tests: validate → render (mocked) → publish dry-run.

These tests exercise the full CLI pipeline without requiring:
  - a live LaTeX installation (render_template is mocked)
  - Google Drive credentials (publish uses --dry-run)
  - an Anthropic API key (compose uses mocked client)

They confirm that the pipeline wires together correctly and that each
stage passes its output to the next as expected.
"""

from __future__ import annotations

import shutil
import yaml
from pathlib import Path
from unittest.mock import patch

import pytest
from typer.testing import CliRunner

from forma.cli.app import app

runner = CliRunner()

FIXTURE_DIR = Path(__file__).parent / "fixtures" / "example-client"

# The example-client forma.yaml hardcodes output_dir: ../../var/builds/example-client
_BUILDS_SUBDIR = Path("var") / "builds" / "example-client"


def _make_project(tmp_path: Path) -> Path:
    """Copy the example-client fixture into tmp_path and return the project dir."""
    project = tmp_path / "example-client"
    shutil.copytree(FIXTURE_DIR, project)
    return project


def _fake_render(tpl_path, document, style, output_path, *, project_dir=None):
    """Mock render_template: writes a stub PDF and returns the path."""
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_bytes(b"%PDF-1.4 stub")
    return output_path


# ---------------------------------------------------------------------------
# Stage 1: validate
# ---------------------------------------------------------------------------

def test_e2e_validate_passes_on_example_client():
    result = runner.invoke(app, ["validate", str(FIXTURE_DIR)])
    assert result.exit_code == 0, result.output


# ---------------------------------------------------------------------------
# Stage 2: render (mocked)
# ---------------------------------------------------------------------------

def test_e2e_render_produces_output_files(tmp_path):
    project = _make_project(tmp_path)
    out_dir = (project / "../../var/builds/example-client").resolve()

    with patch("forma.renderer.engine.render_template", side_effect=_fake_render):
        result = runner.invoke(app, ["render", str(project)])

    assert result.exit_code == 0, result.output
    assert (out_dir / "slides.pdf").exists()
    assert (out_dir / "report.pdf").exists()


def test_e2e_render_single_template(tmp_path):
    project = _make_project(tmp_path)

    with patch("forma.renderer.engine.render_template", side_effect=_fake_render) as mock_render:
        result = runner.invoke(app, ["render", "--template", "slides", str(project)])

    assert result.exit_code == 0, result.output
    assert mock_render.call_count == 1
    rendered_output = mock_render.call_args[0][3]  # 4th positional arg = output_path
    assert rendered_output.name == "slides.pdf"


# ---------------------------------------------------------------------------
# Stage 3: publish --dry-run (no Drive upload)
# ---------------------------------------------------------------------------

def test_e2e_publish_dry_run_skips_upload(tmp_path):
    project = _make_project(tmp_path)

    with patch("forma.renderer.engine.render_template", side_effect=_fake_render):
        with patch("forma.publisher.google_drive.upload_file") as mock_upload:
            result = runner.invoke(app, ["publish", str(project), "--dry-run"])

    assert result.exit_code == 0, result.output
    assert "DRY RUN" in result.output
    mock_upload.assert_not_called()


# ---------------------------------------------------------------------------
# Full pipeline: validate → render → publish dry-run in sequence
# ---------------------------------------------------------------------------

def test_e2e_full_pipeline(tmp_path):
    """
    Runs the complete pipeline in order:
      1. validate — must pass (exit 0)
      2. render   — writes stub PDFs
      3. publish  --dry-run — lists files, no upload
    """
    project = _make_project(tmp_path)
    out_dir = (project / "../../var/builds/example-client").resolve()

    # 1. Validate
    v = runner.invoke(app, ["validate", str(project)])
    assert v.exit_code == 0, f"validate failed:\n{v.output}"

    # 2. Render (mocked)
    with patch("forma.renderer.engine.render_template", side_effect=_fake_render):
        r = runner.invoke(app, ["render", str(project)])
    assert r.exit_code == 0, f"render failed:\n{r.output}"
    assert (out_dir / "slides.pdf").exists()

    # 3. Publish dry-run
    with patch("forma.renderer.engine.render_template", side_effect=_fake_render):
        with patch("forma.publisher.google_drive.upload_file") as mock_upload:
            p = runner.invoke(app, ["publish", str(project), "--dry-run"])

    assert p.exit_code == 0, f"publish failed:\n{p.output}"
    assert "DRY RUN" in p.output
    mock_upload.assert_not_called()


# ---------------------------------------------------------------------------
# compose fill → validate pipeline
# ---------------------------------------------------------------------------

def test_e2e_compose_then_validate(tmp_path):
    """Compose produces a valid content.yaml → validate confirms schema correctness."""
    project = tmp_path / "compose-client"
    project.mkdir()
    shutil.copy(FIXTURE_DIR / "forma.yaml", project / "forma.yaml")

    notes = tmp_path / "notes.md"
    notes.write_text("Client: Widget Corp. We need a digital transformation strategy.")

    composed_yaml = yaml.dump({
        "resourceType": "ProposalContent",
        "engagement": {
            "title": "Widget Corp Transformation",
            "date": "2026-03-21",
        },
        "client": {
            "name": "Widget Corp",
            "contact": {"name": "Bob Smith", "email": "bob@widget.com"},
        },
        "executive_summary": {
            "headline": "We can help Widget Corp grow.",
            "body": "Detailed executive summary here.",
        },
    })

    with patch("forma.composer.filler.FormaClient") as MockClient:
        MockClient.return_value.complete.return_value = composed_yaml
        c = runner.invoke(app, [
            "compose", "fill", str(project),
            "--notes", str(notes),
            "--overwrite",
        ])

    assert c.exit_code == 0, f"compose failed:\n{c.output}"
    assert (project / "content.yaml").exists()

    # Validate the composed output
    v = runner.invoke(app, ["validate", str(project)])
    assert v.exit_code == 0, f"validate after compose failed:\n{v.output}"
