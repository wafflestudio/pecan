#!/usr/bin/env bash
set -euo pipefail

# toolchains/install.sh <manifest.yaml>
# Requires: yq, curl, sha256sum, tar, xz, gzip, make, gcc, build-essential
# Notes:
# - Uses TARGETOS/TARGETARCH if provided (Docker buildx), else uname fallback.
# - Supports tarball with 'artifact_template' (url templating, per-arch sha256).

log() { echo "[toolchains] $*"; }

die() { echo "[toolchains][ERROR] $*" >&2; exit 1; }

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "Required command not found: $1"
}

apt_update_safe() {
  require_cmd apt-get
  DEBIAN_FRONTEND=noninteractive apt-get update
}

apt_cleanup() {
  require_cmd apt-get
  apt-get clean
  rm -rf /var/lib/apt/lists/*
}

download_file() {
  local url="$1" out="$2"
  require_cmd curl
  curl -fsSL "$url" -o "$out"
}

verify_sha256() {
  local file="$1" expect="$2"
  require_cmd sha256sum
  echo "${expect}  ${file}" | sha256sum -c - >/dev/null
}

extract_archive() {
  local file="$1" dest="$2"
  mkdir -p "$dest"
  case "$file" in
    *.tar.xz|*.txz) tar -xJf "$file" -C "$dest" --strip-components=1 ;;
    *.tar.gz|*.tgz) tar -xzf "$file" -C "$dest" --strip-components=1 ;;
    *.tar.bz2)      tar -xjf "$file" -C "$dest" --strip-components=1 ;;
    *.zip)          require_cmd unzip; unzip -oq "$file" -d "$dest" ;;
    *)
      if file "$file" | grep -qi "archive"; then
        die "Unknown archive format: $file"
      else
        cp -a "$file" "$dest/"
      fi
      ;;
  esac
}

detect_platform() {
  local os="${TARGETOS:-}"
  local arch="${TARGETARCH:-}"

  if [[ -z "$os" ]]; then
    # uname -s -> linux/darwin
    case "$(uname -s | tr '[:upper:]' '[:lower:]')" in
      linux)  os="linux" ;;
      darwin) os="darwin" ;;
      *)      os="linux" ;; # default
    esac
  fi

  if [[ -z "$arch" ]]; then
    # Normalize uname -m
    case "$(uname -m)" in
      x86_64|amd64) arch="amd64" ;;
      aarch64|arm64) arch="arm64" ;;
      armv7l|armv7) arch="armv7l" ;;
      ppc64le) arch="ppc64le" ;;
      s390x) arch="s390x" ;;
      *) arch="amd64" ;;
    esac
  else
    # TARGETARCH --> normalize
    case "$arch" in
      amd64|x86_64) arch="amd64" ;;
      arm64|aarch64) arch="arm64" ;;
      arm/v7|armv7|armv7l) arch="armv7l" ;;
      ppc64le) arch="ppc64le" ;;
      s390x) arch="s390x" ;;
      *) arch="amd64" ;;
    esac
  fi

  DETECTED_OS="$os"
  DETECTED_ARCH="$arch"
}

tpl_subst() {
  local input="$1" key="$2" value="$3"
  # Escape replacement slashes
  value="${value//\//\\/}"
  echo "$input" | sed -E "s/\{\{$key\}\}/$value/g"
}

# Apply language-specific template variables to URL
# Uses TEMPLATE_VARS array from hooks.sh and corresponding mapper functions
apply_language_template_vars() {
  local url="$1"
  local arch="$2"
  
  if [[ -n "${TEMPLATE_VARS:-}" ]]; then
    for var_name in "${TEMPLATE_VARS[@]}"; do
      mapper_func="map_${var_name}"
      if declare -f "$mapper_func" >/dev/null 2>&1; then
        mapped_value="$("$mapper_func" "$arch")"
        url="$(tpl_subst "$url" "$var_name" "$mapped_value")"
      fi
    done
  fi
  
  echo "$url"
}

# Resolve SHA256 checksum from manifest using language-specific arch keys
# Uses SHA256_ARCH_KEYS array from hooks.sh for lookup order
resolve_sha256() {
  local manifest="$1"
  local arch="$2"
  local sha256_path="$3"  # e.g., ".artifact_template.sha256_by_arch" or ".artifacts[$i].sha256"
  
  local sha=""
  
  # Try language-specific arch keys first (from hooks.sh)
  if [[ -n "${SHA256_ARCH_KEYS:-}" ]]; then
    for arch_key in "${SHA256_ARCH_KEYS[@]}"; do
      mapper_func="map_${arch_key}"
      if declare -f "$mapper_func" >/dev/null 2>&1; then
        mapped_arch="$("$mapper_func" "$arch")"
        sha="$(yq -r "${sha256_path}.\"${mapped_arch}\" // \"\"" "$manifest")"
        if [[ -n "$sha" && "$sha" != "null" ]]; then
          echo "$sha"
          return 0
        fi
      fi
    done
  fi
  
  sha="$(yq -r "${sha256_path}.\"${arch}\" // \"\"" "$manifest")"
  if [[ -n "$sha" && "$sha" != "null" ]]; then
    echo "$sha"
    return 0
  fi
  
  local base_path="${sha256_path%_by_arch}"
  sha="$(yq -r "${base_path}.sha256 // \"\"" "$manifest")"
  if [[ -n "$sha" && "$sha" != "null" ]]; then
    echo "$sha"
    return 0
  fi
  
  echo ""
}

if [[ $# -ne 1 ]]; then
  echo "Usage: $0 <manifest.yaml>" >&2
  exit 2
fi

MANIFEST="$(realpath "$1")"
require_cmd yq

# Detect platform
detect_platform
OS="$DETECTED_OS"
ARCH="$DETECTED_ARCH"

LANGUAGE="$(yq -r '.language' "$MANIFEST")"
METHOD="$(yq -r '.method' "$MANIFEST")"
VERSION="$(yq -r '.version' "$MANIFEST")"
PREFIX="$(yq -r '.install_prefix' "$MANIFEST")"
SET_DEFAULT="$(yq -r '.set_default // false' "$MANIFEST")"

# Load language-specific hooks if available
LANG_DIR="$(dirname "$MANIFEST")"
HOOKS_FILE="${LANG_DIR}/hooks.sh"
if [[ -f "$HOOKS_FILE" ]]; then
  source "$HOOKS_FILE"
  log "Loaded language-specific hooks from ${HOOKS_FILE}"
fi

log "Installing language=${LANGUAGE} version=${VERSION} method=${METHOD} prefix=${PREFIX} os=${OS} arch=${ARCH}"
mkdir -p "${PREFIX}" >/dev/null 2>&1

case "$METHOD" in
  apt)
    require_cmd apt-get
    SNAPSHOT_URL="$(yq -r '.apt.snapshot // ""' "$MANIFEST")"
    mapfile -t PKGS < <(yq -r '.apt.packages[]' "$MANIFEST")
    if [[ -n "$SNAPSHOT_URL" && "$SNAPSHOT_URL" != "null" ]]; then
      log "Configuring apt snapshot: ${SNAPSHOT_URL}"
      CODENAME="$(. /etc/os-release; echo "${VERSION_CODENAME:-bookworm}")"
      echo "deb [trusted=yes] ${SNAPSHOT_URL} ${CODENAME} main" >/etc/apt/sources.list.d/toolchains-snapshot.list
    fi
    apt_update_safe
    if ((${#PKGS[@]})); then
      log "Installing apt packages: ${PKGS[*]}"
      DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends "${PKGS[@]}"
      for p in "${PKGS[@]}"; do
        apt-mark hold "${p%%=*}" || true
      done
    fi
    apt_cleanup
    ;;

  tarball)
    require_cmd curl
    mkdir -p "${PREFIX}"

    HAS_TEMPLATE="$(yq -r 'has("artifact_template")' "$MANIFEST")"
    if [[ "$HAS_TEMPLATE" == "true" ]]; then
      # Build URL from template
      TPL_URL="$(yq -r '.artifact_template.url' "$MANIFEST")"

      # Generic placeholders
      URL="$TPL_URL"
      URL="$(tpl_subst "$URL" "version" "$VERSION")"
      URL="$(tpl_subst "$URL" "os" "$OS")"
      URL="$(tpl_subst "$URL" "arch" "$ARCH")"

      # Language-specific placeholders (from hooks.sh)
      URL="$(apply_language_template_vars "$URL" "$ARCH")"

      # Optional: filename from template
      OUT="/tmp/$(basename "$URL")"

      # SHA256 resolution using language-specific arch keys
      SHA="$(resolve_sha256 "$MANIFEST" "$ARCH" ".artifact_template.sha256_by_arch")"

      log "Downloading: $URL"
      download_file "$URL" "$OUT"
      if [[ -n "$SHA" && "$SHA" != "null" ]]; then
        verify_sha256 "$OUT" "$SHA"
      else
        log "WARNING: sha256 not provided for ${LANGUAGE} ${VERSION} (${ARCH})"
      fi

      extract_archive "$OUT" "$PREFIX"

      # Post-install hook (from hooks.sh)
      if declare -f post_install_hook >/dev/null 2>&1; then
        post_install_hook "$PREFIX"
      fi
    else
      # Fallback to explicit artifacts array
      ART_COUNT="$(yq -r '.artifacts | length' "$MANIFEST")"
      (( ART_COUNT > 0 )) || die "tarball method requires .artifacts or .artifact_template"
      for i in $(seq 0 $((ART_COUNT-1))); do
        URL="$(yq -r ".artifacts[$i].url" "$MANIFEST")"
        # Placeholder substitutions still supported in artifacts[]
        URL="$(tpl_subst "$URL" "version" "$VERSION")"
        URL="$(tpl_subst "$URL" "os" "$OS")"
        URL="$(tpl_subst "$URL" "arch" "$ARCH")"
        # Language-specific placeholders (from hooks.sh)
        URL="$(apply_language_template_vars "$URL" "$ARCH")"

        SHA="$(yq -r ".artifacts[$i].sha256 // \"\"" "$MANIFEST")"
        # Also try resolve_sha256 for artifacts with sha256_by_arch
        if [[ -z "$SHA" || "$SHA" == "null" ]]; then
          SHA="$(resolve_sha256 "$MANIFEST" "$ARCH" ".artifacts[$i].sha256_by_arch")"
        fi
        
        OUT="/tmp/$(basename "$URL")"
        log "Downloading artifact $i from $URL"
        download_file "$URL" "$OUT"
        if [[ -n "$SHA" && "$SHA" != "null" ]]; then
          verify_sha256 "$OUT" "$SHA"
        fi
        extract_archive "$OUT" "$PREFIX"
      done
      
      # Post-install hook (from hooks.sh)
      if declare -f post_install_hook >/dev/null 2>&1; then
        post_install_hook "$PREFIX"
      fi
    fi
    ;;

  build)
    require_cmd curl
    URL="$(yq -r '.build.url' "$MANIFEST")"
    URL="$(tpl_subst "$URL" "version" "$VERSION")"
    URL="$(tpl_subst "$URL" "os" "$OS")"
    URL="$(tpl_subst "$URL" "arch" "$ARCH")"
    # Language-specific placeholders (from hooks.sh)
    URL="$(apply_language_template_vars "$URL" "$ARCH")"

    SHA="$(yq -r '.build.sha256 // ""' "$MANIFEST")"

    SRC_TGZ="/tmp/$(basename "$URL")"
    log "Downloading source from $URL"
    download_file "$URL" "$SRC_TGZ"
    if [[ -n "$SHA" && "$SHA" != "null" ]]; then
      verify_sha256 "$SRC_TGZ" "$SHA"
    fi

    BUILD_DIR="/tmp/build-$LANGUAGE-$VERSION-$ARCH"
    EXTRACT_DIR="$BUILD_DIR/src"

    rm -rf "$BUILD_DIR" && mkdir -p "$BUILD_DIR"
    extract_archive "$SRC_TGZ" "$EXTRACT_DIR"

    if [[ -f "$EXTRACT_DIR/configure" || -f "$EXTRACT_DIR/CMakeLists.txt" ]]; then
    SRC_ROOT="$EXTRACT_DIR"
    else
    SRC_ROOT="$(find "$EXTRACT_DIR" -mindepth 1 -maxdepth 1 -type d | head -n1)"
    [[ -n "$SRC_ROOT" ]] || die "Unable to locate extracted source root"
    fi

    mapfile -t CFG_ARGS < <(yq -r '.build.configure_args[]? // empty' "$MANIFEST")

    for i in "${!CFG_ARGS[@]}"; do
        CFG_ARGS[$i]="$(tpl_subst "${CFG_ARGS[$i]}" "install_prefix" "$PREFIX")"
        CFG_ARGS[$i]="$(tpl_subst "${CFG_ARGS[$i]}" "version" "$VERSION")"
        CFG_ARGS[$i]="$(tpl_subst "${CFG_ARGS[$i]}" "os" "$OS")"
        CFG_ARGS[$i]="$(tpl_subst "${CFG_ARGS[$i]}" "arch" "$ARCH")"
    done

    pushd "$SRC_ROOT" >/dev/null
    if [[ -x configure || -f configure ]]; then
    log "Configuring with: ${CFG_ARGS[*]}"
    ./configure "${CFG_ARGS[@]}"
    log "Building..."
    make -j"$(nproc)"
    log "Installing to ${PREFIX}"
    make install
    elif [[ -f CMakeLists.txt ]]; then
    require_cmd cmake
    mkdir -p build && cd build
    cmake -DCMAKE_INSTALL_PREFIX="${PREFIX}" ..
    cmake --build . -j"$(nproc)"
    cmake --install .
    else
    die "Unknown build system (no configure or CMakeLists.txt)"
    fi
    popd >/dev/null
    ;;

  *)
    die "Unsupported method: $METHOD"
    ;;
esac

# Set up /opt/toolchains/${LANGUAGE}/current symlink
LANG_BASE="/opt/toolchains/${LANGUAGE}"
mkdir -p "$LANG_BASE"
if [[ -d "$PREFIX" ]]; then
  ln -sfn "$PREFIX" "${LANG_BASE}/current"
fi

# update-alternatives (optional)
ALT_NAME="$(yq -r '.alternatives.name // ""' "$MANIFEST")"
if [[ "$SET_DEFAULT" == "true" && -n "$ALT_NAME" && "$ALT_NAME" != "null" ]]; then
  mapfile -t ALT_KEYS < <(yq -r '.alternatives.links | keys[]?' "$MANIFEST")
  for key in "${ALT_KEYS[@]}"; do
    TARGET_REL="$(yq -r ".alternatives.links[\"${key}\"]" "$MANIFEST")"
    TARGET_ABS="${PREFIX}/${TARGET_REL}"
    if [[ -x "$TARGET_ABS" || -f "$TARGET_ABS" ]]; then
      install -d /usr/local/bin
      update-alternatives --install "/usr/local/bin/${key}" "${key}" "$TARGET_ABS" 100 || true
    else
      log "skip alt link ${key} -> ${TARGET_ABS} (not found)"
    fi
  done
fi

# runtime_env (PATH prepend)
PROFILE_D="/etc/profile.d/toolchains.sh"
PATH_ENTRIES="$(yq -r '.runtime_env.PATH_prepend[]? // empty' "$MANIFEST" | tr '\n' ':')"
if [[ -n "$PATH_ENTRIES" ]]; then
  log "Writing PATH entries to ${PROFILE_D}"
  {
    echo "# auto-generated by toolchains install for ${LANGUAGE} ${VERSION}"
    IFS=":" read -ra ENTRIES <<<"$PATH_ENTRIES"
    for p in "${ENTRIES[@]}"; do
      [[ -n "$p" ]] && echo "export PATH=\"$p:\$PATH\""
    done
  } >>"${PROFILE_D}"
fi

log "Done: ${LANGUAGE} ${VERSION} (${OS}/${ARCH})"