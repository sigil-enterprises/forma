"""
forma CLI — schema-agnostic document rendering framework.

Commands:
  validate   Validate content.yaml + style.yaml against the project schema
  render     Render one or all templates to PDF
  compose    AI-assisted content drafting (fill / enrich)
  publish    Render + upload artifacts to Google Drive
  schema     Export JSON Schema files from Pydantic models
  template   List available templates
  init       Scaffold a new document project directory
"""

from __future__ import annotations

import sys
from pathlib import Path
from typing import Optional

import typer
import yaml
from rich.console import Console
from rich.table import Table

app = typer.Typer(
    name="forma",
    help="Schema-agnostic document rendering framework.",
    no_args_is_help=True,
)
compose_app = typer.Typer(help="AI-assisted content composition.")
app.add_typer(compose_app, name="compose")

render_app = typer.Typer(help="Render templates to PDF.")
app.add_typer(render_app, name="render")

schema_app = typer.Typer(help="Schema utilities.")
app.add_typer(schema_app, name="schema")

template_app = typer.Typer(help="Template utilities.")
app.add_typer(template_app, name="template")

console = Console()


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _load_project(project_dir: Path):
    from forma.core.config import FormaConfig
    from forma.core.loader import load_content_class

    config_path = project_dir / "forma.yaml"
    if not config_path.exists():
        console.print(f"[red]✗ No forma.yaml found in {project_dir}[/red]")
        raise typer.Exit(1)

    config = FormaConfig.from_yaml(config_path)
    schema_cls = load_content_class(config.schema_path, project_root=project_dir)
    return config, schema_cls


# ---------------------------------------------------------------------------
# validate
# ---------------------------------------------------------------------------

@app.command()
def validate(
    project_dir: Path = typer.Argument(Path("."), help="Document project directory"),
    content_file: str = typer.Option("content.yaml", "--content", "-c"),
    style_file: Optional[str] = typer.Option(None, "--style", "-s"),
    strict: bool = typer.Option(False, "--strict", help="Treat warnings as errors"),
):
    """Validate content.yaml (and optionally style.yaml) against the declared schema."""
    from forma.core.validator import validate_content

    project_dir = project_dir.resolve()
    config, schema_cls = _load_project(project_dir)

    content_path = project_dir / content_file
    result = validate_content(content_path, schema_cls, project_dir, strict=strict)
    result.print()

    if not result.ok or (strict and result.warnings):
        raise typer.Exit(1)


# ---------------------------------------------------------------------------
# render
# ---------------------------------------------------------------------------

@render_app.callback(invoke_without_command=True)
def render_default(
    ctx: typer.Context,
    project_dir: Path = typer.Argument(Path("."), help="Document project directory"),
    template_name: Optional[str] = typer.Option(None, "--template", "-t", help="Render a specific template"),
    content_file: str = typer.Option("content.yaml", "--content", "-c"),
    watch: bool = typer.Option(False, "--watch", "-w", help="Re-render on file change"),
):
    """Render all templates (or a specific one) declared in forma.yaml."""
    if ctx.invoked_subcommand is not None:
        return

    project_dir = project_dir.resolve()
    config, schema_cls = _load_project(project_dir)

    def _do_render():
        from forma.core.config import FormaConfig
        from forma.core.loader import load_content_class
        from forma.core.base import FormaStyle
        from forma.renderer.engine import render_template

        cfg = FormaConfig.from_yaml(project_dir / "forma.yaml")
        cls = load_content_class(cfg.schema_path, project_root=project_dir)

        content_path = project_dir / content_file
        content = cls.from_yaml(content_path)

        style_path = cfg.resolve_style_path(project_dir)
        style = FormaStyle.from_yaml(style_path) if style_path.exists() else FormaStyle()

        output_dir = cfg.resolve_output_dir(project_dir)
        templates = {template_name: cfg.templates[template_name]} if template_name else cfg.templates

        for name, entry in templates.items():
            tpl_path = config.resolve_template_path(name, project_dir)
            out = output_dir / f"{name}.pdf"
            console.print(f"[dim]Rendering {name}...[/dim]")
            render_template(tpl_path, content, style, out)

    _do_render()

    if watch:
        from watchfiles import watch as wf
        console.print("[dim]Watching for changes (Ctrl-C to stop)...[/dim]")
        for _ in wf(str(project_dir)):
            _do_render()


# ---------------------------------------------------------------------------
# compose fill
# ---------------------------------------------------------------------------

