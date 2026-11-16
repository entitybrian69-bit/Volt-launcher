# volt-server in a container (docs/server.md, section 3). Build context: the repository root:
#   docker build -f packaging/server/Dockerfile -t volt-server .
#
# The image holds none of Minecraft's files. Prepare them on the host from your own Minecraft 26.3
# (volt-prepare --prepare, see packaging/PLAY.txt) and mount the folder it writes, read-only:
#   docker run -d --name volt -p 25565:25565 --stop-timeout 180 \
#     -v volt-data:/data \
#     -v /path/to/.local/volt-client/26.3:/opt/volt/.local/volt-client/26.3:ro \
#     -e VOLT_EULA=true -e VOLT_WHITELIST=true -e VOLT_PLAYERS=Alex,Steve -e VOLT_OP=Alex \
#     volt-server
#
# VOLT_EULA=true accepts the Minecraft EULA (https://aka.ms/MinecraftEULA) for this server. The first
# start writes /data/server.toml from VOLT_* (EULA, WORLD, SEED, PORT, WHITELIST, PLAYERS, OP); later
# starts use the file. Manage it with `docker exec volt volt-server status|cmd|backup|logs ...`.
FROM rust:1-slim-bookworm AS build
WORKDIR /src
COPY . .
# The server build of the game (no sound, no controllers: it needs neither ALSA nor udev) as volt-engine.
RUN cargo build --release --locked -p volt-client --no-default-features --bin volt-client \
    && cargo build --release --locked -p volt-server --bin volt-server \
    && mkdir /out && cp target/release/volt-client /out/volt-engine && cp target/release/volt-server /out/

FROM debian:bookworm-slim
RUN useradd --system --home-dir /data --shell /usr/sbin/nologin volt \
    && mkdir -p /data /opt/volt/.local/volt-client \
    && chown volt:volt /data
COPY --from=build /out/ /opt/volt/
ENV PATH=/opt/volt:$PATH \
    VOLT_NO_AUDIO=1
WORKDIR /data
VOLUME /data
EXPOSE 25565
USER volt
# SIGTERM makes volt-server stop the game cleanly (it saves first): give it --stop-timeout 180.
CMD ["sh", "-c", "test -f server.toml || volt-server init --yes || exit 1; exec volt-server run"]
