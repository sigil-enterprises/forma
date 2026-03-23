# Renderer API Reference

The `forma.renderer` package contains everything needed to turn a mapping document and style dict into a PDF — Jinja2 template rendering, LaTeX subprocess invocation, and the filter library.

---

## render_template()

```python
def render_template(
    template_dir: Path,
    document: dict[str, Any],
    style: dict[str, Any],
    output_path: Path,
    *,
    project_dir: Path | None = None,
) -> Path
```

**The main entry point.** Runs the full pipeline for one template:

1. Loads `manifest.yaml` from `template_dir`
2. Creates a Jinja2 environment pointing at the template directory
3. Renders the Jinja2 entry template into source text (LaTeX or HTML)
4. Invokes the appropriate compile backend (xelatex / pdflatex / lualatex / Playwright)
5. Writes the PDF to `output_path` and returns it

**Parameters:**

| Parameter | Description |
|-----------|-------------|
| `template_dir` | Directory that contains `manifest.yaml` and all template files. |
| `document` | Fully-resolved mapping dict — the result of `load_document()`. |
| `style` | Style dict loaded from `style.yaml`. |
| `output_path` | Where to write the PDF. Parent directories are created automatically. |
| `project_dir` | Project root, passed through to LaTeX for `\graphicspath` / asset resolution. |

```python
from forma.renderer.engine import render_template

render_template(
    template_dir=Path("templates/proposal-slides"),
    document=doc,
    style=style,
    output_path=Path("var/builds/acme/slides.pdf"),
    project_dir=Path("documents/acme"),
)
```

---

## TemplateManifest

`TemplateManifest` parses a template's `manifest.yaml` file and exposes its fields as attributes. Instantiating it with a `template_dir` that has no `manifest.yaml` raises `FileNotFoundError`.

### manifest.yaml fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | `str` | directory name | Human-readable template name. |
| `description` | `str` | `""` | Short description shown by `forma template list`. |
| `format` | `str` | `"document"` | `slides` or `document` — cosmetic only. |
| `engine` | `str` | `"xelatex"` | Rendering engine: `xelatex`, `pdflatex`, `lualatex`, or `html`. |
| `entry` | `str` | `"main.tex.j2"` | Filename of the root Jinja2 template. |
| `compatible_schemas` | `list[str]` | `[]` | Fully-qualified schema class names (informational only). |

Example `manifest.yaml`:

```yaml
name: "Proposal Slides"
format: slides
engine: xelatex
entry: main.tex.j2
compatible_schemas:
  - "forma.schemas.proposal.content:ProposalContent"
```

---

## Jinja2 environment

The Jinja2 environment uses **non-standard delimiters** to avoid conflicts with LaTeX syntax:

| Role | Delimiter | LaTeX standard conflict |
|------|-----------|------------------------|
| Variables | `(( value ))` | `{{ }}` conflicts with LaTeX grouping |
| Blocks | `(% if/for/block %)` | `{% %}` conflicts with `\%` |
| Comments | `(# comment #)` | `{# #}` conflicts less but kept consistent |

All templates (LaTeX and HTML) use the same delimiters for consistency.

The environment uses `StrictUndefined` — referencing a variable that doesn't exist in the context raises an error immediately rather than silently rendering as an empty string. For optional dict keys, use `.get()`:

```
(( document.meta.get("subtitle", "") | le ))
```

Partial templates in `_slides/` or `_partials/` subdirectories are automatically added to the search path and can be included with `{% include "_cover.tex.j2" %}` (using standard Jinja2 `include` syntax, not the custom `!include` tag).

---

## Template context

`build_context()` assembles the dict that is passed as `**kwargs` to `template.render()`:

| Key | Value |
|-----|-------|
| `document` | The loaded mapping dict (e.g. `SlideDocument`). |
| `content` | Alias for `document` — templates may use either name. |
| `style` | The style dict loaded from `style.yaml`. |
| `meta.rendered_date` | Today's date as an ISO-8601 string (`YYYY-MM-DD`). |
| `meta.forma_version` | The installed forma version string. |
| `meta.project_dir` | Absolute path to the project root (string). |
| `meta.presskit_root` | Absolute path two levels above the template directory (string). |

Access paths freely in templates:

```
(( document.slides[0].title | le ))
(( style.colors.primary | hex_color ))
(( meta.rendered_date | format_date("%B %Y") ))
```

---

## Filters

All filters are registered in the Jinja2 environment automatically. Filters operate on the already-loaded Python values from the mapping dict.

### `latex_escape` / `le`

Escape a string for safe inclusion in LaTeX source. Every string value from YAML **must** pass through this filter before being embedded in `.tex` output.

Handles: `& % $ # _ { } ~ ^ \` and Unicode arrows, dashes, ellipsis, and non-breaking spaces.

```
(( document.slides[0].title | le ))
(( content.client.name | latex_escape ))
```

Returns `""` for `None`.

### `format_date`

Parse a date string or `date`/`datetime` object and reformat it.

```
(( content.engagement.date | format_date ))
(# → "March 21, 2026" (default format) #)

(( content.engagement.date | format_date("%b %Y") ))
(# → "Mar 2026" #)
```

Accepts ISO format (`2026-03-21`), `DD/MM/YYYY`, `DD-MM-YYYY`, and `Month DD, YYYY`. Returns the original string if no pattern matches.

### `currency`

Format a number as currency.

```
(( content.investment.total_usd | currency ))
(# → "$220,000" #)

(( content.investment.total_usd | currency(symbol="€", decimals=2) ))
(# → "€220,000.00" #)
```

### `hex_color`

Strip the leading `#` from a hex color string for use with LaTeX `xcolor`:

```
\definecolor{primary}{HTML}{(( style.colors.primary | hex_color ))}
(# style.colors.primary = "#1A2B3C" → "1A2B3C" #)
```

### `join_oxford`

Join a list of strings with an Oxford comma.

```
(( content.team.members | map(attribute="name") | list | join_oxford ))
(# → "Alice, Bob, and Carol" #)

(( items | join_oxford(conjunction="or") ))
(# → "A, B, or C" #)
```

### `bullet_list`

Render a Python list as a LaTeX `\itemize` environment. Each item is automatically passed through `latex_escape`.

```
(( content.executive_summary.key_points | bullet_list ))
```

Output:

```latex
\begin{itemize}
  \item First point
  \item Second point
\end{itemize}
```

Pass `indent=1` for nested lists.

---

## BaseRenderer

`BaseRenderer` is an abstract base class for LaTeX compilation backends. Subclasses set the `engine` class attribute (`"xelatex"`, `"pdflatex"`, `"lualatex"`).

The `render()` method:

1. Creates a temporary directory
2. Writes the rendered LaTeX source to `document.tex`
3. Runs the LaTeX engine twice (configurable via `passes`) to resolve cross-references
4. Copies `document.pdf` to `output_path`
5. Cleans up the temporary directory

**Font and asset discovery:**

When `project_dir` is provided, it is prepended to `TEXINPUTS` so `\includegraphics` finds local assets without needing absolute paths in templates.

When `fonts_dirs` is provided (a list of directories containing font files), those directories are added to both `TEXINPUTS` and `OSFONTDIR`. This allows `\setmainfont{Rubik-VariableFont.ttf}` in XeTeX/fontspec to find fonts by filename even if they are not registered with the system font manager.

**Error handling:** If no PDF is produced after compilation, the last 60 lines of the `.log` file are included in the `RuntimeError` message to aid debugging.
