# Templates

Templates are Jinja2-wrapped LaTeX files that define document structure.
The same `content.yaml` can render through any number of templates.

## Template layout

```
templates/proposal-slides/
├── manifest.yaml        # metadata: engine, format, compatible schemas
├── main.tex.j2          # entry point
└── _partials/           # included sections
    ├── _cover.tex.j2
    ├── _exec_summary.tex.j2
    ├── _solution.tex.j2
    └── ...
```

## manifest.yaml

```yaml
name: "Proposal Slides"
description: "16:9 Beamer slide deck for proposals"
format: slides           # slides | document
engine: xelatex          # xelatex | pdflatex | lualatex
entry: main.tex.j2
compatible_schemas:
  - "schemas.proposal.content:ProposalContent"
```

## Jinja2 delimiters

Standard `{{ }}` and `{% %}` conflict with LaTeX brace and percent syntax.
forma uses custom delimiters throughout all `.tex.j2` files:

| Purpose | Delimiter |
|---|---|
| Variables | `(( value ))` |
| Blocks | `(% if ... %) ... (% endif %)` |
| Comments | `(# this is ignored #)` |

```latex
\author{ ((- content.client.name | latex_escape -)) }

(% if content.solution %)
  \section{Our Approach}
  (( content.solution.overview | latex_escape ))
(% endif %)
```

## Available filters

All filters are defined in `src/forma/renderer/filters.py`.

| Filter | Alias | Description |
|---|---|---|
| `latex_escape` | `le` | Escape LaTeX special chars (`& % $ # _ { } ~ ^ \`) |
| `format_date` | — | Parse and reformat date strings → `March 21, 2026` |
| `currency` | — | Format numbers as currency → `$50,000` |
| `hex_color` | — | Strip `#` from hex colors for `\definecolor` |
| `join_oxford` | — | Oxford comma list join → `a, b, and c` |
| `bullet_list` | — | Render a list as a LaTeX `itemize` environment |

!!! warning "Always escape user content"
    Every string value from `content.yaml` **must** pass through `latex_escape`
    (or its alias `le`) before being embedded in LaTeX source.

    ```latex
    (( content.client.name | le ))       {# ✓ safe #}
    (( content.client.name ))             {# ✗ unsafe — may break compilation #}
    ```

## Jinja2 context

Templates receive three top-level variables:

| Variable | Type | Description |
|---|---|---|
| `content` | `BaseContent` subclass | The loaded content model |
| `style` | `FormaStyle` | Colors, fonts, layout tokens |
| `meta` | `dict` | `rendered_date`, `forma_version` |

Access any field: `(( content.engagement.title | le ))`, `(( style.colors.primary_dark | hex_color ))`.

## Including partials

Use Jinja2 `{% include %}` (not LaTeX `\include`) to compose partials:

```latex
\begin{document}

(% include '_cover.tex.j2' %)

(% if content.executive_summary %)
(% include '_exec_summary.tex.j2' %)
(% endif %)

\end{document}
```

The `FileSystemLoader` is configured to resolve both `template_dir` and `template_dir/_partials`.

## Creating a new template

1. Create `templates/mytemplate/manifest.yaml` and `main.tex.j2`
2. Add the template to a project's `forma.yaml`:

    ```yaml
    templates:
      mytemplate:
        path: ../../templates/mytemplate
    ```

3. Render:

    ```bash
    forma render documents/my-project --template mytemplate
    ```

## Style tokens in templates

Style values flow from `style.yaml` → `FormaStyle` model → Jinja2 context:

```latex
\definecolor{accent}{HTML}{ ((- style.colors.primary_accent | hex_color -)) }
\setmainfont{ ((- style.typography.font_primary | default('Helvetica') -)) }
```

Default values (via `| default(...)`) protect against minimal `style.yaml` files.
