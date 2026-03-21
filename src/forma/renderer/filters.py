"""
Jinja2 filters for LaTeX rendering.

All string content from YAML must pass through latex_escape before
being embedded in .tex output. Other filters handle formatting.
"""

from __future__ import annotations

import re
from datetime import date, datetime
from typing import Any


_LATEX_SPECIAL = {
    "&":  r"\&",
    "%":  r"\%",
    "$":  r"\$",
    "#":  r"\#",
    "_":  r"\_",
    "{":  r"\{",
    "}":  r"\}",
    "~":  r"\textasciitilde{}",
    "^":  r"\textasciicircum{}",
    "\\": r"\textbackslash{}",
}

_LATEX_RE = re.compile(
    "|".join(re.escape(k) for k in sorted(_LATEX_SPECIAL, key=len, reverse=True))
)


def latex_escape(value: Any) -> str:
    """Escape a value for safe inclusion in LaTeX source."""
    if value is None:
        return ""
    s = str(value)
    return _LATEX_RE.sub(lambda m: _LATEX_SPECIAL[m.group()], s)


def format_date(value: Any, fmt: str = "%B %d, %Y") -> str:
    """Format a date string or date object. Returns '' for None."""
    if value is None:
        return ""
    if isinstance(value, (date, datetime)):
        return value.strftime(fmt)
    if isinstance(value, str):
        for pattern in ("%Y-%m-%d", "%d/%m/%Y", "%d-%m-%Y", "%B %d, %Y"):
            try:
                return datetime.strptime(value, pattern).strftime(fmt)
            except ValueError:
                continue
    return str(value)


def currency(value: Any, symbol: str = "$", decimals: int = 0) -> str:
    """Format a number as currency."""
    try:
        n = float(value)
        if decimals == 0:
            return f"{symbol}{n:,.0f}"
        return f"{symbol}{n:,.{decimals}f}"
    except (TypeError, ValueError):
        return str(value)


def join_oxford(items: list[str], conjunction: str = "and") -> str:
    """Join a list with Oxford comma."""
    if not items:
        return ""
    if len(items) == 1:
        return items[0]
    if len(items) == 2:
        return f"{items[0]} {conjunction} {items[1]}"
    return ", ".join(items[:-1]) + f", {conjunction} {items[-1]}"


def hex_color(value: str) -> str:
    """Strip leading # from a hex color for LaTeX xcolor."""
    return value.lstrip("#")


def bullet_list(items: list[str] | None, indent: int = 0) -> str:
    """Render a list as LaTeX itemize environment. Returns '' for empty/None."""
    if not items:
        return ""
    pad = "  " * indent
    lines = [f"{pad}\\begin{{itemize}}"]
    for item in items:
        lines.append(f"{pad}  \\item {latex_escape(item)}")
    lines.append(f"{pad}\\end{{itemize}}")
    return "\n".join(lines)


# Registry for Jinja2 environment
FILTERS: dict[str, Any] = {
    "latex_escape": latex_escape,
    "le": latex_escape,  # short alias
    "format_date": format_date,
    "currency": currency,
    "join_oxford": join_oxford,
    "hex_color": hex_color,
    "bullet_list": bullet_list,
}
