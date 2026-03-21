"""
ProposalFiller: takes notes text → calls Claude → validates → writes content.yaml.
"""

from __future__ import annotations

from pathlib import Path

import yaml
from rich.console import Console

from forma.composer.client import FormaClient
from forma.composer.prompts import build_system_prompt, build_user_prompt
from forma.core.base import BaseContent

console = Console()


class FillResult:
    def __init__(self, content: BaseContent, raw_yaml: str) -> None:
        self.content = content
        self.raw_yaml = raw_yaml


def fill_from_notes(
    notes: str,
    schema_cls: type[BaseContent],
    model: str = "claude-opus-4-6",
    max_tokens: int = 8192,
    existing_yaml_path: Path | None = None,
) -> FillResult:
    """
    Send notes to Claude and return a validated content instance + raw YAML string.
    Raises ValidationError if the response doesn't conform to the schema.
    """
    existing_yaml: str | None = None
    if existing_yaml_path and existing_yaml_path.exists():
        existing_yaml = existing_yaml_path.read_text()

    client = FormaClient(model=model, max_tokens=max_tokens)
    system = build_system_prompt(schema_cls)
    user = build_user_prompt(notes, existing_yaml)

    console.print(f"[dim]Calling {model}...[/dim]")
    raw = client.complete(system, user)

    # Strip accidental markdown fences
    raw = raw.strip()
    if raw.startswith("```"):
        lines = raw.splitlines()
        raw = "\n".join(lines[1:-1] if lines[-1].strip() == "```" else lines[1:])

    # Parse + validate
    data = yaml.safe_load(raw)
    content = schema_cls.model_validate(data)

    return FillResult(content=content, raw_yaml=raw)
