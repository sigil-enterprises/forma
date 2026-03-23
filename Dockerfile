ARG ORGANIZATION=slivern-corporate-services

# ── base ─────────────────────────────────────────────────────────────────────
# Heavy dependencies: Python + TeX Live + fonts.
# Built and pushed to GHCR as forma-base:latest; only rebuilt when deps change.
FROM python:3.12-slim-bookworm AS base

WORKDIR /app

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
      git make git-lfs curl unzip fontconfig \
      texlive-xetex \
      texlive-fonts-recommended \
      texlive-fonts-extra \
      texlive-latex-extra \
      texlive-lang-arabic \
      fonts-font-awesome \
 && mkdir -p /builds/docs \
 # Install Rubik and Inter variable fonts from google/fonts (OFL licence).
 # Best-effort — templates fall back to Helvetica if the download fails.
 && mkdir -p /usr/share/fonts/truetype/rubik /usr/share/fonts/truetype/inter \
 && BASE="https://github.com/google/fonts/raw/main" \
 && curl -fsSL "${BASE}/ofl/rubik/Rubik%5Bwght%5D.ttf" \
         -o /usr/share/fonts/truetype/rubik/Rubik-VariableFont.ttf 2>/dev/null || true \
 && curl -fsSL "${BASE}/ofl/inter/Inter%5Bslnt%2Cwght%5D.ttf" \
         -o /usr/share/fonts/truetype/inter/Inter-VariableFont.ttf 2>/dev/null || true \
 && fc-cache -fv \
 && rm -rf /var/lib/apt/lists/*

ARG ORGANIZATION
COPY pyproject.toml README.md ./
RUN mkdir -p src/forma

RUN --mount=type=secret,id=github_token,required=false \
  if [ -f /run/secrets/github_token ]; then \
    git config --global url."https://$(cat /run/secrets/github_token):@github.com/$ORGANIZATION/".insteadOf \
      "https://github.com/slivern-corporate-services/"; \
  fi \
  && SETUPTOOLS_SCM_PRETEND_VERSION=0.0.0 pip install -e .[test,ci,docs] \
  && playwright install chromium --with-deps

# ── app ───────────────────────────────────────────────────────────────────────
# Source code layer: any code change only invalidates this COPY layer.
FROM base AS app

COPY . .

ENTRYPOINT ["make"]
