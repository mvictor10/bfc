@echo off
chcp 65001 >nul

echo ====================================================
echo       ORGANIZADOR DE NOMES POR DIMENSÃO - BFC
echo ====================================================

set "PASTA_ATUAL=%~dp0"
cd /d "%PASTA_ATUAL%"

if not exist ".venv\Scripts\activate.bat" (
    echo [ERRO] Ambiente virtual ^(.venv^) não encontrado nesta pasta!
    goto FIM
)

echo [1/2] Ativando ambiente virtual (.venv)...
call ".venv\Scripts\activate.bat"

echo [2/2] Executando o script de organização por resolução...
echo ----------------------------------------------------
python organizar_dimensoes.py
echo ----------------------------------------------------

call deactivate

:FIM
echo.
echo Processo encerrado.
pause
