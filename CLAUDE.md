# forma — AI working notes

This file is read at the start of every Claude Code session. It contains the
full architecture plan, current state, and next steps so work can resume
without re-explanation.

---

## What forma is

`forma` is a **schema-agnostic document rendering framework** owned by
`sliver-corporate-services`. It turns structured YAML content + Jinja2/LaTeX
templates into polished PDF documents (slide decks, full reports, one-pagers,
briefings — whatever templates exist).

### Core design principle

Content YAML describes **semantic meaning** (client, problem, solution, team,
investment…). It contains no document structure, no slide types, no page
layout. Templates define structure and reference YAML paths freely.

```
content.yaml  ──[schema validates]──→  Jinja2 context
                                             ↓
style.yaml    ─────────────────────→  templates/proposal-slides/main.tex.j2  →  slides.pdf
                                             ↓
                                        templates/proposal-report/main.tex.j2  →  report.pdf
                                             ↓
                                        templates/proposal-brief/main.tex.j2   →  brief.pdf
```

The same `content.yaml` renders through any number of templates. New document
types are added by defining a new schema + template pair — the engine doesn't
change.

---

## Repository structure

```
forma/
├── .devcontainer/              lypto-pattern devcontainer
├── .github/workflows/
│   ├── build.yml               on push → opv-actions (lint, test, Docker push to GHCR)
│   ├── build-base.yml          on Dockerfile.base/pyproject.toml change → rebuild base
│   ├── docs.yml                on main/release → mkdocs gh-deploy
│   └── publish.yml             on push to main → render + Google Drive upload
├── skills/                     git submodule → slivern-corporate-services/forma-skills
│   ├── clickup/fetch.py        fetch() -> dict
│   ├── google_docs/fetch.py
│   ├── google_sheets/fetch.py
│   └── meeting_notes/parse.py
├── src/forma/
│   ├── cli/app.py              Typer CLI (validate, render, compose, publish, schema, template, init)
│   ├── core/
│   │   ├── base.py             BaseContent, BaseStyle (Pydantic bases)
│   │   ├── config.py           FormaConfig (forma.yaml loader)
│   │   ├── loader.py           importlib-based schema class loader
│   │   └── validator.py        content + asset validation
│   ├── renderer/
│   │   ├── base.py             BaseRenderer ABC (subprocess xelatex/pdflatex)
│   │   ├── context.py          Jinja2 context builder
│   │   ├── engine.py           TemplateManifest + render_template()
│   │   └── filters.py          latex_escape, format_date, currency, hex_color, etc.
│   ├── composer/
│   │   ├── client.py           Anthropic SDK wrapper
│   │   ├── prompts.py          schema-aware system prompts
│   │   └── filler.py           notes → Claude → validated content.yaml
│   ├── integrations/
│   │   └── skills_loader.py    importlib discovery of skills/*/fetch.py::fetch()
│   └── publisher/
│       └── google_drive.py     service account upload (base64 env var, no disk write)
├── schemas/                    starter schemas — projects copy/extend these
│   ├── proposal/content.py     ProposalContent(BaseContent)
│   ├── case_study/content.py   CaseStudyContent(BaseContent)
│   └── brief/content.py        BriefContent(BaseContent)
├── templates/                  starter templates — projects copy/extend or add their own
│   ├── proposal-slides/        Beamer slide deck
│   │   ├── manifest.yaml       format, engine, entry, compatible_schemas
│   │   ├── main.tex.j2
│   │   └── _partials/          _cover, _exec_summary, _context, _solution,
│   │                           _timeline, _investment, _team, _next_steps, _closing
│   ├── proposal-report/        Full LaTeX article
│   │   ├── manifest.yaml
│   │   └── main.tex.j2
│   └── proposal-brief/         One-pager
│       ├── manifest.yaml
│       └── main.tex.j2
├── documents/                  actual client work
│   └── example-client/
│       ├── forma.yaml          schema, templates, output_dir, publishing
│       ├── content.yaml        semantic content (ProposalContent)
│       └── style.yaml          visual tokens
├── schema/                     auto-generated JSON Schema files (run: forma schema export)
├── tests/
│   ├── test_schema.py
│   └── test_renderer.py
├── Dockerfile                  multi-stage: base (GHCR) + build (copy source)
├── Dockerfile.base             python:3.12-slim-bookworm + texlive-xetex + pip install
├── docker-compose.yaml         app service, github_token secret, GHCR image
└── Makefile                    setup, setup-dev, test, dev, git-config
```

