#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_DIR"

# Read version from Cargo.toml
VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
BINARY="reframe"
DIST_DIR="$PROJECT_DIR/dist"

echo "==> Building reframe v${VERSION} for 3 platforms"
echo ""

# Ensure dist directory is clean
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

# Platform targets
TARGETS=(
    "aarch64-apple-darwin"
    "x86_64-apple-darwin"
    "x86_64-unknown-linux-musl"
)

for TARGET in "${TARGETS[@]}"; do
    echo "--- Building for ${TARGET} ---"
    cargo zigbuild --release --target "$TARGET"

    SRC="$PROJECT_DIR/target/$TARGET/release/$BINARY"
    ZIP_NAME="${BINARY}-${VERSION}-${TARGET}.zip"
    ZIP_PATH="$DIST_DIR/$ZIP_NAME"

    echo "--- Zipping -> ${ZIP_NAME} ---"
    zip -j "$ZIP_PATH" "$SRC"

    echo ""
done

echo "==> All builds complete. Artifacts in dist/:"
ls -lh "$DIST_DIR"
