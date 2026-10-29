#!/usr/bin/env bash
set -e

VERSION="0.3.0"
ARCH="amd64"
PKG_NAME="cruce_${VERSION}_${ARCH}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist"
STAGING_DIR="$DIST_DIR/$PKG_NAME"

echo "==> Building frontend production bundle..."
(cd "$ROOT_DIR/ui" && bun run build)

echo "==> Building optimized release binary for $PKG_NAME..."
cd "$ROOT_DIR"
cargo build --release -p cruce-app

RELEASE_BIN="$ROOT_DIR/target/release/cruce-app"
if [ ! -f "$RELEASE_BIN" ]; then
    echo "Error: Release binary not found at $RELEASE_BIN" >&2
    exit 1
fi

echo "==> Assembling Debian package layout..."
rm -rf "$STAGING_DIR" "$DIST_DIR/${PKG_NAME}.deb"
mkdir -p "$DIST_DIR"
mkdir -p "$STAGING_DIR/DEBIAN"
mkdir -p "$STAGING_DIR/usr/bin"
mkdir -p "$STAGING_DIR/usr/share/applications"
mkdir -p "$STAGING_DIR/usr/share/icons/hicolor/128x128/apps"
mkdir -p "$STAGING_DIR/usr/share/icons/hicolor/32x32/apps"

# Copy binary & strip
cp "$RELEASE_BIN" "$STAGING_DIR/usr/bin/cruce"
strip --strip-all "$STAGING_DIR/usr/bin/cruce" 2>/dev/null || true
chmod 755 "$STAGING_DIR/usr/bin/cruce"

# Copy desktop file
cp "$ROOT_DIR/packaging/cruce.desktop" "$STAGING_DIR/usr/share/applications/cruce.desktop"
chmod 644 "$STAGING_DIR/usr/share/applications/cruce.desktop"

# Copy icons
if [ -f "$ROOT_DIR/crates/cruce-app/icons/128x128.png" ]; then
    cp "$ROOT_DIR/crates/cruce-app/icons/128x128.png" "$STAGING_DIR/usr/share/icons/hicolor/128x128/apps/cruce.png"
fi
if [ -f "$ROOT_DIR/crates/cruce-app/icons/32x32.png" ]; then
    cp "$ROOT_DIR/crates/cruce-app/icons/32x32.png" "$STAGING_DIR/usr/share/icons/hicolor/32x32/apps/cruce.png"
fi

# Calculate installed size in kB
INSTALLED_SIZE=$(du -sk "$STAGING_DIR/usr" | cut -f1)

# Generate control file
cat <<EOF > "$STAGING_DIR/DEBIAN/control"
Package: cruce
Version: ${VERSION}
Section: games
Priority: optional
Architecture: ${ARCH}
Installed-Size: ${INSTALLED_SIZE}
Maintainer: Cruce Team <support@cruce.app>
Depends: libc6 (>= 2.34), libgtk-3-0, libwebkit2gtk-4.1-0 | libwebkit2gtk-4.0-37
Homepage: https://github.com/cruce/cruce
Description: High-Performance Proton & Wine Game Launcher for Linux
 Cruce is an ultra-fast desktop launcher for running Windows games on Linux
 with GE-Proton, UMU, Wine, and micro-compositors like Gamescope. Built with
 pure Rust, Tauri v2, and React 19.
EOF

echo "==> Creating debian package archives..."
TEMP_BUILD="$DIST_DIR/tmp_deb_build"
rm -rf "$TEMP_BUILD"
mkdir -p "$TEMP_BUILD"

echo "2.0" > "$TEMP_BUILD/debian-binary"

# Tar control
(cd "$STAGING_DIR/DEBIAN" && tar -czf "$TEMP_BUILD/control.tar.gz" ./*)

# Tar data
(cd "$STAGING_DIR" && tar --exclude='./DEBIAN' -czf "$TEMP_BUILD/data.tar.gz" ./usr)

# Ar package
cd "$TEMP_BUILD"
ar rcs "$DIST_DIR/${PKG_NAME}.deb" debian-binary control.tar.gz data.tar.gz

rm -rf "$TEMP_BUILD" "$STAGING_DIR"

echo "==> Successfully created Debian package:"
ls -lh "$DIST_DIR/${PKG_NAME}.deb"
