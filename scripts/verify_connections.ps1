#Requires -Version 5.1
$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location (Join-Path $RepoRoot "..")

$VercelUrl = if ($env:CINEMASTUDIO_VERCEL_URL) { $env:CINEMASTUDIO_VERCEL_URL } else { "https://gaf-cinema-studio.vercel.app" }
$CustomUrl = if ($env:CINEMASTUDIO_PUBLIC_URL) { $env:CINEMASTUDIO_PUBLIC_URL } else { "https://gafcinemastudio.com" }
$CustomWwwUrl = "https://www.gafcinemastudio.com"
$GitHubRepo = "aperezavilez-ai/gaf-cinema-studio"

Write-Host ""
Write-Host "=== CinemaStudio connection check ===" -ForegroundColor Cyan

Write-Host ""
Write-Host "[GitHub]" -ForegroundColor Yellow
$remote = git remote get-url origin 2>$null
if ($remote -match "gaf-cinema-studio") {
    Write-Host "  OK  origin -> $remote" -ForegroundColor Green
} else {
    Write-Host "  FAIL origin not linked to gaf-cinema-studio" -ForegroundColor Red
    exit 1
}
$branch = git rev-parse --abbrev-ref HEAD
Write-Host "  OK  branch: $branch"

Write-Host ""
Write-Host "[Vercel]" -ForegroundColor Yellow
Write-Host "  URL: $VercelUrl"
try {
    $status = Invoke-RestMethod -Uri "$VercelUrl/api/status" -Method Get -TimeoutSec 15
    if ($status.connections.github.repo -eq $GitHubRepo) {
        Write-Host "  OK  /api/status - GitHub linked" -ForegroundColor Green
    } else {
        Write-Host "  WARN repo mismatch in status API" -ForegroundColor DarkYellow
    }
    Write-Host "  OK  Vercel deployed (region: $($status.connections.vercel.region))" -ForegroundColor Green
    $sb = $status.connections.supabase.status
    if ($sb -eq "pending") {
        Write-Host "  --  Supabase: pending (expected until env vars set)" -ForegroundColor DarkYellow
    } else {
        Write-Host "  OK  Supabase: $sb" -ForegroundColor Green
    }
}
catch {
    Write-Host "  FAIL cannot reach $VercelUrl/api/status" -ForegroundColor Red
    Write-Host "  $_"
    exit 1
}

try {
    $health = Invoke-RestMethod -Uri "$VercelUrl/api/health" -Method Get -TimeoutSec 10
    Write-Host "  OK  /api/health - v$($health.version)" -ForegroundColor Green
}
catch {
    Write-Host "  WARN /api/health not deployed yet (push latest to main)" -ForegroundColor DarkYellow
}

Write-Host ""
Write-Host "[Custom domain]" -ForegroundColor Yellow
Write-Host "  URL: $CustomUrl"
try {
    $custom = Invoke-WebRequest -Uri "$CustomUrl/api/status" -UseBasicParsing -TimeoutSec 15
    Write-Host "  OK  custom domain live ($($custom.StatusCode))" -ForegroundColor Green
}
catch {
    try {
        $customWww = Invoke-WebRequest -Uri "$CustomWwwUrl/api/status" -UseBasicParsing -TimeoutSec 15
        Write-Host "  OK  www.gafcinemastudio.com live ($($customWww.StatusCode))" -ForegroundColor Green
        Write-Host "  --  apex $CustomUrl not resolving — add A record 76.76.21.21" -ForegroundColor DarkYellow
    }
    catch {
        $code = if ($_.Exception.Response) { [int]$_.Exception.Response.StatusCode } else { "DNS/unreachable" }
        Write-Host "  WARN custom domain not serving yet ($code)" -ForegroundColor DarkYellow
        Write-Host "  Tip: Vercel -> Domains -> verify DNS + redeploy" -ForegroundColor DarkYellow
    }
}

Write-Host ""
Write-Host "=== All core connections OK ===" -ForegroundColor Green
Write-Host "Landing (Vercel): $VercelUrl"
Write-Host "Landing (custom): $CustomUrl"
Write-Host ""
