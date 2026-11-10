@echo off
setlocal
cd /d "%~dp0"
cargo build --release --target x86_64-pc-windows-gnu
if errorlevel 1 exit /b 1
if not exist "tools" mkdir "tools"
copy /Y "target\x86_64-pc-windows-gnu\release\BFCStudio.exe" "tools\BFCStudio.exe" >nul
echo Gerado em target\x86_64-pc-windows-gnu\release\BrazilFightCombat.exe