@compose_app.command("fill")
def compose_fill(
    project_dir: Path = typer.Argument(Path("."), help="Document project directory"),
    notes_file: Path = typer.Option(..., "--notes", "-n", help="Path to notes file"),
    model: str = typer.Option("claude-opus-4-6", "--model", "-m"),
    max_tokens: int = typer.Option(8192, "--max-tokens"),
    dry_run: bool = typer.Option(False, "--dry-run", help="Print YAML without writing"),
    overwrite: bool = typer.Option(False, "--overwrite", help="Overwrite existing content.yaml without prompting"),
):
    """Draft content.yaml from notes using Claude."""
    from forma.composer.filler import fill_from_notes

    project_dir = project_dir.resolve()
    config, schema_cls = _load_project(project_dir)

    notes = notes_file.read_text()
    existing_path = project_dir / "content.yaml"

    result = fill_from_notes(
        notes=notes,
        schema_cls=schema_cls,
        model=model,
        max_tokens=max_tokens,
        existing_yaml_path=existing_path if existing_path.exists() else None,
    )

    if dry_run:
        console.print(result.raw_yaml)
        return

    if existing_path.exists() and not overwrite:
        confirm = typer.confirm(f"content.yaml already exists in {project_dir}. Overwrite?")
        if not confirm:
            raise typer.Abort()

    existing_path.write_text(result.raw_yaml)
    console.print(f"[green]✓ Wrote[/green] {existing_path}")


# ---------------------------------------------------------------------------
# compose enrich
# ---------------------------------------------------------------------------

@compose_app.command("enrich")
def compose_enrich(
    project_dir: Path = typer.Argument(Path("."), help="Document project directory"),
    skills: str = typer.Option(..., "--skills", "-s", help="Comma-separated skill names, e.g. clickup,google_docs"),
    notes_file: Optional[Path] = typer.Option(None, "--notes", "-n"),
    model: str = typer.Option("claude-opus-4-6", "--model", "-m"),
    dry_run: bool = typer.Option(False, "--dry-run"),
):
    """Fetch external data via skills, merge with notes, compose content.yaml."""
    from forma.integrations.skills_loader import load_skills
    from forma.composer.filler import fill_from_notes

    project_dir = project_dir.resolve()
    config, schema_cls = _load_project(project_dir)

    # Find skills dir (submodule)
    repo_root = Path(__file__).parents[4]
    skills_dir = repo_root / "skills"
    if not skills_dir.exists():
        console.print(f"[red]✗ skills/ submodule not found at {skills_dir}[/red]")
        raise typer.Exit(1)

    skill_names = [s.strip() for s in skills.split(",")]
    fetched = load_skills(skills_dir, skill_names)

    # Build combined notes
    combined_parts = []
    if notes_file and notes_file.exists():
        combined_parts.append(notes_file.read_text())

    for name, data in fetched.items():
        combined_parts.append(f"\n\n## {name} data\n{yaml.dump(data, default_flow_style=False)}")

    combined_notes = "\n".join(combined_parts)
    if not combined_notes.strip():
        console.print("[red]✗ No notes or fetched data to work with.[/red]")
        raise typer.Exit(1)

    existing_path = project_dir / "content.yaml"
    result = fill_from_notes(
        notes=combined_notes,
        schema_cls=schema_cls,
        model=model,
        existing_yaml_path=existing_path if existing_path.exists() else None,
    )

    if dry_run:
        console.print(result.raw_yaml)
        return

    existing_path.write_text(result.raw_yaml)
    console.print(f"[green]✓ Wrote enriched content[/green] → {existing_path}")


# ---------------------------------------------------------------------------
# publish
# ---------------------------------------------------------------------------

@app.command()
def publish(
    project_dir: Path = typer.Argument(Path("."), help="Document project directory"),
    template_name: Optional[str] = typer.Option(None, "--template", "-t"),
    folder_id: Optional[str] = typer.Option(None, "--folder-id", help="Override Drive folder ID"),
    dry_run: bool = typer.Option(False, "--dry-run"),
):
    """Render all templates and upload to Google Drive."""
    from forma.publisher.google_drive import upload_file

    project_dir = project_dir.resolve()
    config, schema_cls = _load_project(project_dir)

    output_dir = config.resolve_output_dir(project_dir)
    templates = {template_name: config.templates[template_name]} if template_name else config.templates

    # Render first
    ctx = typer.Context(render_default)
    render_default.callback(ctx, project_dir=project_dir, template_name=template_name)  # type: ignore

    drive_folder = folder_id or config.publishing.google_drive_folder_id
    if not drive_folder and not dry_run:
        console.print("[red]✗ No Google Drive folder ID configured. Set publishing.google_drive_folder_id in forma.yaml or pass --folder-id.[/red]")
        raise typer.Exit(1)

    prefix = config.publishing.filename_prefix
    for name in templates:
        pdf = output_dir / f"{name}.pdf"
        if not pdf.exists():
            console.print(f"[red]✗ {pdf} not found after render[/red]")
            continue
        filename = f"{prefix}-{name}.pdf" if prefix else pdf.name
        if dry_run:
            console.print(f"[dim]DRY RUN: would upload {pdf} → Drive/{drive_folder}/{filename}[/dim]")
        else:
            upload_file(pdf, drive_folder, filename=filename)


