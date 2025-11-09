#!/usr/bin/env bash
# Go-specific installation hooks

# Go binary naming mapper
# Input: normalized arch (amd64|arm64|armv7l|...)
# Output: Go's label (amd64|arm64|armv6l|...)
map_go_arch() {
  local arch="$1"
  case "$arch" in
    amd64)   echo "amd64" ;;
    arm64)   echo "arm64" ;;
    armv7l)  echo "armv6l" ;;
    ppc64le) echo "ppc64le" ;;
    s390x)   echo "s390x" ;;
    *)       echo "$arch" ;;
  esac
}

# Template variables used in manifest.yaml artifact_template.url
TEMPLATE_VARS=("go_arch")

# SHA256 lookup order: Go uses go_arch in sha256_by_arch keys
SHA256_ARCH_KEYS=("go_arch" "arch")

# No post-install hook needed for Go (simple tarball extract)