# Brazil Fight Combat — Port Rust do jogo original

Este projeto é uma conversão progressiva do **Brazil Fight Combat** original (HTML5/JavaScript) para um executável nativo de Windows em Rust. O ZIP inclui o elenco registrado, imagens originais dos personagens, cenários, áudio disponível, configurações dos golpes, histórias de campanha e documentos de referência. A lógica foi portada para os módulos Rust descritos abaixo; não é apenas uma cópia dos assets do jogo web.

> **Estado da conversão:** o carregamento de elenco/configurações, imagens, animações por quadros, rigs, cenários, efeitos e áudio nativo está implementado na base deste projeto. Como este ambiente de preparação não dispõe de `rustc`/`cargo`, a compilação real em Windows ainda precisa ser validada no computador de destino. Não interprete a auditoria de assets como prova de compilação.

## Conteúdo original incluído

- **21 personagens** listados em `chars/chars.json`, cada um com seu `character.json` e assets de arte disponíveis.
- **26 cenários** referenciados por `data/stages.ini`, com as imagens originais em `assets/images/stages/`.
- Frames individuais de animação originais em PNG/JPG/JPEG onde existem, além de artes de corpo inteiro usadas pelo rig legado.
- Efeitos de combate compartilhados e efeitos específicos existentes em `assets/images/effects/` e nas pastas dos personagens.
- Música e SFX disponíveis sob `assets/audio/`, incluindo vozes de personagem que têm arquivo próprio.
- Histórias de campanha em `story/<id>/historia.txt` quando presentes.
- Documentos de referência do projeto original em `docs/original-reference/`.

O port seleciona os personagens por meio da lista `chars/chars.json`; diretórios existentes que não estejam registrados nessa lista não são adicionados automaticamente ao elenco, mantendo o comportamento da seleção original. O pacote também mantém os modelos `chars/_modelo/` e `chars/_exemplo/`, além dos materiais de origem `chars/agente-lya/`, `chars/prof-jeffy/` e `chars/robsom/`. Esses três últimos ainda não fazem parte do elenco: seus `character.json` são modelos de base, e as fotografias JPG incluídas precisam ser escolhidas/importadas e salvas pelo BFC Studio para gerar `sprite.png`/`face.png` corretos e registrá-los no roster.

## Funcionalidades portadas

- Janela nativa Win32/GDI, framebuffer de 960 × 540 e atualização da simulação em ticks de 60 Hz.
- Menu principal, Versus para dois jogadores, CPU, Treinamento, campanha Arcade, modo de observação IA contra IA e tela de história/final.
- Tags BOSS e SUB-BOSS junto aos personagens na seleção; por padrão, Viktor Krayller é o chefe e Amanda é a sub-chefe.
- Quantidade configurável de adversários normais antes da sub-chefe e do chefe (3, 5, 8, 12, 16 ou 24).
- Locomoção, corrida, pulo, agachamento, defesa, ataques, hitstun, empurrão, vida, K.O., rounds e cronômetro.
- Soco, chute, especial e rasteira, com startup/active/recovery, alcance, altura, dano, stun e knockback.
- Colisão de ataque contra hurtbox; processamento sequencial dos ataques no mesmo tick, de modo que um acerto inicial possa interromper o segundo ataque, conforme o jogo JavaScript.
- Defesa frontal com dano reduzido, empurrão reduzido e sem interromper a postura de defesa.
- IA com seis dificuldades: Muito Fácil, Fácil, Normal (aprendizado adaptativo), Difícil, Muito Difícil e Hardcore.
- Aprendizado estatístico persistente somente no nível Normal, em lutas contra CPU e Arcade; a IA registra tendências observadas do jogador e usa heurísticas para ajustar defesa e seleção de golpes. No modo IA vs IA, o perfil aprendido pode ser consultado, sem registrar o comportamento automático como se fosse uma partida de jogador. Não é uma rede neural nem um treinamento de modelo profundo.
- O modo “Observar IA vs IA” põe os dois personagens selecionados sob controle automático, sem comandos de combate dos jogadores. Na dificuldade Normal, os dois agentes consultam o perfil aprendido em partidas contra pessoas, mas partidas IA vs IA não alteram nem sobrescrevem o perfil persistente.
- Animação por frames individuais e rig de arte segmentada quando a personagem tem configuração `rig`; fallback de arte estática ou desenho vetorial caso não existam assets suficientes.
- Remoção aproximada do chroma key verde, carregamento de PNG/JPEG/BMP e redução de tamanho dos quadros ao carregar; o BFC Studio também converte imagens PNG/JPEG/BMP/WebP/GIF/TIFF para PNG (em GIF usa o primeiro quadro).
- Cenários originais, HUD de vida/tempo/combos, hitboxes de depuração (`F1`), efeitos e partículas.
- Áudio do Windows por APIs nativas: música MP3/WAV, efeitos WAV e falas de vitória gravadas MP3/WAV; na ausência de gravação, tenta sintetizar `voices.win.text` com a API de fala disponível no Windows.
- Arquivos editáveis INI e o BFC Studio, dashboard GUI nativo para criar/editar personagens, converter artes, importar mapas e trilhas, configurar fala de vitória e auditar assets sem Python.

