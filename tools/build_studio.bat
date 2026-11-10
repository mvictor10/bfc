@echo off
setlocal
cd /d "%~dp0\.."
where cargo >nul 2>nul
if errorlevel 1 (
  echo ERRO: Rust/Cargo nao encontrado. Instale a toolchain Rust pelo rustup.
  pause
  exit /b 1
)
echo Compilando BFC Studio em modo release...
cargo build --release --bin BFCStudio
if errorlevel 1 (
  echo.
  echo A compilacao do BFC Studio falhou. Leia o log acima.
  pause
  exit /b 1
)
if not exist "tools" mkdir "tools"
copy /Y "target\release\BFCStudio.exe" "tools\BFCStudio.exe" >nul
if errorlevel 1 (
  echo ERRO: nao foi possivel copiar o executavel para tools\.
  pause
  exit /b 1
)
echo.
echo BFC Studio gerado em: tools\BFCStudio.exe
echo O dashboard nao precisa de Python.
pause
