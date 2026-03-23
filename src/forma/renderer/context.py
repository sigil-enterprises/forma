"""
Build the Jinja2 rendering context.

The context exposes:
  document.*  — the loaded mapping document (SlideDocument / ReportDocument dict)
  style.*     — the style dict loaded from style.yaml
  meta.*      — rendering metadata (today's date, forma version, etc.)
"""

from __future__ import annotations

from datetime import date
from typing import Any

from forma import __version__


def build_context(document: dict[str, Any], style: dict[str, Any]) -> dict:
    return {
        "document": document,
        "content": document,   # alias — templates may use either name
        "style": style,
        "meta": {
            "rendered_date": date.today().isoformat(),
            "forma_version": __version__,
            "project_dir": "",
            "presskit_root": "",
        },
    }
