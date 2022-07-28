#!/bin/sh
set -e

# AARCH64
TARGET_ARCH_AARCH64="aarch64-unknown-linux-gnu"
LINK_FLAGS_AARCH64="-L /usr/aarch64-linux-gnu/lib -L /usr/lib/aarch64-linux-gnu"

# AMD64
TARGET_ARCH_AMD64="x86_64-unknown-linux-gnu"
LINK_FLAGS_AMD64=""

# ARMv7
TARGET_ARCH_ARMV7="armv7-unknown-linux-gnueabihf"
LINK_FLAGS_ARMV7="-L /usr/arm-linux-gnueabihf/lib -L /usr/lib/arm-linux-gnueabihf"

build() {
    local target_arch="$1"
    local link_flags="$2"

    env RUSTFLAGS="${link_flags}" cross build \
        --release \
        --target="${target_arch}"
}

main() {
    local target="$1"

    local target_arch
    local link_flags

    case "${target}" in
        "aarch64")
            target_arch="${TARGET_ARCH_AARCH64}"
            link_flags="${LINK_FLAGS_AARCH64}"
            ;;
        "amd64")
            target_arch="${TARGET_ARCH_AMD64}"
            link_flags="${LINK_FLAGS_AMD64}"
            ;;
        "armv7")
            target_arch="${TARGET_ARCH_ARMV7}"
            link_flags="${LINK_FLAGS_ARMV7}"
            ;;
        *)
            echo "Unknown target '${target}'" >&2
            return 1
            ;;
    esac

    build "${target_arch}" "${link_flags}"
}

main "$@"
