@echo off
setlocal
cd /d "%~dp0"
if not exist "target\release\BrazilFightCombat.exe" call build.bat
if not exist "target\release\BrazilFightCombat.exe" exit /b 1
start "" /D "%~dp0" "%~dp0target\release\BrazilFightCombat.exe"
