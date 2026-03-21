"""
System and user prompt builders for the Claude composer.

The system prompt embeds the project's JSON Schema so Claude knows
exactly what shape to produce. The user prompt wraps the notes input.
"""

from __future__ import annotations

from forma.core.base import BaseContent


def build_system_prompt(schema_cls: type[BaseContent]) -> str:
    schema_json = schema_cls.json_schema_str()
    return f"""\
You are an expert business consultant and technical writer helping to draft \
structured proposal content.

Your task is to produce a YAML document that strictly conforms to the JSON \
Schema below. The schema defines the CONTENT of a business document organized \
by semantic domain — NOT by document structure (no slide types, no page layout).

Rules:
1. Output ONLY valid YAML. No markdown code fences, no commentary before or after.
2. Every required field in the schema must be present.
3. Write clear, professional, concise prose appropriate for a senior executive audience.
4. Preserve specific facts, figures, names, and dates from the notes exactly.
5. If a required field has no data in the notes, use a sensible placeholder \
   prefixed with "TODO:" so the human reviewer can find it easily.
6. Do not invent facts. Only use information from the notes.

JSON Schema:
{schema_json}
"""


def build_user_prompt(notes: str, existing_yaml: str | None = None) -> str:
    parts = ["Draft the YAML content document from the following notes:\n\n---\n", notes, "\n---"]
    if existing_yaml:
        parts += [
            "\n\nHere is an existing partial content.yaml to build on. "
            "Preserve all fields that are already filled in:\n\n---\n",
            existing_yaml,
            "\n---",
        ]
    parts.append("\n\nOutput only the YAML document.")
    return "".join(parts)
