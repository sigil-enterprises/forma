"""
forma.yaml project config loader.

Each document project root contains a forma.yaml that declares:
  - resourceType: FormaConfig  (discriminator)
  - content:    path to content.yaml (default: content.yaml)
  - style:      path to style.yaml
  - templates:  named template directories, each with a path + mapping file
  - output_dir: where to write rendered artifacts
  - publishing: Google Drive config
"""

from __future__ import annotations

from pathlib import Path

import yaml
from pydantic import BaseModel, Field


class TemplateEntry(BaseModel):
    path: str          # relative to project root, points to template directory
    mapping: str       # relative to project root, e.g. "slides.yaml"


class PublishOverride(BaseModel):
    google_drive_folder_id: str = ""
    filename_prefix: str = ""


class FormaConfig(BaseModel):
    resource_type: str = Field(alias="resourceType", default="FormaConfig")
    content: str = "content.yaml"
    style: str = "style.yaml"
    templates: dict[str, TemplateEntry] = Field(default_factory=dict)
    output_dir: str = "../../var/builds"
    publishing: PublishOverride = Field(default_factory=PublishOverride)

    model_config = {"populate_by_name": True}

    @classmethod
    def from_yaml(cls, path: Path) -> FormaConfig:
        with open(path) as f:
            data = yaml.safe_load(f)
        return cls.model_validate(data or {})

    def resolve_template_path(self, name: str, project_root: Path) -> Path:
        entry = self.templates[name]
        return (project_root / entry.path).resolve()

    def resolve_mapping_path(self, name: str, project_root: Path) -> Path:
        entry = self.templates[name]
        return (project_root / entry.mapping).resolve()

    def resolve_style_path(self, project_root: Path) -> Path:
        return (project_root / self.style).resolve()

    def resolve_content_path(self, project_root: Path) -> Path:
        return (project_root / self.content).resolve()

    def resolve_output_dir(self, project_root: Path) -> Path:
        return (project_root / self.output_dir).resolve()
