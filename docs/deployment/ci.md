# CI/CD

forma ships with four GitHub Actions workflows.

## build.yml — Test + Docker

Runs on every push.

```
push → test job (pip install, pytest)
            │
            └─ [main only] build-base job (Dockerfile.base → GHCR)
                                │
                                └─ docker job (Dockerfile → GHCR forma:latest)
```

**Jobs:**

| Job | Trigger | What it does |
|---|---|---|
| `test` | every push | `pip install -e .[test,ci]` → `pytest` |
| `build-base` | main + when test passes | Builds `Dockerfile.base`, pushes `forma-base:latest` to GHCR |
| `docker` | main + after build-base | Builds `Dockerfile`, pushes `forma:latest` to GHCR |

**Required secrets:** `PAT_ORG_REPO_READ` (or falls back to `GITHUB_TOKEN`), `PAT_BADGES`

## build-base.yml — Base image

Rebuilds the heavy TeX Live base image only when dependencies change:

- Triggered by changes to `Dockerfile.base` or `pyproject.toml`
- Pushes `ghcr.io/{org}/forma-base:latest`
- Uses `docker/build-push-action` with GitHub Actions layer cache

## docs.yml — GitHub Pages

Deploys the MkDocs site to GitHub Pages on push to `main` or on release:

```bash
mkdocs gh-deploy --force --clean
```

Uses `mike` for version-stamped documentation. The deployed site is available at `https://{org}.github.io/forma/`.

**Required:** GitHub Pages must be enabled and set to deploy from the `gh-pages` branch in **Settings → Pages**.

## publish.yml — Render + Drive upload

Runs on push to `main`:

1. **Discover** — finds all `documents/*/forma.yaml`
2. **Matrix** — one job per document, run in parallel
3. For each document:
   - `forma validate` — fails the job if content is invalid
   - `forma render` — compiles all templates to PDF
   - `forma publish` — uploads to Google Drive (`continue-on-error: true`)
   - Upload PDFs as GitHub artifacts

**Required secrets:** `GOOGLE_SERVICE_ACCOUNT_JSON`

## First push checklist

- [ ] Enable GitHub Pages (Settings → Pages → Source: Deploy from branch `gh-pages`)
- [ ] Add `PAT_ORG_REPO_READ` secret (repo read, write:packages scope)
- [ ] Add `PAT_BADGES` secret
- [ ] Add `GOOGLE_SERVICE_ACCOUNT_JSON` secret
- [ ] Push to `main` — `build.yml` runs tests and builds the Docker image
- [ ] Verify `docs.yml` deploys the site to GitHub Pages
