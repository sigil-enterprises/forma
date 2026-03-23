"""
forma CLI — schema-agnostic document rendering framework.

Commands:
  validate   Validate content.yaml and mapping files against their schemas
  render     Render one or all templates to PDF
  mapping    Mapping file utilities (validate)
  compose    AI-assisted content drafting (fill / enrich)
  publish    Render + upload artifacts to Google Drive
  schema     Export JSON Schema files from Pydantic models
  template   List available templates
  init       Scaffold a new document project directory
"""

from __future__ import annotations

import sys
from pathlib import Path

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

mapping_app = typer.Typer(help="Mapping file utilities.")
app.add_typer(mapping_app, name="mapping")

schema_app = typer.Typer(help="Schema utilities.")
app.add_typer(schema_app, name="schema")

template_app = typer.Typer(help="Template utilities.")
app.add_typer(template_app, name="template")

console = Console()


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _load_config(project_dir: Path):
    from forma.core.config import FormaConfig

    config_path = project_dir / "forma.yaml"
    if not config_path.exists():
        console.print(f"[red]✗ No forma.yaml found in {project_dir}[/red]")
        raise typer.Exit(1)

    return FormaConfig.from_yaml(config_path)


def _format_skill_data(name: str, data: dict) -> str:
    """Format fetched skill data as structured prose for the Claude composer."""
    lines = [f"## Data from {name}"]

    def _render(obj: object, indent: int = 0) -> None:
        pad = "  " * indent
        if isinstance(obj, dict):
            for k, v in obj.items():
                if isinstance(v, (dict, list)):
                    lines.append(f"{pad}**{k}:**")
                    _render(v, indent + 1)
                else:
                    lines.append(f"{pad}**{k}:** {v}")
        elif isinstance(obj, list):
            for item in obj:
                if isinstance(item, (dict, list)):
                    lines.append(f"{pad}-")
                    _render(item, indent + 1)
                else:
                    lines.append(f"{pad}- {item}")
        else:
            lines.append(f"{pad}{obj}")

    _render(data)
    return "\n".join(lines)


# ---------------------------------------------------------------------------
# validate
# ---------------------------------------------------------------------------

@app.command()
def validate(
    project_dir: Path = typer.Argument(Path("."), help="Document project directory"),
    strict: bool = typer.Option(False, "--strict", help="Treat warnings as errors"),
):
    """Validate content.yaml and all mapping files against their declared schemas."""
    from forma.core.validator import validate_project

    project_dir = project_dir.resolve()
    result = validate_project(project_dir)
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
    template_name: str | None = typer.Option(None, "--template", "-t", help="Render a specific template"),
    watch: bool = typer.Option(False, "--watch", "-w", help="Re-render on file change"),
):
    """Render all templates (or a specific one) declared in forma.yaml."""
    if ctx.invoked_subcommand is not None:
        return

    project_dir = project_dir.resolve()

    def _do_render():
        from forma.core.config import FormaConfig
        from forma.core.loader import load_document, load_style
        from forma.renderer.engine import render_template

        cfg = FormaConfig.from_yaml(project_dir / "forma.yaml")

        style_path = cfg.resolve_style_path(project_dir)
        style = load_style(style_path)

        output_dir = cfg.resolve_output_dir(project_dir)
        output_dir.mkdir(parents=True, exist_ok=True)

        templates = (
            {template_name: cfg.templates[template_name]}
            if template_name
            else cfg.templates
        )

        for name in templates:
            mapping_path = cfg.resolve_mapping_path(name, project_dir)
            if not mapping_path.exists():
                console.print(f"[red]✗ Mapping file not found: {mapping_path}[/red]")
                continue

            document = load_document(mapping_path, project_dir)
            tpl_path = cfg.resolve_template_path(name, project_dir)
            out = output_dir / f"{name}.pdf"

            console.print(f"[dim]Rendering {name}...[/dim]")
            render_template(tpl_path, document, style, out, project_dir=project_dir)

    _do_render()

    if watch:
        from watchfiles import watch as wf
        console.print("[dim]Watching for changes (Ctrl-C to stop)...[/dim]")
        for _ in wf(str(project_dir)):
            _do_render()


# ---------------------------------------------------------------------------
# mapping validate
# ---------------------------------------------------------------------------

