FROM rust:1.86-slim-bookworm AS builder

WORKDIR /usr/src/forma

# Install xelatex dependency for LaTeX rendering
RUN apt-get update && apt-get install -y --no-install-recommends \
    texlive-xetex texlive-fonts-recommended texlive-fonts-extra texlive-latex-extra \
    libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY . .
RUN cargo build --release --bin forma

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    texlive-xetex texlive-fonts-recommended texlive-fonts-extra texlive-latex-extra \
    ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /usr/src/forma/target/release/forma /usr/local/bin/forma

ENTRYPOINT ["forma"]
