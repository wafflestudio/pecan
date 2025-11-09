# --- build toolchains ---
FROM debian:bookworm-slim AS toolchain-base

SHELL ["/bin/bash","-euxo","pipefail","-c"]

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl xz-utils tar unzip yq build-essential file vim \
    git build-essential pkg-config libcap-dev libsystemd-dev \
    libbz2-dev libreadline-dev libsqlite3-dev libssl-dev zlib1g-dev libffi-dev \
    && rm -rf /var/lib/apt/lists/* && apt-get clean

COPY toolchains/install.sh /toolchains/install.sh
WORKDIR /toolchains
RUN chmod +x install.sh


# --- build language toolchains in parallel ---
FROM toolchain-base AS toolchain-builder-c
COPY toolchains/c/ /toolchains/c/
RUN ./install.sh c/manifest.yaml

FROM toolchain-base AS toolchain-builder-cpp
COPY toolchains/cpp/ /toolchains/cpp/
RUN ./install.sh cpp/manifest.yaml

FROM toolchain-base AS toolchain-builder-go
COPY toolchains/go/ /toolchains/go/
RUN ./install.sh go/manifest.yaml

FROM toolchain-base AS toolchain-builder-java
COPY toolchains/java/ /toolchains/java/
RUN ./install.sh java/manifest.yaml

FROM toolchain-base AS toolchain-builder-kotlin
COPY toolchains/kotlin/ /toolchains/kotlin/
RUN ./install.sh kotlin/manifest.yaml

FROM toolchain-base AS toolchain-builder-node
COPY toolchains/node/ /toolchains/node/
RUN ./install.sh node/manifest.yaml

FROM toolchain-base AS toolchain-builder-python
COPY toolchains/python/ /toolchains/python/
RUN ./install.sh python/manifest.yaml

FROM toolchain-base AS toolchain-builder-rust
COPY toolchains/rust/ /toolchains/rust/
RUN ./install.sh rust/manifest.yaml

FROM toolchain-base AS toolchain-builder-typescript
COPY toolchains/typescript/ /toolchains/typescript/
RUN ./install.sh typescript/manifest.yaml

# --- build isolate ---
FROM debian:12.6-slim AS isolate-builder

RUN apt-get update && apt-get --no-install-recommends install -y \
    git build-essential pkg-config libcap-dev
RUN git config --global http.sslVerify false
RUN git clone --depth 1 --branch v2.0 https://github.com/ioi/isolate.git /usr/src/isolate
WORKDIR /usr/src/isolate
RUN make isolate


# --- build pecan ---
FROM rust:1.86.0-slim AS pecan-builder

RUN apt-get update && apt-get --no-install-recommends install -y \
    libssl-dev pkg-config && apt-get clean && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/pecan
COPY . .
RUN cargo build --release


# --- build runner ---
FROM debian:12.6-slim AS runner

RUN apt-get update && apt-get --no-install-recommends install -y \
    gcc libc6-dev g++ && apt-get clean && rm -rf /var/lib/apt/lists/*

# copy toolchains
COPY --from=toolchain-builder-c /opt/toolchains/c/current /opt/toolchains/c/current
COPY --from=toolchain-builder-cpp /opt/toolchains/cpp/current /opt/toolchains/cpp/current
COPY --from=toolchain-builder-go /opt/toolchains/go/current /opt/toolchains/go/current
COPY --from=toolchain-builder-java /opt/toolchains/java/current /opt/toolchains/java/current
COPY --from=toolchain-builder-kotlin /opt/toolchains/kotlin/current /opt/toolchains/kotlin/current
COPY --from=toolchain-builder-node /opt/toolchains/node/current /opt/toolchains/node/current
COPY --from=toolchain-builder-python /opt/toolchains/python/current /opt/toolchains/python/current
COPY --from=toolchain-builder-rust /opt/toolchains/rust/current /opt/toolchains/rust/current
COPY --from=toolchain-builder-typescript /opt/toolchains/typescript/current /opt/toolchains/typescript/current

# copy isolate
COPY --from=isolate-builder /usr/src/isolate/isolate /usr/local/bin/isolate
COPY --from=isolate-builder /usr/src/isolate/isolate-check-environment /usr/local/bin/isolate-check-environment
# COPY --from=isolate-builder /usr/src/isolate/default.cf /usr/local/etc/isolate

COPY static/isolate/default.cf /usr/local/etc/isolate
COPY static/isolate/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# copy pecan
COPY --from=pecan-builder /usr/src/pecan/target/release/pecan-api /usr/local/bin/pecan-api

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/entrypoint.sh", "/usr/local/bin/pecan-api"]