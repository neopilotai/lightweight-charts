@echo off
REM Trading Dashboard Quick Setup Script for Windows

setlocal enabledelayedexpansion

echo.
echo [34m========================================[0m
echo [34m  Trading Dashboard Setup[0m
echo [34m========================================[0m
echo.

REM Check if Node.js is installed
where node >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo [33m(!) Node.js not found. Please install Node.js 16+[0m
    exit /b 1
)

for /f "tokens=*" %%i in ('node --version') do set NODE_VERSION=%%i
echo [32m(✓) Node.js found: %NODE_VERSION%[0m
echo.

REM Install dependencies
echo [34m(+) Installing dependencies...[0m
call npm install

echo.
echo [32m========================================[0m
echo [32m  Setup Complete![0m
echo [32m========================================[0m
echo.
echo [32m(✓) To start the development server, run:[0m
echo    npm run dev
echo.
echo [32m(✓) To build for production, run:[0m
echo    npm run build
echo.
echo [33m(!!) Make sure your Rust backend is running on port 3000:[0m
echo    cd ..\backend
echo    cargo run --release
echo.
echo [32m(+) Dashboard will be available at: http://localhost:5173[0m
echo.
