# Core API Reference

The `forma.core` package handles project configuration, schema loading, document parsing, and validation. These are the foundational components that every rendering pipeline depends on.

---

## FormaConfig

`FormaConfig` represents the contents of a project's `forma.yaml` file. It is a Pydantic model and is the authoritative source of truth for which content file, style file, templates, and output location a project uses.

### Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `resourceType` | `str` | `"FormaConfig"` | Discriminator field. Must be `FormaConfig` in YAML. |
| `content` | `str` | `"content.yaml"` | Path to the content file, relative to the project root. |
| `style` | `str` | `"style.yaml"` | Path to the style file, relative to the project root. |
| `templates` | `dict[str, TemplateEntry]` | `{}` | Named template entries. Each key is a template name (e.g. `slides`, `report`). |
| `output_dir` | `str` | `"../../var/builds"` | Where rendered artifacts are written, relative to the project root. |
| `publishing` | `PublishOverride` | (empty) | Google Drive publishing overrides. |

### `FormaConfig.from_yaml(path)`

```python
@classmethod
def from_yaml(cls, path: Path) -> FormaConfig
```

Load a `forma.yaml` file from disk and return a validated `FormaConfig` instance. Raises a Pydantic `ValidationError` if the file does not conform to the schema.

```python
config = FormaConfig.from_yaml(Path("documents/my-client/forma.yaml"))
```

### Resolve methods

These methods convert the relative path strings stored in the config into absolute `Path` objects by resolving them against the project root.

**`resolve_template_path(name, project_root)`** — Returns the absolute path to the template directory for the named entry. Raises `KeyError` if the name is not in `templates`.

**`resolve_mapping_path(name, project_root)`** — Returns the absolute path to the mapping file (e.g. `slides.yaml`) for the named template entry.

**`resolve_style_path(project_root)`** — Returns the absolute path to the style YAML file.

**`resolve_content_path(project_root)`** — Returns the absolute path to the content YAML file.

**`resolve_output_dir(project_root)`** — Returns the absolute path to the output directory. The directory is not created by this call.

All five methods have the same signature pattern:

```python
def resolve_*(self, [name: str,] project_root: Path) -> Path
```

---

## TemplateEntry

`TemplateEntry` is a Pydantic model that represents one entry in the `templates` dict of `forma.yaml`.

| Field | Type | Description |
|-------|------|-------------|
| `path` | `str` | Path to the template directory, relative to the project root. |
| `mapping` | `str` | Path to the mapping file (e.g. `slides.yaml`), relative to the project root. |

Example `forma.yaml` fragment:

```yaml
templates:
  slides:
    path: ../../templates/proposal-slides
    mapping: slides.yaml
  report:
    path: ../../templates/proposal-report
    mapping: report.yaml
```

---

## PublishOverride

`PublishOverride` is a Pydantic model for the `publishing` block inside `forma.yaml`. It overrides per-project publishing settings.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `google_drive_folder_id` | `str` | `""` | Drive folder ID where rendered PDFs are uploaded. |
| `filename_prefix` | `str` | `""` | Prefix prepended to uploaded filenames. |

---

## BaseContent

`BaseContent` is the Pydantic base class that all project content schemas extend. The rendering engine accepts any subclass — it builds a Jinja2 context from the model's fields and passes them to the template.

Projects define their own schema by subclassing `BaseContent`:

```python
from forma.core.base import BaseContent

class ProposalContent(BaseContent):
    engagement: Engagement
    client: Client
    executive_summary: ExecutiveSummary
    # ... additional domain fields
```

### `BaseContent.from_yaml(path)`

```python
@classmethod
def from_yaml(cls, path: Path) -> BaseContent
```

Load a content YAML file and validate it against the subclass schema. This is a plain `yaml.safe_load` — it does not resolve `!include` tags. Use `load_document()` for mapping files that use `!include`.

```python
content = ProposalContent.from_yaml(Path("documents/my-client/content.yaml"))
```

### `BaseContent.json_schema()` and `BaseContent.json_schema_str()`

Return the JSON Schema for the content class as a dict or formatted string respectively. Used internally by the composer to build Claude's system prompt.

---

## BaseStyle and FormaStyle

`BaseStyle` is the Pydantic base for `style.yaml` models. Like `BaseContent`, projects may extend it with custom branding tokens. It has a single typed field:

| Field | Type | Description |
|-------|------|-------------|
| `publishing` | `PublishingConfig` | Google Drive settings embedded in the style file. |

`FormaStyle` extends `BaseStyle` with fully typed branding sections:

| Field | Type | Description |
|-------|------|-------------|
| `brand` | `BrandConfig` | Logo paths. |
| `colors` | `ColorConfig` | Hex color tokens (primary, accent, grays, text). |
| `typography` | `TypographyConfig` | Font families and size scale. |
| `layout` | `LayoutConfig` | Page size and slide aspect ratio. |

Both classes accept `extra = "allow"`, meaning arbitrary additional fields in `style.yaml` will pass through to templates without validation errors.

---

## Document loaders

These functions are in `forma.core.loader` and handle reading YAML files from disk.

