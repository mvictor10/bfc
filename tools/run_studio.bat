@echo off
setlocal
cd /d "%~dp0\.."
if not exist "tools\BFCStudio.exe" call "tools\build_studio.bat"
if not exist "tools\BFCStudio.exe" (
  echo ERRO: BFCStudio.exe nao foi gerado. Confira a compilacao acima.
  pause
  exit /b 1
)
start "" /D "%~dp0\.." "%~dp0BFCStudio.exe"