## Requisitos e compilação

Use Windows 10/11, Rust estável (recomenda-se a versão estável atual) e Cargo. O código define `rust-version = "1.70"`, mas isso não substitui o teste com a toolchain recomendada.

1. Instale Rust a partir de <https://www.rust-lang.org/tools/install>.
2. Extraia o ZIP preservando todas as pastas.
3. Abra um terminal na raiz do projeto, onde está `Cargo.toml`.
4. Execute:

```bat
cargo build --release
cargo test
```

Para padronizar a formatação e executar a análise estática:

```bat
cargo fmt
cargo clippy --all-targets
```

Também estão incluídos `build.bat`, `run.bat`, `test.bat` e scripts para alvos GNU/MSVC. Abra `target\release\BrazilFightCombat.exe` depois da compilação em release. Para o dashboard, execute `tools\build_studio.bat`, que gera e copia `BFCStudio.exe` para `tools\`; depois use `tools\run_studio.bat`. O jogo e o dashboard carregam configuração/assets a partir da pasta do projeto; mantenha a hierarquia original de diretórios.

**Dependências do build:** o `Cargo.toml` usa as crates `image` (habilitada para PNG, JPEG, BMP, GIF, WebP e TIFF) e `serde_json` para decodificar imagens/configurações JSON. Na primeira compilação, Cargo precisa baixar essas crates, normalmente com conexão à internet. Elas não exigem que DLLs dessas crates sejam distribuídas ao lado do executável, mas o backend de áudio usa APIs nativas do Windows. Não é um projeto “sem crates”; é um projeto com um conjunto pequeno de dependências.

## Controles

| Ação | Jogador 1 | Jogador 2 |
|---|---|---|
| Andar | `A` / `D` | Setas esquerda/direita |
| Pular | `W` | Seta para cima |
| Agachar | `S` | Seta para baixo |
| Soco | `F` | `J` |
| Chute | `G` | `K` |
| Especial | `H` | `L` |
| Defender | `R` | `I` |
| Navegação de menu | Setas | — |
| Confirmar/iniciar | `Enter` | — |
| Mostrar hitboxes | `F1` | — |
| Voltar ao menu | `Esc` | — |
| Treinamento: alternar defesa do boneco | `B` | — |
| Treinamento: reiniciar | `N` | — |
| Trocar personagem do P1 | `Q` / `E` | — |
| Trocar personagem do P2 | — | `U` / `O` |
| Escolher cenário | `[` / `]` | — |

No menu principal, `Q`/`E` e `U`/`O` percorrem a lista registrada, e `[`/`]` selecionam um cenário manualmente. Em Configurações, as teclas `Q`/`E` modificam a opção atual.

## Onde personalizar o jogo

- `chars/chars.json`: elenco e ordem da seleção.
- `chars/<id>/character.json`: nome, atributos, corpo, direção original, arte, frames, rig, golpes específicos e efeitos.
- `chars/<id>/idle_01.png`, `attack_01.png`, `kick_01.png` etc.: frames individuais disponíveis para aquele personagem.
- `assets/images/stages/`: imagens de cenários.
- `assets/images/effects/`: efeitos de fallback compartilhados.
- `assets/audio/music/`: música de menu, luta e temas específicos.
- `assets/audio/sfx/`: efeitos sonoros.
- `assets/audio/voice/<id>/`: vozes específicas, quando disponíveis.
- `story/<id>/historia.txt`: texto do final de campanha.
- `data/engine.ini`: gravidade, salto, chão, tempo, rounds e dificuldade inicial.
- `data/moves.ini`: parâmetros globais de soco/chute/especial/rasteira.
- `data/stages.ini`: nomes, caminhos de imagem, música e altura de chão dos cenários.

As opções `moves` existentes no JSON de um personagem sobrescrevem apenas os parâmetros correspondentes daquele golpe. O formato de nomes das animações mantém o padrão original `prefixo_01`, `prefixo_02`, etc. Se uma sequência tem lacunas, o carregador interrompe a leitura no primeiro quadro ausente. Para associar música ao personagem e garantir que só toque em sua fase, preencha `stage` com o ID do mapa da casa e `music` com o caminho/ID da trilha. Se a escolha automática usar um mapa neutro, o motor não usa o tema pessoal nesse mapa; seleção manual usa a música do próprio cenário.

## Arquitetura de software

| Módulo | Responsabilidade |
|---|---|
| `config` | Carregar e validar INI, roster, personagens, estatísticas, overrides de golpes e cenários |
| `geometry` | Retângulos e interseção de colisão |
| `input` | Comandos independentes do sistema operacional |
| `controller` | Trait `Controller` e controle humano |
| `ai` | Estratégia de decisões da CPU |
| `fighter` | Estados, física, movimentação, hurtbox, hitbox e temporizadores |
| `combat` | Colisão, defesa, dano, stun, empurrão, combos e eventos |
| `animation` | Leitura de imagens, sequências, chroma key, efeitos e seleção de quadros |
| `game` | Cenas, regras de luta, rounds, modos, campanha e história |
| `render` | Renderização por software de cenários, rig, sprites, HUD, efeitos e debug |
| `platform/win32` | Janela, leitura de teclas, temporização e apresentação com GDI |
| `platform/audio` | Música/SFX/voz por APIs do Windows |

Princípios aplicados: separação de responsabilidades, composição, configuração orientada a dados, estados explícitos com `enum`, estratégia de controle via `trait`, ownership/referências do Rust e dependência da camada de plataforma apenas no backend. Esses padrões foram usados pragmaticamente; não há uma hierarquia de classes artificial imitando C++.

## Desempenho e memória

A resolução lógica de 960 × 540 com buffer de pixels de 32 bits ocupa cerca de 1,98 MiB. O motor limita a apresentação a 60 FPS e carrega o conjunto de sprites dos lutadores ativos e o cenário selecionado, em vez de decodificar todo o elenco de uma só vez. O perfil `release` habilita otimização, LTO, uma unidade de geração de código e remoção de símbolos.

O objetivo é funcionar em computadores modestos com processador Dual Core e 500 MB de RAM disponível, mas isso **não é uma garantia de consumo total inferior a 500 MB**: memória do Windows, drivers, imagens carregadas e outros processos também contam. Meça a versão compilada no computador de destino com o Gerenciador de Tarefas ou uma ferramenta de profiling. Uma compilação Rust pode consumir muito mais memória durante o build do que o jogo em execução.

## Limitações atuais que devem ser consideradas

- O backend gráfico/janela é Windows-only (Win32/GDI); a maior parte do domínio não depende da plataforma.
- A seleção de elenco usa navegação por teclado e prévia de personagens, mas ainda não reproduz a grade visual completa do jogo HTML original.
- A conversão de rig é aproximada da implementação JavaScript original; personagens com atlas legado `sheet.png`/`atlas.json` não têm o formato de atlas reproduzido diretamente. O port prioriza frames individuais, arte única, rig procedimental e fallback.
- A aprendizagem da IA é uma heurística estatística sobre padrões observados, não uma rede neural. O registro e a persistência ocorrem somente em CPU/Arcade na dificuldade Normal; o modo IA vs IA pode consultar esse perfil, mas não grava observações automáticas nem o sobrescreve.
- O BFC Studio oferece campos textuais para papéis (`normal`, `subboss`, `boss`), mapa da casa, música e fala; importe trilhas, escolha o caminho e salve o personagem/mapa para associar. Pode substituir uma trilha catalogada reutilizando o mesmo ID.
- Não foi possível executar `cargo build`/`cargo test` neste ambiente de preparação, pois `rustc` e `cargo` não estão instalados aqui. Execute os comandos acima no Windows antes de considerar o port validado.

## Dashboard nativo (BFC Studio)

O `BFCStudio` é uma segunda aplicação Rust/Win32 definida em `src/bin/bfc_studio.rs`, sem dependência de Python em execução. Ele pode ser compilado e copiado para `tools/BFCStudio.exe` com `tools\build_studio.bat`. O dashboard cria/edita `character.json`, registra personagens no roster, converte JPG/JPEG/PNG/BMP para PNG, gera `sprite.png` e `face.png`, importa mapas para `data/stages.ini`, registra/atualiza trilhas MP3/WAV e executa uma auditoria do roster, arte, frames e cenários.

A lista de trilhas inicial fica em `data/music_library.json`. As trilhas associadas em `character.json` ou `stages.ini` usam caminhos relativos à raiz do projeto para que o motor as toque no cenário correspondente.
