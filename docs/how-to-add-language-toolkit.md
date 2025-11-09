# How to Add a New Language Toolkit

This guide explains how to add support for a new programming language toolkit to the toolchains installation system.

## Overview

The toolchains system uses a modular architecture where language-specific logic is separated from the universal installation framework. Each language toolkit requires:

1. **`manifest.yaml`** - Configuration file defining the installation method, URLs, and metadata
2. **`hooks.sh`** (optional) - Language-specific hooks for architecture mapping, template variables, and post-installation steps

## Directory Structure

Create a new directory under `toolchains/` with your language name:

```
toolchains/
└── <language>/
    ├── manifest.yaml      # Required: Installation configuration
    └── hooks.sh           # Optional: Language-specific hooks
```

## Required Files

### 1. `manifest.yaml`

The manifest file defines how the toolkit is installed. It must contain the following required fields:

#### Required Fields

- `language` - Language identifier (e.g., `rust`, `go`, `node`)
- `version` - Version string (e.g., `"1.81.0"`, `"system"`)
- `method` - Installation method: `apt`, `tarball`, or `build`
- `install_prefix` - Installation directory path (e.g., `"/opt/toolchains/rust/1.81.0"`)

#### Optional Fields

- `binaries` - List of binary paths for verification
- `set_default` - Boolean to enable update-alternatives (default: `false`)
- `alternatives` - Configuration for `update-alternatives`:
  - `name` - Alternative group name
  - `links` - Map of command names to binary paths
- `runtime_env` - Runtime environment configuration:
  - `PATH_prepend` - List of directories to prepend to PATH

### 2. `hooks.sh` (Optional)

Create `hooks.sh` if your language requires:

- Custom architecture mapping (e.g., Node.js uses `x64` instead of `amd64`)
- Custom template variables in URLs
- Post-installation steps (e.g., Rust runs an internal installer)

## Installation Methods

### Method 1: `apt` (Package Manager)

Use this method when the toolkit is available via apt packages.

**Example:**

```yaml
language: c
version: "system"
method: apt
install_prefix: "/opt/toolchains/c/system"

apt:
  # Optional: snapshot URL for reproducible builds
  snapshot: "http://snapshot.debian.org/archive/debian/20241010T000000Z"
  packages:
    - "build-essential"
    - "gcc"
    - "libc6-dev"
    - "binutils"
```

**Fields:**

- `apt.snapshot` (optional) - Debian snapshot URL for reproducible builds
- `apt.packages` - Array of package names (can include version pinning, e.g., `"gcc=4:13.x-yy"`)

**Notes:**

- Packages are automatically pinned with `apt-mark hold` after installation
- No `hooks.sh` needed for most apt-based installations

### Method 2: `tarball` (Binary Distribution)

Use this method when pre-built binaries are available as tarballs.

#### Option A: `artifact_template` (Recommended)

Use a template URL with placeholders that get substituted automatically.

```yaml
language: go
version: "1.23.3"
method: tarball
install_prefix: "/opt/toolchains/go/1.23.3"

artifact_template:
  url: "https://go.dev/dl/go{{version}}.{{os}}-{{go_arch}}.tar.gz"
  sha256_by_arch:
    amd64: "a0afb9744c00648bafb1b90b4aba5bdb86f424f02f9275399ce0c20b93a2c3a8"
    arm64: "1f7cbd7f668ea32a107ecd41b6488aaee1f5d77a66efd885b175494439d4e1ce"
```

**Template Variables:**

- `{{version}}` - Version string
- `{{os}}` - Operating system (normalized: `linux`, `darwin`)
- `{{arch}}` - Architecture (normalized: `amd64`, `arm64`, `armv7l`, `ppc64le`, `s390x`)
- Custom variables (defined in `hooks.sh`): `{{node_arch}}`, `{{go_arch}}`, `{{rust_arch_tag}}`, etc.

**SHA256 Checksums:**

