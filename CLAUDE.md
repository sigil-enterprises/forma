# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

---

## What forma is

`forma` is a **schema-agnostic document rendering framework**. It turns structured YAML content + Jinja2 templates into PDF documents (slide decks, reports, briefs).

The central idea: `content.yaml` describes semantic meaning only (client, problem, solution, investment…). Mapping files (`slides.yaml`, `report.yaml`) assemble that content into document structure using `!include "@content.yaml:dot.path"` tags. Templates render the assembled document into LaTeX or HTML, which is then compiled to PDF.

```
content.yaml ──[!include]──→  slides.yaml / report.yaml (mapping)
                                        ↓
style.yaml   ───────────────→  Jinja2 template → xelatex/Playwright → PDF
```

---

## Commands

```bash
# Install for development
pip install -e .[dev]

# Run tests
pytest .
# or
make test

# Run a single test
pytest tests/test_renderer.py::test_html_slides_template_renders

# Watch mode
make dev   # runs ptw (pytest-watch)

# CLI (after pip install -e .)
forma validate <project-dir>
forma render <project-dir> [--template slides]
forma compose fill <project-dir> --notes notes.md [--dry-run] [--overwrite]
forma publish <project-dir> [--dry-run]
forma template list
forma schema export
forma init "Client Name" [--dir documents]
```

---

## Architecture

### Source layout

```
src/forma/
├── cli/app.py              Typer CLI entry point; all commands defined here
├── core/
│   ├── base.py             BaseContent, BaseStyle (Pydantic v2 bases for schemas)
│   ├── config.py           FormaConfig — loads forma.yaml per project
│   ├── include_loader.py   Custom YAML loader resolving !include "@file:dot.path" tags
│   ├── loader.py           load_document/load_content/load_style + _SCHEMA_REGISTRY
│   └── validator.py        validate_file() / validate_project() using jsonschema
├── renderer/
│   ├── engine.py           TemplateManifest + render_template() — full pipeline
│   ├── context.py          build_context() → {content, document, style, meta}
│   ├── filters.py          Jinja2 filters: latex_escape/le, currency, format_date, hex_color
│   └── base.py             BaseRenderer ABC (subprocess xelatex/pdflatex/lualatex)
├── composer/
│   ├── client.py           Anthropic SDK wrapper (FormaClient)
│   ├── prompts.py          Schema-aware system/user prompt builders
│   └── filler.py           fill_from_notes() → FillResult (validated content + raw YAML)
├── integrations/
│   └── skills_loader.py    Discovers skills/*/fetch.py::fetch() via importlib
├── publisher/
│   └── google_drive.py     Service account upload — credentials from base64 env var
├── schema/                 Built-in JSON Schema files (YAML format, draft-07)
│   ├── forma-config.schema.yaml
│   ├── slide-document.schema.yaml
│   ├── report-document.schema.yaml
│   └── proposal-content.schema.yaml
└── schemas/                Pydantic content schema classes
    ├── proposal/content.py  ProposalContent(BaseContent) — used by compose commands
    ├── brief/content.py
    └── case_study/content.py
```

### Per-project document structure

Each document project is a directory containing:
- `forma.yaml` — `FormaConfig`: which schema, templates, output dir, publishing config
- `content.yaml` — semantic content (`resourceType: ProposalContent`)
- `slides.yaml` / `report.yaml` — mapping documents (`resourceType: SlideDocument / ReportDocument`) that pull from content.yaml via `!include`
- `style.yaml` — visual tokens (colors, fonts)

### Schema registry

`loader.py` maintains `_SCHEMA_REGISTRY: dict[str, Path]` mapping `resourceType` values to JSON Schema files. `validate_file()` reads `resourceType` from the document and looks up the schema. To add a new content type, call `register_schema()` at runtime or extend the registry dict.

### Jinja2 template conventions

- **Delimiters**: `(( var ))`, `(% block %)`, `(# comment #)` — avoids conflict with LaTeX `{}` and `%`
- **Context keys**: `content` (alias `document`) — the resolved mapping dict; `style` — style dict; `meta` — `{rendered_date, forma_version, project_dir, presskit_root}`
- **Partials**: placed in `_partials/` or `_slides/` subdirectory; auto-added to Jinja2 search path
- **Optional fields**: use `slide.get('key')` not `slide.key` for optional dict fields with `StrictUndefined`; use `slide['items']` not `slide.items` when the key shadows a Python dict method
- **LaTeX escaping**: ALL string content embedded in LaTeX must pass through `| latex_escape` (alias `| le`); currency values need `| currency | le`
- **Computed values**: templates work with plain dicts, not Pydantic models — compute aggregates (subtotals) using Jinja2 `namespace` rather than relying on Python properties

### Rendering pipeline

1. `FormaConfig.from_yaml()` loads `forma.yaml`
2. `load_document()` loads the mapping file, resolving `!include` tags against project root
3. `load_style()` loads `style.yaml`
4. `build_context()` creates the Jinja2 context
5. `TemplateManifest` reads `manifest.yaml` to discover engine and entry template
6. Jinja2 renders the template to LaTeX or HTML source
7. `BaseRenderer` (LaTeX) or `HtmlRenderer` (Playwright) compiles to PDF

### Testing

Tests live in `tests/`. All external dependencies are mocked — no live API keys, no xelatex, no Playwright required. Template rendering tests invoke Jinja2 only (no compilation).

- `tests/fixtures/example-client/` — full project fixture (content, slides, report, forma.yaml)
- `tests/fixtures/templates/` — template fixtures (proposal-slides-html, proposal-report, etc.)

```bash
pytest tests/test_renderer.py   # Jinja2 rendering + filter tests
pytest tests/test_schema.py     # Schema loading, validation, config
pytest tests/test_cli.py        # CLI commands via typer.testing.CliRunner
pytest tests/test_e2e.py        # Full pipeline (render + publish mocked)
pytest tests/test_composer.py   # Compose fill/enrich (Anthropic mocked)
pytest tests/test_publisher.py  # Google Drive upload (Drive API mocked)
```

---

## Key conventions

**forma.yaml** (per project):
```yaml
resourceType: FormaConfig
content: content.yaml
style: style.yaml
templates:
  slides:
    path: ../../tests/fixtures/templates/proposal-slides-html
    mapping: slides.yaml
  report:
    path: ../../tests/fixtures/templates/proposal-report
    mapping: report.yaml
output_dir: ../../var/builds/my-client
publishing:
  google_drive_folder_id: ""
  filename_prefix: "SLVR"
```

**Template manifest.yaml**:
```yaml
name: "Proposal Slides"
format: slides       # slides | document
engine: html         # xelatex | pdflatex | lualatex | html
entry: main.html.j2
```

**!include syntax** (in mapping files):
```yaml
title: !include "@content.yaml:engagement.title"
logo:  !include "@content.yaml:closing.logo"
```

**CI/CD secrets** (GitHub Actions):
- `PAT_ORG_REPO_READ` — private org repo access
- `GOOGLE_SERVICE_ACCOUNT_JSON` — base64-encoded service account JSON
- `ANTHROPIC_API_KEY` — compose commands only
