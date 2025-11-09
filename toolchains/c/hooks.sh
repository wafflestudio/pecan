#!/usr/bin/env bash
# C-specific installation hooks

# C toolchain uses apt method, no special architecture mapping needed
TEMPLATE_VARS=()

# SHA256 lookup: not applicable for apt method
SHA256_ARCH_KEYS=("arch")

# No post-install hook needed for C (apt install)