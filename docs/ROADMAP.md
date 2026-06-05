# CinemaStudio — Roadmap

> Original MVP: Phases **0–6**. Post-MVP integration: **7–12**.

## Completed

| Phase | Focus |
|-------|--------|
| 0 | Foundation, schemas, architecture |
| 1 | Project state, storage, recovery |
| 2 | Video engine MVP (timeline + playback) |
| 3 | Minimal editing + export stub |
| 4 | AI orchestrator v1 |
| 5 | Device adaptive + performance |
| 6 | Beta + optional cloud |
| 7 | Integration scaffold (FFI, decode, render) |
| 8 | App shell + CI |
| 9 | Native preview pipeline |
| **10** | **C ABI + mobile engine link** ← current |

## Remaining (2 fases to MVP ship)

| Phase | Focus | Gate |
|-------|--------|------|
| **11** | FFmpeg H.264 export + full edit workflow on device | Export 1080p real, workflow end-to-end |
| **12** | TestFlight / Play internal beta + MVP release | 10 beta projects, crash rate < 1% |

## After MVP (out of scope until approved)

- Real cloud OAuth + Stripe production
- UniFFI callback for decode (replace C ABI shim optional)
- Multicam, color grading, plugins — see `OUT_OF_SCOPE_MVP.md`

## Dependency wiring order

1. **Phase 10** — Link `libcinemastudio_engine` (C ABI or UniFFI)
2. **Phase 11** — FFmpeg + `CINEMASTUDIO_FFMPEG_LINKED=1`
3. **Phase 12** — TestFlight, beta cohort, store assets
