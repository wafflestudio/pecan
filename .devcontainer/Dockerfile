FROM rust:1.86.0-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl xz-utils tar unzip yq build-essential file vim \
    git build-essential pkg-config libcap-dev libsystemd-dev \
    libbz2-dev libreadline-dev libsqlite3-dev libssl-dev zlib1g-dev libffi-dev \
    && rm -rf /var/lib/apt/lists/* && apt-get clean

RUN git clone --depth 1 --branch v2.0 https://github.com/ioi/isolate.git /opt/isolate
RUN make -C /opt/isolate isolate isolate-cg-keeper \
    && cp /opt/isolate/isolate /usr/local/bin/isolate \
    && cp /opt/isolate/isolate-check-environment /usr/local/bin/isolate-check-environment \
    && cp /opt/isolate/isolate-cg-keeper /usr/local/bin/isolate-cg-keeper
# && cp /opt/isolate/default.cf /usr/local/etc/isolate

COPY static/isolate/default.cf /usr/local/etc/isolate
COPY static/isolate/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

RUN rustup component add clippy
RUN rustup component add --toolchain nightly rustfmt

WORKDIR /workspace