- `sha256_by_arch` - Map of architecture → checksum (recommended)
- `sha256` - Single checksum for all architectures (fallback)

#### Option B: `artifacts` Array

Use explicit artifact definitions (useful for multiple artifacts or complex scenarios).

```yaml
language: example
version: "1.0.0"
method: tarball
install_prefix: "/opt/toolchains/example/1.0.0"

artifacts:
  - url: "https://example.com/toolkit-{{version}}-{{os}}-{{arch}}.tar.gz"
    sha256: "abc123..."
  - url: "https://example.com/extra-{{version}}.zip"
    sha256: "def456..."
```

### Method 3: `build` (Source Compilation)

Use this method when building from source.

```yaml
language: python
version: "3.12.7"
method: build
install_prefix: "/opt/toolchains/python/3.12.7"

build:
  url: "https://www.python.org/ftp/python/{{version}}/Python-{{version}}.tgz"
  sha256: "73ac8fe780227bf371add8373c3079f42a0dc62deff8d612cd15a618082ab623"
  configure_args:
    - "--prefix={{install_prefix}}"
    - "--enable-optimizations"
    - "--with-lto"
```

**Fields:**

- `build.url` - Source tarball URL (supports template variables)
- `build.sha256` - Checksum for the source tarball
- `build.configure_args` - Array of arguments passed to `./configure`

**Build Systems Supported:**

- Autotools (`./configure`)
- CMake (`CMakeLists.txt`)

**Template Variables in `configure_args`:**

- `{{install_prefix}}` - Installation prefix
- `{{version}}`, `{{os}}`, `{{arch}}` - Standard variables
- Custom variables from `hooks.sh`

## Architecture Mapping

If your language uses non-standard architecture names in URLs (e.g., Node.js uses `x64` instead of `amd64`), create a `hooks.sh` file with a mapping function.

### Creating Architecture Mappers

**Function Naming Convention:**

```bash
map_<template_var_name>() {
  local arch="$1"
  case "$arch" in
    amd64)   echo "language_specific_arch" ;;
    arm64)   echo "language_specific_arch64" ;;
    # ... more cases
    *)       echo "default_arch" ;;
  esac
}
```

**Example: Node.js**

```bash
# hooks.sh
map_node_arch() {
  case "${1:-}" in
    amd64)  echo "x64" ;;
    arm64)  echo "arm64" ;;
    armv7l) echo "armv7l" ;;
    *)      echo "x64" ;;
  esac
}

# Declare which template variables this mapper provides
TEMPLATE_VARS=("node_arch")
```

**Example: Rust (Target Triples)**

```bash
# hooks.sh
map_rust_arch_tag() {
  local arch="$1"
  case "$arch" in
    amd64)   echo "x86_64-unknown-linux-gnu" ;;
    arm64)   echo "aarch64-unknown-linux-gnu" ;;
    armv7l)  echo "armv6l-unknown-linux-gnueabihf" ;;
    ppc64le) echo "powerpc64le-unknown-linux-gnu" ;;
    s390x)   echo "s390x-unknown-linux-gnu" ;;
    *)       echo "$arch" ;;
  esac
}

TEMPLATE_VARS=("rust_arch_tag")
```

## SHA256 Checksum Resolution

The system supports flexible SHA256 checksum lookup with the following precedence:

1. Language-specific mapped architecture keys (from `SHA256_ARCH_KEYS` in `hooks.sh`)
2. Direct architecture key (normalized: `amd64`, `arm64`, etc.)
3. Single `sha256` value (fallback)

### Configuring SHA256 Lookup Order

In `hooks.sh`, define the lookup order:

```bash
# SHA256 lookup order: which arch keys to try (in order)
SHA256_ARCH_KEYS=("node_arch" "arch")  # Try node_arch first, then arch
```

**Example: Node.js**

