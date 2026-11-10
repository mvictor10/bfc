@echo off
setlocal
cd /d "%~dp0"
cargo test
if errorlevel 1 exit /b 1
echo Todos os testes Cargo terminaram sem erro.
