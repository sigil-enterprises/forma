"""
Skills submodule loader.

Discovers fetch() functions in skills/*/fetch.py and runs them,
returning a merged dict of external context data for the composer.

Each skill module must expose:
    def fetch(**kwargs) -> dict: ...
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path
from typing import Any

from rich.console import Console

console = Console()


def load_skills(
    skills_dir: Path,
    names: list[str],
    **kwargs: Any,
) -> dict[str, Any]:
    """
    Run the named skills and merge their output dicts.

    Args:
        skills_dir: path to the skills/ submodule root
        names: list of skill names to run (e.g. ["clickup", "google_docs"])
        **kwargs: passed to each skill's fetch() function
    """
    merged: dict[str, Any] = {}

    for name in names:
        fetch_path = skills_dir / name / "fetch.py"
        if not fetch_path.exists():
            console.print(f"[yellow]⚠ skill not found: {name} ({fetch_path})[/yellow]")
            continue

        spec = importlib.util.spec_from_file_location(f"skills.{name}.fetch", fetch_path)
        if spec is None or spec.loader is None:
            console.print(f"[yellow]⚠ could not load skill: {name}[/yellow]")
            continue

        module = importlib.util.module_from_spec(spec)
        sys.modules[f"skills.{name}.fetch"] = module
        spec.loader.exec_module(module)  # type: ignore[union-attr]

        if not hasattr(module, "fetch"):
            console.print(f"[yellow]⚠ skill {name}/fetch.py has no fetch() function[/yellow]")
            continue

        try:
            console.print(f"[dim]Running skill: {name}...[/dim]")
            result = module.fetch(**kwargs)
            merged[name] = result
        except Exception as e:
            console.print(f"[red]✗ skill {name} failed: {e}[/red]")

    return merged
