# Built from the repository root: docker build -f crates/volt-telemetry-server/Dockerfile .
FROM rust:1-slim AS build
WORKDIR /src
# The root manifest only for the shared crate's workspace settings (edition, lints).
COPY Cargo.toml Cargo.toml
COPY crates/volt-telemetry crates/volt-telemetry
COPY crates/volt-telemetry-server crates/volt-telemetry-server
RUN cargo build --release --locked --manifest-path crates/volt-telemetry-server/Cargo.toml

FROM debian:stable-slim
COPY --from=build /src/crates/volt-telemetry-server/target/release/volt-telemetry-server /usr/local/bin/
ENV VOLT_TELEMETRY_DB=/data/telemetry.sqlite3 VOLT_TELEMETRY_TRUST_PROXY=1
CMD ["volt-telemetry-server"]
