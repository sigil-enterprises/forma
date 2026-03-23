"""
Tests for forma.core.include_loader — custom !include YAML tag resolver.
"""

from __future__ import annotations

import textwrap
from pathlib import Path

import pytest
import yaml

from forma.core.include_loader import load_mapping, make_loader


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _write(tmp_path: Path, name: str, content: str) -> Path:
    p = tmp_path / name
    p.write_text(textwrap.dedent(content), encoding="utf-8")
    return p


# ---------------------------------------------------------------------------
# make_loader — unit tests
# ---------------------------------------------------------------------------

class TestMakeLoader:
    def test_scalar_value(self, tmp_path):
        """Resolves a simple string value from the referenced file."""
        _write(tmp_path, "data.yaml", """\
            name: Acme Corp
        """)
        _write(tmp_path, "main.yaml", """\
            company: !include "@data.yaml:name"
        """)
        result = load_mapping(tmp_path / "main.yaml", tmp_path)
        assert result["company"] == "Acme Corp"

    def test_entire_file(self, tmp_path):
        """Loads the entire referenced file when no dot-path is given."""
        _write(tmp_path, "colors.yaml", """\
            primary: "#061E30"
            accent: "#FDB62B"
        """)
        _write(tmp_path, "main.yaml", """\
            palette: !include "@colors.yaml"
        """)
        result = load_mapping(tmp_path / "main.yaml", tmp_path)
        assert result["palette"] == {"primary": "#061E30", "accent": "#FDB62B"}

    def test_nested_dot_path(self, tmp_path):
        """Traverses a nested dict using dot-separated path."""
        _write(tmp_path, "content.yaml", """\
            client:
              contact:
                email: "hello@example.com"
        """)
        _write(tmp_path, "main.yaml", """\
            email: !include "@content.yaml:client.contact.email"
        """)
        result = load_mapping(tmp_path / "main.yaml", tmp_path)
        assert result["email"] == "hello@example.com"

    def test_list_value(self, tmp_path):
        """Resolves a list value from the referenced file."""
        _write(tmp_path, "content.yaml", """\
            key_points:
              - "Point A"
              - "Point B"
              - "Point C"
        """)
        _write(tmp_path, "main.yaml", """\
            points: !include "@content.yaml:key_points"
        """)
        result = load_mapping(tmp_path / "main.yaml", tmp_path)
        assert result["points"] == ["Point A", "Point B", "Point C"]

    def test_list_index_traversal(self, tmp_path):
        """Traverses into a list using a numeric index in the dot-path."""
        _write(tmp_path, "content.yaml", """\
            items:
              - name: "first"
              - name: "second"
        """)
        _write(tmp_path, "main.yaml", """\
            second: !include "@content.yaml:items.1.name"
        """)
        result = load_mapping(tmp_path / "main.yaml", tmp_path)
        assert result["second"] == "second"

    def test_relative_path(self, tmp_path):
        """Resolves refs relative to base_dir (not the mapping file's location)."""
        sub = tmp_path / "sub"
        sub.mkdir()
        _write(tmp_path, "style.yaml", "color: navy\n")
        _write(sub, "main.yaml", """\
            c: !include "@style.yaml:color"
        """)
        # base_dir is tmp_path, so @style.yaml resolves there
        result = load_mapping(sub / "main.yaml", tmp_path)
        assert result["c"] == "navy"

    def test_file_caching(self, tmp_path, mocker):
        """The same file is opened only once per load_mapping() call."""
        _write(tmp_path, "data.yaml", "x: 1\ny: 2\n")
        _write(tmp_path, "main.yaml", """\
            a: !include "@data.yaml:x"
            b: !include "@data.yaml:y"
        """)
        open_spy = mocker.patch("builtins.open", wraps=open)
        load_mapping(tmp_path / "main.yaml", tmp_path)

        # data.yaml should be opened once; main.yaml also opened once
        data_opens = [
            c for c in open_spy.call_args_list
            if "data.yaml" in str(c.args[0])
        ]
        assert len(data_opens) == 1

    def test_missing_file_raises(self, tmp_path):
        """FileNotFoundError if the @-referenced file does not exist."""
        _write(tmp_path, "main.yaml", """\
            x: !include "@missing.yaml:key"
        """)
        with pytest.raises(FileNotFoundError, match="missing.yaml"):
            load_mapping(tmp_path / "main.yaml", tmp_path)

    def test_missing_key_raises(self, tmp_path):
        """KeyError if the dot-path key does not exist in the file."""
        _write(tmp_path, "data.yaml", "a: 1\n")
        _write(tmp_path, "main.yaml", """\
            x: !include "@data.yaml:b"
        """)
        with pytest.raises(KeyError, match="'b' not found"):
            load_mapping(tmp_path / "main.yaml", tmp_path)

    def test_missing_list_index_raises(self, tmp_path):
        """KeyError if a numeric list index is out of range."""
        _write(tmp_path, "data.yaml", "items: [1, 2]\n")
        _write(tmp_path, "main.yaml", """\
            x: !include "@data.yaml:items.5"
        """)
        with pytest.raises(KeyError, match="list index"):
            load_mapping(tmp_path / "main.yaml", tmp_path)

    def test_invalid_include_value_raises(self, tmp_path):
        """ValueError if the !include value doesn't start with '@'."""
        _write(tmp_path, "main.yaml", """\
            x: !include "data.yaml:key"
        """)
        with pytest.raises(ValueError, match="must start with '@'"):
            load_mapping(tmp_path / "main.yaml", tmp_path)

    def test_traverse_none_raises(self, tmp_path):
        """KeyError with helpful message when traversal hits a None value."""
        _write(tmp_path, "data.yaml", "a: ~\n")
        _write(tmp_path, "main.yaml", """\
            x: !include "@data.yaml:a.b"
        """)
        with pytest.raises(KeyError, match="parent is None"):
            load_mapping(tmp_path / "main.yaml", tmp_path)

    def test_traverse_scalar_raises(self, tmp_path):
        """KeyError when trying to traverse into a scalar value."""
        _write(tmp_path, "data.yaml", "a: 42\n")
        _write(tmp_path, "main.yaml", """\
            x: !include "@data.yaml:a.b"
        """)
        with pytest.raises(KeyError, match="not a dict"):
            load_mapping(tmp_path / "main.yaml", tmp_path)

    def test_fresh_cache_per_call(self, tmp_path):
        """Each load_mapping() call gets an independent file cache."""
        _write(tmp_path, "data.yaml", "v: 1\n")
        _write(tmp_path, "main.yaml", "x: !include \"@data.yaml:v\"\n")

        # Two independent calls — both succeed independently
        r1 = load_mapping(tmp_path / "main.yaml", tmp_path)
        r2 = load_mapping(tmp_path / "main.yaml", tmp_path)
        assert r1 == r2 == {"x": 1}