### `load_document(path, base_dir)`

```python
def load_document(path: Path, base_dir: Path) -> dict[str, Any]
```

Load a mapping file (`slides.yaml`, `report.yaml`, etc.) and resolve all `!include` tags. The `base_dir` argument is the project root — all `@file` references in `!include` tags are resolved relative to it.

This function does **not** validate the result against a schema. Call `validate_file()` separately if validation is needed.

```python
doc = load_document(
    Path("documents/my-client/slides.yaml"),
    base_dir=Path("documents/my-client"),
)
```

### `load_content(path)`

```python
def load_content(path: Path) -> dict[str, Any]
```

Load a plain YAML file (no `!include` resolution). Use this for `content.yaml` and `forma.yaml`, which do not use the `!include` tag. Raises `FileNotFoundError` if the file does not exist.

The key difference from `load_document()` is that this function uses `yaml.safe_load` directly, while `load_document()` installs a custom constructor that understands the `!include` tag.

### `load_style(path)`

```python
def load_style(path: Path) -> dict[str, Any]
```

Load a `style.yaml` file into a plain dict. Returns an empty dict if the file does not exist — style is always optional.

### `register_schema(resource_type, schema_path)`

```python
def register_schema(resource_type: str, schema_path: Path) -> None
```

Register a new `resourceType` → JSON Schema file mapping at runtime. The built-in registry covers `FormaConfig`, `SlideDocument`, `ReportDocument`, and `ProposalContent`. Call this function before any validation if your project defines a custom `resourceType`.

```python
register_schema("MyCustomDocument", Path("schemas/my-custom.schema.yaml"))
```

---

## `!include` tag reference

The `!include` tag is a YAML extension that mapping files use to pull values from other YAML files — typically `content.yaml`. It is only active when loading with `load_document()`. Plain `yaml.safe_load` does not recognise it.

### Syntax

```
!include "@<file-ref>[:<dot-path>]"
```

The `@` prefix is required. The file reference is resolved relative to the project root (where `forma.yaml` lives), regardless of where the mapping file itself sits.

### Load an entire file

```yaml
data: !include "@content.yaml"
```

The value of `data` becomes the entire parsed content of `content.yaml` as a dict.

### Extract a string field

```yaml
title: !include "@content.yaml:engagement.title"
```

Loads `content.yaml` and traverses the path `engagement → title`, returning the string value.

### Extract a list

```yaml
key_points: !include "@content.yaml:executive_summary.key_points"
```

Returns the list at `executive_summary.key_points`. The result is a proper YAML list, not a string — it can be iterated in templates.

### Traverse nested dicts

```yaml
email: !include "@content.yaml:client.contact.email"
```

Each dot-separated segment is treated as a dict key lookup.

### Traverse list elements by index

When a path segment is a plain integer, it is used as a list index:

```yaml
first_phase: !include "@content.yaml:timeline.phases.0"
```

Retrieves the first element of the `phases` list.

### Relative paths across directories

The `@` path is always relative to the project root, not to the mapping file's location. Use `../` to escape the project root if needed:

```yaml
colors: !include "@../clients/sigil/style.yaml:colors.primary_dark"
```

### Caching

Within a single `load_document()` call, each file is read from disk only once. Subsequent `!include` tags referencing the same file reuse the cached parse result.

### Error messages

If the referenced file does not exist, `FileNotFoundError` is raised with the resolved path, the `base_dir`, and the original `@ref` string. If a dot-path key does not exist, `KeyError` is raised with the full reference and the available keys at the failing level.

---

## Validation

`forma.core.validator` validates YAML documents against their JSON Schema files. Schemas are stored in YAML format using JSON Schema draft-07.

### `ValidationResult`

`ValidationResult` is returned by all validation functions. It has two fields:

- `errors: list[str]` — fatal errors that make the document invalid.
- `warnings: list[str]` — non-fatal notices (e.g. missing optional schema, `jsonschema` not installed).

The `.ok` property returns `True` when `errors` is empty. Call `.print()` to display a formatted summary to the terminal using Rich.

### `validate_file(path, base_dir, *, schema_path)`

```python
def validate_file(
    path: Path,
    base_dir: Path | None = None,
    *,
    schema_path: Path | None = None,
) -> ValidationResult
```

Load a single YAML file and validate it. If `base_dir` is provided, `!include` tags are resolved before validation (appropriate for mapping files). If `base_dir` is `None`, the file is loaded as plain YAML (appropriate for `content.yaml` and `forma.yaml`).

The schema is determined automatically from the `resourceType` field in the document, using the built-in registry. Supply `schema_path` explicitly to override.

### `validate_project(project_dir)`

```python
def validate_project(project_dir: Path) -> ValidationResult
```

Validate all YAML files in a project directory in one call. This covers:

- `content.yaml` — loaded as plain YAML
- `slides.yaml`, `report.yaml`, `brief.yaml` — loaded with `!include` resolution if present
- `forma.yaml` — loaded as plain YAML

Returns a single `ValidationResult` that aggregates errors and warnings from all files.
