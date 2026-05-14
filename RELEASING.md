# Releasing

Universal release policy for Sigil repositories. This file is shipped by `toolchain/base` and pinned — every Sigil repo that adopts the toolchain inherits the same SemVer rules and the same enforcement workflow.

> **Why this exists:** The `sigil-enterprises/toolchain` release history (April 2026) had multiple backward jumps and orphaned numbers because there was no policy. The `Latest` GitHub flag had been manually pinned to `v0.3.3` while local clones held tags up through `v0.16.0` — `tc upgrade` was silently downgrading consumers. This document plus the reusable enforcement workflow exist so that situation cannot recur in any Sigil repo.

## Versioning policy

This repository follows [SemVer 2.0.0](https://semver.org/) **strictly**:

- `vMAJOR.MINOR.PATCH` — no pre-release suffixes, no build metadata, no extra components.
- Versions only ever increase. **No backward jumps. No re-numbering. No skipping.**
- Tags are immutable. No `git tag -f`, no deleting and re-pushing.

### Pre-1.0 vs post-1.0

While on `0.x.y`:

- **Minor bump** (`0.16.0 → 0.17.0`) — any breaking change to the public contract (CLI flags, file formats, layer boundaries — whatever this repo exposes to consumers).
- **Patch bump** (`0.16.0 → 0.16.1`) — non-breaking additions, fixes, doc-only changes.

Bumping to **`1.0.0`** is a deliberate decision, not automatic — make it once the contract is stable in production for ≥ 4 weeks with no breaking changes pending. After 1.0.0, standard SemVer applies (major = breaking, minor = additive, patch = fix).

### The `Latest` flag

GitHub's `Latest` flag must always point at the highest SemVer tag. Release workflows set `make_latest: true` automatically — **do not override this in the GitHub UI**. Consumer tooling (`tc upgrade`, `tc update`, `install.sh`) resolves "latest" via the GitHub API; pointing it at a numerically-lower release silently downgrades every consumer.

Some repos additionally maintain a moving `latest` tag (e.g. `tc` re-publishes `tc-<arch>` under a sticky ref so first-install is fast). Such moving pointers are managed by the release workflow itself and are exempt from validation.

## How to cut a release

Repos using the standard `.toolchain/targets/release.mk` (also shipped by `base`) get a `make release` wrapper that runs the same checks locally before pushing:

```sh
make release VERSION=v0.17.0
```

This wrapper:

1. Refuses to run if your working tree is dirty or you are not on `main`.
2. Refuses to run if `VERSION` does not match `^v[0-9]+\.[0-9]+\.[0-9]+$`.
3. Refuses to run if `VERSION` is not strictly greater than the current highest `vX.Y.Z` release.
4. Pulls `origin/main` to confirm you are tagging the correct SHA.
5. Creates the annotated tag and pushes it. The release workflow takes over.

Repos with extra release prerequisites (changelog updates, version-string bumps, etc.) extend the wrapper with a repo-local `release-pre` target — see `.toolchain/targets/release.mk`.

## What the release workflow enforces

`base/.github/workflows/release-validate.yml` is a **reusable workflow** (`on: workflow_call`). Each repo's release pipeline invokes it as a gate before any artifacts are built or published:

```yaml
jobs:
  validate-tag:
    uses: ./.github/workflows/release-validate.yml
  build:
    needs: validate-tag
    # ...repo-specific build...
```

The release fails entirely if any of these are true:

| Check | What it prevents |
|-------|------------------|
| Tag format `^v\d+\.\d+\.\d+$` | Typos like `v0.17`, `v0.17.0-rc1`, `0.17.0` (no `v`) |
| Tag commit reachable from `origin/main` | Releasing from a feature branch by accident |
| Tag does not already have a release | Overwriting a published release |
| Tag `>` highest existing `vX.Y.Z` release | Backward jumps, re-numbering |

If your repo also publishes a moving sticky tag (e.g. `latest`), pass it via `with: allow_moving_tag: latest` so re-pushes of that pointer skip validation.

## Tag protection (admin)

Every Sigil repo should configure a tag protection rule for `v*`:

- Force-pushes blocked.
- Deletion requires admin role.

This prevents `git push --delete origin v0.17.0` from rewriting history even if the local tag is mishandled. Tag protection is configured per-repo in GitHub settings; it cannot be enforced from the toolchain side.