```yaml
# manifest.yaml
artifact_template:
  url: "https://nodejs.org/dist/v{{version}}/node-v{{version}}-{{os}}-{{node_arch}}.tar.xz"
  sha256_by_arch:
    x64: "4543670b589593f8fa5f106111fd5139081da42bb165a9239f05195e405f240a"
    arm64: "a9ce85675ba33f00527f6234d90000946c0936fb4fca605f1891bb5f4fe6fb0a"
    armv7l: "964110a031c467a555aa889a6f2f13f7a1d5d4b834403aaa6d9148de050e1563"
```

The system will:

1. Map `amd64` → `x64` using `map_node_arch()`
2. Look up `sha256_by_arch.x64`
3. Fall back to `sha256_by_arch.amd64` if not found
4. Finally use `sha256` if available

## Post-Installation Hooks

Some toolkits require additional steps after extraction. Define a `post_install_hook()` function in `hooks.sh`.

**Example: Rust**

Rust tarballs contain an internal `install.sh` that must be executed:

```bash
# hooks.sh
post_install_hook() {
  local prefix="$1"
  if [[ -x "${prefix}/install.sh" ]]; then
    log "Running Rust internal installer into ${prefix}"
    local prefix_preinstall="${prefix}.preinstall"
    mv "${prefix}" "${prefix_preinstall}" >/dev/null 2>&1
    bash "${prefix_preinstall}/install.sh" --prefix="${prefix}" >/dev/null 2>&1
    rm -rf "${prefix_preinstall}" >/dev/null 2>&1
  fi
}
```

**When to Use:**

- Running internal installers
- Setting up environment variables
- Creating symlinks
- Any post-extraction setup

## Complete Examples

### Example 1: Simple Tarball (No hooks.sh needed)

```yaml
# toolchains/example/manifest.yaml
language: example
version: "1.0.0"
method: tarball
install_prefix: "/opt/toolchains/example/1.0.0"

artifact_template:
  url: "https://example.com/toolkit-{{version}}-{{os}}-{{arch}}.tar.gz"
  sha256: "abc123def456..."

set_default: true
alternatives:
  name: example
  links:
    example: "bin/example"

runtime_env:
  PATH_prepend:
    - "/opt/toolchains/example/current/bin"
```

### Example 2: Tarball with Custom Architecture Mapping

```yaml
# toolchains/mylang/manifest.yaml
language: mylang
version: "2.0.0"
method: tarball
install_prefix: "/opt/toolchains/mylang/2.0.0"

artifact_template:
  url: "https://mylang.org/releases/v{{version}}/mylang-{{mylang_arch}}-{{os}}.tar.xz"
  sha256_by_arch:
    special: "sha256_for_amd64..."
    special64: "sha256_for_arm64..."
```

```bash
#!/usr/bin/env bash
# toolchains/mylang/hooks.sh

map_mylang_arch() {
  case "${1:-}" in
    amd64) echo "special" ;;
    arm64) echo "special64" ;;
    *)     echo "special" ;;
  esac
}

TEMPLATE_VARS=("mylang_arch")
SHA256_ARCH_KEYS=("mylang_arch" "arch")
```

### Example 3: Build from Source

```yaml
# toolchains/mylang/manifest.yaml
language: mylang
version: "3.0.0"
method: build
install_prefix: "/opt/toolchains/mylang/3.0.0"

build:
  url: "https://mylang.org/source/mylang-{{version}}.tar.gz"
  sha256: "abc123..."
  configure_args:
    - "--prefix={{install_prefix}}"
    - "--enable-feature"
    - "--with-library=/usr/local"
```

### Example 4: APT Packages

```yaml
# toolchains/mylang/manifest.yaml
language: mylang
version: "system"
method: apt
install_prefix: "/opt/toolchains/mylang/system"

apt:
  snapshot: "http://snapshot.debian.org/archive/debian/20241010T000000Z"
  packages:
    - "mylang"
    - "mylang-dev"
    - "mylang-tools"

set_default: true
alternatives:
  name: mylang
  links:
    mylang: "/usr/bin/mylang"
```

