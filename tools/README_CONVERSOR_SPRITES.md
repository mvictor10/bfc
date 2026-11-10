# BFC Sprite PNG Converter

Ferramenta auxiliar para o projeto Brazil Fight Combat em Rust.

## Arquivos deste pacote

- `src/bin/converter_sprites.rs`: código-fonte do conversor.
- `tools/compilar_conversor_sprites.bat`: compila e copia o executável para `tools/`.

Não é necessário substituir o `Cargo.toml` do projeto recebido: ele já declara a crate `image` 0.24.9 com suporte a PNG, JPEG e BMP, que são os formatos necessários para os sprites atuais.

## Instalação

Copie os arquivos deste pacote para a raiz do projeto, preservando a estrutura:

```text
BrazilFightCombat_Rust_Port/
├── Cargo.toml
├── chars/
├── src/
│   └── bin/
│       └── converter_sprites.rs
└── tools/
    ├── compilar_conversor_sprites.bat
    └── README_CONVERSOR_SPRITES.md
```

Se `src/bin/` ou `tools/` já existirem, copie somente os arquivos correspondentes; não apague os demais arquivos dessas pastas.

## Compilação

No Windows, dê dois cliques em:

```text
tools\compilar_conversor_sprites.bat
```

Ou execute o comando abaixo na raiz do projeto:

```bat
cargo build --release --bin converter_sprites
copy /Y target\release\converter_sprites.exe tools\converter_sprites.exe
```

O executável final ficará em:

```text
tools\converter_sprites.exe
```

## Execução

Execute `tools\converter_sprites.exe`. Não é necessário informar argumentos ou selecionar personagem. A ferramenta localiza a raiz do projeto e percorre automaticamente:

```text
chars/
├── amanda/
│   ├── idle_01.png
│   ├── jump_01.jpg
│   └── character.json
├── ayana/
│   └── ...
└── outro-personagem/
    └── ...
```

A análise percorre os arquivos diretamente dentro de cada pasta de personagem. Não percorre subpastas internas.

## Regras de conversão

1. A ferramenta identifica o formato real pelo conteúdo do arquivo, e não confia somente na extensão.
2. Um arquivo chamado `idle_01.png` que seja realmente JPEG ou BMP é decodificado e regravado como PNG no mesmo caminho, mantendo o nome `idle_01.png`.
3. Um arquivo chamado `jump_01.jpg` gera `jump_01.png`; o arquivo original é preservado.
4. Um PNG válido é apenas validado e não é regravado.
5. Se o PNG de destino de um arquivo com outra extensão já existir, ele não é sobrescrito.
6. Arquivos JSON são ignorados. Outros arquivos que não possam ser decodificados como imagens também são ignorados e informados no relatório.
7. A execução pode ser repetida: arquivos PNG já válidos não são convertidos novamente.

## Dependências e formatos

O conversor reutiliza a dependência já existente no `Cargo.toml`:

```toml
image = { version = "0.24.9", default-features = false, features = ["png", "jpeg", "bmp"] }
```

Assim, reconhece e converte os formatos PNG, JPEG e BMP habilitados no projeto. Formatos como GIF, TIFF, WebP e HEIC não estão habilitados nessa configuração e não serão convertidos por esta versão.

## Observação

Este pacote contém somente os arquivos novos necessários para integrar o utilitário. O executável `.exe` é produzido no computador Windows do projeto pelo arquivo `.bat`; ele não está pré-compilado neste ZIP.
