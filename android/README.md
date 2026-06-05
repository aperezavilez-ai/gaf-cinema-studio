# CinemaStudio Android

Jetpack Compose shell — engine via UniFFI/JNI (wire at integration).

## Structure

```
android/CinemaStudio/
├── MainActivity.kt          Compose home (scaffold)
└── engine/
    └── EngineBridge.kt      Kotlin → Rust FFI facade
```

## Integration checklist

1. Install Android NDK + Rust targets (`aarch64-linux-android`, etc.)
2. Run `scripts/generate_bindings.sh android`
3. Set `EngineBridge.useNativeEngine = true`
4. Wire `MediaCodec` decode callback → Rust `native_bridge`

See [docs/INTEGRATION.md](../docs/INTEGRATION.md).
