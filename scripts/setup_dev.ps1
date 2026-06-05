# CinemaStudio dev environment check (Windows)
$ErrorActionPreference = "Continue"

Write-Host "=== CinemaStudio Dev Setup Check ===" -ForegroundColor Cyan

function Test-Command($name) {
    $cmd = Get-Command $name -ErrorAction SilentlyContinue
    if ($cmd) {
        Write-Host "[OK] $name -> $($cmd.Source)" -ForegroundColor Green
        return $true
    }
    Write-Host "[!!] $name not found" -ForegroundColor Yellow
    return $false
}

Test-Command "git" | Out-Null
Test-Command "rustc" | Out-Null
Test-Command "cargo" | Out-Null

$link = Get-Command link.exe -ErrorAction SilentlyContinue
if ($link) {
    Write-Host "[OK] link.exe (MSVC) found" -ForegroundColor Green
} else {
    Write-Host "[!!] link.exe not found - Rust cannot compile on Windows yet" -ForegroundColor Red
    Write-Host ""
    Write-Host "Option A - Install MSVC Build Tools:" -ForegroundColor White
    Write-Host "  https://visualstudio.microsoft.com/visual-cpp-build-tools/" -ForegroundColor Gray
    Write-Host "  Select: Desktop development with C++" -ForegroundColor Gray
    Write-Host ""
    Write-Host "Option B - Use GNU toolchain:" -ForegroundColor White
    Write-Host "  rustup toolchain install stable-x86_64-pc-windows-gnu" -ForegroundColor Gray
    Write-Host "  rustup default stable-x86_64-pc-windows-gnu" -ForegroundColor Gray
    Write-Host ""
    Write-Host "Option C - Push to GitHub; CI runs tests on Ubuntu (engine-ci.yml)" -ForegroundColor White
}

Write-Host ""
Write-Host "Engine tests (local):" -ForegroundColor Cyan
Write-Host "  cd engine; cargo test" -ForegroundColor Gray

Write-Host ""
Write-Host "Android (JDK 17 + Android Studio):" -ForegroundColor Cyan
Write-Host "  Open android/ folder in Android Studio" -ForegroundColor Gray

Write-Host ""
Write-Host "iOS (Mac + XcodeGen):" -ForegroundColor Cyan
Write-Host "  brew install xcodegen" -ForegroundColor Gray
Write-Host "  cd ios; xcodegen generate" -ForegroundColor Gray
Write-Host "  open CinemaStudio.xcodeproj" -ForegroundColor Gray

Write-Host ""
Write-Host "UniFFI bindings:" -ForegroundColor Cyan
Write-Host "  scripts/generate_bindings.ps1" -ForegroundColor Gray
