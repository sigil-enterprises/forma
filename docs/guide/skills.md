# Skills

Skills are data-fetching modules in the `skills/` git submodule
(`slivern-corporate-services/forma-skills`). Each skill pulls data from an
external system and returns a dict that the composer can use.

## Available skills

| Skill | Fetches | Env vars required |
|---|---|---|
| `clickup` | Tasks from a ClickUp list | `CLICKUP_API_TOKEN`, `CLICKUP_LIST_ID` |
| `google_docs` | Document text from Google Docs | `GOOGLE_SERVICE_ACCOUNT_JSON`, `GOOGLE_DOCS_ID` |
| `google_sheets` | Rows from a Google Sheet | `GOOGLE_SERVICE_ACCOUNT_JSON`, `GOOGLE_SHEETS_ID` |
| `meeting_notes` | Parsed structured markdown notes | `MEETING_NOTES_PATH` (or pass `path` kwarg) |

## Using skills

```bash
forma compose enrich documents/acme-corp \
  --skills clickup,google_docs \
  --notes extra-context.md
```

Skills are loaded dynamically from `skills/<name>/fetch.py`. Each must expose:

```python
def fetch(**kwargs) -> dict:
    ...
```

## meeting_notes format

The `meeting_notes` skill parses markdown files in this structure:

```markdown
# Meeting: Q1 Strategy Review

Date: 2026-03-21
Attendees: Alice Chen, Bob Smith, Carol Jones

## Action Items
- [x] Send proposal draft to client
- [ ] Schedule kick-off call

## Notes
We agreed on the three-phase approach. Budget is confirmed at $220k.
Alice flagged a compliance requirement around data residency.
```

Returns:

```python
{
    "title": "Q1 Strategy Review",
    "date": "2026-03-21",
    "attendees": ["Alice Chen", "Bob Smith", "Carol Jones"],
    "action_items": ["Send proposal draft to client", "Schedule kick-off call"],
    "notes": "We agreed on the three-phase approach..."
}
```

## Writing a new skill

1. Create `skills/my_skill/fetch.py` in the submodule:

    ```python
    import os

    def fetch(list_id: str | None = None, **kwargs) -> dict:
        list_id = list_id or os.environ["MY_API_LIST_ID"]
        # ... fetch and return data
        return {"items": [...]}
    ```

2. Add its env vars to `.env.example`.

3. Use it:

    ```bash
    forma compose enrich documents/acme-corp \
      --skills my_skill
    ```

## Skills loader

`forma.integrations.skills_loader.load_skills()` loads skills dynamically:

- Missing skills (no `fetch.py`) produce a warning and are skipped
- Skills that raise exceptions are skipped; other skills still run
- `**kwargs` from `load_skills()` are forwarded to each skill's `fetch()`

```python
from forma.integrations.skills_loader import load_skills
from pathlib import Path

results = load_skills(
    Path("skills"),
    ["clickup", "meeting_notes"],
    path="notes/meeting.md",
)
# results = {"clickup": {...}, "meeting_notes": {...}}
```
