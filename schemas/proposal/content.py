"""
ProposalContent — starter schema for client proposals.

Organizes content by semantic domain. Templates reference these fields
freely via Jinja2 paths (e.g. content.client.name, content.investment.phases).

Projects copy and extend this schema to fit their specific needs.
"""

from __future__ import annotations

from typing import Optional

from pydantic import BaseModel, Field

from forma.core.base import BaseContent, PublishingConfig


# ---------------------------------------------------------------------------
# Engagement metadata
# ---------------------------------------------------------------------------

class Engagement(BaseModel):
    title: str
    subtitle: str = ""
    reference: str = ""
    date: str
    version: str = "1.0"
    confidentiality: str = "confidential"  # public | confidential | strictly-confidential
    language: str = "en"


# ---------------------------------------------------------------------------
# Client
# ---------------------------------------------------------------------------

class ClientContact(BaseModel):
    name: str
    title: str = ""
    email: str = ""


class Client(BaseModel):
    name: str
    industry: str = ""
    size: str = ""
    contact: Optional[ClientContact] = None


# ---------------------------------------------------------------------------
# Executive summary
# ---------------------------------------------------------------------------

class ExecutiveSummary(BaseModel):
    headline: str
    body: str
    key_points: list[str] = Field(default_factory=list)


# ---------------------------------------------------------------------------
# Context (problem / current state)
# ---------------------------------------------------------------------------

class PainPoint(BaseModel):
    title: str
    description: str


class Metric(BaseModel):
    label: str
    value: str
    note: str = ""


class CurrentState(BaseModel):
    description: str = ""
    metrics: list[Metric] = Field(default_factory=list)


class Context(BaseModel):
    problem_statement: str
    pain_points: list[PainPoint] = Field(default_factory=list)
    current_state: Optional[CurrentState] = None


# ---------------------------------------------------------------------------
# Solution
# ---------------------------------------------------------------------------

class Pillar(BaseModel):
    title: str
    description: str
    icon: str = ""


class Differentiator(BaseModel):
    title: str
    description: str


class Solution(BaseModel):
    overview: str
    pillars: list[Pillar] = Field(default_factory=list)
    differentiators: list[Differentiator] = Field(default_factory=list)


# ---------------------------------------------------------------------------
# Timeline
# ---------------------------------------------------------------------------

class Phase(BaseModel):
    name: str
    duration: str
    activities: list[str] = Field(default_factory=list)
    deliverables: list[str] = Field(default_factory=list)
    start_week: Optional[int] = None
    end_week: Optional[int] = None


class Timeline(BaseModel):
    phases: list[Phase] = Field(default_factory=list)


# ---------------------------------------------------------------------------
# Investment
# ---------------------------------------------------------------------------

class LineItem(BaseModel):
    service: str
    quantity: float = 1
    unit: str = ""
    rate_usd: float = 0
    total_usd: float = 0  # computed or explicit

    def model_post_init(self, __context) -> None:
        if not self.total_usd:
            self.total_usd = self.quantity * self.rate_usd


class InvestmentPhase(BaseModel):
    name: str
    duration: str = ""
    line_items: list[LineItem] = Field(default_factory=list)

    @property
    def subtotal_usd(self) -> float:
        return sum(item.total_usd for item in self.line_items)


class Investment(BaseModel):
    currency: str = "USD"
    secondary_currency: str = ""
    exchange_rate: float = 1.0
    notes: list[str] = Field(default_factory=list)
    phases: list[InvestmentPhase] = Field(default_factory=list)

    @property
    def total_usd(self) -> float:
        return sum(p.subtotal_usd for p in self.phases)


# ---------------------------------------------------------------------------
# Team
# ---------------------------------------------------------------------------

class Consultant(BaseModel):
    name: str
    role: str
    credentials: str = ""
    experience_years: Optional[int] = None
    education: list[str] = Field(default_factory=list)
    expertise: list[str] = Field(default_factory=list)
    photo: str = ""


class Partner(BaseModel):
    name: str
    logo: str = ""
    description: str = ""


class Team(BaseModel):
    consultants: list[Consultant] = Field(default_factory=list)
    partners: list[Partner] = Field(default_factory=list)


# ---------------------------------------------------------------------------
# Next steps
# ---------------------------------------------------------------------------

class Step(BaseModel):
    title: str
    description: str
    icon: str = ""


class NextSteps(BaseModel):
    intro: str = ""
    steps: list[Step] = Field(default_factory=list)


# ---------------------------------------------------------------------------
# Closing
# ---------------------------------------------------------------------------

class Closing(BaseModel):
    tagline: str = ""
    email: str = ""
    website: str = ""
    phone: str = ""
    logo: str = ""


# ---------------------------------------------------------------------------
# Root content model
# ---------------------------------------------------------------------------

class ProposalContent(BaseContent):
    engagement: Engagement
    client: Client
    executive_summary: ExecutiveSummary
    context: Optional[Context] = None
    solution: Optional[Solution] = None
    timeline: Optional[Timeline] = None
    investment: Optional[Investment] = None
    team: Optional[Team] = None
    next_steps: Optional[NextSteps] = None
    closing: Optional[Closing] = None
