# CinemaStudio

Mobile-first cinematic editing studio. Local-first architecture with a native video engine core.

## Principles

- **MVP first** — stability over feature count
- **Performance first** — mobile GPU, RAM, battery, thermal budgets
- **Project State is the source of truth** — not UI history
- **AI as orchestrator** — not a chatbot
- **Local-first** — cloud is optional, never critical for core

## Monorepo Structure

```
/engine          Rust video core + project state
/shared          Schemas, contracts, shared types
/ios             SwiftUI shell (Phase 1+)
/android         Compose shell (Phase 1+)
/docs            Architecture, gates, decisions
/tests           Cross-module integration tests
```

## Development Phases

| Phase | Focus | Status |
|-------|-------|--------|
| 0 | Foundation, schemas, architecture | Complete |
| 1 | Project State + Storage + Recovery | Complete |
| 2 | Video Engine MVP (playback + timeline) | Complete |
| 3 | Minimal cinematic editing + export | Complete |
| 4 | AI Orchestrator v1 | Complete |
| 5 | Device adaptive + performance hardening | Complete |
| 6 | Beta + optional cloud sync | Complete |
| 7 | Integration scaffold (FFI, decode, render) | Complete |
| 8 | App shell + CI (XcodeGen, Gradle, GitHub Actions) | Complete |
| 9 | Native preview pipeline (AVFoundation / MediaMetadataRetriever) | Complete |

See [docs/PHASE_GATES.md](docs/PHASE_GATES.md) for gate criteria.

## Quick Start (Engine)

```bash
cd engine
cargo test          # requires MSVC on Windows, or use GitHub CI
cargo run --example create_project
```

Windows dev check:

```powershell
.\scripts\setup_dev.ps1
```

## Quick Start (Mobile)

- **iOS (Mac):** `cd ios && xcodegen generate && open CinemaStudio.xcodeproj`
- **Android:** Open `android/` in Android Studio

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [Integration guide](docs/INTEGRATION.md)
- [Project State Schema](docs/PROJECT_STATE.schema.json)
- [Phase Gates](docs/PHASE_GATES.md)
- [Out of Scope MVP](docs/OUT_OF_SCOPE_MVP.md)
- [Event Bus](docs/EVENT_BUS.md)

## Tech Stack (frozen for MVP)

| Layer | Technology |
|-------|------------|
| Engine core | Rust |
| iOS shell | Swift + SwiftUI |
| Android shell | Kotlin + Compose |
| Project DB | SQLite (rusqlite) |
| Media processing | FFmpeg (Phase 2+) |
| AI (optional) | Cloud API, non-blocking |
