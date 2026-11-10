# BFC AI Sprite Lab - Atualização para Python 3.14.2

Esta atualização habilita o inicializador para Python 3.14.x, incluindo Python 3.14.2.

## Arquivos incluídos

- `INICIAR_BFC_AI_SPRITE_LAB.bat`: prioriza Python 3.14 e aceita também 3.13, 3.12 e 3.11.
- `requirements.txt`: instala o rembg e o ONNX Runtime para execução pela CPU.
- `README_PYTHON_314.md`: estas instruções.

## Como aplicar

Extraia o ZIP na raiz do projeto BFC, mantendo os caminhos. Permita substituir os arquivos existentes dentro de `tools/BFC_AI_Sprite_Lab/`.

**Não apague `ai_sprite_remover.py`**: ele continua sendo o programa executado pelo inicializador e não está incluído neste pacote de atualização.

Depois, execute `tools/BFC_AI_Sprite_Lab/INICIAR_BFC_AI_SPRITE_LAB.bat`.

## O que o inicializador faz

1. Procura primeiro Python 3.14; depois tenta 3.13, 3.12 e 3.11.
2. Cria o ambiente virtual `.venv` se ainda não existir.
3. Reutiliza uma `.venv` criada com Python 3.11 a 3.14.
4. Recria a `.venv` se ela usar uma versão fora desse intervalo.
5. Instala as dependências e executa `ai_sprite_remover.py`.

## Dependências

- `rembg[cpu]==2.0.86`
- `onnxruntime==1.31.1`

O pacote ONNX Runtime 1.31.1 publica wheel CPython 3.14 para Windows x64. O rembg 2.0.86 declara Python >=3.11 e lista Python 3.14 entre seus classificadores.

A primeira execução requer internet para instalar as bibliotecas e baixar o modelo de IA. Depois, o ambiente virtual e o modelo em cache são reutilizados.

## Observação

Este ZIP é somente uma atualização do inicializador, das dependências e das instruções. Ele não contém o script `ai_sprite_remover.py` nem os arquivos do projeto inteiro.
