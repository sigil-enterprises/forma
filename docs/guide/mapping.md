# Mapping files

Mapping files (`slides.yaml`, `report.yaml`, etc.) are the bridge between semantic content and template structure. They define what each slide or section contains, pulling values from `content.yaml` via the `!include` tag.

The two-level split means:

- `content.yaml` is **schema-driven** — it describes *what*, not *how*. It never contains slide structure.
- The mapping file is **template-driven** — it defines the exact structure the template expects, and cherry-picks the relevant fields from `content.yaml`.

```
content.yaml
    │
    │  !include "@content.yaml:engagement.title"
    ▼
slides.yaml  ──────────────────────────────────→  proposal-slides template  →  slides.pdf
report.yaml  ──────────────────────────────────→  proposal-report template  →  report.pdf
```

---

## Structure

### SlideDocument

```yaml
resourceType: SlideDocument

slides:
  - type: cover
    title: !include "@content.yaml:engagement.title"
    subtitle: !include "@content.yaml:engagement.subtitle"
    client: !include "@content.yaml:client.name"
    date: !include "@content.yaml:engagement.date"

  - type: exec_summary
    headline: !include "@content.yaml:executive_summary.headline"
    key_points: !include "@content.yaml:executive_summary.key_points"

  - type: closing
    tagline: !include "@content.yaml:closing.tagline"
    email: !include "@content.yaml:closing.email"
```

The `slides` list is passed directly to the template as `document.slides`. Each slide dict is free-form — the template decides what fields to render for each `type`.

### ReportDocument

```yaml
resourceType: ReportDocument

meta:
  title: !include "@content.yaml:engagement.title"
  client: !include "@content.yaml:client.name"
  date: !include "@content.yaml:engagement.date"

chapters:
  - title: "Executive Summary"
    sections:
      - title: "Overview"
        blocks:
          - type: paragraph
            text: !include "@content.yaml:executive_summary.headline"
```

Report templates typically iterate over `document.chapters` and then over `chapter.sections` and `section.blocks`.

---

## `!include` tag reference

The `!include` tag is available only inside mapping files. It is resolved by `load_document()` before the mapping dict is passed to the template renderer. Plain `yaml.safe_load` does not recognise it.

### Syntax

```
!include "@<file-ref>[:<dot-path>]"
```

The `@` prefix is required. The file reference is resolved relative to the **project root** (the directory containing `forma.yaml`), regardless of where the mapping file itself lives.

### Load an entire file

```yaml
all_content: !include "@content.yaml"
```

The value becomes the entire parsed dict from `content.yaml`.

### Extract a single field

```yaml
title: !include "@content.yaml:engagement.title"
```

Loads `content.yaml` and traverses `engagement → title`.

### Extract a list

```yaml
key_points: !include "@content.yaml:executive_summary.key_points"
```

Returns the list at that path. The result is a proper YAML sequence — the template can iterate over it.

### Traverse nested structures

Each dot-separated segment is a dict key:

```yaml
email: !include "@content.yaml:client.contact.email"
```

To access a list element by index, use a plain integer as the segment:

```yaml
first_phase: !include "@content.yaml:timeline.phases.0"
```

### Relative paths

The `@` path is always relative to the project root. To reference files outside the project:

```yaml
colors: !include "@../shared/brand.yaml:colors"
```

### Caching

Within one `load_document()` call, each file is read from disk only once. All `!include` tags for the same file share the cached parse result.

### Error messages

- **Missing file:** `FileNotFoundError` with the resolved path, the `base_dir`, and the original `@ref` string.
- **Bad dot-path:** `KeyError` with the full reference and the available keys at the failing level.

---

## Configuring mapping files

Each template entry in `forma.yaml` declares its mapping file:

```yaml
templates:
  slides:
    path: ../../templates/proposal-slides
    mapping: slides.yaml
  report:
    path: ../../templates/proposal-report
    mapping: report.yaml
```

`mapping` is relative to the project root. The CLI loads the mapping file with `load_document()` and validates it against the `resourceType`'s JSON Schema before rendering.

---

## JSON Schema validation

Mapping files declare their `resourceType` and are validated against the corresponding JSON Schema in `src/forma/schema/`:

| `resourceType` | Schema file |
|----------------|-------------|
| `SlideDocument` | `slide_document.schema.yaml` |
| `ReportDocument` | `report_document.schema.yaml` |

Run `forma validate documents/my-project` to check all mapping files and `content.yaml` before rendering.

---

## Template context

After loading and validating, the mapping dict is passed to `render_template()` as `document`. Templates receive it under both `document` and `content` (an alias):

```
(% for slide in document.slides %)
  (% if slide.type == "cover" %)
    \frametitle{(( slide.title | le ))}
  (% endif %)
(% endfor %)
```

The full context also includes:

| Key | Description |
|-----|-------------|
| `style` | Style dict from `style.yaml` |
| `meta.rendered_date` | ISO-8601 date of the render (e.g. `"2026-03-23"`) |
| `meta.forma_version` | Installed forma version |
| `meta.project_dir` | Absolute path to the project root |
| `meta.presskit_root` | Absolute path two levels above the template directory |

---

## Worked example: cover slide

The following traces a cover slide from `content.yaml` source values through the mapping file to the rendered LaTeX.

**`content.yaml`** (excerpt):

```yaml
engagement:
  title: "Digital Transformation Strategy"
  subtitle: "A phased modernisation roadmap for Acme Corp"
  reference: "SCS-2026-001"
  date: "2026-03-21"

client:
  name: "Acme Corp"
```

**`slides.yaml`**:

```yaml
resourceType: SlideDocument

slides:
  - type: cover
    title: !include "@content.yaml:engagement.title"
    subtitle: !include "@content.yaml:engagement.subtitle"
    client: !include "@content.yaml:client.name"
    reference: !include "@content.yaml:engagement.reference"
    date: !include "@content.yaml:engagement.date"
```

After `load_document()` resolves the `!include` tags, the first slide dict is:

```python
{
    "type": "cover",
    "title": "Digital Transformation Strategy",
    "subtitle": "A phased modernisation roadmap for Acme Corp",
    "client": "Acme Corp",
    "reference": "SCS-2026-001",
    "date": "2026-03-21",
}
```

**`_slides/cover.tex.j2`** (partial template):

```latex
\begin{frame}[plain]
  \vfill
  \textbf{(( slide.title | le ))}\\[0.5em]
  \textit{(( slide.subtitle | le ))}\\[1em]
  (( slide.client | le )) \quad \textbar \quad (( slide.date | format_date ))
  \vfill
\end{frame}
```

The `| le` filter (alias for `latex_escape`) ensures that any special characters in the content — ampersands, underscores, percent signs — are escaped before LaTeX processes them. The `| format_date` filter converts `"2026-03-21"` to `"March 21, 2026"`.
