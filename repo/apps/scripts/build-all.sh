#!/bin/bash
#
# Rustica Applications Build Script
#
# Builds all applications in the workspace for multiple architectures

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    log_error "Must be run from apps directory"
    exit 1
fi

# Parse arguments
TARGETS="${TARGETS:-amd64}"
BUILD_TYPE="${BUILD_TYPE:-debug}"

# Set build flags
if [ "$BUILD_TYPE" = "release" ]; then
    CARGO_FLAGS="--release"
else
    CARGO_FLAGS=""
fi

log_info "Building Rustica applications..."
log_info "  Targets: $TARGETS"
log_info "  Type: $BUILD_TYPE"
echo

# Build for each target
for target in $TARGETS; do
    case $target in
        amd64)
            TARGET_TRIPLE="x86_64-unknown-linux-gnu"
            ;;
        arm64)
            TARGET_TRIPLE="aarch64-unknown-linux-gnu"
            ;;
        riscv64)
            TARGET_TRIPLE="riscv64gc-unknown-linux-gnu"
            ;;
        *)
            log_error "Unknown target: $target"
            exit 1
            ;;
    esac

    log_info "Building for $target ($TARGET_TRIPLE)..."

    # Add target if not already installed
    if ! rustup target list --installed 2>/dev/null | grep -q "$TARGET_TRIPLE"; then
        log_info "  Adding target $TARGET_TRIPLE..."
        rustup target add "$TARGET_TRIPLE"
    fi

    # Build
    cargo build $CARGO_FLAGS --target "$TARGET_TRIPLE" --workspace

    if [ $? -eq 0 ]; then
        log_info "  ✓ Build successful for $target"
    else
        log_error "  ✗ Build failed for $target"
        exit 1
    fi

    echo
done

log_info "All builds completed successfully!"

# Show summary
echo
log_info "Build Summary:"
for target in $TARGETS; do
    case $target in
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

    BIN_DIR="target/$TARGET_TRIPLE/$BUILD_TYPE"
    if [ "$BUILD_TYPE" = "release" ]; then
        BIN_DIR="target/$TARGET_TRIPLE/release"
    else
        BIN_DIR="target/$TARGET_TRIPLE/debug"
    fi

    if [ -d "$BIN_DIR" ]; then
        BIN_COUNT=$(find "$BIN_DIR" -maxdepth 1 -executable -type f 2>/dev/null | wc -l)
        echo "  $target: $BIN_COUNT binaries"
    fi
done
