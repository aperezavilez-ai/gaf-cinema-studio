# Supabase — Pending

CinemaStudio MVP does **not** require Supabase. This folder holds the future schema.

## When you create the project

1. [supabase.com](https://supabase.com) → New project
2. SQL Editor → run `migrations/001_initial.sql`
3. Storage → create bucket `project-backups` (private)
4. Auth → enable Email (or OAuth providers later)
5. Copy URL + anon key → Vercel env vars (see `docs/DEPLOYMENT.md`)

## Scope (post-MVP)

- Optional account login (core stays guest/local-first)
- Cloud backup metadata + Storage paths
- Beta telemetry events (crash-free sessions)

Do **not** store raw project SQLite or media in Postgres — use Storage only.
