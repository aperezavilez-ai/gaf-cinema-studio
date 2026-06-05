# Phase 7 Report — Integration Scaffold

**Date:** 2026-06-05  
**Status:** Complete — wire dependencies at integration time

---

## Philosophy

Build all layers first with traits + stubs. Connect AVFoundation, MediaCodec, FFmpeg, UniFFI, and cloud APIs **at the end** without blocking development.

---

## Deliverables

| Module | Status |
|--------|--------|
| `media_decoder` | `VideoDecoder` trait + `StubDecoder` + `DecoderRegistry` |
| `render_pipeline` | `RenderBackendImpl` + stub + FFmpeg scaffold (`feature ffmpeg`) |
| `native_bridge` | Callback slots, backend selection, `decode_frame_at` |
| `export` | Refactored to use `render_pipeline` via `native_bridge` |
| `ffi` | Full `cs_*` API (~40 functions), JSON DTOs |
| `manager` | `decode_at_playhead()` |
| iOS | `EngineBridge.swift` — mock/native toggle |
| Android | `MainActivity.kt` + `EngineBridge.kt` |
| Scripts | `generate_bindings.ps1` / `.sh` |
| Docs | `docs/INTEGRATION.md` |

---

## API additions

```rust
// Decode at playhead (stub until native wired)
manager.decode_at_playhead()?;

// Bridge control
set_decoder_backend(DecoderBackend::AvFoundation);
set_render_backend(RenderBackend::Ffmpeg);
bridge_status(); // JSON diagnostic
decode_frame_at(path, time_ms, w, h);
```

---

## Integration hooks (FFI)

```text
cs_bridge_status()
cs_set_decoder_backend("avfoundation" | "mediacodec" | "stub")
cs_set_render_backend("ffmpeg" | "stub")
```

---

## Tests

```powershell
cargo test --test integration_phase7
cargo run --example integration_scaffold_demo
```

| Test | Validates |
|------|-----------|
| `phase7_decoder_stub` | Stub decode metadata |
| `phase7_render_pipeline_via_export` | Export uses pipeline |
| `phase7_decode_at_playhead` | Manager + bridge |
| `phase7_bridge_status_json` | Backend selection |
| `phase7_render_pipeline_direct` | Pipeline unit path |

---

## Wiring checklist (when ready)

| # | Task | Doc section |
|---|------|-------------|
| 1 | Install MSVC, run full test suite | INTEGRATION §1 |
| 2 | `generate_bindings.ps1` → link in Xcode | §2 |
| 3 | AVFoundation → `register_decode_callback` | §3 |
| 4 | MediaCodec (Android) | §4 |
| 5 | FFmpeg + `CINEMASTUDIO_FFMPEG_LINKED=1` | §5 |
| 6 | Cloud OAuth + Stripe production | §6 |

---

## Risks

| Risk | Mitigation |
|------|------------|
| UniFFI callbacks for decode not yet in UDL | Phase 8: UniFFI callback trait or C ABI shim |
| FFmpeg not in crate yet | Scaffold returns stub fallback with log |
| Android Gradle project not full | Compose scaffold only; Gradle wrapper later |
| `EngineBridge.useNativeEngine = false` | UI dev continues with mocks |

---

## STOP

Integration scaffold complete. Next: install MSVC → run tests → generate bindings → wire AVFoundation first.
