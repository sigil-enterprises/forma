# forma

Schema-agnostic document rendering framework.

Content YAML + Jinja2/LaTeX templates → polished PDF slides and reports.

## Concept

`forma` separates **content** from **presentation**:

- **`content.yaml`** — domain-semantic data (client, problem, solution, investment, team…). No document structure. No slide types.
- **`style.yaml`** — visual tokens (colors, fonts, spacing).
- **`templates/`** — Jinja2/LaTeX templates that define document structure and reference content paths freely.

The same `content.yaml` can be rendered as a slide deck, a full report, or a one-pager — each template decides its own structure.

## Quick start

```bash
pip install -e .[dev]

# Scaffold a new document project
forma init acme-corp

# Fill content from notes using Claude
forma compose fill documents/acme-corp --notes meeting-notes.md

# Validate
forma validate documents/acme-corp

# Render
forma render documents/acme-corp

# Render specific template
forma render documents/acme-corp --template slides
```

## Repository structure

```
forma/
├── src/forma/          # the engine (schema-agnostic)
├── schemas/            # starter content schemas (Pydantic)
│   ├── proposal/       # ProposalContent
│   ├── case_study/     # CaseStudyContent
│   └── brief/          # BriefContent
├── templates/          # starter Jinja2/LaTeX templates
│   ├── proposal-slides/
│   ├── proposal-report/
│   └── proposal-brief/
├── documents/          # document projects (one dir per client/project)
│   └── example-client/
│       ├── forma.yaml  # schema, templates, output config
│       ├── content.yaml
│       └── style.yaml
└── skills/             # git submodule: forma-skills (external data fetchers)
```

## CLI reference

```
forma validate [PROJECT_DIR]               validate content.yaml
forma render   [PROJECT_DIR]               render all templates
forma render   [PROJECT_DIR] -t slides     render specific template
forma compose fill  [PROJECT_DIR] -n FILE  draft content from notes (Claude)
forma compose enrich [PROJECT_DIR] -s clickup,gdocs
forma publish  [PROJECT_DIR]               render + upload to Google Drive
forma schema export                        regenerate JSON Schema files
forma template list                        list available templates
forma init CLIENT_NAME                     scaffold new project directory
```

## Environment variables

| Variable | Purpose |
|---|---|
| `ANTHROPIC_API_KEY` | Required for `forma compose` commands |
| `GOOGLE_SERVICE_ACCOUNT_JSON` | Base64-encoded service account JSON for Drive publishing |
| `GITHUB_TOKEN` | Used by devcontainer setup and CI |

## Devcontainer

Open in VS Code → "Reopen in Container". The devcontainer:
- Installs all Python dependencies (`make setup-dev`)
- Installs zsh + oh-my-zsh + tmux
- Installs Claude Code CLI
- Includes LaTeX Workshop extension for template editing

## Adding a new document type

1. Define a schema in `schemas/mytype/content.py` extending `BaseContent`
2. Create a template in `templates/mytype/` with a `manifest.yaml` and `main.tex.j2`
3. Point a document project's `forma.yaml` at your schema and template

## CI/CD

- `build.yml` — validate + test on every push (via `opv-actions`)
- `build-base.yml` — rebuild Docker base image when dependencies change
- `docs.yml` — deploy MkDocs to GitHub Pages on main/release
- `publish.yml` — render all documents + upload to Google Drive on push to main

## Secrets required

| Secret | Where |
|---|---|
| `PAT_ORG_REPO_READ` | GitHub personal access token with repo read scope |
| `PAT_BADGES` | PAT for writing coverage badges |
| `GOOGLE_SERVICE_ACCOUNT_JSON` | Base64-encoded Drive service account |
| `ANTHROPIC_API_KEY` | (optional) Only if running compose in CI |
