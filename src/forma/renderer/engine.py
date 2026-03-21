"""
Main rendering engine.

Loads the template manifest, sets up a Jinja2 environment pointing at
the template directory, renders the Jinja2 template into LaTeX source,
then delegates compilation to the appropriate BaseRenderer subclass.
"""

from __future__ import annotations

from pathlib import Path

import yaml
from jinja2 import Environment, FileSystemLoader, StrictUndefined

from forma.core.base import BaseContent, BaseStyle
from forma.renderer.base import BaseRenderer
from forma.renderer.context import build_context
from forma.renderer.filters import FILTERS


class _XelatexRenderer(BaseRenderer):
    engine = "xelatex"


class _PdflatexRenderer(BaseRenderer):
    engine = "pdflatex"


class _LualatexRenderer(BaseRenderer):
    engine = "lualatex"


_ENGINES: dict[str, type[BaseRenderer]] = {
    "xelatex": _XelatexRenderer,
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


def render_template(
    template_dir: Path,
    content: BaseContent,
    style: BaseStyle,
    output_path: Path,
    *,
    project_dir: Path | None = None,
) -> Path:
    """
    Full pipeline: Jinja2 render → LaTeX compile → PDF at output_path.
    """
    manifest = TemplateManifest(template_dir)

    # Build Jinja2 environment with LaTeX-safe delimiters.
    # Standard {{ }} and {% %} conflict with LaTeX's brace/percent syntax.
    # We use (( )) for variables and (% %) for blocks throughout all .tex.j2 files.
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

    # Render template
    template = env.get_template(manifest.entry)
    context = build_context(content, style)
    tex_source = template.render(**context)

    # Compile
    renderer_cls = _ENGINES.get(manifest.engine)
    if renderer_cls is None:
        raise ValueError(f"Unknown LaTeX engine: {manifest.engine!r}. Choose from {list(_ENGINES)}")

    renderer = renderer_cls()
    return renderer.render(tex_source, output_path, project_dir=project_dir)
