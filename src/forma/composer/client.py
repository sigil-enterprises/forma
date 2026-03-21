"""
Thin wrapper around the Anthropic SDK.
Reads ANTHROPIC_API_KEY from the environment.
"""

from __future__ import annotations

import os

import anthropic


class FormaClient:
    def __init__(self, model: str = "claude-opus-4-6", max_tokens: int = 8192) -> None:
        api_key = os.environ.get("ANTHROPIC_API_KEY")
        if not api_key:
            raise EnvironmentError(
                "ANTHROPIC_API_KEY is not set. "
                "Export it before running compose commands."
            )
        self._client = anthropic.Anthropic(api_key=api_key)
        self.model = model
        self.max_tokens = max_tokens

    def complete(self, system_prompt: str, user_prompt: str) -> str:
        message = self._client.messages.create(
            model=self.model,
            max_tokens=self.max_tokens,
            system=system_prompt,
            messages=[{"role": "user", "content": user_prompt}],
        )
        return message.content[0].text
