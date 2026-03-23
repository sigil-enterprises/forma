"""
StatusReportContent — schema for project status reports.

Covers RAG status, milestones, budget, risks, and actions for ongoing
engagement updates.
"""

from __future__ import annotations

from typing import Optional

from pydantic import BaseModel, Field

from forma.core.base import BaseContent


class ProjectInfo(BaseModel):
    name: str
    client: str
    period_start: str               # ISO date: "2026-03-01"
    period_end: str                 # ISO date: "2026-03-31"
    overall_rag: str = "green"      # "green" | "amber" | "red"
    phase: str = ""                 # e.g. "Phase 2 — Development"


class Milestone(BaseModel):
    name: str
    due_date: str                   # ISO date
    status: str = "on-track"        # "done" | "on-track" | "at-risk" | "delayed"
    notes: str = ""


class Budget(BaseModel):
    planned_usd: float
    actual_usd: float
    forecast_usd: float
    notes: str = ""


class Risk(BaseModel):
    description: str
    severity: str = "medium"        # "high" | "medium" | "low"
    mitigation: str
    owner: str = ""


class Action(BaseModel):
    action: str
    owner: str = ""
    due_date: str = ""
    status: str = "pending"         # "done" | "in-progress" | "pending"


class StatusReportContent(BaseContent):
    project: ProjectInfo
    summary: str
    milestones: list[Milestone] = Field(default_factory=list)
    budget: Optional[Budget] = None
    risks: list[Risk] = Field(default_factory=list)
    actions: list[Action] = Field(default_factory=list)
    next_period_focus: str = ""
