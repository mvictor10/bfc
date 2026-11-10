@echo off
setlocal
cd /d "%~dp0"
cargo build --release --target x86_64-pc-windows-msvc
if errorlevel 1 exit /b 1
if not exist "tools" mkdir "tools"
copy /Y "target\x86_64-pc-windows-msvc\release\BFCStudio.exe" "tools\BFCStudio.exe" >nul
echo Jogo e BFC Studio gerados para x64 MSVC.
