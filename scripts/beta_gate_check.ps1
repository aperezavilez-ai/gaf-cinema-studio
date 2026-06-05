# Beta gate validation — run before TestFlight / Play upload
$ErrorActionPreference = "Continue"

Write-Host "=== CinemaStudio MVP Beta Gate Check ===" -ForegroundColor Cyan

$engine = Join-Path $PSScriptRoot "..\engine"
Push-Location $engine

Write-Host "`nRunning Phase 12 integration tests..." -ForegroundColor White
cargo test --test integration_phase12 2>&1
$testOk = $LASTEXITCODE -eq 0

Pop-Location

Write-Host "`n--- Manual checklist ---" -ForegroundColor Cyan
@(
    "[ ] 10 beta testers completed a real project",
    "[ ] Crash rate < 1% (telemetry opt-in)",
    "[ ] TestFlight / Play Internal build uploaded",
    "[ ] Privacy policy URL live",
    "[ ] App Store / Play metadata filled (docs/store/)"
) | ForEach-Object { Write-Host $_ }

if ($testOk) {
    Write-Host "`nAutomated gates: PASS" -ForegroundColor Green
} else {
    Write-Host "`nAutomated gates: FAIL (install MSVC or rely on GitHub CI)" -ForegroundColor Yellow
}

Write-Host "`nSee docs/BETA_RELEASE.md for upload steps."
