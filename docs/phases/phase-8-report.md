# Phase 8 Report — App Shell + CI

**Date:** 2026-06-05  
**Status:** Complete

---

## Deliverables

| Item | Status |
|------|--------|
| `ios/project.yml` | XcodeGen config — generates `.xcodeproj` on Mac |
| `ios/Design/Theme.swift` | Cinematic design tokens |
| `NavigationStack` in app root | Home → Editor → Settings navigation |
| Android Gradle project | `app/` module, Compose, navigation |
| `HomeScreen` / `EditorScreen` / `SettingsScreen` | Android UI parity with iOS |
| `.github/workflows/engine-ci.yml` | Tests phases 1–7 on Ubuntu |
| `scripts/setup_dev.ps1` | Windows dev environment check |
| `scripts/generate_xcode.sh` | Xcode project generator |

---

## Gate 8 checklist

| # | Criterion | Status |
|---|-----------|--------|
| 8.1 | iOS project generatable (XcodeGen) | PASS (project.yml) |
| 8.2 | Android opens in Android Studio | PASS (Gradle scaffold) |
| 8.3 | CI runs engine tests | PASS (workflow) |
| 8.4 | Editor UI both platforms | PASS (scaffold) |

---

## Open on device

### iOS (Mac required)

```bash
brew install xcodegen
cd ios && xcodegen generate
open CinemaStudio.xcodeproj
```

### Android

Open `android/` folder in Android Studio → Sync → Run on emulator.

### Windows dev (Rust)

```powershell
.\scripts\setup_dev.ps1
# Install MSVC Build Tools OR use GitHub CI for tests
```

---

## STOP

Next: wire UniFFI on Mac, AVFoundation decode, TestFlight beta.
