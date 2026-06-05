#!/usr/bin/env bash
# Cross-compile Rust engine for iOS and pack XCFramework scaffold.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ENGINE="$ROOT/engine"
OUT="$ROOT/ios/Generated"
LIB_NAME="libcinemastudio_engine.a"

echo "==> CinemaStudio iOS engine build"

rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios 2>/dev/null || true

cd "$ENGINE"

echo "==> Building device (aarch64-apple-ios)..."
cargo build --release --target aarch64-apple-ios --features ffi

echo "==> Building simulator (aarch64-apple-ios-sim)..."
cargo build --release --target aarch64-apple-ios-sim --features ffi

mkdir -p "$OUT/lib" "$OUT/include"

DEVICE_LIB="$ENGINE/target/aarch64-apple-ios/release/$LIB_NAME"
SIM_LIB="$ENGINE/target/aarch64-apple-ios-sim/release/$LIB_NAME"

cp "$DEVICE_LIB" "$OUT/lib/device.a"
cp "$SIM_LIB" "$OUT/lib/sim.a"

if command -v xcodebuild &>/dev/null; then
  echo "==> Creating XCFramework..."
  rm -rf "$OUT/CinemaStudioEngine.xcframework"
  xcodebuild -create-xcframework \
    -library "$OUT/lib/device.a" \
    -library "$OUT/lib/sim.a" \
    -output "$OUT/CinemaStudioEngine.xcframework"
  echo "XCFramework: $OUT/CinemaStudioEngine.xcframework"
else
  echo "xcodebuild not found — static libs at $OUT/lib/"
fi

echo "==> Generate Swift bindings:"
echo "  ./scripts/generate_bindings.sh"
