#!/usr/bin/env bash
set -e

VERSION="0.3.0"
ARCH="x86_64"
DIST_NAME="cruce-${VERSION}-${ARCH}-linux"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist"
STAGING_DIR="$DIST_DIR/$DIST_NAME"

echo "==> Building frontend production bundle..."
(cd "$ROOT_DIR/ui" && bun run build)

echo "==> Building optimized release binary for $DIST_NAME..."
cd "$ROOT_DIR"
cargo build --release -p cruce-app

RELEASE_BIN="$ROOT_DIR/target/release/cruce-app"
if [ ! -f "$RELEASE_BIN" ]; then
    echo "Error: Release binary not found at $RELEASE_BIN" >&2
    exit 1
fi

echo "==> Assembling standalone tarball layout..."
rm -rf "$STAGING_DIR" "$DIST_DIR/${DIST_NAME}.tar.gz"
mkdir -p "$DIST_DIR"
mkdir -p "$STAGING_DIR/icons"

# Copy binary & strip
cp "$RELEASE_BIN" "$STAGING_DIR/cruce"
strip --strip-all "$STAGING_DIR/cruce" 2>/dev/null || true
chmod 755 "$STAGING_DIR/cruce"

# Copy desktop and install scripts
cp "$ROOT_DIR/packaging/cruce.desktop" "$STAGING_DIR/cruce.desktop"
cp "$ROOT_DIR/packaging/install.sh" "$STAGING_DIR/install.sh"
chmod +x "$STAGING_DIR/install.sh"

# Copy documentation
cp "$ROOT_DIR/README.md" "$STAGING_DIR/README.md"

# Copy icons
if [ -d "$ROOT_DIR/crates/cruce-app/icons" ]; then
    cp -r "$ROOT_DIR/crates/cruce-app/icons/"* "$STAGING_DIR/icons/"
fi

echo "==> Compressing tarball..."
(cd "$DIST_DIR" && tar -czf "${DIST_NAME}.tar.gz" "$DIST_NAME")
rm -rf "$STAGING_DIR"

echo "==> Successfully created standalone distribution tarball:"
ls -lh "$DIST_DIR/${DIST_NAME}.tar.gz"
