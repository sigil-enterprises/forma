"""
Tests for the skills submodule loader (integrations/skills_loader.py).
"""

from __future__ import annotations

import pytest
from pathlib import Path
from unittest.mock import patch

SKILLS_DIR = Path(__file__).parent.parent / "skills"


# ---------------------------------------------------------------------------
# load_skills — happy path
# ---------------------------------------------------------------------------

def test_load_skills_returns_empty_dict_for_empty_list(tmp_path):
    from forma.integrations.skills_loader import load_skills

    result = load_skills(tmp_path, [])
    assert result == {}


def test_load_skills_warns_on_missing_skill_dir(tmp_path, capsys):
    from forma.integrations.skills_loader import load_skills

    result = load_skills(tmp_path, ["nonexistent_skill"])
    assert result == {}


def test_load_skills_warns_on_missing_fetch_function(tmp_path):
    """A skill fetch.py with no fetch() should be skipped gracefully."""
    skill_dir = tmp_path / "mocked_skill"
    skill_dir.mkdir()
    fetch_py = skill_dir / "fetch.py"
    fetch_py.write_text("# no fetch function here\nsome_var = 42\n")

    from forma.integrations.skills_loader import load_skills
    result = load_skills(tmp_path, ["mocked_skill"])
    assert result == {}


def test_load_skills_calls_fetch_and_returns_result(tmp_path):
    """A skill fetch.py with a working fetch() should be called and its result returned."""
    skill_dir = tmp_path / "mock_skill"
    skill_dir.mkdir()
    (skill_dir / "fetch.py").write_text(
        "def fetch(**kwargs):\n"
        "    return {'tasks': ['task-1', 'task-2']}\n"
    )

    from forma.integrations.skills_loader import load_skills
    result = load_skills(tmp_path, ["mock_skill"])
    assert "mock_skill" in result
    assert result["mock_skill"] == {"tasks": ["task-1", "task-2"]}


def test_load_skills_continues_on_fetch_exception(tmp_path):
    """If fetch() raises, the skill is skipped and other skills still run."""
    failing_dir = tmp_path / "failing_skill"
    failing_dir.mkdir()
    (failing_dir / "fetch.py").write_text(
        "def fetch(**kwargs):\n"
        "    raise RuntimeError('external API is down')\n"
    )

    ok_dir = tmp_path / "ok_skill"
    ok_dir.mkdir()
    (ok_dir / "fetch.py").write_text(
        "def fetch(**kwargs):\n"
        "    return {'ok': True}\n"
    )

    from forma.integrations.skills_loader import load_skills
    result = load_skills(tmp_path, ["failing_skill", "ok_skill"])
    assert "failing_skill" not in result
    assert result.get("ok_skill") == {"ok": True}


def test_load_skills_passes_kwargs_to_fetch(tmp_path):
    """kwargs passed to load_skills are forwarded to each skill's fetch()."""
    skill_dir = tmp_path / "kwarg_skill"
    skill_dir.mkdir()
    (skill_dir / "fetch.py").write_text(
        "def fetch(**kwargs):\n"
        "    return {'received': kwargs}\n"
    )

    from forma.integrations.skills_loader import load_skills
    result = load_skills(tmp_path, ["kwarg_skill"], list_id="ABC123")
    assert result["kwarg_skill"]["received"] == {"list_id": "ABC123"}


# ---------------------------------------------------------------------------
# meeting_notes skill (if submodule is present)
# ---------------------------------------------------------------------------

@pytest.mark.skipif(
    not (SKILLS_DIR / "meeting_notes" / "fetch.py").exists(),
    reason="skills submodule not checked out",
)
def test_meeting_notes_skill_parses_markdown(tmp_path):
    """meeting_notes/fetch.py parses a structured markdown meeting note."""
    import importlib.util, sys

    fetch_path = SKILLS_DIR / "meeting_notes" / "fetch.py"
    spec = importlib.util.spec_from_file_location("skills.meeting_notes.fetch", fetch_path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)

    notes_file = tmp_path / "meeting.md"
    notes_file.write_text(
        "# Meeting: Q1 Strategy Review\n"
        "Date: 2026-03-21\n"
        "Attendees: Alice, Bob, Carol\n\n"
        "## Action Items\n"
        "- [x] Send proposal draft\n"
        "- [ ] Schedule follow-up\n\n"
        "## Notes\n"
        "We agreed on the Q1 roadmap.\n"
    )

    result = module.fetch(path=str(notes_file))

    assert result.get("title") == "Q1 Strategy Review"
    assert result.get("date") == "2026-03-21"
    assert "Alice" in result.get("attendees", [])
    assert len(result.get("action_items", [])) == 2
