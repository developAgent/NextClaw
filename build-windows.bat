@echo off
REM Windows Build Script for CEOClaw
echo ========================================
echo   CEOClaw Windows Build Script
echo ========================================
echo.

REM Check if Rust is installed
cargo --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Rust is not installed!
    echo Please install Rust from: https://rustup.rs/
    pause
    exit /b 1
)

echo [1/4] Checking dependencies...
npm --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Node.js is not installed!
    pause
    exit /b 1
)
echo OK - Node.js and Rust found

echo.
echo [2/4] Installing dependencies...
call npm install
if %errorlevel% neq 0 (
    echo ERROR: Failed to install dependencies
    pause
    exit /b 1
)
echo OK - Dependencies installed

echo.
echo [3/4] Building CEOClaw...
call npm run tauri:build
if %errorlevel% neq 0 (
    echo ERROR: Build failed
    pause
    exit /b 1
)
echo OK - Build successful

echo.
echo [4/4] Build artifacts location:
echo   Installer: src-tauri\target\release\bundle\nsis\
echo   EXE: src-tauri\target\release\bundle\nsis\CEOClaw_*.exe
echo.
echo ========================================
echo   Build completed successfully!
echo ========================================
pause