"""
Tests for the rendering pipeline (Jinja2 context + filters).
Does NOT invoke xelatex or Playwright — tests template rendering only.
"""

import pytest
from pathlib import Path

FIXTURE_DIR = Path(__file__).parent / "fixtures" / "example-client"
REPO_ROOT = Path(__file__).parent.parent


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
    """build_context returns document/style/meta keys."""
    from forma.renderer.context import build_context
    document = {"resourceType": "SlideDocument", "slides": [{"type": "cover", "title": "Test"}]}
    style = {"colors": {"primary_dark": "#061E30"}}
    ctx = build_context(document, style)

    assert "document" in ctx
    assert "style" in ctx
    assert "meta" in ctx
    assert ctx["meta"]["forma_version"] is not None
    assert ctx["document"]["slides"][0]["title"] == "Test"


def _make_jinja_env(template_dir: Path):
    from forma.renderer.filters import FILTERS
    from jinja2 import Environment, FileSystemLoader, StrictUndefined

    search_paths = [str(template_dir)]
    for sub in ("_partials", "_slides"):
        if (template_dir / sub).is_dir():
            search_paths.append(str(template_dir / sub))

    env = Environment(
        loader=FileSystemLoader(search_paths),
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
        autoescape=False,
    )
    for name, fn in FILTERS.items():
        env.filters[name] = fn
    return env


def test_html_slides_template_renders():
    """Render the HTML slides main.html.j2 to a string (no Playwright)."""
    from forma.core.loader import load_document, load_style
    from forma.renderer.context import build_context

    template_dir = Path(__file__).parent / "fixtures" / "templates" / "proposal-slides-html"
    if not template_dir.exists():
        pytest.skip("proposal-slides-html template not found")

    doc = load_document(FIXTURE_DIR / "slides.yaml", FIXTURE_DIR)
    style = load_style(Path(__file__).parent / "fixtures" / "templates" / "style.yaml")
    ctx = build_context(doc, style)

    env = _make_jinja_env(template_dir)
    html = env.get_template("main.html.j2").render(**ctx)

    assert "<!DOCTYPE html>" in html
    assert "Digital Transformation Strategy" in html
    assert "Acme Corp" in html


def test_report_template_renders():
    """Render the LaTeX report main.tex.j2 to a string (no xelatex)."""
    from forma.core.loader import load_document, load_style
    from forma.renderer.context import build_context

    template_dir = Path(__file__).parent / "fixtures" / "templates" / "proposal-report"
    if not template_dir.exists():
        pytest.skip("proposal-report template not found")

    doc = load_document(FIXTURE_DIR / "content.yaml", FIXTURE_DIR)
    style = load_style(Path(__file__).parent / "fixtures" / "templates" / "style.yaml")
    ctx = build_context(doc, style)

    env = _make_jinja_env(template_dir)
    tex = env.get_template("main.tex.j2").render(**ctx)

    assert r"\documentclass" in tex
    assert "Digital Transformation Strategy" in tex


def test_bullet_list_filter():
    from forma.renderer.filters import bullet_list
    result = bullet_list(["Alpha", "Beta", "Gamma"])
    assert r"\begin{itemize}" in result
    assert r"\item Alpha" in result
    assert r"\end{itemize}" in result


def test_bullet_list_filter_empty():
    from forma.renderer.filters import bullet_list
    assert bullet_list([]) == ""


def test_bullet_list_filter_none():
    from forma.renderer.filters import bullet_list
    assert bullet_list(None) == ""


def test_format_date_various_formats():
    from forma.renderer.filters import format_date
    assert format_date(None) == ""
    assert format_date("not-a-date") == "not-a-date"
