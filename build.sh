#!/bin/sh
# Builds FaceCam.app.
#   ./build.sh                    debug build (arm64)
#   ./build.sh --release          release build (native arch)
#   ./build.sh --release --universal  release universal binary (arm64 + x86_64)
set -eu

MODE=dev
CARGO_ARGS=""
UNIVERSAL=0
for arg in "$@"; do
    case "$arg" in
        --release) MODE=release; CARGO_ARGS="--release" ;;
        --universal) UNIVERSAL=1 ;;
        *) echo "usage: $0 [--release] [--universal]" >&2; exit 2 ;;
    esac
done

APP_NAME=FaceCam

if [ "$UNIVERSAL" = 1 ]; then
    [ "$MODE" = "release" ] || { echo "--universal requires --release" >&2; exit 2; }
    MODE=universal
    OUT="target/universal/facecam"

    rustup target add aarch64-apple-darwin x86_64-apple-darwin

    cargo build --release --target aarch64-apple-darwin
    cargo build --release --target x86_64-apple-darwin

    mkdir -p target/universal
    lipo -create \
        target/aarch64-apple-darwin/release/facecam \
        target/x86_64-apple-darwin/release/facecam \
        -output "$OUT"
else
    cargo build $CARGO_ARGS
    OUT="target/$MODE/facecam"
fi

BUNDLE="target/$MODE/FaceCam.app"

rm -rf "$BUNDLE"
mkdir -p "$BUNDLE/Contents/MacOS"

cp "$OUT" "$BUNDLE/Contents/MacOS/$APP_NAME"
cp Info.plist "$BUNDLE/Contents/Info.plist"

printf 'APPL????' > "$BUNDLE/Contents/PkgInfo"

codesign --force --sign - \
    --entitlements entitlements.plist \
    "$BUNDLE" >/dev/null

echo "Built $BUNDLE"
