@echo off
setlocal
cd /d "%~dp0"
where cargo >nul 2>nul
if errorlevel 1 (
  echo ERRO: Rust/Cargo nao encontrado. Instale o Rust pelo rustup e reabra o terminal.
  pause
  exit /b 1
)
echo Compilando o jogo...
cargo build --release --bin BrazilFightCombat
if errorlevel 1 goto :failed
echo Compilando o dashboard BFC Studio...
cargo build --release --bin BFCStudio
if errorlevel 1 goto :failed
if not exist "tools" mkdir "tools"
copy /Y "target\release\BFCStudio.exe" "tools\BFCStudio.exe" >nul
if errorlevel 1 goto :failed
echo.
echo Compilacao concluida.
echo Jogo: target\release\BrazilFightCombat.exe
echo Dashboard: tools\BFCStudio.exe
echo Execute run.bat para abrir o jogo ou tools\run_studio.bat para abrir o dashboard.
pause
exit /b 0
:failed
echo.
echo A compilacao falhou. Leia as mensagens acima.
pause
exit /b 1
