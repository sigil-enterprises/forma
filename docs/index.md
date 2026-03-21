# forma

**Schema-agnostic document rendering framework.**

Content YAML + Jinja2/LaTeX templates → polished PDF slides and reports.

## Concept

`forma` separates **content** from **presentation**:

- **`content.yaml`** — domain-semantic data (client, problem, solution, investment, team…). No document structure, no slide types.
- **`style.yaml`** — visual tokens (colors, fonts, spacing).
- **`templates/`** — Jinja2/LaTeX templates that define document structure and reference content paths freely.

The same `content.yaml` renders as a slide deck, a full report, or a one-pager — each template decides its own structure.

## Quick start

```bash
pip install -e .[dev]

# Scaffold a new document project
forma init acme-corp

# Fill content from notes using Claude
forma compose fill documents/acme-corp --notes meeting-notes.md

# Validate
forma validate documents/acme-corp

# Render to PDF
forma render documents/acme-corp

# Publish to Google Drive
forma publish documents/acme-corp
```

## CLI reference

| Command | Description |
|---|---|
| `forma validate [DIR]` | Validate content.yaml and style.yaml |
| `forma render [DIR]` | Render all templates to PDF |
| `forma render [DIR] -t slides` | Render a specific template |
| `forma render [DIR] --watch` | Re-render on file change |
| `forma compose fill [DIR] -n FILE` | Draft content.yaml from notes using Claude |
| `forma compose enrich [DIR] -s clickup,google_docs` | Enrich with external data then compose |
| `forma publish [DIR]` | Render + upload to Google Drive |
| `forma schema export` | Export JSON Schema files |
| `forma template list` | List available templates |
| `forma init CLIENT_NAME` | Scaffold a new document project |

## Architecture

```
content.yaml  ──[schema validates]──→  Jinja2 context
                                             ↓
style.yaml    ─────────────────────→  templates/proposal-slides/main.tex.j2  →  slides.pdf
                                             ↓
                                        templates/proposal-report/main.tex.j2  →  report.pdf
                                             ↓
                                        templates/proposal-brief/main.tex.j2   →  brief.pdf
```

## Environment variables

| Variable | Purpose |
|---|---|
| `ANTHROPIC_API_KEY` | Required for `forma compose` commands |
| `GOOGLE_SERVICE_ACCOUNT_JSON` | Base64-encoded service account JSON for Drive publishing |
| `GITHUB_TOKEN` | CI and devcontainer setup |

See `.env.example` for the full list.
