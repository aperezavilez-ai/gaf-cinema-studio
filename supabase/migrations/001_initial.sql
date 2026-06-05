-- CinemaStudio — initial schema (run when Supabase project is created)
-- Local-first: mobile keeps SQLite; this stores optional cloud metadata only.

create extension if not exists "pgcrypto";

create table if not exists public.profiles (
  id uuid primary key references auth.users(id) on delete cascade,
  email text,
  display_name text,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table if not exists public.project_backups (
  id uuid primary key default gen_random_uuid(),
  user_id uuid not null references public.profiles(id) on delete cascade,
  project_id uuid not null,
  project_name text not null,
  storage_path text not null,
  size_bytes bigint,
  created_at timestamptz not null default now()
);

create index if not exists idx_project_backups_user on public.project_backups(user_id);

create table if not exists public.beta_events (
  id uuid primary key default gen_random_uuid(),
  user_id uuid references public.profiles(id) on delete set null,
  event_type text not null,
  payload jsonb default '{}',
  created_at timestamptz not null default now()
);

alter table public.profiles enable row level security;
alter table public.project_backups enable row level security;
alter table public.beta_events enable row level security;

create policy "profiles_own" on public.profiles
  for all using (auth.uid() = id);

create policy "backups_own" on public.project_backups
  for all using (auth.uid() = user_id);

create policy "beta_events_insert" on public.beta_events
  for insert with check (auth.uid() = user_id or user_id is null);

create policy "beta_events_select_own" on public.beta_events
  for select using (auth.uid() = user_id);
