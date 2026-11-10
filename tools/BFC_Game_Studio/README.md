# BFC Game Studio — Editor Web do Brazil Fight Combat

Editor local para administrar os dados e os recursos do Brazil Fight Combat. A interface usa HTML5, CSS3 e JavaScript; o servidor local usa Python/Flask; imagens são inspecionadas e convertidas com Pillow. O motor de jogo continua em Rust.

## Instalação

1. Faça uma cópia de segurança do projeto BFC.
2. Extraia a pasta `tools/BFC_Game_Studio` deste pacote para dentro da raiz do projeto, mantendo a estrutura:

```text
BrazilFightCombat_Rust_Port/
├── Cargo.toml
├── chars/
├── assets/
├── data/
├── src/
└── tools/
    └── BFC_Game_Studio/
        ├── app.py
        ├── requirements.txt
        ├── INICIAR_BFC_STUDIO.bat
        ├── templates/
        └── static/
```

3. Instale Python 3.10 ou superior no Windows. Marque `Add Python to PATH` durante a instalação.
4. Execute `tools\BFC_Game_Studio\INICIAR_BFC_STUDIO.bat`.
5. Na primeira execução, o BAT cria `.venv` e instala Flask e Pillow. É necessária conexão com a internet nessa primeira instalação.
6. O navegador abrirá `http://127.0.0.1:8765`. Se o projeto não for detectado automaticamente, clique em **Abrir projeto** e informe a pasta que contém `Cargo.toml` e `chars/chars.json`.

Mantenha a janela de terminal aberta enquanto o Studio estiver em uso. `Ctrl+C` encerra o servidor.

## Módulos

- **Visão geral:** quantidade de personagens, cenários, sprites e recursos de áudio; compilação e execução do jogo (executável Windows previamente compilado).
- **Personagens:** lê o elenco de `chars/chars.json` e edita `chars/<id>/character.json`; atributos, fase, música, voz, arte, âncoras, chroma, rig, animações, efeitos, overrides, vozes e metadados avançados. Permite registrar novo personagem e retirar do elenco sem apagar sua pasta.
- **Fases e cenários:** lê/grava `data/stages.ini`, visualiza as imagens e importa recursos em `assets/images/stages/`.
- **Animações e sprites:** lista frames pelo prefixo, pré-visualiza a sequência, configura prefixo/FPS/modo e permite enviar frames.
- **Áudio:** lista e reproduz música, SFX e vozes; permite importar arquivos. Vozes são colocadas em `assets/audio/voice/<id-do-personagem>/`.
- **Histórias:** edita `story/<id>/historia.txt`.
- **Golpes e dano:** edita os parâmetros globais de `data/moves.ini`.
- **Regras gerais:** edita os parâmetros existentes em `data/engine.ini`.
- **Gerenciador de recursos:** navega por imagens de fases, efeitos e bibliotecas de áudio; upload e remoção reversível para `.bfc_studio_trash/`.
- **Conversor de imagens:** analisa o conteúdo real dos arquivos em todas as subpastas de `chars/` e em `assets/images/stages/`; converte imagens reconhecidas para PNG e atualiza referências explícitas de arquivos de configuração.
- **Validação:** confere JSON, referências de arte, imagens de fase e integridade básica das imagens.

## Conversor PNG

O conversor não confia na extensão. Pillow tenta decodificar o conteúdo real do arquivo. Para imagens com extensão diferente de `.png`, cria um PNG com o mesmo nome-base e preserva o original. Se um arquivo chamado `.png` contiver outro formato, o programa salva uma cópia de segurança com o sufixo `.original.bak` antes de substituir o arquivo por PNG válido.

Se um arquivo de destino `.png` diferente já existir, ele não será sobrescrito; a colisão aparece no relatório. GIFs e imagens animadas são convertidos usando o primeiro quadro. O conversor **não remove chroma key** e não altera pixels verdes dos cenários. A transparência de sprites continua configurada pelo campo `chroma` em cada `character.json` e pelo processamento existente no motor.

## Segurança e backups

- O servidor escuta somente em `127.0.0.1`; não foi projetado para ser publicado na rede ou na internet.
- O backend bloqueia caminhos que tentem sair da pasta do projeto.
- Antes de gravar um arquivo de configuração existente, cria uma cópia `.bak` caso ela ainda não exista. Para revisões futuras, faça cópias manuais adicionais se quiser manter vários pontos de restauração.
- Arquivos removidos pelo gerenciador são movidos para `.bfc_studio_trash/`, não apagados definitivamente.
- Faça backup completo do projeto antes de conversões em lote ou alterações de configuração.

## O que permanece no motor Rust

Este é um editor de dados/recursos do BFC, não uma reimplementação do runtime. Física, colisões, IA, estados dos lutadores, leitura de teclado, lógica de cenas, desenho e áudio de runtime continuam em `src/`. O Studio edita os parâmetros que `src/config.rs` já lê. O vínculo inicial das posições P1/P2 e o mapeamento de teclas ainda são definidos no código Rust. O editor de fase corresponde ao formato atual do motor (uma imagem de fundo, ID, nome, música e altura do chão); paralaxe, objetos interativos ou múltiplas camadas exigem expansão do renderizador Rust. Personalizar algoritmo da IA, adicionar novos estados de combate, alterar o mapeamento de teclas, criar novos tipos de hitbox, editar o código do renderizador ou mudar a lógica de gameplay exige implementação Rust adicional.

A ferramenta **Compilar** executa `cargo build --release`. **Executar jogo** procura `target/release/BrazilFightCombat.exe` (ou os caminhos debug/raiz) e inicia o executável no Windows.

## Dependências

- Python 3.10+
- Flask 3.x
- Pillow 10.x–12.x
- Rust/Cargo apenas para compilar o jogo pelo botão do Studio

## Desenvolvimento

```bash
python -m venv .venv
.venv\Scripts\activate
pip install -r requirements.txt
python app.py
```

O endereço local é `http://127.0.0.1:8765`.
