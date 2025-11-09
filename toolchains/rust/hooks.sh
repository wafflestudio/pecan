#!/usr/bin/env bash
# Rust-specific installation hooks

# Rust binary naming mapper
# Input: normalized arch (amd64|arm64|armv7l|...)
# Output: Rust's target triple (x86_64-unknown-linux-gnu|aarch64-unknown-linux-gnu|armv6l-unknown-linux-gnueabihf|...)
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

# Template variables used in manifest.yaml artifact_template.url
TEMPLATE_VARS=("rust_arch_tag")

# SHA256 lookup order: Rust uses rust_arch_tag for sha256 lookup
SHA256_ARCH_KEYS=("rust_arch_tag" "arch")

# Post-install hook: Rust tarball contains an internal install.sh that needs to be run
post_install_hook() {
  local prefix="$1"
  if [[ -x "${prefix}/install.sh" ]]; then
    log "Running Rust internal installer into ${prefix}"
    local prefix_preinstall="${prefix}.preinstall"
    mv "${prefix}" "${prefix_preinstall}" >/dev/null 2>&1

    # do not install unnecessary components
    bash "${prefix_preinstall}/install.sh" \
      --prefix="${prefix}" \
      --without=rust-docs,rust-docs-json-preview,rustfmt-preview,rls-preview,rust-analyzer-preview,llvm-tools-preview,clippy-preview,rust-analysis-x86_64-unknown-linux-gnu,llvm-bitcode-linker-preview \
      >/dev/null 2>&1
    rm -rf "${prefix_preinstall}" >/dev/null 2>&1
  fi
}