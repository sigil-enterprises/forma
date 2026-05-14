# Forma — Rust Binary

Schema-agnostic document rendering framework. Ported from Python to Rust.

## Build

```bash
cargo build          # debug
cargo build --release # optimized (~12MB, ~10MB stripped)
cargo test            # 74 tests
```

## Architecture

5-crates workspace:

```
crates/
  forma-schema/     — embedded JSON schemas, serde content/document types
  forma-core/       — FormaConfig, !include resolver, schema registry, validation
  forma-render/     — TemplateManifest, Tera rendering, LaTeX/HTML→PDF
  forma-composer/   — Anthropic API client, prompt builders, fill_from_notes
  forma-cli/        — binary (clap), 7 commands
```

`forma-schema → forma-core → forma-render/composer → forma-cli`

## CLI

```bash
forma validate <project-dir> [--strict]
forma render <project-dir> [-t NAME]
forma mapping validate <project-dir> [-f FILE]
forma compose fill <project-dir> -n NOTES [--dry-run] [--overwrite]
forma schema export [-o DIR]
forma template list [-d DIR]
forma init <client-name> [-d DIR]
```

## Template Engine

Tera with custom delimiters matching Python Jinja2: `(( ))`, `(%)`, `(# #)`.
No template rewrite needed — builder config in `forma-render/src/filters.rs`.

## Removed from Python

Google Drive publishing, skills loading, Playwright, watch mode.

## Toolchain

- `toolchain.toml` → `$ORGANIZATION/toolchain/rust-tools`
- `tc check` runs 10 policy checks (9 from chain + 1 local `cargo-lock-exists`)
- Pre-commit hook installed at `.toolchain/hooks/pre-commit`
- Run `tc update` to refresh toolchain chain
- Run `tc check` to verify compliance
