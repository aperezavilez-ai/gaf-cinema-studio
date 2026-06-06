#Requires -Version 5.1
# Install CinemaStudio debug APK on connected Android phone (Option B).
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Apk = Join-Path $Root "android\app\build\outputs\apk\debug\app-debug.apk"
$Adb = Join-Path $env:LOCALAPPDATA "Android\Sdk\platform-tools\adb.exe"

Write-Host ""
Write-Host "=== CinemaStudio - Install on phone ===" -ForegroundColor Cyan

if (-not (Test-Path $Adb)) {
    Write-Host "adb not found. Install Android SDK platform-tools." -ForegroundColor Red
    exit 1
}

if (-not (Test-Path $Apk)) {
    Write-Host "APK not built yet." -ForegroundColor Yellow
    Write-Host "Build once in Android Studio: open android/ -> Run (green play)." -ForegroundColor Yellow
    Write-Host "Expected: $Apk" -ForegroundColor Gray
    exit 1
}

& $Adb devices
$devices = (& $Adb devices) | Select-String "device$"
if (-not $devices) {
    Write-Host ""
    Write-Host "No phone detected." -ForegroundColor Red
    Write-Host "1. Enable Developer options + USB debugging on phone" -ForegroundColor Yellow
    Write-Host "2. Connect USB and accept trust prompt on phone" -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "Installing..." -ForegroundColor Green
& $Adb install -r $Apk
Write-Host "Launching CinemaStudio..." -ForegroundColor Green
& $Adb shell am start -n com.cinemastudio/.MainActivity
Write-Host "Done." -ForegroundColor Green
