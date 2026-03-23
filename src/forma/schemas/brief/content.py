"""
BriefContent — minimal one-pager schema.

For executive briefs, summaries, and one-page overviews.
Projects can extend or replace this with their own fields.
"""

from __future__ import annotations

from typing import Optional

from pydantic import BaseModel, Field

from forma.core.base import BaseContent


class BriefMeta(BaseModel):
    title: str
    subtitle: str = ""
    date: str
    prepared_for: str = ""
    prepared_by: str = ""


class BriefSection(BaseModel):
    heading: str
    body: str
    bullets: list[str] = Field(default_factory=list)


class BriefContent(BaseContent):
    meta: BriefMeta
    sections: list[BriefSection] = Field(default_factory=list)
    call_to_action: str = ""
    contact_email: str = ""
    logo: str = ""
