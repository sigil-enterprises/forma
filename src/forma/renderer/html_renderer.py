"""
HTML → PDF renderer using Playwright (Chromium headless).

Converts a rendered HTML string (1280×720 slide deck or A4 document) into
a PDF file. Writes the HTML to a temp file in the template directory so
relative asset paths (fonts, images) resolve correctly via file:// URLs.

Slide dimensions: 1280×720 px (16:9).
Each slide is a <div class="slide"> with CSS break-after: page.
"""

from __future__ import annotations

from pathlib import Path

from rich.console import Console

console = Console()


class HtmlRenderer:
    """Renders an HTML string to PDF via Playwright Chromium."""

    def render(
        self,
        html_source: str,
        output_path: Path,
        *,
        workdir: Path | None = None,
    ) -> Path:
        """
        Convert html_source to a PDF file at output_path.

        The HTML is written to a temp file in workdir (defaults to
        output_path.parent) so that relative asset paths in the HTML
        resolve correctly via file:// URLs.

        Args:
            html_source: Fully-rendered HTML string (from Jinja2).
            output_path: Where to write the PDF.
            workdir:     Directory to write the temp HTML file into.
                         Use the template directory so relative font/image
                         paths resolve relative to the template.

        Returns:
            output_path on success.
        """
        try:
            from playwright.sync_api import sync_playwright
        except ImportError as exc:
            raise RuntimeError(
                "Playwright is required for HTML rendering. "
                "Install it with: pip install playwright && playwright install chromium"
            ) from exc

        output_path.parent.mkdir(parents=True, exist_ok=True)

        effective_workdir = workdir or output_path.parent
        effective_workdir.mkdir(parents=True, exist_ok=True)

        # Write HTML to a temp file so relative file:// paths resolve
        html_file = effective_workdir / "_forma_render.html"
        html_file.write_text(html_source, encoding="utf-8")

        try:
            with sync_playwright() as pw:
                browser = pw.chromium.launch()
                page = browser.new_page()

                page.goto(f"file://{html_file.resolve()}", wait_until="networkidle")

                page.pdf(
                    path=str(output_path),
                    print_background=True,
                    width="1280px",
                    height="720px",
                    margin={"top": "0", "right": "0", "bottom": "0", "left": "0"},
                )

                browser.close()
        finally:
            html_file.unlink(missing_ok=True)

        console.print(f"[green]✓[/green] Rendered → {output_path}")
        return output_path
