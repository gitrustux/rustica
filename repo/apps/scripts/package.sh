#!/bin/bash
#
# Rustica Applications Package Script
#
# Packages built applications for distribution

set -e

# Colors
GREEN='\033[0;32m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

# Configuration
OUTPUT_BASE="../../releases"
VERSION="${VERSION:-0.1.0}"
DIST_DIR="dist-$VERSION"

log_info "Packaging Rustica applications..."
log_info "  Version: $VERSION"
echo

# Create distribution directory
mkdir -p "$DIST_DIR"

# Package CLI applications
log_info "Packaging CLI applications..."

for arch in amd64 arm64 riscv64; do
    case $arch in
        amd64)
            TARGET_TRIPLE="x86_64-unknown-linux-gnu"
            ;;
        arm64)
            TARGET_TRIPLE="aarch64-unknown-linux-gnu"
            ;;
        riscv64)
            TARGET_TRIPLE="riscv64gc-unknown-linux-gnu"
            ;;
    esac

    RELEASE_DIR="$OUTPUT_BASE/cli/$arch"
    ARCH_DIR="$DIST_DIR/cli-$arch"

    log_info "  $arch ($TARGET_TRIPLE)..."

    # Create archive directory structure
    mkdir -p "$ARCH_DIR/bin"
    mkdir -p "$ARCH_DIR/lib"

    # Copy binaries if they exist
    if [ -d "target/$TARGET_TRIPLE/release" ]; then
        cp target/$TARGET_TRIPLE/release/redit "$ARCH_DIR/bin/" 2>/dev/null || true
        cp target/$TARGET_TRIPLE/release/net-tools "$ARCH_DIR/bin/" 2>/dev/null || true
        cp target/$TARGET_TRIPLE/release/sys-tools "$ARCH_DIR/bin/" 2>/dev/null || true
    fi

    # Create tarball
    tar -czf "$DIST_DIR/rustica-cli-$arch-$VERSION.tar.gz" -C "$ARCH_DIR" . 2>/dev/null || true

    log_info "    Created rustica-cli-$arch-$VERSION.tar.gz"
done

# Package Aurora Desktop
log_info "Packaging Aurora Desktop applications..."

for arch in amd64 arm64 riscv64; do
    case $arch in
        amd64)
            TARGET_TRIPLE="x86_64-unknown-linux-gnu"
            ;;
        arm64)
            TARGET_TRIPLE="aarch64-unknown-linux-gnu"
            ;;
        riscv64)
            TARGET_TRIPLE="riscv64gc-unknown-linux-gnu"
            ;;
    esac

    RELEASE_DIR="$OUTPUT_BASE/desktop/aurora/$arch"
    ARCH_DIR="$DIST_DIR/aurora-$arch"

    log_info "  Aurora for $arch ($TARGET_TRIPLE)..."

    # Create archive directory structure
    mkdir -p "$ARCH_DIR/bin"
    mkdir -p "$ARCH_DIR/lib"
    mkdir -p "$ARCH_DIR/share"

    # Copy binaries if they exist
    if [ -d "target/$TARGET_TRIPLE/release" ]; then
        cp target/$TARGET_TRIPLE/release/aurora-shell "$ARCH_DIR/bin/" 2>/dev/null || true
        cp target/$TARGET_TRIPLE/release/aurora-panel "$ARCH_DIR/bin/" 2>/dev/null || true
        cp target/$TARGET_TRIPLE/release/aurora-launcher "$ARCH_DIR/bin/" 2>/dev/null || true
    fi

    # Create tarball
    tar -czf "$DIST_DIR/rustica-aurora-$arch-$VERSION.tar.gz" -C "$ARCH_DIR" . 2>/dev/null || true

    log_info "    Created rustica-aurora-$arch-$VERSION.tar.gz"
done

log_info "Packaging complete!"
echo
log_info "Packages created in: $DIST_DIR"

# List packages
echo
log_info "Available packages:"
ls -lh "$DIST_DIR"/*.tar.gz 2>/dev/null || echo "  No packages found"
