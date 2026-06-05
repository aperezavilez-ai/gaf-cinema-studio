# CinemaStudio — Integration Guide

> Build-first, connect-later. All modules exist as scaffolds; wire external dependencies at the end.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  SwiftUI / Compose  →  EngineBridge  →  UniFFI (cs_*)       │
├─────────────────────────────────────────────────────────────┤
│  ProjectStateManager (single mutation authority)            │
├─────────────────────────────────────────────────────────────┤
│  native_bridge  ←── callbacks from AVFoundation / MediaCodec│
│  media_decoder  │   trait VideoDecoder                      │
│  render_pipeline│   trait RenderBackendImpl                 │
└─────────────────────────────────────────────────────────────┘
```

## Module map (Phase 7)

| Module | Purpose | Wire when |
|--------|---------|-----------|
| `media_decoder` | Decode frames at playhead | AVFoundation (iOS) / MediaCodec (Android) |
| `render_pipeline` | H.264 export | FFmpeg + `--features ffmpeg` |
| `native_bridge` | Callback registry | Mobile shell registers decode fn |
| `ffi` | Full `cs_*` API | UniFFI bindgen → Swift/Kotlin |

## Step 1 — MSVC + Rust (Windows dev)

Install [Build Tools for Visual Studio](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with **Desktop development with C++**.

```powershell
cd engine
cargo test
```

## Step 2 — UniFFI bindings

```powershell
.\scripts\generate_bindings.ps1
```

Then in Xcode:
1. Link `libcinemastudio_engine.a` (or `.xcframework`)
2. Add generated Swift to target
3. Set `EngineBridge.useNativeEngine = true`

## Step 3 — AVFoundation decode (iOS)

1. Create `VideoDecoderService.swift` using `AVAssetReader`
2. On each frame request from `cs_scrub_to` / playback tick, decode via native service
3. Register callback:

```swift
// Future UniFFI callback interface — until then use cs_set_decoder_backend("avfoundation")
EngineBridge.shared.registerNativeDecoder()
```

Rust side receives frames via `native_bridge::register_decode_callback`.

## Step 4 — MediaCodec decode (Android)

Mirror iOS in `VideoDecoderService.kt` → register with `native_bridge`.

## Step 5 — FFmpeg export

1. Install FFmpeg dev libraries on build machine
2. Implement `render_pipeline/ffmpeg.rs` libav calls (concat + libx264)
3. Build with:

```powershell
$env:CINEMASTUDIO_FFMPEG_LINKED = "1"
cargo build --features ffmpeg
```

4. From app: `cs_set_render_backend("ffmpeg")`

Until linked, export uses **stub backend** (file copy + sidecar JSON).

## Step 6 — Cloud / Stripe / OAuth

Phase 6 stubs in `cloud/`, `billing/`. Replace:
- `login_stub` → OAuth provider SDK
- `CloudBackupService` local dir → S3 presigned URLs
- `activate_pro_stub` → Stripe SDK + webhook server

## FFI API surface

| Function | Description |
|----------|-------------|
| `cs_engine_init(data_root?)` | Boot manager |
| `cs_create_project` / `cs_open_project` / `cs_save_project` | Project lifecycle |
| `cs_import_media` / `cs_add_clip` | Media + timeline |
| `cs_scrub_to` / `cs_playback_*` | Playback (JSON frame DTO) |
| `cs_split_at_playhead` / `cs_undo` / `cs_redo` | Edit |
| `cs_start_export` / `cs_export_status` | Export |
| `cs_ai_*` | AI orchestrator |
| `cs_set_device_hints` / `cs_set_thermal` | Performance |
| `cs_cloud_*` / `cs_activate_pro` | Optional cloud/billing |
| `cs_set_decoder_backend` / `cs_set_render_backend` | Integration hooks |
| `cs_bridge_status` | Diagnostic JSON |

## Verify scaffolds

```powershell
cd engine
cargo test --test integration_phase7
cargo run --example integration_scaffold_demo
```

## Order of wiring (recommended)

1. MSVC → `cargo test` all phases
2. UniFFI → iOS `ProjectStore` via `EngineBridge`
3. AVFoundation decode → smooth preview
4. FFmpeg → real export
5. Cloud OAuth + Stripe (optional)
6. TestFlight beta (10 users)
