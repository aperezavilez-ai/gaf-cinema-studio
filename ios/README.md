# iOS Shell — CinemaStudio

SwiftUI app wrapping the Rust engine via UniFFI.

## Status (Phase 8)

- SwiftUI sources: Home, Editor, Settings, Guidance, DeviceStatus
- `EngineBridge` + `VideoDecoderService` (AVFoundation scaffold)
- `project.yml` for XcodeGen — generates openable Xcode project on Mac

## Generate Xcode project (Mac)

```bash
brew install xcodegen
cd ios
xcodegen generate
open CinemaStudio.xcodeproj
```

Or: `./scripts/generate_xcode.sh`

## Wire Rust engine

1. Build engine for iOS: `cargo build --release --target aarch64-apple-ios --features ffi`
2. Run `scripts/generate_bindings.sh`
3. Link static lib in Xcode (see `ios/project.yml` comments)
4. Set `EngineBridge.shared.useNativeEngine = true`

## Structure

```
ios/
├── project.yml              XcodeGen spec
├── CinemaStudio/
│   ├── App/                 CinemaStudioApp.swift
│   ├── Design/              Theme.swift
│   ├── Engine/              EngineBridge, ProjectStore, VideoDecoderService
│   └── Features/            Home, Editor, Settings
└── CinemaStudio.xcodeproj   (generated — not committed)
```

## Requirements

- iOS 17+
- Xcode 15+
- Swift 5.9+
