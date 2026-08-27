#!/bin/sh
# Installs (or updates) facecam from the latest GitHub release.
#   curl -fsSL https://raw.githubusercontent.com/trouze/facecam/main/install.sh | sh
#
# Optional install location (defaults to /Applications):
#   INSTALL_DIR="$HOME/Applications" sh install.sh
set -eu

REPO="trouze/facecam"
INSTALL_DIR="${INSTALL_DIR:-/Applications}"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "Downloading latest facecam..."
curl -fsSL --fail \
    "https://github.com/$REPO/releases/latest/download/facecam.app.zip" \
    -o "$TMP/facecam.app.zip"

echo "Extracting..."
ditto -x -k "$TMP/facecam.app.zip" "$TMP/app"

if [ ! -d "$TMP/app/facecam.app" ]; then
    echo "error: downloaded archive did not contain facecam.app" >&2
    exit 1
fi

mkdir -p "$INSTALL_DIR"
if [ -d "$INSTALL_DIR/facecam.app" ]; then
    echo "Removing existing installation..."
    rm -rf "$INSTALL_DIR/facecam.app"
fi

echo "Installing to $INSTALL_DIR..."
mv "$TMP/app/facecam.app" "$INSTALL_DIR/"

echo
echo "facecam installed! Launch it with:"
echo "  open -a facecam"
