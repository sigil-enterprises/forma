"""
Document and style loaders.

Loads YAML files with optional !include resolution and validates them against
their JSON Schema (YAML format, draft-07) using the jsonschema library.

The resourceType field on each document is used to discover the matching
schema file from the registry.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

import yaml

from forma.core.include_loader import load_mapping


# ---------------------------------------------------------------------------
# Schema registry
# ---------------------------------------------------------------------------

# Maps resourceType value → schema file path.
# Extend this dict when adding new content or document types.
_FORMA_PKG = Path(__file__).parents[1]  # src/forma/
_REPO_ROOT = Path(__file__).parents[3]

_SCHEMA_REGISTRY: dict[str, Path] = {
    "FormaConfig":     _FORMA_PKG / "schema" / "forma-config.schema.yaml",
    "SlideDocument":   _FORMA_PKG / "schema" / "slide-document.schema.yaml",
    "ReportDocument":  _FORMA_PKG / "schema" / "report-document.schema.yaml",
    "ProposalContent": _FORMA_PKG / "schema" / "proposal-content.schema.yaml",
}


def register_schema(resource_type: str, schema_path: Path) -> None:
    """Register an additional resourceType → schema path mapping at runtime."""
    _SCHEMA_REGISTRY[resource_type] = schema_path


def load_document(path: Path, base_dir: Path) -> dict[str, Any]:
    """
    Load a YAML mapping file (slides.yaml / report.yaml), resolving all
    !include "@file:path" tags relative to base_dir.

    Does NOT validate — call validate_document() separately if needed.

    Args:
        path:     Path to the mapping YAML file.
        base_dir: Project root; all @file references resolve relative to this.

    Returns:
        Fully-resolved dict with all !include tags substituted.
    """
    return load_mapping(path, base_dir)


def load_content(path: Path) -> dict[str, Any]:
    """
    Load a plain content.yaml file (no !include tags).

    Returns the raw dict — useful for validation or reference lookups.
    """
    if not path.exists():
        raise FileNotFoundError(f"Content file not found: {path}")
    with open(path, encoding="utf-8") as fh:
        return yaml.safe_load(fh) or {}


def load_style(path: Path) -> dict[str, Any]:
    """
    Load a style.yaml file into a plain dict.
    """
    if not path.exists():
        return {}
    with open(path, encoding="utf-8") as fh:
        return yaml.safe_load(fh) or {}


def get_schema_for(resource_type: str) -> Path | None:
    """Return the schema file path for a given resourceType, or None."""
    return _SCHEMA_REGISTRY.get(resource_type)
