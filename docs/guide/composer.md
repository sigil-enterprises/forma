# Claude Composer

forma integrates with Claude to draft `content.yaml` from unstructured notes.

## How it works

```
meeting notes  →  build_user_prompt()  →  Claude (claude-opus-4-6)
                                               │
JSON Schema  →  build_system_prompt()  →  ────┘
                                               │
                                          raw YAML response
                                               │
                                        strip markdown fences
                                               │
                                        yaml.safe_load()
                                               │
                                        schema_cls.model_validate()
                                               │
                                          FillResult (content + raw_yaml)
```

1. The system prompt embeds the full JSON Schema so Claude knows exactly what shape to produce.
2. Claude outputs only valid YAML — no code fences, no commentary.
3. forma validates the YAML against the Pydantic schema. If validation fails, the error is raised immediately.
4. Required fields with no data are filled with `TODO:` placeholders for human review.

## forma compose fill

```bash
forma compose fill documents/acme-corp \
  --notes meeting-notes.md
```

| Option | Default | Description |
|---|---|---|
| `--notes / -n` | required | Path to notes file |
| `--model / -m` | `claude-opus-4-6` | Claude model ID |
| `--max-tokens` | `8192` | Maximum output tokens |
| `--dry-run` | off | Print YAML, do not write |
| `--overwrite` | off | Skip confirmation when `content.yaml` exists |

### Incremental filling

If `content.yaml` already exists, forma reads it and passes it to Claude alongside the notes. Claude preserves all existing fields and only fills gaps:

```bash
# First pass — draft from initial notes
forma compose fill documents/acme-corp --notes discovery.md

# Second pass — add investment detail from follow-up notes
forma compose fill documents/acme-corp \
  --notes followup.md \
  --overwrite
```

### Dry run

Preview the output without writing:

```bash
forma compose fill documents/acme-corp \
  --notes notes.md \
  --dry-run
```

## forma compose enrich

Fetches external data via the [skills submodule](skills.md), then composes:

```bash
forma compose enrich documents/acme-corp \
  --skills clickup,google_docs \
  --notes context.md
```

forma calls each named skill's `fetch()` function, formats the results as structured prose, combines them with your notes, and passes everything to Claude.

| Option | Description |
|---|---|
| `--skills / -s` | Comma-separated skill names (e.g. `clickup,google_docs`) |
| `--notes / -n` | Optional supplementary notes file |
| `--model / -m` | Claude model ID |
| `--dry-run` | Print YAML, do not write |

## Prompt design

The system prompt:

- Embeds the full JSON Schema of the target schema class
- Instructs Claude to output **only** valid YAML
- Requires `TODO:` prefixes on fields with no data
- Prohibits invented facts

The user prompt wraps the notes and, if present, the existing `content.yaml`.

## Environment

```bash
ANTHROPIC_API_KEY=sk-ant-...   # required
```

Raises `EnvironmentError` at invocation time if the key is missing.
