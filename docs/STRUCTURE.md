# CinemaStudio — Monorepo Structure

> Build-first, connect-later. Scaffolds exist before external wiring.

```
/
├── engine/              Rust core — source of truth
├── shared/              JSON schemas + API contracts
├── ios/                 SwiftUI shell
├── android/             Jetpack Compose shell
├── web/                 Vercel static landing
├── api/                 Vercel serverless routes (status live, cloud stubs)
├── supabase/            SQL migrations (run when project created)
├── scripts/             Build, release, verify_connections
├── docs/                Architecture, gates, deployment
└── tests/               Pointer to engine integration tests
```

## Connection layers (connect at end)

| Layer | Scaffold | Wire to |
|-------|----------|---------|
| Engine decode | `media_decoder/stub` | AVFoundation / MediaCodec |
| Engine export | `render_pipeline/stub` | FFmpeg binary |
| Engine FFI | `ffi/capi.rs` | iOS/Android XCFramework |
| Cloud auth | `cloud/providers/local` | Supabase Auth |
| Cloud backup | local dir copy | Supabase Storage + `/api/cloud/backup` |
| Billing | `billing/activate_pro_stub` | Stripe + `/api/webhooks/stripe` |
| Web status | `/api/status` | Vercel env vars |

## Verified endpoints (production)

| URL | Status |
|-----|--------|
| https://gafcinemastudio.com | Custom domain (apex — configure DNS) |
| https://www.gafcinemastudio.com | Custom domain (live) |
| https://gaf-cinema-studio.vercel.app | Vercel default |
| https://gaf-cinema-studio.vercel.app/api/status | Live |
| https://gaf-cinema-studio.vercel.app/api/health | Live |
| `/api/auth/session` | Stub 501 |
| `/api/cloud/backup` | Stub 501 |
| `/api/webhooks/stripe` | Stub 501 |

## Verify locally

```powershell
.\scripts\verify_connections.ps1
```
