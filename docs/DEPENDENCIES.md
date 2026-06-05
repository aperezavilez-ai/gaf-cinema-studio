# Approved Dependencies

> New dependencies require entry here before adding to Cargo.toml or native projects.

## Rust Engine (Phase 0)

| Crate | Version | Purpose | Approved |
|-------|---------|---------|----------|
| serde | 1.x | Serialization | Phase 0 |
| serde_json | 1.x | JSON project files | Phase 0 |
| uuid | 1.x | IDs | Phase 0 |
| chrono | 0.4 | Timestamps | Phase 0 |
| thiserror | 2.x | Error types | Phase 0 |
| rusqlite | 0.32 | SQLite persistence | Phase 1 |
| sha2 | 0.10 | State checksums | Phase 1 |
| mime_guess | 2.x | MIME detection on import | Phase 1 |
| tempfile | 3.x | Test fixtures | Phase 0 (dev) |
| uniffi | 0.28 | Mobile FFI (optional) | Phase 1 |

## Planned (not yet added)

| Dependency | Purpose | Phase |
|------------|---------|-------|
| rusqlite | SQLite persistence | 1 |
| uniffi | Mobile FFI bindings | 1 |
| ffmpeg-next | Media processing | 2 |