## Best Practices

### 1. Directory Naming

- Use lowercase, hyphenated names (e.g., `my-language`, not `MyLanguage` or `my_language`)
- Match the language identifier exactly

### 2. Version Formatting

- Use semantic versioning strings in quotes: `"1.2.3"`
- Use `"system"` for apt-based installations

### 3. Installation Prefix

- Follow the pattern: `/opt/toolchains/<language>/<version>`
- Use `system` as version for apt installations

### 4. Template Variables

- Use standard variables when possible: `{{version}}`, `{{os}}`, `{{arch}}`
- Create custom variables only when necessary
- Document custom variables in `hooks.sh` comments

### 5. SHA256 Checksums

- Prefer `sha256_by_arch` for architecture-specific checksums
- Always verify checksums before committing
- Include checksums for all supported architectures

### 6. Binary Paths

- List all important binaries in the `binaries` field for verification
- Use relative paths from `install_prefix` (e.g., `"bin/tool"`)
- Use absolute paths for apt-installed binaries (e.g., `"/usr/bin/tool"`)

### 7. PATH Configuration

- Use `runtime_env.PATH_prepend` to add toolkit directories
- Reference the `current` symlink: `/opt/toolchains/<language>/current/bin`
- List multiple paths if needed (e.g., TypeScript needs both TypeScript and Node.js paths)

### 8. Testing

- Test installation on multiple architectures if possible
- Verify all binaries are accessible after installation
- Test `update-alternatives` setup if `set_default: true`
- Verify PATH entries are correctly added

## Common Patterns

### Pattern 1: Language with Standard Architecture Names

No `hooks.sh` needed - use standard `{{arch}}` variable.

```yaml
artifact_template:
  url: "https://example.com/tool-{{version}}-{{os}}-{{arch}}.tar.gz"
  sha256_by_arch:
    amd64: "..."
    arm64: "..."
```

### Pattern 2: Language with Custom Architecture Names

Create `hooks.sh` with architecture mapper.

```bash
# hooks.sh
map_lang_arch() {
  case "${1:-}" in
    amd64) echo "custom64" ;;
    arm64) echo "customarm64" ;;
    *)     echo "custom64" ;;
  esac
}
TEMPLATE_VARS=("lang_arch")
SHA256_ARCH_KEYS=("lang_arch" "arch")
```

### Pattern 3: Language Requiring Post-Install Steps

Add `post_install_hook()` to `hooks.sh`.

```bash
# hooks.sh
post_install_hook() {
  local prefix="$1"
  # Custom post-installation logic
  # e.g., run internal installer, create symlinks, etc.
}
```

## Troubleshooting

### Issue: Template variables not substituted

**Solution:** Ensure `TEMPLATE_VARS` array is defined in `hooks.sh` and the mapper function exists.

```bash
# hooks.sh
TEMPLATE_VARS=("my_var")
map_my_var() { ... }
```

### Issue: SHA256 not found

**Solution:** Check `SHA256_ARCH_KEYS` order in `hooks.sh` and ensure checksums are in `manifest.yaml` with correct architecture keys.

### Issue: Post-install hook not executed

**Solution:** Verify `post_install_hook()` function is defined in `hooks.sh` and is executable (no syntax errors).

### Issue: Binaries not found after installation

**Solution:**

- Check `install_prefix` path
- Verify binary paths in `binaries` field
- For tarballs, check if extraction preserves directory structure
- For build method, verify `configure_args` include correct prefix

## Reference

### Supported Architectures

The system normalizes architectures to:

- `amd64` (x86_64)
- `arm64` (aarch64)
- `armv7l` (32-bit ARM)
- `ppc64le` (PowerPC 64-bit little-endian)
- `s390x` (IBM Z)

### Supported Operating Systems

