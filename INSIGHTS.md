# INSIGHTS.md — Forma

## What Forma Is

Forma is a **schema-agnostic document rendering framework**. It turns structured YAML content combined with Jinja2 templates into PDF documents — slide decks, reports, briefs, case studies, and status reports.

The core abstraction separates **semantic content** (what you want to say) from **document structure** (how it is arranged) and **visual style** (how it looks):

```
content.yaml ──[!include]──> slides.yaml / report.yaml (mapping)
                                        |
style.yaml   ───────────────> Jinja2 template --> xelatex/Playwright --> PDF
```

## The Presskit Ecosystem

Forma powers a document generation ecosystem across **7 organizations**, each with its own presskit containing brand assets, fonts, and pre-configured document projects:

| Organization | Focus |
|---|---|
| **Sigil Enterprises** | Cloud architecture, healthcare data, digital transformation |
| **MTG Research** | Research advisory, publications, certifications |
| **Asklepic** | Healthcare data platform (Weave product) |
| **Slivern Corporate Services** | Corporate services and consulting |
| **Glacerion** | Private holding company (minimal disclosure) |
| **Medipal** | PROMs/PREMs, remote monitoring, EHR integration (Stockholm) |
| **Opvance** | Operations and performance consulting |

Each presskit typically contains proposal, report, and brief projects that render through forma's pipeline.

## The Landing-Page Ecosystem

Six of the organizations share a common **Vite + React + TypeScript + Radix UI + Tailwind CSS + Framer Motion** landing page, originally built for MTG Research and forked/restyled per org:

| Site | Brand Colors | Status |
|---|---|---|
| MTG Research | Teal `#003B50` + Cyan `#35C5F5` | Most complete (template origin) |
| Sigil Enterprises | Navy `#061E30` + Gold `#FDB62B` | Content customized, needs styling pass |
| Asklepic | Slate `#0B1120` + Blue `#3B82F6` | Fully styled and deployed |
| Slivern | Indigo `#1A1A2E` + Silver `#C0C0C0` | Styled, placeholder content |
| Glacerion | Arctic `#0C1B2A` + Ice `#7DD3FC` | Intentionally minimal |
| Medipal | Stone `#1C1917` + Amber `#F59E0B` | Styled, deployed |

All landing pages use YAML-driven content (`content/site.yaml`) — the same philosophy as forma's document pipeline.

## Architecture

### Rendering Pipeline

1. `FormaConfig.from_yaml()` loads `forma.yaml` (per-project configuration)
2. `load_document()` loads the mapping file, resolving `!include "@content.yaml:dot.path"` tags
3. `load_style()` loads visual tokens (colors, fonts, spacing)
4. `build_context()` assembles the Jinja2 context: `{content, document, style, meta}`
5. `TemplateManifest` reads `manifest.yaml` to select engine and entry template
6. Jinja2 renders to LaTeX or HTML source (using `(( ))` / `(% %)` delimiters to avoid LaTeX conflicts)
7. `BaseRenderer` (xelatex/pdflatex/lualatex) or `HtmlRenderer` (Playwright) compiles to PDF

### Content Schemas (Pydantic v2)

- `ProposalContent` — engagement proposals with investment tables
- `BriefContent` — executive briefs
- `CaseStudyContent` — case study narratives
- `StatusReportContent` — project status reports

### JSON Schema Validation (draft-07)

- `forma-config.schema.yaml` — project configuration
- `proposal-content.schema.yaml` — proposal content validation
- `slide-document.schema.yaml` — slide deck structure
- `report-document.schema.yaml` — report document structure

## Template Inventory

| Template | Engine | Format | Description |
|---|---|---|---|
| `proposal-slides` | xelatex | slides | LaTeX Beamer proposal deck |
| `proposal-slides-html` | html | slides | HTML/Playwright proposal deck |
| `proposal-report` | xelatex | document | Multi-page proposal report |
| `proposal-brief` | xelatex | document | Executive brief (1-2 pages) |
| `case-study` | xelatex | document | Case study narrative |
| `status-report` | (schema defined) | document | Project status report |
| `pitch-deck` | (planned) | slides | Investor/partner pitch deck |

## Composer (AI-Assisted Content)

The `forma compose fill` command uses the Anthropic SDK to generate structured YAML content from unstructured notes. It is schema-aware — the Pydantic content models are converted to prompts so the LLM output validates against the schema.

## Publisher

`forma publish` uploads rendered PDFs to Google Drive via service account credentials (base64-encoded in CI). Supports configurable folder IDs and filename prefixes per project.

## Known Limitations

- **No pitch-deck template yet** — schema and template planned but not implemented
- **Status-report template** — Pydantic schema exists (`StatusReportContent`) but no rendering template in fixtures
- **LaTeX dependency** — xelatex-based templates require a full TeX Live installation; the HTML engine (Playwright) is more portable but only covers proposal-slides so far
- **No incremental rendering** — every render rebuilds from scratch; no caching of intermediate LaTeX/HTML
- **Single-language** — no i18n support in templates or content schemas
- **Font management** — fonts are threaded via `TEXINPUTS` + `OSFONTDIR` from presskit directories, which works but is fragile across environments

## Next Steps

1. **Expand HTML engine coverage** — port proposal-report and proposal-brief to HTML/Playwright to reduce xelatex dependency
2. **Implement pitch-deck template** — high demand from the sales workflow
3. **Add status-report template** — schema already defined, needs Jinja2 template and manifest
4. **Template versioning** — templates are currently path-referenced; add version pinning for reproducibility
5. **Incremental builds** — cache intermediate artifacts to speed up re-renders
6. **Multi-language support** — add i18n layer for content schemas and templates
7. **Landing page template extraction** — extract the MTG template into a reusable scaffold with a `forma init`-style CLI for new org sites
