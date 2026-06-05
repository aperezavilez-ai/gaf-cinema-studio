#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERCEL_URL="${CINEMASTUDIO_VERCEL_URL:-https://gaf-cinema-studio.vercel.app}"

echo "=== CinemaStudio connection check ==="
git -C "$ROOT" remote get-url origin | grep -q gaf-cinema-studio && echo "[GitHub] OK"
curl -sf "$VERCEL_URL/api/status" | grep -q gaf-cinema-studio && echo "[Vercel] OK — $VERCEL_URL"
curl -sf "$VERCEL_URL/api/health" | grep -q cinemastudio && echo "[Health] OK"
echo "=== Done ==="
