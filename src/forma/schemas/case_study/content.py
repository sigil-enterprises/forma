"""
CaseStudyContent — schema for client case studies and success stories.
"""

from __future__ import annotations

from typing import Optional

from pydantic import BaseModel, Field

from forma.core.base import BaseContent


class CaseStudyMeta(BaseModel):
    title: str
    subtitle: str = ""
    client_name: str
    industry: str = ""
    date: str
    confidentiality: str = "public"


class Challenge(BaseModel):
    statement: str
    details: list[str] = Field(default_factory=list)


class Approach(BaseModel):
    overview: str
    steps: list[str] = Field(default_factory=list)


class Outcome(BaseModel):
    headline: str
    results: list[str] = Field(default_factory=list)
    quote: str = ""
    quote_attribution: str = ""


class CaseStudyContent(BaseContent):
    meta: CaseStudyMeta
    challenge: Challenge
    approach: Approach
    outcomes: Outcome
    technologies: list[str] = Field(default_factory=list)
    logo: str = ""
    hero_image: str = ""
