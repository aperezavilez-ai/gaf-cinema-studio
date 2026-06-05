# iOS Shell — CinemaStudio

> Phase 1+: SwiftUI app that wraps the Rust engine via FFI.

## Status

Phase 0: **Placeholder only**. No Xcode project yet.

## Planned stack

- Swift 5.9+
- SwiftUI
- UniFFI or C ABI bridge to `cinemastudio-engine`
- AVFoundation for media capture/import (Phase 1)
- Metal for GPU preview (Phase 2)

## Phase 1 deliverables

- [ ] Xcode project setup
- [ ] FFI bindings to Rust engine
- [ ] Home screen: New Project / Open Project
- [ ] Project browser (local `.csproj` folders)
- [ ] Wire create/open/save to `ProjectStateManager`

## Directory plan (Phase 1)

```
ios/
├── CinemaStudio/
│   ├── App/
│   ├── Features/
│   │   ├── Home/
│   │   └── Project/
│   ├── Engine/          # FFI wrapper
│   └── Design/          # UI components
└── CinemaStudio.xcodeproj
```

## Build note

Engine must be compiled as `staticlib` for iOS targets before linking.
See `docs/ARCHITECTURE.md` FFI section.
