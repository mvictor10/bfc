@echo off
setlocal EnableExtensions
cd /d "%~dp0"
title BFC Sprite Organizer

REM Raiz do projeto: tools\BFC_Sprite_Organizer -> dois níveis acima.
for %%I in ("%~dp0..\..") do set "BFC_ROOT=%%~fI"

if not exist "%BFC_ROOT%\chars\" (
    echo [ERRO] A pasta chars nao foi encontrada em:
    echo        %BFC_ROOT%
    echo Extraia esta pasta dentro de bfc\tools\BFC_Sprite_Organizer.
    pause
    exit /b 1
)

REM Reutiliza um ambiente virtual existente. Nao cria uma nova .venv.
set "VENV_PY="

if defined BFC_VENV_PY if exist "%BFC_VENV_PY%" set "VENV_PY=%BFC_VENV_PY%"
if not defined VENV_PY if defined VIRTUAL_ENV if exist "%VIRTUAL_ENV%\Scripts\python.exe" set "VENV_PY=%VIRTUAL_ENV%\Scripts\python.exe"

REM Ambiente criado pelo BFC AI Sprite Lab.
if not defined VENV_PY if exist "%~dp0..\BFC_AI_Sprite_Lab\.venv\Scripts\python.exe" set "VENV_PY=%~dp0..\BFC_AI_Sprite_Lab\.venv\Scripts\python.exe"

REM Outros locais comuns de .venv no projeto.
if not defined VENV_PY if exist "%~dp0..\.venv\Scripts\python.exe" set "VENV_PY=%~dp0..\.venv\Scripts\python.exe"
if not defined VENV_PY if exist "%BFC_ROOT%\.venv\Scripts\python.exe" set "VENV_PY=%BFC_ROOT%\.venv\Scripts\python.exe"
if not defined VENV_PY if exist "%~dp0..\BFC_Game_Studio\.venv\Scripts\python.exe" set "VENV_PY=%~dp0..\BFC_Game_Studio\.venv\Scripts\python.exe"

if not defined VENV_PY (
    echo [ERRO] Nao encontrei um ambiente virtual existente.
    echo.
    echo Procurei em:
    echo   tools\BFC_AI_Sprite_Lab\.venv
    echo   tools\.venv
    echo   .venv na raiz do projeto
    echo.
    echo Execute primeiro o inicializador que criou a .venv ou defina BFC_VENV_PY.
    pause
    exit /b 1
)

echo [BFC] Ambiente virtual selecionado:
echo       %VENV_PY%
"%VENV_PY%" --version
if errorlevel 1 goto :erro

REM Confere se o ambiente existente usa Python 3.11 ou superior.
"%VENV_PY%" -c "import sys; raise SystemExit(0 if sys.version_info >= (3,11) else 1)"
if errorlevel 1 (
    echo [ERRO] O ambiente virtual precisa usar Python 3.11 ou superior.
    goto :erro
)

REM Instala/atualiza somente a dependencia declarada neste pacote.
echo.
echo [SETUP] Verificando Pillow...
"%VENV_PY%" -m pip install --disable-pip-version-check -r "%~dp0requirements.txt"
if errorlevel 1 goto :erro

echo.
echo [BFC] Iniciando organizador de sprites...
echo.
"%VENV_PY%" "%~dp0organizar_sprites.py" --root "%BFC_ROOT%" %*
set "APP_EXIT=%ERRORLEVEL%"

echo.
if "%APP_EXIT%"=="0" (
    echo [OK] Operacao finalizada.
) else (
    echo [AVISO] O programa terminou com codigo %APP_EXIT%.
)
pause
exit /b %APP_EXIT%

:erro
echo.
echo [ERRO] Nao foi possivel preparar o ambiente ou executar o organizador.
pause
exit /b 1
