"""
BaseRenderer ABC.

Subclasses implement _compile(tex_source, output_path) to invoke
the appropriate LaTeX engine (xelatex, pdflatex, lualatex).
"""

from __future__ import annotations

import os
import shutil
import subprocess
import tempfile
from abc import ABC
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
        project_dir: Path | None = None,
        fonts_dirs: list[Path] | None = None,
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
                self._compile(tex_file, tmp, project_dir=project_dir, fonts_dirs=fonts_dirs)

            pdf_file = tmp / "document.pdf"
            if not pdf_file.exists():
                raise RuntimeError(
                    f"Compilation produced no PDF. Check {tex_file} for errors."
                )

            shutil.copy2(pdf_file, output_path)

        console.print(f"[green]✓[/green] Rendered → {output_path}")
        return output_path

    def _compile(self, tex_file: Path, workdir: Path, *, project_dir: Path | None = None, fonts_dirs: list[Path] | None = None) -> None:
        cmd = [
            self.engine,
            "-interaction=nonstopmode",
            f"-output-directory={workdir}",
            str(tex_file),
        ]
        env = os.environ.copy()
        if project_dir:
            # Prepend project dir to TEXINPUTS so \includegraphics finds local assets.
            # The trailing // means recursive search; the trailing : keeps defaults.
            env["TEXINPUTS"] = f"{project_dir}//::{env.get('TEXINPUTS', '')}"
        if fonts_dirs:
            # Add fonts dirs to both TEXINPUTS and OSFONTDIR so fontspec can find
            # font files by filename (e.g. \setmainfont{Rubik-VariableFont.ttf})
            # regardless of whether they are registered with fontconfig.
            extra = ":".join(str(d) + "//" for d in fonts_dirs)
            env["TEXINPUTS"] = f"{extra}:{env.get('TEXINPUTS', '')}"
            env["OSFONTDIR"] = f"{extra}:{env.get('OSFONTDIR', '')}"
        result = subprocess.run(
            cmd,
            cwd=workdir,
            capture_output=True,
            text=True,
            env=env,
        )
        pdf_produced = (workdir / "document.pdf").exists()
        if not pdf_produced:
            log = (workdir / "document.log").read_text(errors="replace") if (workdir / "document.log").exists() else result.stdout
            raise RuntimeError(
                f"{self.engine} failed (exit {result.returncode}) — no PDF produced.\n"
                f"--- LaTeX log (last 60 lines) ---\n"
                + "\n".join(log.splitlines()[-60:])
            )
