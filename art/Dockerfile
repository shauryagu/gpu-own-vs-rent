# Build from the repo root:
#   docker build -f art/Dockerfile -t chi .
# Compose: docker compose -f art/compose.yaml run --rm invert

FROM rust:1-bookworm AS build
WORKDIR /src

RUN apt-get update \
    && apt-get install -y --no-install-recommends cmake pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock rust-toolchain.toml rustfmt.toml ./
COPY crates ./crates
RUN cargo build --release -p chi

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --home-dir /app --create-home chi

WORKDIR /app
COPY --from=build /src/target/release/chi /usr/local/bin/chi
COPY art/entrypoint.sh /usr/local/bin/entrypoint.sh
COPY fixtures /app/fixtures
RUN chmod 755 /usr/local/bin/entrypoint.sh \
    && mkdir -p /data \
    && chown -R chi:chi /app /data

USER chi
VOLUME ["/data"]
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD ["--help"]