# ---------------------------------------------------------------------------
# load_mapping — integration tests
# ---------------------------------------------------------------------------

class TestLoadMapping:
    def test_mapping_not_found_raises(self, tmp_path):
        """FileNotFoundError if the mapping file itself does not exist."""
        with pytest.raises(FileNotFoundError, match="Mapping file not found"):
            load_mapping(tmp_path / "nonexistent.yaml", tmp_path)

    def test_empty_mapping_returns_empty_dict(self, tmp_path):
        """An empty YAML file returns an empty dict (not None)."""
        p = tmp_path / "empty.yaml"
        p.write_text("", encoding="utf-8")
        result = load_mapping(p, tmp_path)
        assert result == {}

    def test_plain_yaml_no_includes(self, tmp_path):
        """A mapping file with no !include tags is loaded as plain YAML."""
        _write(tmp_path, "main.yaml", """\
            title: "Hello"
            count: 42
        """)
        result = load_mapping(tmp_path / "main.yaml", tmp_path)
        assert result == {"title": "Hello", "count": 42}

    def test_multiple_includes_from_same_file(self, tmp_path):
        """Multiple !include tags referencing the same file all resolve."""
        _write(tmp_path, "content.yaml", """\
            client:
              name: "Acme"
              city: "London"
        """)
        _write(tmp_path, "main.yaml", """\
            name: !include "@content.yaml:client.name"
            city: !include "@content.yaml:client.city"
        """)
        result = load_mapping(tmp_path / "main.yaml", tmp_path)
        assert result == {"name": "Acme", "city": "London"}
