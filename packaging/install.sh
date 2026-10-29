#!/usr/bin/env bash
set -e

PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="$PREFIX/bin"
APP_DIR="$PREFIX/share/applications"
ICON_DIR="$PREFIX/share/icons/hicolor/128x128/apps"

echo "==> Installing Cruce to $PREFIX..."

mkdir -p "$BIN_DIR" "$APP_DIR" "$ICON_DIR"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Copy binary
install -m 755 "$SCRIPT_DIR/cruce" "$BIN_DIR/cruce"

# Copy icon
if [ -f "$SCRIPT_DIR/icons/128x128.png" ]; then
    install -m 644 "$SCRIPT_DIR/icons/128x128.png" "$ICON_DIR/cruce.png"
fi

# Copy and patch desktop entry
if [ -f "$SCRIPT_DIR/cruce.desktop" ]; then
    sed -e "s|Exec=cruce|Exec=$BIN_DIR/cruce|" "$SCRIPT_DIR/cruce.desktop" > "$APP_DIR/cruce.desktop"
    chmod 644 "$APP_DIR/cruce.desktop"
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database "$APP_DIR" || true
    fi
fi

echo "==> Cruce installed successfully!"
echo "    Run 'cruce' or launch it from your application menu."
