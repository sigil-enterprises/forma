# Composer API Reference

The `forma.composer` package wraps the Anthropic API to draft `content.yaml` from unstructured meeting notes. It is only active when `ANTHROPIC_API_KEY` is set in the environment.

---

## fill_from_notes()

```python
def fill_from_notes(
    notes: str,
    schema_cls: type[BaseContent],
    model: str = "claude-opus-4-6",
    max_tokens: int = 8192,
    existing_yaml_path: Path | None = None,
) -> FillResult
```

The main entry point. Sends notes to Claude and returns a validated content instance.

**Parameters:**

| Parameter | Description |
|-----------|-------------|
| `notes` | Raw text notes — meeting transcript, bullet points, any format. |
| `schema_cls` | The `BaseContent` subclass to validate against. Its JSON Schema is embedded in the system prompt. |
| `model` | Claude model ID. Defaults to `claude-opus-4-6`. |
| `max_tokens` | Maximum tokens for the response. 8192 is sufficient for most proposals. |
| `existing_yaml_path` | Optional path to an existing `content.yaml`. If it exists, its contents are appended to the user prompt so Claude preserves already-filled fields. |

**Returns:** a `FillResult` instance.

**Raises:** `pydantic.ValidationError` if the Claude response does not conform to the schema after YAML parsing. This typically means the notes lacked required fields — the raw YAML is included in the exception for debugging.

```python
from forma.composer.filler import fill_from_notes
from forma.schemas.proposal.content import ProposalContent

result = fill_from_notes(
    notes=open("meeting-notes.md").read(),
    schema_cls=ProposalContent,
)
result.content.model_dump()   # validated Pydantic model
print(result.raw_yaml)        # raw YAML string from Claude
```

---

## FillResult

```python
class FillResult:
    content: BaseContent    # validated Pydantic model instance
    raw_yaml: str           # raw YAML string returned by Claude (after fence stripping)
```

The `raw_yaml` field contains the exact text Claude produced (minus any accidental markdown code fences). It is written directly to `content.yaml` by the CLI when `--overwrite` is passed.

---

## Prompts

### build_system_prompt()

```python
def build_system_prompt(schema_cls: type[BaseContent]) -> str
```

Builds the system prompt sent to Claude. It includes:

- Role instruction: expert business consultant drafting structured proposal content
- The full JSON Schema of `schema_cls` embedded as a code block
- Six rules Claude must follow:
  1. Output only valid YAML — no markdown fences
  2. Every required field must be present
  3. Write concise, professional prose for a senior executive audience
  4. Preserve all specific facts, figures, names, and dates from the notes exactly
  5. Use `TODO: ` prefix for required fields that have no data in the notes
  6. Do not invent facts

### build_user_prompt()

```python
def build_user_prompt(notes: str, existing_yaml: str | None = None) -> str
```

Builds the user-turn prompt. Wraps the notes between `---` fences. If `existing_yaml` is provided, appends a second fenced section asking Claude to preserve already-filled fields.

---

## FormaClient

```python
class FormaClient:
    def __init__(self, model: str = "claude-opus-4-6", max_tokens: int = 8192) -> None
    def complete(self, system_prompt: str, user_prompt: str) -> str
```

Thin wrapper around the Anthropic SDK's `messages.create()`. Reads `ANTHROPIC_API_KEY` from the environment — raises `OSError` if not set.

`complete()` sends a single-turn conversation (system + user) and returns the first content block's text. Raises `ValueError` if the response contains a non-text block.

The client is not intended to be called directly; use `fill_from_notes()` instead. It is a separate class to allow test mocking without patching the Anthropic SDK internals.
