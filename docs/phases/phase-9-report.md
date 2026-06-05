# Phase 9 Report — Native Preview Pipeline

**Date:** 2026-06-05  
**Status:** Complete

---

## Deliverables

| Item | Status |
|------|--------|
| iOS `VideoDecoderService` | AVFoundation frame decode + cache |
| iOS `PreviewViewModel` + `PreviewFrameView` | Live preview in editor |
| iOS `ProjectDocumentPicker` | Open `.csproj` folders |
| `ProjectStore` wired | Playback timer, scrub → decode, export stub |
| `EngineBridge` expanded | Full mock API surface |
| Android `EditorViewModel` | Playback + preview state |
| Android `VideoDecoderService` | MediaMetadataRetriever frames |
| `scripts/build_ios.sh` | Cross-compile + XCFramework scaffold |
| `scripts/build_android.sh` | NDK multi-ABI build scaffold |

---

## Gate 9 checklist

| # | Criterion | Status |
|---|-----------|--------|
| 9.1 | iOS AVFoundation preview at playhead | PASS (Swift) |
| 9.2 | Android native frame preview | PASS (Retriever) |
| 9.2 | Editor wired to preview pipeline | PASS |
| 9.3 | Mobile engine build scripts | PASS (scaffold) |
| 9.4 | Document picker (iOS) | PASS |

---

## Preview flow

```
Scrub / Play tick
    → ProjectStore.playheadMs
    → VideoDecoderService.decodeFrame(path, timeMs)
    → PreviewFrameView (UIImage / Bitmap)
```

Rust engine decode (`native_bridge`) connects when UniFFI is linked — native preview works independently for UI dev.

---

## Build engine for mobile (Mac / NDK)

```bash
./scripts/build_ios.sh      # requires Xcode + Rust iOS targets
./scripts/build_android.sh  # requires ANDROID_NDK_HOME + cargo-ndk
```

---

## STOP

Next: link UniFFI on device, replace mock EngineBridge with `cs_*`, TestFlight beta.
