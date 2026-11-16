# Built from the repository root: docker build -f crates/volt-mod-registry/Dockerfile .
FROM rust:1-slim AS build
WORKDIR /src
# The root manifest only for the shared crate's workspace settings (edition, lints).
COPY Cargo.toml Cargo.toml
COPY crates/volt-mods crates/volt-mods
# volt-mods' optional dependency: cargo needs its manifest even with the feature off.
COPY crates/volt-shader-api crates/volt-shader-api
COPY crates/volt-mod-registry crates/volt-mod-registry
RUN cargo build --release --locked --manifest-path crates/volt-mod-registry/Cargo.toml

FROM debian:stable-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/crates/volt-mod-registry/target/release/volt-mod-registry /usr/local/bin/
ENV REGISTRY_DB=/data/registry.sqlite3
# Railway mounts the ratings' volume at /data owned by root: hand it to nobody, then run as nobody (setpriv is
# util-linux's, in every Debian image).
CMD ["sh", "-c", "mkdir -p /data && chown nobody:nogroup /data && exec setpriv --reuid=nobody --regid=nogroup --clear-groups volt-mod-registry"]
