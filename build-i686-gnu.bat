@echo off
setlocal
cd /d "%~dp0"
rem Requer rustup target add i686-pc-windows-gnu e linker MinGW de 32 bits.
cargo build --release --target i686-pc-windows-gnu
if errorlevel 1 exit /b 1
if not exist "tools" mkdir "tools"
copy /Y "target\i686-pc-windows-gnu\release\BFCStudio.exe" "tools\BFCStudio.exe" >nul
echo Gerado em target\i686-pc-windows-gnu\release\BrazilFightCombat.exe