@mapping_app.command("validate")
def mapping_validate(
    project_dir: Path = typer.Argument(Path("."), help="Document project directory"),
    mapping_file: str | None = typer.Option(None, "--file", "-f", help="Specific mapping file (e.g. slides.yaml)"),
):
    """Validate mapping file(s) against their declared schemas."""
    from forma.core.validator import validate_file

    project_dir = project_dir.resolve()
    mapping_names = [mapping_file] if mapping_file else ["slides.yaml", "report.yaml", "brief.yaml"]

    found = False
    combined_ok = True
    for name in mapping_names:
        path = project_dir / name
        if not path.exists():
            continue
        found = True
        console.print(f"[dim]Validating {name}...[/dim]")
        result = validate_file(path, base_dir=project_dir)
        result.print()
        if not result.ok:
            combined_ok = False

    if not found:
        console.print("[yellow]⚠ No mapping files found in project directory[/yellow]")
        raise typer.Exit(1)

    if not combined_ok:
        raise typer.Exit(1)


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

    # Derive a simple schema class for the composer — still uses ProposalContent for now.
    # This can be extended to support other schema types via a registry lookup.
    try:
        from forma.schemas.proposal.content import ProposalContent as schema_cls
    except ImportError:
        console.print("[red]✗ ProposalContent schema not found. Check that schemas/ is on sys.path.[/red]")
        raise typer.Exit(1)

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
    skills: str = typer.Option(..., "--skills", "-s", help="Comma-separated skill names"),
    notes_file: Path | None = typer.Option(None, "--notes", "-n"),
    model: str = typer.Option("claude-opus-4-6", "--model", "-m"),
    dry_run: bool = typer.Option(False, "--dry-run"),
):
    """Fetch external data via skills, merge with notes, compose content.yaml."""
    from forma.composer.filler import fill_from_notes
    from forma.integrations.skills_loader import load_skills

    project_dir = project_dir.resolve()

    try:
        from forma.schemas.proposal.content import ProposalContent as schema_cls
    except ImportError:
        console.print("[red]✗ ProposalContent schema not found.[/red]")
        raise typer.Exit(1)

    repo_root = Path(__file__).parents[3]
    skills_dir = repo_root / "skills"
    if not skills_dir.exists():
        console.print(f"[red]✗ skills/ submodule not found at {skills_dir}[/red]")
        raise typer.Exit(1)

    skill_names = [s.strip() for s in skills.split(",")]
    fetched = load_skills(skills_dir, skill_names)

    combined_parts = []
    if notes_file and notes_file.exists():
        combined_parts.append(notes_file.read_text().strip())

    for name, data in fetched.items():
        combined_parts.append(_format_skill_data(name, data))

    combined_notes = "\n\n".join(p for p in combined_parts if p)
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
    template_name: str | None = typer.Option(None, "--template", "-t"),
    folder_id: str | None = typer.Option(None, "--folder-id", help="Override Drive folder ID"),
    dry_run: bool = typer.Option(False, "--dry-run"),
):
    """Render all templates and upload to Google Drive."""
    from forma.core.loader import load_document, load_style
    from forma.publisher.google_drive import upload_file
    from forma.renderer.engine import render_template

    project_dir = project_dir.resolve()
    config = _load_config(project_dir)

    output_dir = config.resolve_output_dir(project_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    style_path = config.resolve_style_path(project_dir)
    style = load_style(style_path)

    templates = (
        {template_name: config.templates[template_name]}
        if template_name
        else config.templates
    )

    for name in templates:
        mapping_path = config.resolve_mapping_path(name, project_dir)
        if not mapping_path.exists():
            console.print(f"[red]✗ Mapping file not found: {mapping_path}[/red]")
            continue
        document = load_document(mapping_path, project_dir)
        tpl_path = config.resolve_template_path(name, project_dir)
        out = output_dir / f"{name}.pdf"
        console.print(f"[dim]Rendering {name}...[/dim]")
        render_template(tpl_path, document, style, out, project_dir=project_dir)

    drive_folder = folder_id or config.publishing.google_drive_folder_id
    if not drive_folder and not dry_run:
        console.print("[red]✗ No Google Drive folder ID configured.[/red]")
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
):
    """List built-in YAML schemas from the package schema/ directory."""
    pkg_dir = Path(__file__).parents[1]  # src/forma/
    schemas_dir = pkg_dir / "schema"

    if not schemas_dir.exists():
        console.print(f"[red]✗ schema/ directory not found at {schemas_dir}[/red]")
        raise typer.Exit(1)

    for f in sorted(schemas_dir.glob("*.schema.yaml")):
        console.print(f"[green]✓[/green] {f}")

    repo_root = Path(__file__).parents[3]
    client_schemas = sorted(repo_root.glob("templates/*.schema.yaml"))
    for f in client_schemas:
        console.print(f"[cyan]✓[/cyan] {f}")


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
        repo_root = Path(__file__).parents[3]
        templates_dir = repo_root / "tests" / "fixtures" / "templates"

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
    client_name: str = typer.Argument(..., help="Client or project name"),
    documents_dir: Path = typer.Option(Path("documents"), "--dir", "-d"),
):
    """Scaffold a new document project directory."""
    slug = client_name.lower().replace(" ", "-")
    project_dir = documents_dir / slug
    project_dir.mkdir(parents=True, exist_ok=True)

    # Determine template base relative path
    repo_root = Path(__file__).parents[3]
    try:
        rel = project_dir.resolve().relative_to(repo_root)
        depth = len(rel.parts)
    except ValueError:
        depth = 3  # fallback
    up = "/".join([".."] * depth)

    forma_config = {
        "resourceType": "FormaConfig",
        "content": "content.yaml",
        "style": "style.yaml",
        "templates": {
            "slides": {
                "path": "path/to/proposal-slides-html",
                "mapping": "slides.yaml",
            },
            "report": {
                "path": "path/to/proposal-report",
                "mapping": "report.yaml",
            },
        },
        "output_dir": f"{up}/var/builds/{slug}",
        "publishing": {
            "google_drive_folder_id": "",
            "filename_prefix": slug.upper()[:8],
        },
    }

    (project_dir / "forma.yaml").write_text(
        yaml.dump(forma_config, default_flow_style=False, allow_unicode=True, sort_keys=False)
    )

    # Minimal content.yaml with resourceType
    (project_dir / "content.yaml").write_text(
        f"resourceType: ProposalContent\n\n"
        f"# {client_name} — content.yaml\n"
        "# Fill in your content here.\n\n"
        "engagement:\n"
        f'  title: "{client_name}"\n'
        "  subtitle: \"\"\n"
        "  reference: \"\"\n"
        "  date: \"\"\n\n"
        "client:\n"
        f'  name: "{client_name}"\n\n'
        "executive_summary:\n"
        "  headline: \"\"\n"
        "  key_points: []\n"
    )

    # Minimal slides.yaml skeleton
    (project_dir / "slides.yaml").write_text(
        "resourceType: SlideDocument\n\n"
        "slides:\n"
        "  - type: cover\n"
        '    title: !include "@content.yaml:engagement.title"\n'
        '    client: !include "@content.yaml:client.name"\n\n'
        "  - type: exec_summary\n"
        '    headline: !include "@content.yaml:executive_summary.headline"\n'
        '    key_points: !include "@content.yaml:executive_summary.key_points"\n\n'
        "  - type: closing\n"
        "    tagline: \"\"\n"
    )

    # Minimal report.yaml skeleton
    (project_dir / "report.yaml").write_text(
        "resourceType: ReportDocument\n\n"
        "meta:\n"
        '  title: !include "@content.yaml:engagement.title"\n'
        '  client: !include "@content.yaml:client.name"\n\n'
        "chapters:\n"
        "  - title: Executive Summary\n"
        "    sections:\n"
        "      - title: Overview\n"
        "        blocks:\n"
        "          - type: paragraph\n"
        '            text: !include "@content.yaml:executive_summary.headline"\n'
    )

    console.print(f"[green]✓ Created[/green] {project_dir}/")
    console.print(f"  • Edit [cyan]{project_dir}/content.yaml[/cyan] with your semantic content")
    console.print(f"  • Edit [cyan]{project_dir}/slides.yaml[/cyan] to map content → slides")
    console.print(f"  • Edit [cyan]{project_dir}/report.yaml[/cyan] to map content → report chapters")
    console.print(f"  • Run: [cyan]forma render {project_dir} --template slides[/cyan]")
