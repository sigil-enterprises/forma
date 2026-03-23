"""
Custom YAML loader that resolves !include tags.

Syntax:
  !include "@content.yaml:dot.path.to.key"   — extract specific key from a file
  !include "@content.yaml"                   — load entire file
  !include "@../clients/sigil/style.yaml:colors.primary_dark"  — relative paths ok

The base_dir is always the project root (where forma.yaml lives), so
@content.yaml resolves to <project_root>/content.yaml regardless of which
mapping file is being loaded.

Files are cached within a single load_mapping() call to avoid re-reading the
same content.yaml for every !include tag in slides.yaml.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

import yaml


def make_loader(base_dir: Path) -> type:
    """
    Return a yaml.SafeLoader subclass that resolves !include "@file:path" tags.

    Each call produces a fresh subclass with its own file cache, so concurrent
    loads don't share state.

    Args:
        base_dir: Project root. All @file references are resolved relative to it.
    """
    _cache: dict[str, Any] = {}

    def _include(loader: yaml.SafeLoader, node: yaml.ScalarNode) -> Any:
        value = loader.construct_scalar(node)

        if not isinstance(value, str) or not value.startswith("@"):
            raise ValueError(
                f"!include value must start with '@', got: {value!r}\n"
                "Usage: !include \"@filename.yaml\" or !include \"@filename.yaml:dot.path\""
            )

        ref = value[1:]  # strip leading @

        # Split file path from optional dot-path on first ':'
        if ":" in ref:
            file_ref, dot_path = ref.split(":", 1)
        else:
            file_ref, dot_path = ref, None

        # Resolve relative to base_dir
        file_path = (base_dir / file_ref).resolve()

        if not file_path.exists():
            raise FileNotFoundError(
                f"!include referenced file not found: {file_path}\n"
                f"  base_dir : {base_dir}\n"
                f"  ref      : {file_ref!r}"
            )

        # Load and cache (plain safe_load — no nested !include support)
        cache_key = str(file_path)
        if cache_key not in _cache:
            with open(file_path, encoding="utf-8") as fh:
                _cache[cache_key] = yaml.safe_load(fh)

        data = _cache[cache_key]

        # Traverse dot-path if given
        if dot_path:
            for part in dot_path.split("."):
                if data is None:
                    raise KeyError(
                        f"!include: cannot traverse '{part}' — parent is None\n"
                        f"  full ref: {value!r}"
                    )
                if isinstance(data, list):
                    try:
                        data = data[int(part)]
                    except (ValueError, IndexError) as exc:
                        raise KeyError(
                            f"!include: list index '{part}' is invalid in {value!r}"
                        ) from exc
                elif isinstance(data, dict):
                    if part not in data:
                        available = list(data.keys())
                        raise KeyError(
                            f"!include: key '{part}' not found in {value!r}\n"
                            f"  available keys: {available}"
                        )
                    data = data[part]
                else:
                    raise KeyError(
                        f"!include: cannot traverse '{part}' — "
                        f"value is {type(data).__name__}, not a dict\n"
                        f"  full ref: {value!r}"
                    )

        return data

    class _IncludeLoader(yaml.SafeLoader):
        pass

    _IncludeLoader.add_constructor("!include", _include)
    return _IncludeLoader


def load_mapping(mapping_path: Path, base_dir: Path) -> dict:
    """
    Load a YAML mapping file, resolving all !include tags against base_dir.

    Args:
        mapping_path: Path to slides.yaml or report.yaml.
        base_dir: Project root; all @file references resolve relative to this.

    Returns:
        Fully-resolved dict (all !include tags substituted).

    Raises:
        FileNotFoundError: mapping_path or any included file does not exist.
        KeyError: a dot-path in an !include tag does not exist in the target file.
    """
    if not mapping_path.exists():
        raise FileNotFoundError(f"Mapping file not found: {mapping_path}")

    loader_cls = make_loader(base_dir)
    with open(mapping_path, encoding="utf-8") as fh:
        result = yaml.load(fh, Loader=loader_cls)  # noqa: S506 — custom loader, not unsafe

    return result or {}
