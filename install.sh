#!/bin/sh
# Installs (or updates) FaceCam from the latest GitHub release.
#   curl -fsSL https://raw.githubusercontent.com/trouze/facecam/main/install.sh | sh
#
# Optional install location (defaults to /Applications):
#   INSTALL_DIR="$HOME/Applications" sh install.sh
set -eu

REPO="trouze/facecam"
INSTALL_DIR="${INSTALL_DIR:-/Applications}"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "Downloading latest FaceCam..."
curl -fsSL --fail \
    "https://github.com/$REPO/releases/latest/download/FaceCam.app.zip" \
    -o "$TMP/FaceCam.app.zip"

echo "Extracting..."
ditto -x -k "$TMP/FaceCam.app.zip" "$TMP/app"

if [ ! -d "$TMP/app/FaceCam.app" ]; then
    echo "error: downloaded archive did not contain FaceCam.app" >&2
    exit 1
fi

mkdir -p "$INSTALL_DIR"
if [ -d "$INSTALL_DIR/FaceCam.app" ]; then
    echo "Removing existing installation..."
    rm -rf "$INSTALL_DIR/FaceCam.app"
fi

echo "Installing to $INSTALL_DIR..."
mv "$TMP/app/FaceCam.app" "$INSTALL_DIR/"

echo
echo "FaceCam installed! Launch it with:"
echo "  open -a FaceCam"
