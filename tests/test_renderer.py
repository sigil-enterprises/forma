"""
Tests for the rendering pipeline (Jinja2 context + filters).
Does NOT invoke xelatex — tests the template rendering stage only.
"""

import pytest
from pathlib import Path


def test_latex_escape_basics():
    from forma.renderer.filters import latex_escape
    assert latex_escape("Hello & World") == r"Hello \& World"
    assert latex_escape("50% off") == r"50\% off"
    assert latex_escape("cost: $100") == r"cost: \$100"
    assert latex_escape(None) == ""


def test_format_date_iso():
    from forma.renderer.filters import format_date
    assert format_date("2026-03-21") == "March 21, 2026"


def test_currency_filter():
    from forma.renderer.filters import currency
    assert currency(50000) == "$50,000"
    assert currency(1234.5, decimals=2) == "$1,234.50"


def test_hex_color_strips_hash():
    from forma.renderer.filters import hex_color
    assert hex_color("#F58220") == "F58220"
    assert hex_color("0B1D2A") == "0B1D2A"


def test_join_oxford():
    from forma.renderer.filters import join_oxford
    assert join_oxford(["a"]) == "a"
    assert join_oxford(["a", "b"]) == "a and b"
    assert join_oxford(["a", "b", "c"]) == "a, b, and c"


def test_build_context_structure():
    from schemas.proposal.content import ProposalContent
    from forma.core.base import FormaStyle
    from forma.renderer.context import build_context
    from pathlib import Path

    fixture = Path(__file__).parents[1] / "documents" / "example-client"
    content = ProposalContent.from_yaml(fixture / "content.yaml")
    style = FormaStyle.from_yaml(fixture / "style.yaml")
    ctx = build_context(content, style)

    assert "content" in ctx
    assert "style" in ctx
    assert "meta" in ctx
    assert ctx["meta"]["forma_version"] is not None


def test_jinja2_template_renders_to_string():
    """Render the slides main.tex.j2 to a string (no LaTeX compile)."""
    from schemas.proposal.content import ProposalContent
    from forma.core.base import FormaStyle
    from forma.renderer.context import build_context
    from forma.renderer.filters import FILTERS
    from jinja2 import Environment, FileSystemLoader, StrictUndefined
    from pathlib import Path

    fixture = Path(__file__).parents[1] / "documents" / "example-client"
    template_dir = Path(__file__).parents[1] / "templates" / "proposal-slides"

    content = ProposalContent.from_yaml(fixture / "content.yaml")
    style = FormaStyle.from_yaml(fixture / "style.yaml")
    ctx = build_context(content, style)

    env = Environment(
        loader=FileSystemLoader([str(template_dir), str(template_dir / "_partials")]),
        undefined=StrictUndefined,
        keep_trailing_newline=True,
        trim_blocks=True,
        lstrip_blocks=True,
        block_start_string="(%",
        block_end_string="%)",
        variable_start_string="((",
        variable_end_string="))",
        comment_start_string="(#",
        comment_end_string="#)",
    )
    for name, fn in FILTERS.items():
        env.filters[name] = fn

    tex = env.get_template("main.tex.j2").render(**ctx)
    assert r"\documentclass" in tex
    assert "Digital Transformation Strategy" in tex
    assert "Acme Corp" in tex


def _make_jinja_env(template_dir: Path):
    from forma.renderer.filters import FILTERS
    from jinja2 import Environment, FileSystemLoader, StrictUndefined

    env = Environment(
        loader=FileSystemLoader([str(template_dir), str(template_dir / "_partials")]),
        undefined=StrictUndefined,
        keep_trailing_newline=True,
        trim_blocks=True,
        lstrip_blocks=True,
        block_start_string="(%",
        block_end_string="%)",
        variable_start_string="((",
        variable_end_string="))",
        comment_start_string="(#",
        comment_end_string="#)",
    )
    for name, fn in FILTERS.items():
        env.filters[name] = fn
    return env


def test_jinja2_report_template_renders_to_string():
    """Render proposal-report main.tex.j2 to a string (no LaTeX compile)."""
    from schemas.proposal.content import ProposalContent
    from forma.core.base import FormaStyle
    from forma.renderer.context import build_context
    from pathlib import Path

    fixture = Path(__file__).parents[1] / "documents" / "example-client"
    template_dir = Path(__file__).parents[1] / "templates" / "proposal-report"

    content = ProposalContent.from_yaml(fixture / "content.yaml")
    style = FormaStyle.from_yaml(fixture / "style.yaml")
    ctx = build_context(content, style)

    env = _make_jinja_env(template_dir)
    tex = env.get_template("main.tex.j2").render(**ctx)
    assert r"\documentclass" in tex
    assert "Digital Transformation Strategy" in tex
    assert "Acme Corp" in tex


def test_jinja2_brief_template_renders_to_string():
    """Render proposal-brief main.tex.j2 with BriefContent (no LaTeX compile)."""
    from schemas.brief.content import BriefContent
    from forma.core.base import FormaStyle
    from forma.renderer.context import build_context
    from pathlib import Path

    template_dir = Path(__file__).parents[1] / "templates" / "proposal-brief"

    content = BriefContent.model_validate({
        "meta": {
            "title": "Q1 Brief",
            "subtitle": "Digital Transformation",
            "date": "2026-03-21",
            "prepared_for": "Acme Corp",
            "prepared_by": "Sliver",
        },
        "sections": [
            {"heading": "Overview", "body": "We can help.", "bullets": ["Point A", "Point B"]},
        ],
        "call_to_action": "Contact us today.",
        "contact_email": "hello@sliver.co",
    })
    style = FormaStyle()
    ctx = build_context(content, style)

    env = _make_jinja_env(template_dir)
    tex = env.get_template("main.tex.j2").render(**ctx)
    assert r"\documentclass" in tex
    assert "Q1 Brief" in tex
    assert "Acme Corp" in tex


def test_bullet_list_filter():
    from forma.renderer.filters import bullet_list

    result = bullet_list(["Alpha", "Beta", "Gamma"])
    assert r"\begin{itemize}" in result
    assert r"\item Alpha" in result
    assert r"\end{itemize}" in result


def test_bullet_list_filter_empty():
    from forma.renderer.filters import bullet_list

    result = bullet_list([])
    assert result == ""


def test_bullet_list_filter_none():
    from forma.renderer.filters import bullet_list

    result = bullet_list(None)
    assert result == ""


def test_format_date_various_formats():
    from forma.renderer.filters import format_date

    assert format_date(None) == ""
    # Unknown format: falls back to str(value)
    assert format_date("not-a-date") == "not-a-date"
