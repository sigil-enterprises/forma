"""
Build the Jinja2 rendering context from content + style models.

The context exposes:
  content.*  — all fields from the content model
  style.*    — all fields from the style model
  meta.*     — rendering metadata (today's date, forma version, etc.)
"""

from __future__ import annotations

from datetime import date

from forma.core.base import BaseContent, BaseStyle, FormaStyle
from forma import __version__


def build_context(content: BaseContent, style: BaseStyle) -> dict:
    return {
        "content": content,
        "style": style,
        "meta": {
            "rendered_date": date.today().isoformat(),
            "forma_version": __version__,
        },
    }
