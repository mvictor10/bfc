# BFC Studio — Dashboard nativo do Brazil Fight Combat

O BFC Studio é um aplicativo gráfico nativo de Windows escrito em Rust/Win32. Depois de compilado, ele funciona sem Python. É incluído como binário `BFCStudio` do Cargo; por não haver toolchain Rust instalada no ambiente de geração, o arquivo `.exe` é gerado no Windows pelo script de build.

## Gerar o executável

1. Instale Rust estável e Cargo pelo instalador oficial: https://www.rust-lang.org/tools/install
2. Na raiz do projeto, execute `tools\build_studio.bat`.
3. O binário será copiado para `tools\BFCStudio.exe`. Para abrir, execute-o ou use `tools\run_studio.bat`.

Na primeira compilação, Cargo precisa baixar as crates `image` e `serde_json`. Essa necessidade é só do build, não para rodar o executável já gerado.

## Personagens

- Informe nome, ID, velocidade, poder, vida e papel (`normal`, `subboss` ou `boss`).
- Indique o mapa da casa e o caminho da trilha de música, quando desejado.
- Digite a frase de vitória ou selecione uma gravação MP3/WAV. A fala gravada tem prioridade; a frase de texto é usada como alternativa pelo mecanismo de síntese de fala do Windows.
- Selecione arte principal e, opcionalmente, um rosto separado. JPG/JPEG, PNG, BMP, WebP, GIF e TIFF são convertidos para PNG (no GIF, o primeiro quadro é usado); o dashboard cria `sprite.png` e `face.png`, tenta remover chroma key verde/branco detectável e limita o tamanho de saída. Se não houver rosto fornecido, ele deriva um recorte da arte principal.
- Clique em “Criar / salvar personagem” para gerar/atualizar `chars/<id>/character.json`, atualizar `chars/chars.json` e criar `story/<id>/historia.txt` quando for um personagem novo. Ao marcar um boss/sub-boss, o mesmo papel é removido de outro personagem para manter no máximo um de cada. Por padrão, Viktor Krayller é boss e Amanda é sub-boss.

## Mapas e trilhas

- Para mapa novo, informe ID, nome, coordenada do chão, caminho da imagem e música opcional. A imagem é convertida para PNG em `assets/images/stages/` e o cadastro é atualizado em `data/stages.ini`.
- Para editar um mapa existente, digite o ID e use “Carregar mapa pelo ID”; ajuste os campos e salve.
- Importe áudio MP3/WAV com ID único. O arquivo vai para `assets/audio/music/` e é catalogado em `data/music_library.json`. Se reutilizar um ID já catalogado, pode substituir a faixa; o catálogo é atualizado.
- Após importar a faixa, associe o caminho que aparece no campo Música do personagem e/ou do mapa e salve esses cadastros. A música do personagem é usada quando o motor escolhe automaticamente o mapa da casa desse personagem; se o cenário for escolhido manualmente, prevalece a trilha cadastrada no próprio mapa.

## Auditoria

O botão “Auditar personagens/mapas” verifica arquivos JSON, artes, faces, imagens de mapas, trilhas com caminhos explícitos e contagem de frames PNG reconhecidos. Os relatórios são diagnósticos estáticos; não substituem compilar ou testar o jogo em execução.


## Compilar jogo e dashboard de uma vez

Na raiz do projeto, execute `build.bat`. Ele compila explicitamente `BrazilFightCombat` e `BFCStudio` em modo `release` e copia `target\release\BFCStudio.exe` para `tools\BFCStudio.exe`. O compilador Rust e o Cargo são necessários apenas para gerar os executáveis; depois de compilado, o dashboard não depende de Python.


## Materiais de origem que ainda não estão no elenco

O projeto preserva `chars/agente-lya/`, `chars/prof-jeffy/` e `chars/robsom/` com fotografias JPG originais e JSON de base. Eles não são incluídos no roster até serem configurados. Para finalizar um deles: abra o BFC Studio, digite o ID correspondente, informe o nome e escolha a foto em Arte principal; opcionalmente escolha outro arquivo em Rosto (face); revise os atributos e clique em Criar / salvar personagem. O Studio converte a imagem para PNG, gera a face derivada quando necessário e registra o ID em `chars/chars.json`. Os arquivos de entrada não são apagados.
