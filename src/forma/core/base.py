"""
Base Pydantic models for forma content and style.

Projects extend BaseContent and BaseStyle to define their own
domain-specific schemas. The engine works with any subclass.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

import yaml
from pydantic import BaseModel, Field


class PublishingConfig(BaseModel):
    google_drive_folder_id: str = ""
    filename_prefix: str = ""


class BrandConfig(BaseModel):
    logo: str = ""
    logo_white: str = ""

    model_config = {"extra": "allow"}


class ColorConfig(BaseModel):
    primary_dark: str = "#0B1D2A"
    primary_accent: str = "#F58220"
    white: str = "#FFFFFF"
    gray_light: str = "#F5F5F5"
    gray_medium: str = "#E0E0E0"
    gray_dark: str = "#666666"
    text_primary: str = "#333333"
    text_secondary: str = "#666666"

    model_config = {"extra": "allow"}


class TypographyConfig(BaseModel):
    font_primary: str = "Helvetica"
    font_secondary: str = "Helvetica"
    font_mono: str = "Courier"
    sizes: dict[str, int] = Field(default_factory=lambda: {
        "xs": 9, "sm": 10, "base": 11, "md": 12,
        "lg": 14, "xl": 16, "xl2": 20, "xl3": 24, "xl4": 32,
    })

    model_config = {"extra": "allow"}


class LayoutConfig(BaseModel):
    page_size: str = "a4"
    slides_aspect_ratio: str = "169"

    model_config = {"extra": "allow"}


class BaseStyle(BaseModel):
    """
    Base for style.yaml models. Projects extend this with their
    own branding tokens (colors, typography, layout, etc.).

    FormaStyle (below) provides the full default implementation.
    Projects that need custom fields should subclass FormaStyle.
    """

    publishing: PublishingConfig = Field(default_factory=PublishingConfig)

    model_config = {"extra": "allow"}

    @classmethod
    def from_yaml(cls, path: Path) -> BaseStyle:
        with open(path) as f:
            data = yaml.safe_load(f)
        return cls.model_validate(data or {})


class FormaStyle(BaseStyle):
    """
    Full style model with typed branding tokens.
    Loaded by the renderer from style.yaml.
    """
    brand: BrandConfig = Field(default_factory=BrandConfig)
    colors: ColorConfig = Field(default_factory=ColorConfig)
    typography: TypographyConfig = Field(default_factory=TypographyConfig)
    layout: LayoutConfig = Field(default_factory=LayoutConfig)


class BaseContent(BaseModel):
    """
    Base for content.yaml models. Projects extend this with their
    own domain-specific fields (engagement, client, solution, etc.).

    The engine accepts any subclass — it builds a Jinja2 context
    from the model's fields and passes it to the template.
    """

    publishing: PublishingConfig = Field(default_factory=PublishingConfig)

    @classmethod
    def from_yaml(cls, path: Path) -> BaseContent:
        with open(path) as f:
            data = yaml.safe_load(f)
        return cls.model_validate(data or {})

    @classmethod
    def json_schema(cls) -> dict[str, Any]:
        return cls.model_json_schema()

    @classmethod
    def json_schema_str(cls) -> str:
        return json.dumps(cls.model_json_schema(), indent=2)
