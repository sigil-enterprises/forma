# Devcontainer

forma ships with a VS Code devcontainer that provides a complete development environment including Python, TeX Live, tmux, and Claude Code CLI.

## Opening the devcontainer

1. Install [Docker Desktop](https://www.docker.com/products/docker-desktop/) and [VS Code](https://code.visualstudio.com) with the [Dev Containers extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers)
2. Open the forma repository in VS Code
3. Click **Reopen in Container** when prompted (or `Ctrl+Shift+P` → "Dev Containers: Reopen in Container")

The first build takes a few minutes (TeX Live is large). Subsequent starts are fast.

## What's included

| Tool | Version | Purpose |
|---|---|---|
| Python | 3.12 | Runtime |
| xelatex | TeX Live | PDF compilation |
| tmux | latest | Terminal multiplexer (default terminal) |
| zsh + oh-my-zsh | latest | Shell |
| Claude Code CLI | latest | AI pair programming |

**VS Code extensions installed automatically:**

- Python, Pylance, Debugpy
- LaTeX Workshop (`.tex.j2` editing + PDF preview)
- GitHub Actions
- YAML, TOML
- Claude Code

## setup-host

`initializeCommand` runs `.devcontainer/setup-host` on the **host** machine before the container starts. It:

- Reads GITHUB credentials from the host's `gh` CLI or existing `.env`
- Writes `.env` with `GITHUB_TOKEN`, `PROJECT`, `ORGANIZATION`, `DEV_CONTAINER_NAME`
- Logs in to GHCR with the token (so Docker can pull the private base image)

## setup-editor

`onCreateCommand` runs `.devcontainer/setup-editor` inside the container after creation. It:

- Installs zsh, oh-my-zsh, and the gpakosz tmux config
- Runs `make setup-dev` (installs forma in dev mode with all extras)
- Installs Claude Code CLI globally

## Terminal

The default VS Code terminal opens a tmux session named `dev`. This persists across terminal close/reopen in the same container session.

## make targets

| Target | Description |
|---|---|
| `make init` | Init submodules + create `.env` |
| `make setup` | `pip install -e .[test,ci,docs]` |
| `make setup-dev` | `pip install -e .[dev]` (includes all extras + pytest-watch) |
| `make test` | `pytest .` |
| `make dev` | `ptw` (pytest-watch, re-runs tests on save) |
| `make submodule-status` | `git submodule status` |
