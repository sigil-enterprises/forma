"""
Content and style validation with human-readable error output.

Validates content.yaml against the project's declared schema class,
checks that all asset paths referenced in the content exist on disk,
and prints structured errors via rich.
"""

from __future__ import annotations

from pathlib import Path

import yaml
from pydantic import ValidationError
from rich.console import Console

from forma.core.base import BaseContent, BaseStyle

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


def validate_content(
    content_path: Path,
    schema_cls: type[BaseContent],
    project_root: Path,
    strict: bool = False,
) -> ValidationResult:
    result = ValidationResult()

    # Load raw YAML
    try:
        with open(content_path) as f:
            raw = yaml.safe_load(f)
    except Exception as e:
        result.add_error(f"Failed to parse YAML: {e}")
        return result

    if not raw:
        result.add_error("content.yaml is empty")
        return result

    # Pydantic validation
    try:
        content = schema_cls.model_validate(raw)
    except ValidationError as e:
        for err in e.errors():
            loc = ".".join(str(p) for p in err["loc"])
            result.add_error(f"{loc}: {err['msg']}")
        return result

    # Asset path checks — walk all string values ending in common image extensions
    _check_assets(content.model_dump(), project_root, result)

    return result


def validate_style(
    style_path: Path,
    schema_cls: type[BaseStyle],
    project_root: Path,
) -> ValidationResult:
    result = ValidationResult()

    try:
        with open(style_path) as f:
            raw = yaml.safe_load(f)
    except Exception as e:
        result.add_error(f"Failed to parse style YAML: {e}")
        return result

    try:
        schema_cls.model_validate(raw or {})
    except ValidationError as e:
        for err in e.errors():
            loc = ".".join(str(p) for p in err["loc"])
            result.add_error(f"style.yaml {loc}: {err['msg']}")

    return result


def _check_assets(
    data: object,
    project_root: Path,
    result: ValidationResult,
    _path: str = "",
) -> None:
    asset_exts = {".png", ".jpg", ".jpeg", ".svg", ".pdf", ".eps"}

    if isinstance(data, dict):
        for k, v in data.items():
            _check_assets(v, project_root, result, f"{_path}.{k}" if _path else k)
    elif isinstance(data, list):
        for i, v in enumerate(data):
            _check_assets(v, project_root, result, f"{_path}[{i}]")
    elif isinstance(data, str):
        p = Path(data)
        if p.suffix.lower() in asset_exts:
            full = (project_root / data).resolve()
            if not full.exists():
                result.add_warning(f"{_path}: asset not found: {data}")
