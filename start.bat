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
echo [oclive-launcher] Tauri 窗口 ^(npm run tauri:dev^)...
echo 需已安装 Rust；控制台保持运行，结束请关闭启动器窗口或按 Ctrl+C。
call npm run tauri:dev
exit /b %errorlevel%

:webdev
echo [oclive-launcher] 仅浏览器 ^(npm run dev，http://127.0.0.1:5174^)...
echo 无 Tauri 时界面无法调用本机能力，仅适合看布局。
call npm run dev
exit /b %errorlevel%

:usage
echo Usage:
echo   start.bat           - 桌面启动器 ^(Tauri，推荐^)
echo   start.bat web       - 仅 Vite 浏览器预览
exit /b 0
