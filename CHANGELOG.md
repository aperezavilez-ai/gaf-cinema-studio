# Changelog

## [1.0.0] — MVP Beta — 2026-06-05

First public beta release of CinemaStudio mobile cinematic studio.

### Core
- Rust video engine with project state, SQLite, autosave, recovery
- Timeline + playback + minimal cinematic editing
- Undo/redo (50 snapshots)
- FFmpeg H.264 export (CLI backend)

### AI
- Offline AI orchestrator — contextual suggestions, no chatbot

### Mobile
- iOS SwiftUI app (XcodeGen) with AVFoundation preview
- Android Jetpack Compose app with Gradle project
- C ABI engine link scaffold

### Optional
- Cloud backup stub, Stripe Pro stub, telemetry opt-in
- Beta program tracking (10-user gate)

### Known limitations
- Engine mock mode until `CINEMASTUDIO_ENGINE_LINKED` on device
- MSVC required for local Rust builds on Windows
- Real beta cohort requires TestFlight / Play Internal upload

---

## Pre-release (Phases 0–11)

See `docs/phases/` for phase-by-phase development history.
