#!/usr/bin/env bash
# Cross-compile Rust engine for Android ABIs.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ENGINE="$ROOT/engine"
OUT="$ROOT/android/app/src/main/jniLibs"

: "${ANDROID_NDK_HOME:?Set ANDROID_NDK_HOME to your NDK path}"

API=26
ABIS=(aarch64-linux-android armv7-linux-androideabi x86_64-linux-android)

cd "$ENGINE"

for abi in "${ABIS[@]}"; do
  echo "==> Building $abi..."
  rustup target add "$abi" 2>/dev/null || true
  cargo ndk -t "$abi" -P $API -o "$OUT" build --release --features ffi 2>/dev/null || {
    echo "Install cargo-ndk: cargo install cargo-ndk"
    echo "Or manual: cargo build --release --target $abi --features ffi"
    cargo build --release --target "$abi" --features ffi
    mkdir -p "$OUT/$abi"
    cp "target/$abi/release/libcinemastudio_engine.so" "$OUT/$abi/" 2>/dev/null || true
  }
done

echo "==> Libraries in $OUT"
echo "==> Generate Kotlin bindings: ./scripts/generate_bindings.sh"
