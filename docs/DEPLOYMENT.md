# CinemaStudio — Deployment & Infrastructure

> Post-MVP infrastructure. Core app remains **local-first**; cloud is optional.

## Connection status

| Service | Status | Notes |
|---------|--------|-------|
| **GitHub** | Linked | `aperezavilez-ai/gaf-cinema-studio` — CI on push |
| **Vercel** | Deployed | Static landing + `/api/status` |
| **Supabase** | Pending | Auth, backup metadata, beta telemetry (Phase 13+) |

---

## Vercel

**Root:** repo root (uses `vercel.json` → `web/` static output + `/api` serverless).

| Path | Purpose |
|------|---------|
| `/` | Landing page |
| `/api/status` | JSON — GitHub/Vercel/Supabase connection state |

### Vercel project settings

- **Framework preset:** Other
- **Root directory:** `.` (repository root)
- **Build command:** (empty — static)
- **Output directory:** `web` (via `vercel.json`)

Redeploy after push to `main`.

---

## GitHub

- **Remote:** `https://github.com/aperezavilez-ai/gaf-cinema-studio`
- **CI:** `.github/workflows/engine-ci.yml` — Rust tests phases 1–12 on Ubuntu
- **Vercel integration:** auto-deploy on `main` (confirm in Vercel dashboard → Git)

---

## Supabase (pending)

When ready, create a Supabase project and add env vars in **Vercel → Settings → Environment Variables**:

| Variable | Scope | Purpose |
|----------|-------|---------|
| `SUPABASE_URL` | Production | Project API URL |
| `SUPABASE_ANON_KEY` | Production | Public anon key (mobile + web) |
| `SUPABASE_SERVICE_ROLE_KEY` | Production only | Server-side backup/webhooks — **never** in mobile apps |

Optional for local dev — copy `docs/env.template` to `.env` (gitignored).

### Planned tables (not migrated yet)

See `supabase/migrations/001_initial.sql` — profiles, project_backups, beta_events.

### Wiring order (when Supabase is live)

1. Run migration in Supabase SQL editor
2. Add env vars to Vercel → redeploy → `/api/status` shows `supabase: configured`
3. Replace `engine/src/cloud/auth.rs` `login_stub` with Supabase JWT validation
4. Replace local backup dir with Storage bucket + presigned URLs
5. Mobile: Supabase Auth SDK (optional login in Settings)

Until then, **all core features work offline** — no blocker for TestFlight/Play beta.

---

## Verify

```powershell
# Local (optional — needs Vercel CLI)
npx vercel dev

# Production
curl https://YOUR-VERCEL-URL.vercel.app/api/status
```

Expected Supabase field while pending:

```json
"supabase": { "status": "pending", "note": "Add SUPABASE_URL + ..." }
```
