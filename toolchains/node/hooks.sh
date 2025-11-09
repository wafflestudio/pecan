#!/usr/bin/env bash
# Node.js-specific installation hooks

# Node.js binary naming mapper
# Input: normalized arch (amd64|arm64|armv7l|...)
# Output: Node.js's label (x64|arm64|armv7l|...)
map_node_arch() {
  case "${1:-}" in
    amd64)  echo "x64" ;;
    arm64)  echo "arm64" ;;
    armv7l) echo "armv7l" ;;
    *)      echo "x64" ;;
  esac
}

# Template variables used in manifest.yaml artifact_template.url
TEMPLATE_VARS=("node_arch")

# SHA256 lookup order: Node uses node_arch in sha256_by_arch keys
SHA256_ARCH_KEYS=("node_arch" "arch")

# No post-install hook needed for Node.js (simple tarball extract)