"""
Document validation using jsonschema.

Validates YAML documents (content.yaml, slides.yaml, report.yaml, forma.yaml)
against their corresponding JSON Schema draft-07 files (stored as YAML).

The resourceType field on each document is used to look up the schema.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

import yaml
from rich.console import Console

console = Console()


class ValidationResult:
    def __init__(self) -> None:
        self.errors: list[str] = []
        self.warnings: list[str] = []

    @property
    def ok(self) -> bool:
        return len(self.errors) == 0

    def add_error(self, msg: str) -> None:
        self.errors.append(msg)

    def add_warning(self, msg: str) -> None:
        self.warnings.append(msg)

    def print(self) -> None:
        if self.errors:
            console.print(f"\n[bold red]✗ {len(self.errors)} validation error(s)[/bold red]")
            for e in self.errors:
                console.print(f"  [red]•[/red] {e}")
        if self.warnings:
            console.print(f"\n[bold yellow]⚠ {len(self.warnings)} warning(s)[/bold yellow]")
            for w in self.warnings:
                console.print(f"  [yellow]•[/yellow] {w}")
        if self.ok and not self.warnings:
            console.print("[bold green]✓ Validation passed[/bold green]")


def _load_schema(schema_path: Path) -> dict:
    """Load a JSON Schema file written in YAML format."""
    with open(schema_path, encoding="utf-8") as fh:
        return yaml.safe_load(fh)


def validate_document(
    doc: dict[str, Any],
    schema_path: Path,
    *,
    label: str = "document",
) -> ValidationResult:
    """
    Validate a loaded dict against a JSON Schema file.

    Args:
        doc:         The loaded YAML dict to validate.
        schema_path: Path to the JSON Schema YAML file.
        label:       Human-readable name for error messages.

    Returns:
        ValidationResult with any errors/warnings.
    """
    try:
        import jsonschema
    except ImportError:
        result = ValidationResult()
        result.add_warning(
            "jsonschema not installed — skipping schema validation. "
            "Run: pip install jsonschema"
        )
        return result

    result = ValidationResult()
    schema = _load_schema(schema_path)

    validator_cls = jsonschema.Draft7Validator
    validator = validator_cls(schema)

    for error in validator.iter_errors(doc):
        path = ".".join(str(p) for p in error.absolute_path) or "(root)"
        result.add_error(f"{label}: [{path}] {error.message}")

    return result


def validate_file(
    path: Path,
    base_dir: Path | None = None,
    *,
    schema_path: Path | None = None,
) -> ValidationResult:
    """
    Load a YAML file and validate it against its schema.

    The schema is determined by the resourceType field if schema_path is
    not provided explicitly.

    Args:
        path:        YAML file to validate.
        base_dir:    Project root (used for !include resolution). If None,
                     the file is loaded as plain YAML.
        schema_path: Explicit schema path. If None, looked up from registry.
    """
    from forma.core.loader import get_schema_for, load_content, load_document

    result = ValidationResult()

    if not path.exists():
        result.add_error(f"File not found: {path}")
        return result

    # Load the document
    try:
        if base_dir is not None:
            doc = load_document(path, base_dir)
        else:
            doc = load_content(path)
    except Exception as exc:
        result.add_error(f"Failed to load {path.name}: {exc}")
        return result

    if not isinstance(doc, dict):
        result.add_error(f"{path.name}: expected a YAML mapping (dict) at root")
        return result

    # Determine schema
    resolved_schema = schema_path
    if resolved_schema is None:
        rt = doc.get("resourceType")
        if rt:
            resolved_schema = get_schema_for(rt)
        if resolved_schema is None:
            result.add_warning(
                f"{path.name}: no resourceType or schema found — skipping validation"
            )
            return result

    if not resolved_schema.exists():
        result.add_warning(f"Schema file not found: {resolved_schema} — skipping")
        return result

    return validate_document(doc, resolved_schema, label=path.name)


def validate_project(project_dir: Path) -> ValidationResult:
    """
    Validate all YAML files in a project directory:
      - content.yaml
      - slides.yaml (if present)
      - report.yaml (if present)
      - forma.yaml

    Returns a combined ValidationResult.
    """
    combined = ValidationResult()

    def _merge(r: ValidationResult) -> None:
        combined.errors.extend(r.errors)
        combined.warnings.extend(r.warnings)

    # content.yaml — plain load (no !include)
    content_path = project_dir / "content.yaml"
    if content_path.exists():
        _merge(validate_file(content_path))
    else:
        combined.add_warning("No content.yaml found in project directory")

    # Mapping files — resolve !include tags relative to project_dir
    for mapping_name in ("slides.yaml", "report.yaml", "brief.yaml"):
        mapping_path = project_dir / mapping_name
        if mapping_path.exists():
            _merge(validate_file(mapping_path, base_dir=project_dir))

    # forma.yaml — plain load
    forma_path = project_dir / "forma.yaml"
    if forma_path.exists():
        _merge(validate_file(forma_path))

    return combined