---

## Key conventions

### forma.yaml (per document project)

```yaml
schema: schemas.proposal.content:ProposalContent   # importable Pydantic class
style: style.yaml
templates:
  slides:
    path: ../../templates/proposal-slides
  report:
    path: ../../templates/proposal-report
output_dir: ../../var/builds/example-client
publishing:
  google_drive_folder_id: ""
  filename_prefix: "SLVR"
```

### Template manifest.yaml

```yaml
name: "Proposal Slides"
format: slides            # slides | document
engine: xelatex           # xelatex | pdflatex | lualatex
entry: main.tex.j2
compatible_schemas:
  - "schemas.proposal.content:ProposalContent"
```

### Content schema pattern

All schemas extend `BaseContent` from `forma.core.base`. The engine works
with any subclass — it builds a Jinja2 context from the model fields and
renders the template.

```python
class ProposalContent(BaseContent):
    engagement: Engagement
    client: Client
    executive_summary: ExecutiveSummary
    context: Optional[Context] = None
    solution: Optional[Solution] = None
    timeline: Optional[Timeline] = None
    investment: Optional[Investment] = None
    team: Optional[Team] = None
    next_steps: Optional[NextSteps] = None
    closing: Optional[Closing] = None
```

### Template Jinja2 context

Templates receive:
- `content` — the loaded content model instance
- `style` — the loaded style model instance
- `meta` — `{rendered_date, forma_version}`

Reference paths freely: `{{ content.client.name }}`, `{{ content.investment.total_usd | currency }}`.

### Jinja2 filters

Defined in `src/forma/renderer/filters.py`. All string content MUST pass
through `latex_escape` (alias: `le`) before embedding in LaTeX source.

| Filter | Purpose |
|--------|---------|
| `latex_escape` / `le` | Escape LaTeX special chars |
| `format_date` | Parse + reformat dates |
| `currency` | Format numbers as currency |
| `hex_color` | Strip `#` from hex color for xcolor |
| `join_oxford` | Oxford-comma list join |
| `bullet_list` | Render list as `\itemize` |

---

## CI/CD patterns (from lypto)

- `build.yml` delegates to `{owner}/opv-actions@v3` — org-invariant
- `build-base.yml` uses `opv-actions/build-base` — rebuilds only when deps change
- `docs.yml` is self-contained — full history fetch, mkdocs gh-deploy
- `publish.yml` is forma-specific — auto-discovers `documents/*/forma.yaml`, renders, uploads

Secrets:
- `PAT_ORG_REPO_READ` — private org repo access
- `PAT_BADGES` — write coverage badges
- `GOOGLE_SERVICE_ACCOUNT_JSON` — base64-encoded service account JSON
- `ANTHROPIC_API_KEY` — for compose commands only

---

## Devcontainer (from lypto pattern)

- `setup-host`: extracts ORGANIZATION/PROJECT from git remote, GHCR login, writes `.env`
- `setup-editor`: installs zsh + oh-my-zsh + tmux (gpakosz), `make setup-dev`, Claude Code CLI
- Uses `Dockerfile` target `base` via `.devcontainer/docker-compose.yaml`
- Terminal default: tmux (session name `dev`)
- VS Code extensions: Python, YAML, LaTeX Workshop, GitHub Actions, Claude Code

---

## Milestones and open issues

### ✅ v0.1 — Engine Foundation (COMPLETE)
- BaseContent, BaseStyle, FormaConfig, loader, validator
- ProposalContent, CaseStudyContent, BriefContent starter schemas
- forma schema export
- forma validate CLI
- example-client fixture with full content.yaml

### ✅ v0.2 — Rendering Pipeline (COMPLETE)
- BaseRenderer, context, filters, engine (TemplateManifest + render_template)
- proposal-slides (Beamer) template + all _partials
- proposal-report (LaTeX article) template
- proposal-brief (one-pager) template
- forma render CLI (--template, --watch)

