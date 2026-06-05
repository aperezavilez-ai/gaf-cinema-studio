#!/usr/bin/env bash
# Generate UniFFI bindings for iOS and Android.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ENGINE="$ROOT/engine"
OUT_IOS="$ROOT/ios/CinemaStudio/Generated"
OUT_ANDROID="$ROOT/android/CinemaStudio/generated"

echo "Building Rust engine with FFI..."
cd "$ENGINE"
cargo build --release --features ffi

echo "Generating Swift bindings..."
mkdir -p "$OUT_IOS"
if command -v uniffi-bindgen &>/dev/null; then
  uniffi-bindgen generate src/ffi/mod.rs --language swift --out-dir "$OUT_IOS"
else
  echo "Install: cargo install uniffi-bindgen-cli"
fi

echo "Generating Kotlin bindings..."
mkdir -p "$OUT_ANDROID"
if command -v uniffi-bindgen &>/dev/null; then
  uniffi-bindgen generate src/ffi/mod.rs --language kotlin --out-dir "$OUT_ANDROID"
fi

echo "Done. Wire static lib in Xcode / Gradle, then enable useNativeEngine."
