# AirShare Build Script
# Builds Go Engine for Windows + Android and packages Tauri app

$ErrorActionPreference = "Stop"

Write-Host "🚀 AirShare Build Script" -ForegroundColor Cyan
Write-Host "========================" -ForegroundColor Cyan

# Check Go is installed
if (-not (Get-Command "go" -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Go is not installed!" -ForegroundColor Red
    exit 1
}

# Navigate to engine directory
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RootDir = Split-Path -Parent $ScriptDir
$EngineDir = Join-Path $RootDir "engine"
$BinariesDir = Join-Path $RootDir "clients\desktop\src-tauri\binaries"
$DesktopDir = Join-Path $RootDir "clients\desktop"

Write-Host ""
Write-Host "📁 Engine Dir: $EngineDir" -ForegroundColor Gray
Write-Host "📁 Binaries Dir: $BinariesDir" -ForegroundColor Gray

# Ensure binaries directory exists
New-Item -ItemType Directory -Force -Path $BinariesDir | Out-Null

# === Build Go Engine ===
Write-Host ""
Write-Host "🔨 Building Go Engine..." -ForegroundColor Yellow

Set-Location $EngineDir

# Windows (x86_64)
Write-Host "  → Windows (x86_64)..." -ForegroundColor White
$env:GOOS = "windows"
$env:GOARCH = "amd64"
$env:CGO_ENABLED = "0"
go build -ldflags="-s -w" -o "$BinariesDir\airshare-engine-x86_64-pc-windows-msvc.exe" main.go
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Windows build failed!" -ForegroundColor Red
    exit 1
}
Write-Host "  ✅ Windows build complete" -ForegroundColor Green

# Android ARM64 (if cross-compile is available)
Write-Host "  → Android (arm64)..." -ForegroundColor White
$env:GOOS = "android"
$env:GOARCH = "arm64"
$env:CGO_ENABLED = "0"
try {
    go build -ldflags="-s -w" -o "$BinariesDir\airshare-engine-aarch64-linux-android" main.go
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  ✅ Android ARM64 build complete" -ForegroundColor Green
    } else {
        Write-Host "  ⚠️ Android ARM64 build skipped (may need NDK)" -ForegroundColor Yellow
    }
} catch {
    Write-Host "  ⚠️ Android build skipped: $_" -ForegroundColor Yellow
}

# List built binaries
Write-Host ""
Write-Host "📦 Built Binaries:" -ForegroundColor Yellow
Get-ChildItem $BinariesDir | ForEach-Object {
    $size = [math]::Round($_.Length / 1MB, 2)
    Write-Host "  - $($_.Name) ($size MB)" -ForegroundColor Gray
}

# === Build Tauri App ===
Write-Host ""
Write-Host "🔧 Building Tauri App..." -ForegroundColor Yellow

Set-Location $DesktopDir

# Reset Go env
$env:GOOS = "windows"
$env:GOARCH = "amd64"

# Check if we should build
$Build = Read-Host "Build Windows EXE now? (y/n)"
if ($Build -eq "y") {
    Write-Host "  → Running npm run tauri build..." -ForegroundColor White
    npm run tauri build
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host ""
        Write-Host "🎉 BUILD COMPLETE!" -ForegroundColor Green
        Write-Host ""
        Write-Host "📍 Windows Installer:" -ForegroundColor Cyan
        Get-ChildItem "$DesktopDir\src-tauri\target\release\bundle\msi" -ErrorAction SilentlyContinue | ForEach-Object {
            Write-Host "   $($_.FullName)" -ForegroundColor White
        }
        Get-ChildItem "$DesktopDir\src-tauri\target\release\bundle\nsis" -ErrorAction SilentlyContinue | ForEach-Object {
            Write-Host "   $($_.FullName)" -ForegroundColor White
        }
    } else {
        Write-Host "❌ Tauri build failed!" -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "  ⏭️ Skipping Tauri build" -ForegroundColor Gray
}

Write-Host ""
Write-Host "✨ Done!" -ForegroundColor Cyan
