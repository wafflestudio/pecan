# Deployment Guide

This guide covers deploying Pecan in production environments.

## Prerequisites

- **Linux host** with cgroup v2 support
- **Root privileges** (required for Isolate sandbox)
- **Docker** (recommended for containerized deployment)
- **Rust 1.86.0+** (if building from source)

## Environment Variables

### Server Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `HOST` | `0.0.0.0` | Server bind address |
| `PORT` | `8080` | Server port |

### Service Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `ENABLE_BG_WORKER_LOOP` | `true` | Enable background worker for sandbox health management |
| `MAX_QUEUE_SIZE` | `100` | Maximum pending execution requests |
| `MAX_CONCURRENT_EXECUTIONS` | `20` | Maximum concurrent sandbox executions |

### Sandbox Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `MAX_PREWARMED_SANDBOXES` | `1000` | Maximum number of prewarmed sandbox instances |
| `SANDBOX_TYPE` | `isolate` | Sandbox backend (`isolate`, `isolate-cg`) |

### Logging

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Log level filter (`trace`, `debug`, `info`, `warn`, `error`) |

## Build Configuration

### Sandbox Backend Selection

Set `SANDBOX_TYPE` at build time to select the sandbox backend:

```bash
# Standard Isolate (without cgroup memory tracking)
SANDBOX_TYPE=isolate cargo build --release

# Isolate with cgroup support (recommended for production)
SANDBOX_TYPE=isolate-cg cargo build --release
```

The `isolate-cg` backend provides accurate memory usage tracking via cgroups.

### Release Build

```bash
cargo build --release -p pecan-api
```

The binary is output to `target/release/pecan-api`.

## Container Specification

### Base Image Requirements

The production container must include:

- **Isolate v2.0** - sandbox execution tool
- **cgroup v2** - process isolation and resource limits
- **Language toolchains** - compilers/interpreters for supported languages

### Dockerfile Example

```dockerfile
FROM rust:1.86.0-slim

# Install system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl xz-utils tar unzip yq build-essential file \
    git pkg-config libcap-dev libsystemd-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Isolate
RUN git clone --depth 1 --branch v2.0 https://github.com/ioi/isolate.git /opt/isolate
RUN make -C /opt/isolate isolate isolate-cg-keeper \
    && cp /opt/isolate/isolate /usr/local/bin/isolate \
    && cp /opt/isolate/isolate-check-environment /usr/local/bin/isolate-check-environment \
    && cp /opt/isolate/isolate-cg-keeper /usr/local/bin/isolate-cg-keeper

# Copy Isolate configuration
COPY static/isolate/default.cf /usr/local/etc/isolate
COPY static/isolate/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# Build application
COPY . /app
WORKDIR /app
RUN SANDBOX_TYPE=isolate-cg cargo build --release -p pecan-api

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD ["/app/target/release/pecan-api"]
```

### Isolate Configuration

The default Isolate configuration (`static/isolate/default.cf`):

```
box_root = /var/local/lib/isolate
lock_root = /run/isolate/locks
cg_root = /sys/fs/cgroup/isolate
first_uid = 60000
first_gid = 60000
num_boxes = 1000
```

Key settings:
- `num_boxes`: Maximum concurrent sandboxes (should match `MAX_PREWARMED_SANDBOXES`)
- `first_uid`/`first_gid`: UID/GID range for sandbox users

### Cgroup Initialization

The entrypoint script (`static/isolate/entrypoint.sh`) initializes cgroups:

```bash
#!/bin/bash

# Create initial cgroup and migrate processes
mkdir -p /sys/fs/cgroup/init
xargs -rn1 < /sys/fs/cgroup/cgroup.procs > /sys/fs/cgroup/init/cgroup.procs
echo +cpu +cpuset +memory +pids > /sys/fs/cgroup/cgroup.subtree_control

# Create isolate cgroup
mkdir -p /sys/fs/cgroup/isolate
echo +cpu +cpuset +memory +pids > /sys/fs/cgroup/isolate/cgroup.subtree_control

exec "$@"
```

### Container Runtime Requirements

Run the container with:

```bash
docker run --privileged \
    -p 8080:8080 \
    -e MAX_PREWARMED_SANDBOXES=100 \
    -e MAX_CONCURRENT_EXECUTIONS=20 \
    pecan:latest
```

**Required flags:**
- `--privileged`: Isolate requires root and cgroup access

## Installing Language Toolchains

Language toolchains are installed separately using the provided installation scripts.

```bash
cd toolchains
./install.sh <language>/manifest.yaml
```

Available toolchains:
- `c/manifest.yaml` - GCC
- `cpp/manifest.yaml` - G++
- `java/manifest.yaml` - OpenJDK
- `kotlin/manifest.yaml` - Kotlin compiler
- `python/manifest.yaml` - CPython
- `node/manifest.yaml` - Node.js
- `typescript/manifest.yaml` - TypeScript (requires Node.js)
- `go/manifest.yaml` - Go compiler
- `rust/manifest.yaml` - Rust toolchain

Toolchains are installed to `/opt/toolchains/<language>/<version>` with a `current` symlink.

## CI/CD with GitHub Actions

### Build and Test Workflow

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1
        with:
          toolchain: 1.86.0
          components: clippy, rustfmt
      
      - name: Check formatting
        run: cargo +nightly fmt --all -- --check
      
      - name: Lint
        run: cargo clippy --all-targets --all-features -- -D warnings
      
      - name: Build
        run: cargo build --release -p pecan-api
      
      - name: Run tests
        run: cargo test --workspace
```

### Docker Build and Push

```yaml
name: Docker

on:
  push:
    tags: ['v*']

jobs:
  docker:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3
      
      - name: Login to Registry
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}
      
      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: ghcr.io/${{ github.repository }}:${{ github.ref_name }}
          platforms: linux/amd64,linux/arm64
```

## Health Checks

Monitor the server with the health endpoint:

```bash
curl http://localhost:8080/v1/health
# Returns: OK
```

Check sandbox pool status:

```bash
curl http://localhost:8080/v1/manager/sandbox-status
# Returns: {"available_sandboxes":100,"idle_sandboxes":95,"running_sandboxes":5,"error_sandboxes":0}
```

## Production Recommendations

1. **Set resource limits** based on expected load:
   - `MAX_PREWARMED_SANDBOXES`: 2-3x expected concurrent users
   - `MAX_CONCURRENT_EXECUTIONS`: Match CPU core count

2. **Enable cgroup backend** (`SANDBOX_TYPE=isolate-cg`) for accurate memory tracking

3. **Configure log level** to `warn` or `error` in production:
   ```bash
   RUST_LOG=warn ./pecan-api
   ```

4. **Use a reverse proxy** (nginx, Caddy) for TLS termination and rate limiting

5. **Monitor sandbox health** via `/v1/manager/sandbox-status` endpoint
