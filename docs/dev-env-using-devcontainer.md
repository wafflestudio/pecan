# Building a Development Environment Using Devcontainer

This guide explains how to set up a development environment for Pecan using devcontainers.

## Overview

Developing Pecan requires Rust, Isolate, and additional language toolchains to be installed on your machine. While this process can be straightforward on Linux or Unix-based operating systems, it can be challenging on Windows.

Devcontainers enable you to use a container as a full-featured development environment. If you are using VS Code (or compatible IDEs) and have Docker installed on your machine, you are ready to get started!

## Prerequisites

- **VS Code** (or a compatible IDE with devcontainer support)
- **Docker** installed and running on your machine
- **Docker Compose** (optional, for advanced configurations)

## What's Included

The devcontainer image is based on `rust:1.86.0-slim` and includes:

- **Rust toolchain** (stable and nightly)
  - `rustfmt` (nightly toolchain)
  - `clippy` (Rust linter)
- **Isolate** (v2.0) - sandboxing tool for code execution
- **Build tools** - essential development tools and libraries
- **VS Code extensions** (automatically installed):
  - `rust-lang.rust-analyzer`
  - `fill-labs.dependi`
  - `vadimcn.vscode-lldb`

## Setting Up the Environment

You can review the devcontainer specification for this project in [`.devcontainer/devcontainer.json`](../.devcontainer/devcontainer.json).

The devcontainer configuration:

- Mounts the project root as the workspace folder (`/workspace`)
- Exposes container port 8080 to host port 17360
- Runs in privileged mode (required for Isolate's cgroup access)
- Executes a post-start script to initialize the environment

## Installing Language Toolchains

The devcontainer Docker image does not include language toolchains by default. You need to install each toolchain using the installation scripts in the [`toolchains/`](../toolchains/) folder.

Available toolchains include:

- C/C++
- Go
- Java
- Kotlin
- Node.js
- Python
- Rust
- TypeScript

To install a toolchain, run:

```bash
cd toolchains
./install.sh <language>/manifest.yaml
```

For example, to install the Node.js runtime for testing the Pecan API:

```bash
cd toolchains
./install.sh node/manifest.yaml
```

The installation script will:

- Download the appropriate toolchain for your platform
- Verify SHA256 checksums
- Install to `/opt/toolchains/<language>/<version>`
- Set up symlinks and PATH entries
- Configure `update-alternatives` (if specified in the manifest)

## Important Notes

### Privileged Mode

Since Isolate (and other tools that require cgroup access for sandboxing) requires root privileges, the container runs in privileged mode. As a result, files or folders created inside the devcontainer will have root ownership.

**Recommendation**: To avoid permission issues, develop your code outside the container in most cases. Launch the devcontainer primarily when you need to:

- Run end-to-end tests
- Test sandbox functionality
- Build and run the full application

### File Permissions

If you do create files inside the devcontainer, you may need to adjust their permissions afterward:

```bash
sudo chown -R $USER:$USER /workspace
```

### Port Mapping

The devcontainer maps container port 8080 to host port 17360. If you need to change this, modify the `appPort` setting in `devcontainer.json`.
