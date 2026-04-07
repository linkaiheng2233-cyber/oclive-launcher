@echo off
setlocal
cd /d "%~dp0"

if not exist "node_modules\" (
  echo [oclive-launcher] Installing npm dependencies...
  call npm install
  if errorlevel 1 (
    echo.
    echo npm install failed.
    pause
    exit /b 1
  )
  echo.
)

if /i "%~1"=="/?" goto :usage
if /i "%~1"=="--help" goto :usage

if /i "%~1"=="web" goto :webdev
if /i "%~1"=="browser" goto :webdev
if /i "%~1"=="vite" goto :webdev

if "%~1"=="" goto :tauridev
if /i "%~1"=="tauri" goto :tauridev

echo Unknown option: %~1
goto :usage

:tauridev
echo [oclive-launcher] Tauri ^(npm run tauri:dev^)...
echo Requires Rust. Leave this window open; close the app or press Ctrl+C to stop.
call npm run tauri:dev
exit /b %errorlevel%

:webdev
echo [oclive-launcher] Browser only ^(npm run dev, http://127.0.0.1:5174^)...
echo Without Tauri, IPC is unavailable. For layout preview only.
call npm run dev
exit /b %errorlevel%

:usage
echo Usage:
echo   start.bat           - Tauri hub ^(recommended^)
echo   start.bat web       - Vite in browser only
exit /b 0
