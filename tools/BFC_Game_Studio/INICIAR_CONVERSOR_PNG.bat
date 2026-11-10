@echo off
:: Altera a codificação do prompt para UTF-8 (corrige acentos no terminal)
chcp 65001 >nul

echo ====================================================
echo         INICIANDO CONVERSOR DE SPRITES - BFC
echo ====================================================

:: Define o caminho da pasta atual do script
set "PASTA_ATUAL=%~dp0"
cd /d "%PASTA_ATUAL%"

:: Verifica se a pasta do ambiente virtual (.venv) existe
if not exist ".venv\Scripts\activate.bat" (
    echo [ERRO] Ambiente virtual ^(.venv^) não encontrado nesta pasta!
    echo Certifique-se de que este arquivo .bat está na mesma pasta que a pasta .venv.
    goto FIM
)

echo [1/3] Ativando ambiente virtual (.venv)...
call ".venv\Scripts\activate.bat"

echo [2/3] Instalando/Atualizando dependências do requirements.txt...
python -m pip install --upgrade pip -q
pip install -r requirements.txt

echo [3/3] Executando o script de conversão...
echo ----------------------------------------------------
python converter_sprite.py
echo ----------------------------------------------------

:: Desativa o ambiente virtual ao terminar
call deactivate

:FIM
echo.
echo Processo encerrado.
pause
