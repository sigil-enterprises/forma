# forma

**Schema-agnostic document rendering framework.**

Turn structured YAML content and Jinja2/LaTeX templates into polished PDF slides, reports, and briefings — driven by Claude AI.

---

## How it works

```
content.yaml  ──[!include]──→  slides.yaml / report.yaml  (mapping)
                                        │
style.yaml   ───────────────→  Jinja2 template  →  xelatex / Playwright  →  PDF
```

**Content** describes *what* — client, problem, solution, team, investment. No slide types, no page layout.

**Templates** describe *how* — each template independently decides structure, layout, and which content fields to include.

---

## Key features

<div class="grid cards" markdown>

- :material-file-code: **Schema-first content**

    Define your content structure with Pydantic. The engine works with any subclass of `BaseContent`.

- :material-robot: **Claude Composer**

    Draft `content.yaml` from meeting notes in seconds. `forma compose fill` sends notes to Claude and validates the output against your schema.

- :simple-latex: **LaTeX rendering**

    Templates are Jinja2-wrapped LaTeX. xelatex produces pixel-perfect PDFs with full font and color control.

- :material-google-drive: **Google Drive publishing**

    `forma publish` renders all templates and uploads to a Drive folder. Service account credentials never touch disk.

- :material-puzzle: **Skills integration**

    Pull live data from ClickUp, Google Docs, Google Sheets, or meeting notes files and feed them directly into the composer.

- :material-test-tube: **84 tests, all mocked**

    Full test suite covering every CLI command, filter, schema, publisher, and skills loader — no live API keys required.

</div>

---

## Quick start

```bash
# Install
pip install -e .[dev]

# Scaffold a new project
forma init acme-corp

# Draft content from meeting notes using Claude
forma compose fill documents/acme-corp --notes meeting-notes.md

# Validate
forma validate documents/acme-corp

# Render to PDF (requires xelatex)
forma render documents/acme-corp

# Publish to Google Drive
forma publish documents/acme-corp
```

---

## CLI reference

| Command | Description |
|---|---|
| `forma validate [DIR]` | Validate `content.yaml` and `style.yaml` against the schema |
| `forma render [DIR]` | Render all templates to PDF |
| `forma render [DIR] -t slides` | Render a specific template |
| `forma render [DIR] --watch` | Re-render on file change |
| `forma compose fill [DIR] -n FILE` | Draft `content.yaml` from notes using Claude |
| `forma compose enrich [DIR] -s clickup,gdocs` | Fetch external data, then compose |
| `forma publish [DIR]` | Render + upload artifacts to Google Drive |
| `forma schema export` | Export JSON Schema files from all Pydantic schemas |
| `forma template list` | List available templates and their manifests |
| `forma init NAME` | Scaffold a new document project directory |
