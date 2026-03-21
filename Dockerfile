ARG ORGANIZATION=slivern-corporate-services
FROM ghcr.io/${ORGANIZATION}/forma-base:latest AS base

ENTRYPOINT ["make"]


FROM base AS build

# Copy source on top of the pre-built base. Any code change only
# invalidates this COPY layer — all heavy installs are in the base image.
COPY . .
