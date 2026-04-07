# CEOClaw Build Script for Windows
# PowerShell version

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  CEOClaw Windows Build Script" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check Rust
Write-Host "[1/5] Checking Rust installation..." -ForegroundColor Yellow
try {
    $rustVersion = cargo --version 2>$null
    if ($rustVersion) {
        Write-Host "✓ $rustVersion" -ForegroundColor Green
    } else {
        throw "Rust not found"
    }
} catch {
    Write-Host "✗ ERROR: Rust is not installed!" -ForegroundColor Red
    Write-Host "  Please install Rust from: https://rustup.rs/" -ForegroundColor White
    Read-Host "Press Enter to exit"
    exit 1
}

# Check Node.js
Write-Host "[2/5] Checking Node.js installation..." -ForegroundColor Yellow
try {
    $nodeVersion = node --version 2>$null
    if ($nodeVersion) {
        Write-Host "✓ Node.js $nodeVersion" -ForegroundColor Green
    } else {
        throw "Node.js not found"
    }
} catch {
    Write-Host "✗ ERROR: Node.js is not installed!" -ForegroundColor Red
    Write-Host "  Please install Node.js from: https://nodejs.org/" -ForegroundColor White
    Read-Host "Press Enter to exit"
    exit 1
}

# Change to project directory
$projectDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $projectDir
Write-Host "📂 Working directory: $projectDir" -ForegroundColor Cyan
Write-Host ""

# Install dependencies
Write-Host "[3/5] Installing dependencies..." -ForegroundColor Yellow
npm install
if ($LASTEXITCODE -ne 0) {
    Write-Host "✗ ERROR: Failed to install dependencies" -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit 1
}
Write-Host "✓ Dependencies installed" -ForegroundColor Green
Write-Host ""

# Build
Write-Host "[4/5] Building CEOClaw..." -ForegroundColor Yellow
Write-Host "  This may take several minutes..." -ForegroundColor Cyan
npm run tauri:build
if ($LASTEXITCODE -ne 0) {
    Write-Host "✗ ERROR: Build failed" -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit 1
}
Write-Host "✓ Build successful" -ForegroundColor Green
Write-Host ""

# Show results
Write-Host "[5/5] Build artifacts:" -ForegroundColor Yellow
$bundleDir = "src-tauri\target\release\bundle"
if (Test-Path "$bundleDir\nsis") {
    $installer = Get-ChildItem "$bundleDir\nsis\*.exe" | Select-Object -First 1
    if ($installer) {
        Write-Host "  Installer: $($installer.FullName)" -ForegroundColor Cyan
        $size = [math]::Round($installer.Length / 1MB, 2)
        Write-Host "  Size: $size MB" -ForegroundColor Gray
    }
}

if (Test-Path "src-tauri\target\release\ceo-claw.exe") {
    Write-Host "  Portable EXE: src-tauri\target\release\ceo-claw.exe" -ForegroundColor Cyan
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "  Build completed successfully!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Read-Host "Press Enter to exit"