### ✅ v0.3 — Devcontainer & CI/CD (COMPLETE)
- devcontainer.json, setup-host, setup-editor, .tmux.conf
- docker-compose.yaml, Dockerfile, Dockerfile.base
- Makefile
- build.yml, build-base.yml, docs.yml, publish.yml

### 🔲 v0.4 — Claude Composer (NOT STARTED)
The engine + prompts + filler are written. The CLI commands exist.
Remaining work:
- [ ] Integration test for `forma compose fill` with a real ANTHROPIC_API_KEY
- [ ] Test that `filler.py` validates against schema and raises cleanly on bad output
- [ ] `forma init` scaffold test
- [ ] Review prompt quality with actual meeting notes

### 🔲 v0.5 — Skills Submodule (NOT STARTED)
- [ ] Create `slivern-corporate-services/forma-skills` repository
- [ ] Add as git submodule: `git submodule add https://github.com/slivern-corporate-services/forma-skills skills`
- [ ] Implement `skills/clickup/fetch.py` — fetch tasks from a ClickUp list by ID
- [ ] Implement `skills/google_docs/fetch.py` — fetch document text by doc ID
- [ ] Implement `skills/google_sheets/fetch.py` — fetch sheet as a list of dicts
- [ ] Implement `skills/meeting_notes/parse.py` — parse structured meeting notes format
- [ ] `forma compose enrich` integration test

### 🔲 v1.0 — Publishing (PARTIAL)
- [x] `publisher/google_drive.py` — written, needs integration test
- [x] `publish.yml` workflow — written
- [ ] Integration test for publish (mock Drive API)
- [ ] End-to-end CI test: validate → render → mock publish
- [ ] Deployment + secrets documentation in README

---

## Companion repositories needed

1. **`slivern-corporate-services/forma-skills`** — the skills submodule repo.
   Each skill is a directory with `fetch.py` exposing `fetch(**kwargs) -> dict`.

2. **`slivern-corporate-services/opv-actions`** — org-level reusable CI actions.
   Currently exists in `sigil-enterprises/opv-actions`. Either fork it into
   `slivern-corporate-services` or update `build.yml` + `build-base.yml` to
   point at the sigil repo directly if cross-org access is acceptable.

---

## Known issues / TODOs

- `style.yaml` is loaded as plain `BaseStyle` in the renderer. A richer
  `FormaStyle` model with all branding.css-derived fields should be defined
  (colors, typography, layout with proper defaults). Currently templates use
  `style.colors.primary_dark` etc. with `| default(...)` fallbacks.

- The `publish` CLI command calls `render_default.callback()` inline which is
  fragile. It should call `render_template()` directly for each template.

- `forma compose enrich` builds combined notes as a YAML dump of fetched data,
  which is not ideal. A structured note-assembly step would improve quality.

- LaTeX font availability: `Rubik` and `Inter` must be installed in the TeX
  live distribution. The `Dockerfile.base` installs `texlive-fonts-extra` which
  should cover them, but this needs verification in CI.

- The `\include` commands in `proposal-slides/main.tex.j2` reference partial
  files. xelatex `\include` expects `.tex` files by name without extension.
  The Jinja2-rendered partials need to be written to disk before the main
  template tries to include them. The current `engine.py` does not handle this.
  **Fix needed**: either use `\input` with inline Jinja2 expansion (render
  everything to a single .tex file), or write partials to the temp dir.
  → Recommended fix: render partials inline in `main.tex.j2` using
  Jinja2 `{% include '_partials/_cover.tex.j2' %}` (already handled by
  Jinja2 FileSystemLoader — just change LaTeX `\include` to Jinja2 `{% include %}`).

- `BaseStyle.from_yaml` uses `model_validate` but `BaseStyle` has no fields
  beyond `publishing`. A `FormaStyle` subclass with all branding tokens needs
  to be created and used as the default style model.

---

## How to run tests

```bash
make setup-dev
make test
```

Or for continuous testing:

```bash
make dev   # runs ptw (pytest-watch)
```

---

## How to render the example document

```bash
pip install -e .[dev]
cd /path/to/forma
forma validate documents/example-client
forma render documents/example-client --template slides
forma render documents/example-client --template report
# Output: var/builds/example-client/slides.pdf, report.pdf
```

Requires xelatex in PATH (`which xelatex`). Inside devcontainer it's available
via the `texlive-xetex` package installed in `Dockerfile.base`.
