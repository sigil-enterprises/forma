# Installation

## Requirements

- Python 3.12+
- xelatex (for rendering PDFs)

## Install from source

```bash
git clone https://github.com/slivern-corporate-services/forma.git
cd forma
pip install -e .[dev]
```

The `[dev]` extra includes test dependencies and pytest-watch for continuous testing.

## Verify the install

```bash
forma --help
```

```
Usage: forma [OPTIONS] COMMAND [ARGS]...

  Schema-agnostic document rendering framework.

Options:
  --help  Show this message and exit.

Commands:
  compose   AI-assisted content composition.
  init      Scaffold a new document project directory.
  publish   Render all templates and upload to Google Drive.
  render    Render templates to PDF.
  schema    Schema utilities.
  template  Template utilities.
  validate  Validate content.yaml (and optionally style.yaml)...
```

## Install xelatex

Rendering requires xelatex. Inside the devcontainer it is pre-installed.
For local use:

=== "macOS"

    ```bash
    brew install --cask mactex
    ```

=== "Ubuntu / Debian"

    ```bash
    sudo apt-get install texlive-xetex texlive-fonts-extra texlive-latex-extra
    ```

=== "Docker"

    The `Dockerfile` multi-stage build produces a `base` target with all TeX Live dependencies:

    ```bash
    docker build --target base -t forma-base .
    docker run --rm -v $(pwd):/app forma-base pip install -e . && forma render .
    ```

    Or use the pre-built GHCR image:

    ```bash
    docker run --rm -v $(pwd):/app \
      ghcr.io/slivern-corporate-services/forma:latest \
      forma render .
    ```

## Environment variables

Create a `.env` file in the repo root with your credentials:

```bash
ANTHROPIC_API_KEY=sk-ant-...
GOOGLE_SERVICE_ACCOUNT_JSON=<base64-encoded service account JSON>
```

See [Secrets & Environment](../deployment/secrets.md) for a full reference.
