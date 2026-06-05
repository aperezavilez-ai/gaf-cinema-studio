# Generate UniFFI bindings for mobile shells.
# Requires: Rust + MSVC (Windows) or clang (macOS), UniFFI via --features ffi

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Engine = Join-Path $Root "engine"
$OutIos = Join-Path $Root "ios\CinemaStudio\Generated"
$OutAndroid = Join-Path $Root "android\CinemaStudio\generated"

Write-Host "Building Rust engine with FFI..."
Push-Location $Engine
cargo build --release --features ffi
if ($LASTEXITCODE -ne 0) { Pop-Location; exit $LASTEXITCODE }

Write-Host "Generating Swift bindings..."
New-Item -ItemType Directory -Force -Path $OutIos | Out-Null
cargo run --release --features ffi --bin uniffi-bindgen -- `
    generate src/ffi/mod.rs --language swift --out-dir $OutIos 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Host "Note: uniffi-bindgen CLI may need: cargo install uniffi-bindgen-cli"
    Write-Host "Manual: uniffi-bindgen generate engine/src/ffi/mod.rs --language swift --out-dir $OutIos"
}

Write-Host "Generating Kotlin bindings..."
New-Item -ItemType Directory -Force -Path $OutAndroid | Out-Null
cargo run --release --features ffi --bin uniffi-bindgen -- `
    generate src/ffi/mod.rs --language kotlin --out-dir $OutAndroid 2>$null

Pop-Location
Write-Host "Done. Set EngineBridge.useNativeEngine = true after linking static lib."