# ---------------------------------------------------------------------------
# schema export
# ---------------------------------------------------------------------------

@schema_app.command("export")
def schema_export(
    output_dir: Path = typer.Option(Path("schema"), "--output-dir", "-o"),
    project_dir: Path = typer.Argument(Path("."), help="Document project directory (for schema path)"),
):
    """Export JSON Schema files from all starter Pydantic schemas."""
    import json

    project_dir = project_dir.resolve()

    # Export all schemas found in the repo's schemas/ directory
    repo_root = Path(__file__).parents[4]
    schemas_dir = repo_root / "schemas"

    if str(repo_root) not in sys.path:
        sys.path.insert(0, str(repo_root))

    output_dir.mkdir(parents=True, exist_ok=True)

    exported = 0
    for schema_file in schemas_dir.glob("*/content.py"):
        schema_name = schema_file.parent.name
        try:
            import importlib
            module = importlib.import_module(f"schemas.{schema_name}.content")
            # Find BaseContent subclass
            from forma.core.base import BaseContent
            for attr_name in dir(module):
                cls = getattr(module, attr_name)
                if (
                    isinstance(cls, type)
                    and issubclass(cls, BaseContent)
                    and cls is not BaseContent
                ):
                    out_path = output_dir / f"{schema_name}.schema.json"
                    out_path.write_text(json.dumps(cls.model_json_schema(), indent=2))
                    console.print(f"[green]✓[/green] {out_path}")
                    exported += 1
        except Exception as e:
            console.print(f"[yellow]⚠ Could not export {schema_name}: {e}[/yellow]")

    console.print(f"\nExported {exported} schema(s) to {output_dir}/")


# ---------------------------------------------------------------------------
# template list
# ---------------------------------------------------------------------------

@template_app.command("list")
def template_list(
    templates_dir: Path = typer.Option(None, "--dir", "-d", help="Templates root directory"),
):
    """List available templates and their manifests."""
    import yaml as _yaml

    if templates_dir is None:
        templates_dir = Path(__file__).parents[4] / "templates"

    table = Table(title="Available Templates")
    table.add_column("Name", style="cyan")
    table.add_column("Format", style="green")
    table.add_column("Engine")
    table.add_column("Description")

    for manifest_path in sorted(templates_dir.glob("*/manifest.yaml")):
        with open(manifest_path) as f:
            data = _yaml.safe_load(f) or {}
        table.add_row(
            manifest_path.parent.name,
            data.get("format", "?"),
            data.get("engine", "xelatex"),
            data.get("description", ""),
        )

    console.print(table)


# ---------------------------------------------------------------------------
# init
# ---------------------------------------------------------------------------

@app.command()
def init(
    client_name: str = typer.Argument(..., help="Client or project name (used as directory name)"),
    documents_dir: Path = typer.Option(Path("documents"), "--dir", "-d"),
    schema: str = typer.Option("schemas.proposal.content:ProposalContent", "--schema"),
    template: str = typer.Option("proposal-slides,proposal-report", "--templates"),
):
    """Scaffold a new document project directory."""
    slug = client_name.lower().replace(" ", "-")
    project_dir = documents_dir / slug
    project_dir.mkdir(parents=True, exist_ok=True)
    (project_dir / "assets").mkdir(exist_ok=True)

    templates_config = {
        t.strip(): {"path": f"../../templates/{t.strip()}"}
        for t in template.split(",")
    }

    forma_config = {
        "schema": schema,
        "style": "style.yaml",
        "templates": templates_config,
        "output_dir": f"../../var/builds/{slug}",
        "publishing": {
            "google_drive_folder_id": "",
            "filename_prefix": slug.upper()[:8],
        },
    }

    (project_dir / "forma.yaml").write_text(
        yaml.dump(forma_config, default_flow_style=False, allow_unicode=True)
    )

    # Minimal content.yaml skeleton
    (project_dir / "content.yaml").write_text(
        f"# {client_name} — content.yaml\n"
        "# Fill in your content here. Run: forma compose fill . --notes your-notes.md\n\n"
        "publishing:\n"
        "  google_drive_folder_id: ''\n"
        "  filename_prefix: ''\n"
    )

    # Minimal style.yaml
    (project_dir / "style.yaml").write_text(
        "# Style overrides for this project.\n"
        "# See style defaults in the root style.yaml.\n"
    )

    console.print(f"[green]✓ Created[/green] {project_dir}/")
    console.print(f"  • Edit [cyan]{project_dir}/content.yaml[/cyan]")
    console.print(f"  • Or run: [cyan]forma compose fill {project_dir} --notes notes.md[/cyan]")
