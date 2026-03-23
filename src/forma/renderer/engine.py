"""
Main rendering engine.

Loads the template manifest, sets up a Jinja2 environment pointing at
the template directory, renders the Jinja2 template into source (LaTeX or HTML),
then delegates compilation to the appropriate renderer.

Dispatch logic:
  engine: xelatex | pdflatex | lualatex  →  BaseRenderer (LaTeX subprocess)
  engine: html                            →  HtmlRenderer (Playwright PDF)
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

import yaml
from jinja2 import Environment, FileSystemLoader, StrictUndefined

from forma.renderer.base import BaseRenderer
from forma.renderer.context import build_context
from forma.renderer.filters import FILTERS


class _XelatexRenderer(BaseRenderer):
    engine = "xelatex"


class _PdflatexRenderer(BaseRenderer):
    engine = "pdflatex"


class _LualatexRenderer(BaseRenderer):
    engine = "lualatex"


_LATEX_ENGINES: dict[str, type[BaseRenderer]] = {
    "xelatex":  _XelatexRenderer,
    "pdflatex": _PdflatexRenderer,
    "lualatex": _LualatexRenderer,
}


class TemplateManifest:
    def __init__(self, template_dir: Path) -> None:
        self.template_dir = template_dir
        manifest_path = template_dir / "manifest.yaml"

        if not manifest_path.exists():
            raise FileNotFoundError(f"No manifest.yaml found in {template_dir}")

        with open(manifest_path) as f:
            data = yaml.safe_load(f) or {}

        self.name: str = data.get("name", template_dir.name)
        self.description: str = data.get("description", "")
        self.format: str = data.get("format", "document")
        self.engine: str = data.get("engine", "xelatex")
        self.entry: str = data.get("entry", "main.tex.j2")
        self.compatible_schemas: list[str] = data.get("compatible_schemas", [])


def _make_jinja_env(template_dir: Path) -> Environment:
    """
    Create a Jinja2 environment with LaTeX-safe delimiters and custom filters.

    Standard {{ }} and {% %} conflict with LaTeX brace/percent syntax.
    We use (( )) for variables, (% %) for blocks, (# #) for comments.
    HTML templates use the same delimiters for consistency.
    """
    partials = template_dir / "_slides"
    partials_alt = template_dir / "_partials"
    search_paths = [str(template_dir)]
    if partials.is_dir():
        search_paths.append(str(partials))
    if partials_alt.is_dir():
        search_paths.append(str(partials_alt))

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


def render_template(
    template_dir: Path,
    document: dict[str, Any],
    style: dict[str, Any],
    output_path: Path,
    *,
    project_dir: Path | None = None,
) -> Path:
    """
    Full pipeline: Jinja2 render → compile → PDF at output_path.

    Args:
        template_dir: Directory containing manifest.yaml and template files.
        document:     Fully-resolved mapping dict (SlideDocument / ReportDocument).
        style:        Style dict loaded from style.yaml.
        output_path:  Where to write the output PDF.
        project_dir:  Project root, used for asset resolution in LaTeX.

    Returns:
        output_path on success.
    """
    manifest = TemplateManifest(template_dir)
    env = _make_jinja_env(template_dir)

    template = env.get_template(manifest.entry)
    context = build_context(document, style)

    # Expose absolute paths for \graphicspath / asset resolution in templates.
    presskit_root = template_dir.parent.parent
    context["meta"]["project_dir"] = str(project_dir.resolve()) if project_dir else ""
    context["meta"]["presskit_root"] = str(presskit_root.resolve())

    rendered_source = template.render(**context)

    if manifest.engine == "html":
        from forma.renderer.html_renderer import HtmlRenderer
        renderer = HtmlRenderer()
        return renderer.render(rendered_source, output_path, workdir=template_dir)

    # LaTeX path
    renderer_cls = _LATEX_ENGINES.get(manifest.engine)
    if renderer_cls is None:
        raise ValueError(
            f"Unknown engine: {manifest.engine!r}. "
            f"Choose from {list(_LATEX_ENGINES)} or 'html'."
        )

    # Collect fonts + presskit root for TEXINPUTS / OSFONTDIR
    fonts_dirs: list[Path] = []
    presskit_root = template_dir.parent.parent
    fonts_candidate = presskit_root / "fonts"
    if fonts_candidate.is_dir():
        fonts_dirs.append(fonts_candidate)
    if presskit_root.is_dir():
        fonts_dirs.append(presskit_root)

    renderer = renderer_cls()
    return renderer.render(
        rendered_source,
        output_path,
        project_dir=project_dir,
        fonts_dirs=fonts_dirs or None,
    )
