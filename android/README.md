# CinemaStudio Android

Jetpack Compose shell — Gradle project openable in Android Studio.

## Status (Phase 8)

- Full Gradle project (`app/` module)
- Compose UI: Home, Editor, Settings
- `EngineBridge` + `VideoDecoderService` (MediaCodec scaffold)
- Navigation with NavHost

## Open in Android Studio

1. Open the `android/` folder (not repo root)
2. Wait for Gradle sync
3. Run on emulator or device (API 26+)

Requires **JDK 17**.

## Wire Rust engine

1. Add UniFFI Kotlin bindings to `app/src/main/java`
2. Build Rust for Android targets (`aarch64-linux-android`, etc.)
3. Set `EngineBridge.useNativeEngine = true`
4. Register MediaCodec decode callback

See [docs/INTEGRATION.md](../docs/INTEGRATION.md).

## Structure

```
android/
├── app/
│   ├── build.gradle.kts
│   └── src/main/java/com/cinemastudio/
│       ├── MainActivity.kt
│       ├── engine/
│       └── ui/
├── settings.gradle.kts
└── build.gradle.kts
```
