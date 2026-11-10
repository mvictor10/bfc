@echo off
setlocal
cd /d "%~dp0"
title BFC Game Studio - Flask Editor

where py >nul 2>nul
if not errorlevel 1 (
    set "PYTHON_LAUNCHER=py -3"
) else (
    where python >nul 2>nul
    if errorlevel 1 (
        echo [ERRO] Python 3 nao foi encontrado.
        echo Instale Python 3.10 ou superior e marque a opcao Add Python to PATH.
        pause
        exit /b 1
    )
    set "PYTHON_LAUNCHER=python"
)

if not exist ".venv\Scripts\python.exe" (
    echo [1/3] Criando ambiente virtual...
    %PYTHON_LAUNCHER% -m venv .venv
    if errorlevel 1 goto :error
)

echo [2/3] Verificando dependencias Flask e Pillow...
".venv\Scripts\python.exe" -m pip install --disable-pip-version-check -r requirements.txt
if errorlevel 1 goto :error

echo [3/3] Iniciando BFC Game Studio...
echo.
echo Abra: http://127.0.0.1:8765
echo Mantenha esta janela aberta enquanto utilizar o editor.
start "" "http://127.0.0.1:8765"
".venv\Scripts\python.exe" app.py

echo.
echo Servidor encerrado.
pause
exit /b 0

:error
echo.
echo [ERRO] Nao foi possivel preparar o ambiente Python.
echo Verifique a conexao com a internet e a instalacao do Python.
pause
exit /b 1
