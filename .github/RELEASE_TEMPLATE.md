Complete port of the forma document rendering framework from Python to a single, lean Rust binary. This release eliminates all external runtime dependencies (Node.js, Playwright, Python) and consolidates the codebase into a fast, statically-linked binary.

**What's New:** Single Rust binary (5 crates), Tera template engine, headless_chrome crate, raw reqwest HTTP client, embedded JSON schemas.
**Removed:** Google Drive publishing, dynamic skill loading, Node.js/Playwright, watch mode.
**Kept:** Anthropic compose fill, full LaTeX rendering pipeline, HTML rendering with headless Chrome.
**Verification:** 153 tests passing, binary compiles clean, all Python source removed.

Thanks to @mateusz and @rafal for their work on the original Python implementation and for the design insights that made this port possible.
