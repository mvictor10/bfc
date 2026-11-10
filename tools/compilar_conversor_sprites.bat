@echo off
setlocal

REM Compila o binario a partir da raiz do projeto.
cd /d "%~dp0\.."

where cargo >nul 2>nul
if errorlevel 1 (
    echo [ERRO] Cargo nao foi encontrado. Instale o Rust antes de compilar.
    exit /b 1
)

echo ================================================
echo Compilando BFC Sprite PNG Converter
echo ================================================

cargo build --release --bin converter_sprites
if errorlevel 1 (
    echo.
    echo [ERRO] A compilacao falhou.
    exit /b 1
)

if not exist "tools" mkdir "tools"
copy /Y "target\release\converter_sprites.exe" "tools\converter_sprites.exe" >nul
if errorlevel 1 (
    echo [ERRO] Nao foi possivel copiar o executavel para tools.
    exit /b 1
)

echo.
echo [OK] Executavel criado em tools\converter_sprites.exe
echo Execute esse arquivo sempre que quiser validar os sprites.
echo.
endlocal