- `linux` (default)

### Template Variables Reference

| Variable             | Description             | Example                              |
| -------------------- | ----------------------- | ------------------------------------ |
| `{{version}}`        | Version string          | `1.81.0`                             |
| `{{os}}`             | Operating system        | `linux`, `darwin`                    |
| `{{arch}}`           | Normalized architecture | `amd64`, `arm64`                     |
| `{{install_prefix}}` | Installation prefix     | `/opt/toolchains/rust/1.81.0`        |
| Custom variables     | Defined in `hooks.sh`   | `{{node_arch}}`, `{{rust_arch_tag}}` |

### hooks.sh Interface

```bash
#!/usr/bin/env bash
# Language-specific installation hooks

# Architecture mapping functions (optional)
map_<var_name>() {
  local arch="$1"
  # Return language-specific architecture string
}

# Template variables to substitute (required if using custom variables)
TEMPLATE_VARS=("var1" "var2")

# SHA256 lookup order (optional, defaults to ["arch"])
SHA256_ARCH_KEYS=("var1" "arch")

# Post-installation hook (optional)
post_install_hook() {
  local prefix="$1"
  # Custom installation steps
}
```

## Dockerfile Integration

After creating your language toolkit, you need to add it to the Dockerfile to include it in the Docker image build.

### Step 1: Add Build Stage

Add a new build stage for your language toolkit in `docker/Dockerfile`:

```dockerfile
FROM toolchain-base AS toolchain-builder-<language>
COPY toolchains/<language>/ /toolchains/<language>/
RUN ./install.sh <language>/manifest.yaml
```

**Example (for a hypothetical `dart` language):**

```dockerfile
FROM toolchain-base AS toolchain-builder-dart
COPY toolchains/dart/ /toolchains/dart/
RUN ./install.sh dart/manifest.yaml
```

**Placement:** Add the build stage after other toolchain build stages, following the alphabetical order convention (e.g., after `c`, `cpp`, `go`, etc.).

### Step 2: Copy Toolchain to Final Image

In the final `runner` stage, add a `COPY` instruction to include your toolchain:

```dockerfile
COPY --from=toolchain-builder-<language> /opt/toolchains/<language>/current /opt/toolchains/<language>/current
```

**Example:**

```dockerfile
COPY --from=toolchain-builder-dart /opt/toolchains/dart/current /opt/toolchains/dart/current
```

**Placement:** Add this line in the `# copy toolchains` section, maintaining alphabetical order.

### Complete Example

Here's how a complete integration looks in the Dockerfile:

```dockerfile
# Build stage (in the toolchain-builder section)
FROM toolchain-base AS toolchain-builder-dart
COPY toolchains/dart/ /toolchains/dart/
RUN ./install.sh dart/manifest.yaml

# Final image (in the runner stage)
COPY --from=toolchain-builder-dart /opt/toolchains/dart/current /opt/toolchains/dart/current
```

### Notes

- **Build stages are independent:** Each language toolkit is built in its own stage, so they can be built in parallel and cached separately.
- **Use `current` symlink:** Always copy the `current` symlink, not the version-specific directory. This allows switching versions without changing the Dockerfile.
- **Order matters for clarity:** Maintaining alphabetical order makes it easier to find and manage toolchains.

## Next Steps

1. Create the language directory: `toolchains/<language>/`
2. Create `manifest.yaml` with required fields
3. Create `hooks.sh` if needed (architecture mapping, custom variables, post-install hooks)
4. Test installation: `./toolchains/install.sh toolchains/<language>/manifest.yaml`
5. Verify binaries and PATH configuration
6. **Add to Dockerfile** (see "Dockerfile Integration" section above)
7. Test Docker build: `docker build -f docker/Dockerfile -t pecan .`
8. Document any special requirements or quirks

For questions or issues, refer to existing language toolkits as examples (e.g., `rust/`, `go/`, `node/`, `python/`).
