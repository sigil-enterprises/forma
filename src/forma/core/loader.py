"""
Dynamic schema class loader.

Given a dotted path like "schemas.proposal.content:ProposalContent",
imports the module and returns the class. Works with any BaseContent subclass
defined anywhere on sys.path — including in-project schemas/ directories.
"""

from __future__ import annotations

import importlib
import sys
from pathlib import Path

from forma.core.base import BaseContent, BaseStyle


def load_content_class(schema_path: str, project_root: Path | None = None) -> type[BaseContent]:
    """
    Load a BaseContent subclass from a dotted import path.

    Args:
        schema_path: "module.path:ClassName"
        project_root: if provided, added to sys.path so project-local
                      schemas/ directory is importable.
    """
    if project_root and str(project_root) not in sys.path:
        sys.path.insert(0, str(project_root))

    module_path, class_name = _split_path(schema_path)
    module = importlib.import_module(module_path)
    cls = getattr(module, class_name)

    if not (isinstance(cls, type) and issubclass(cls, BaseContent)):
        raise TypeError(f"{schema_path} must be a subclass of BaseContent, got {cls}")

    return cls


def load_style_class(schema_path: str, project_root: Path | None = None) -> type[BaseStyle]:
    if project_root and str(project_root) not in sys.path:
        sys.path.insert(0, str(project_root))

    module_path, class_name = _split_path(schema_path)
    module = importlib.import_module(module_path)
    cls = getattr(module, class_name)

    if not (isinstance(cls, type) and issubclass(cls, BaseStyle)):
        raise TypeError(f"{schema_path} must be a subclass of BaseStyle, got {cls}")

    return cls


def _split_path(path: str) -> tuple[str, str]:
    if ":" not in path:
        raise ValueError(
            f"Schema path must be 'module.path:ClassName', got: {path!r}"
        )
    module_path, class_name = path.rsplit(":", 1)
    return module_path, class_name
