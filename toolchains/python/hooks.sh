#!/usr/bin/env bash
# Python-specific installation hooks

# Python binary naming mapper
# Input: normalized arch (amd64|arm64|armv7l|...)
# Output: Python's target triple (x86_64-unknown-linux-gnu|aarch64-unknown-linux-gnu|...)
map_python_arch() {
  case "${1:-}" in
    amd64)   echo "x86_64-unknown-linux-gnu" ;;
    arm64)   echo "aarch64-unknown-linux-gnu" ;;
    *)       echo "x86_64-unknown-linux-gnu" ;;
  esac
}

# Template variables: Python build method may use standard placeholders
TEMPLATE_VARS=("python_arch")

# SHA256 lookup order: Python uses standard arch keys
SHA256_ARCH_KEYS=("python_arch" "arch")

# No post-install hook needed for Python (standard build process)