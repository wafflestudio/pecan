#!/usr/bin/env bash
# Java-specific installation hooks

# Java binary naming mapper
# Input: normalized arch (amd64|arm64|armv7l|...)
# Output: Java's label (x64|aarch64|...)
map_java_arch() {
  case "${1:-}" in
    amd64)   echo "x64" ;;
    arm64)   echo "aarch64" ;;
    *)       echo "x64" ;;
  esac
}

# Template variables used in manifest.yaml artifact_template.url
TEMPLATE_VARS=("java_arch")

# SHA256 lookup order: Java uses java_arch in sha256_by_arch keys
SHA256_ARCH_KEYS=("java_arch" "arch")

# No post-install hook needed for Java (simple tarball extract)