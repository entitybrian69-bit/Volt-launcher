# The Play with friends relay, for anyone to run (docs/play-with-friends.md, "Running your own relay").
# Build context: the repository root (the workspace's manifests and lockfile):
#   docker build -f crates/volt-relay/Dockerfile -t volt-relay .
FROM rust:1-slim-bookworm AS build
WORKDIR /src
COPY . .
RUN cargo build --release --locked -p volt-relay \
    && cp target/release/volt-relay /volt-relay

FROM debian:bookworm-slim
COPY --from=build /volt-relay /usr/local/bin/volt-relay
ENV PORT=7000
EXPOSE 7000
USER nobody
CMD ["volt-relay"]
