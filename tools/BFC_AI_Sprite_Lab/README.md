# BFC Sprite Organizer

Ferramenta Python para verificar o formato real das imagens dentro de `chars/<personagem>/`, normalizar os nomes e garantir que os arquivos finais sejam PNGs verdadeiros.

**Este programa não remove fundo, não aplica chroma key, não altera pixels verdes e não usa IA.** O conteúdo visual é preservado; imagens que precisam de conversão são apenas codificadas no formato PNG.

## Instalação

1. Extraia a pasta `BFC_Sprite_Organizer` dentro de `bfc/tools/`.
2. O programa reutiliza uma `.venv` já existente. Ele procura, nesta ordem:
   - `tools/BFC_AI_Sprite_Lab/.venv`
   - `tools/.venv`
   - `.venv` na raiz de `bfc/`
   - `tools/BFC_Game_Studio/.venv`
   - ou o caminho definido pela variável de ambiente `BFC_VENV_PY`.
3. Execute `tools/BFC_Sprite_Organizer/ORGANIZAR_SPRITES.bat`.

O BAT **não cria outra `.venv`**. Ele reutiliza o ambiente encontrado e instala/atualiza somente a dependência `Pillow` declarada em `requirements.txt`.

Python 3.14.2 é compatível com Pillow 13.0.0. Se a sua `.venv` tiver sido criada com Python 3.14, ela será reutilizada.

## Testar antes de alterar

Execute:

```bat
tools\BFC_Sprite_Organizer\ORGANIZAR_SPRITES.bat --dry-run
```

A simulação mostra o que será renomeado ou convertido sem modificar os arquivos. Para aplicar, execute o BAT normalmente.

## Regras de nomes

A proporção é calculada após aplicar a orientação EXIF. As regras usam dimensões exatas:

| Condição | Nome final |
|---|---|
| Imagem quadrada (1:1) que não seja um frame de animação conhecido | `face.png` |
| Nome começando com `WhatsApp Image` e proporção exata 9:16 | `sprite.png` |
| Frames reconhecidos (`attack_01`, `jump_01`, `idle_01` etc.), mesmo em 1:1 | Mantém o nome-base, com `.png` |
| Demais imagens | Mesmo nome-base, com extensão `.png` |

Exemplos:

- `defense_01.jpeg` -> `defense_01.png`.
- `attack_01.png` cujo conteúdo é JPEG -> converte para um PNG verdadeiro chamado `attack_01.png`.
- `foto_quadrada.webp` (1:1) -> `face.png`.
- `attack_01.png` (1:1) -> permanece `attack_01.png`; é um frame de animação do jogo, não um retrato.
- `WhatsApp Image 2025-08-20 at 10.30.22.jpg` (9:16) -> `sprite.png`.
- `WhatsApp Image 2025-08-20 at 10.30.22.jpg` (1:1) -> `face.png`.
- `jump_01.gif` -> `jump_01.png`.

**Proteção específica do BFC:** na cópia do projeto analisada existem frames de animação quadrados, como `attack_01.png`, `idle_01.png` e `jump_01.jpg` (560x560). Renomear todos para `face.png` quebraria os nomes das sequências e entraria em conflito com o retrato já existente. Por isso, o padrão preserva nomes reconhecidos de frames (`attack`, `crouch`, `dead`, `defense`, `idle`, `jump`, `kick`, `run`, `special`, `walk` etc.).

Se você realmente quiser forçar literalmente todos os arquivos 1:1 para `face.png`, inclusive os frames de animação, use `--force-square-to-face`. Isso não é recomendado para o BFC atual: várias imagens tentariam ocupar o mesmo `face.png` e seriam reportadas como conflito.

## Backup e segurança

- Antes de alterar um arquivo, o original é copiado para `chars/<personagem>/backup_sprite/`.
- A origem só é removida depois que o PNG de destino foi gravado e validado.
- Arquivos de destino existentes nunca são sobrescritos. Em caso de colisão (por exemplo, duas imagens quadradas tentando se tornar `face.png`), o arquivo conflitante é mantido no lugar e o programa informa a situação.
- Backups anteriores não são sobrescritos. Se já existir um backup com o mesmo nome, o programa cria um nome `__backup_001`, `__backup_002` etc.
- O programa analisa somente arquivos diretamente dentro de `chars/<personagem>/`; não percorre subpastas e não reprocessa `backup_sprite/`.
- Arquivos que não forem imagens válidas são ignorados.

## Observações sobre formatos

O formato é identificado pelo conteúdo usando Pillow, não pela extensão. São reconhecidos os formatos suportados pela instalação do Pillow, incluindo PNG, JPEG, GIF, BMP, TIFF e WebP, entre outros.

GIFs e imagens com múltiplos quadros são convertidos usando o primeiro quadro, pois o arquivo final solicitado é PNG estático. O programa avisa quando isso acontecer. Para manter animações, seria necessário definir outro fluxo de exportação.

A proporção 1:1 é detectada quando largura e altura são iguais. A proporção 9:16 é detectada quando `largura * 16 == altura * 9`. Imagens próximas, mas não exatamente nessas proporções, não são classificadas como 1:1 ou 9:16.

O código verifica o conteúdo real da imagem. Por exemplo, um JPEG com extensão `.png` é recodificado como PNG verdadeiro; um PNG com extensão `.jpg` apenas recebe o nome correto, sem recodificação desnecessária.

## Estrutura

```text
bfc/
├── chars/
│   └── amanda/
│       ├── character.json
│       ├── face.png
│       ├── attack_01.png
│       ├── backup_sprite/
│       └── ...
└── tools/
    └── BFC_Sprite_Organizer/
        ├── organizar_sprites.py
        ├── requirements.txt
        ├── ORGANIZAR_SPRITES.bat
        └── README.md
```
