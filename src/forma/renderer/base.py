"""
BaseRenderer ABC.

Subclasses implement _compile(tex_source, output_path) to invoke
the appropriate LaTeX engine (xelatex, pdflatex, lualatex).
"""

from __future__ import annotations

import shutil
import subprocess
import tempfile
from abc import ABC, abstractmethod
from pathlib import Path

from rich.console import Console

console = Console()


class BaseRenderer(ABC):
    engine: str = "xelatex"  # override in subclasses

    def render(
        self,
        tex_source: str,
        output_path: Path,
        *,
        passes: int = 2,
    ) -> Path:
        """
        Write tex_source to a temp dir, compile it, copy the result to output_path.
        Returns the output_path on success, raises on failure.
        """
        output_path.parent.mkdir(parents=True, exist_ok=True)

        with tempfile.TemporaryDirectory() as tmpdir:
            tmp = Path(tmpdir)
            tex_file = tmp / "document.tex"
            tex_file.write_text(tex_source, encoding="utf-8")

            for _ in range(passes):
                self._compile(tex_file, tmp)

            pdf_file = tmp / "document.pdf"
            if not pdf_file.exists():
                raise RuntimeError(
                    f"Compilation produced no PDF. Check {tex_file} for errors."
                )

            shutil.copy2(pdf_file, output_path)

        console.print(f"[green]✓[/green] Rendered → {output_path}")
        return output_path

    def _compile(self, tex_file: Path, workdir: Path) -> None:
        cmd = [
            self.engine,
            "-interaction=nonstopmode",
            "-halt-on-error",
            f"-output-directory={workdir}",
            str(tex_file),
        ]
        result = subprocess.run(
            cmd,
            cwd=workdir,
            capture_output=True,
            text=True,
        )
        if result.returncode != 0:
            log = (workdir / "document.log").read_text(errors="replace") if (workdir / "document.log").exists() else result.stdout
            raise RuntimeError(
                f"{self.engine} failed (exit {result.returncode}).\n"
                f"--- LaTeX log (last 60 lines) ---\n"
                + "\n".join(log.splitlines()[-60:])
            )
