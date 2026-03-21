# Installation

## Requirements

- Python 3.12+
- xelatex (for rendering PDFs)
- Git (for the skills submodule)

## Install from source

```bash
git clone https://github.com/slivern-corporate-services/forma.git
cd forma
git submodule update --init --recursive
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

    Use `Dockerfile.devcontainer` which includes all TeX Live dependencies:

    ```bash
    docker build -f Dockerfile.devcontainer -t forma-dev .
    docker run --rm -v $(pwd):/app forma-dev forma render documents/example-client
    ```

## Environment variables

Copy `.env.example` to `.env` and fill in your values:

```bash
cp .env.example .env
```

See [Secrets & Environment](../deployment/secrets.md) for a full reference.
