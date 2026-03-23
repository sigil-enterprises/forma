# Schemas

Schemas define the structure and validation rules for `content.yaml` files.
They are Pydantic models that extend `BaseContent`.

## Starter schemas

Three schemas ship with forma:

| Schema | Class | Use case |
|---|---|---|
| `src/forma/schemas/proposal/` | `ProposalContent` | Full proposal (slides + report) |
| `src/forma/schemas/brief/` | `BriefContent` | One-pager / executive brief |
| `src/forma/schemas/case_study/` | `CaseStudyContent` | Client case study |

## How a schema works

```python
# schemas/proposal/content.py

from forma.core.base import BaseContent
from pydantic import BaseModel
from typing import Optional

class Client(BaseModel):
    name: str
    industry: Optional[str] = None
    contact: Contact           # nested model

class ProposalContent(BaseContent):
    engagement: Engagement     # required
    client: Client             # required
    executive_summary: ExecutiveSummary
    context: Optional[Context] = None
    solution: Optional[Solution] = None
    investment: Optional[Investment] = None
    team: Optional[Team] = None
```

The engine works with **any** `BaseContent` subclass. Templates receive the model instance as `content` and can reference any field: `(( content.client.name ))`.

## Creating a new schema

1. Create `src/forma/schemas/mytype/content.py` extending `BaseContent`:

    ```python
    from forma.core.base import BaseContent
    from pydantic import BaseModel
    from typing import Optional

    class MyMeta(BaseModel):
        title: str
        date: str
        prepared_for: str

    class MyContent(BaseContent):
        meta: MyMeta
        body: Optional[str] = None
    ```

2. Export its JSON Schema:

    ```bash
    forma schema export
    ```

3. Register it in `src/forma/core/loader.py` if you want `forma validate` to check it:

    ```python
    from forma.core.loader import register_schema
    from pathlib import Path
    register_schema("MyContent", Path("path/to/my-content.schema.yaml"))
    ```

## Computed properties

Use Pydantic `@property` or `model_validator` for derived values:

```python
from pydantic import computed_field

class Investment(BaseModel):
    phases: list[InvestmentPhase]

    @computed_field
    @property
    def total_usd(self) -> float:
        return sum(p.subtotal_usd for p in self.phases)
```

Templates can then use `(( content.investment.total_usd | currency ))`.

## Asset paths

Any string field ending in `.png`, `.jpg`, `.svg`, `.pdf`, or `.eps` is treated as an asset path and checked by `forma validate`. Missing assets produce warnings (or errors with `--strict`).

## Listing schemas

```bash
forma schema export
```

Lists the built-in JSON Schema YAML files bundled with the package (under `src/forma/schema/`). These can be used by editors for YAML autocompletion via YAML Language Server.
