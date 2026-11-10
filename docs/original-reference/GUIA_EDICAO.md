# Guia de edição — onde mexer para cada coisa

Os arquivos que você edita no dia a dia estão em `js/data/` (todos comentados). O resto do código só precisa ser mexido para criar mecânicas novas.

| Quero... | Arquivo |
|---|---|
| adicionar/alterar personagens (frames, atributos, stage, música, voz, efeitos) | `chars/<pasta>/character.json` — veja `docs/PERSONAGENS.md` |
| adicionar/alterar stages (imagem, música, altura do chão) | `js/data/stages.js` |
| registrar músicas, efeitos sonoros e vozes | `js/data/audio.js` |
| mudar tamanho da tela, grade de seleção, golpes e teclas | `js/data/config.js` |
| quais animações tocam em cada estado, efeitos padrão | `js/data/animations.js` |
| (legado) sprite sheet | `docs/legado/ANIMACOES.md` |
| escrever/alterar o final (história) de um personagem | `story/<pasta>/historia.txt` (+ `img_01.png` opcional) — veja `story/LEIA-ME.md` |
| trocar quem é o sub-chefe / chefe final da campanha | `BFC.CAMPAIGN_SUBBOSS` / `BFC.CAMPAIGN_BOSS` em `js/data/config.js` |

## 9. Campanha (Arcade / Desafio Solo) e finais
O modo **Arcade** agora é uma campanha: o jogador enfrenta **todo o elenco** (em ordem aleatória, cada
adversário no seu próprio *stage* e trilha sonora — regra `'auto'` de `MatchRules.js`), depois o
**sub-chefe** e por fim o **chefe final** (`BFC.CAMPAIGN_SUBBOSS` / `BFC.CAMPAIGN_BOSS`, por padrão
`amanda` e `viktor-krayller`). Se o jogador escolher o próprio sub-chefe ou chefe final para jogar, essa
luta é pulada automaticamente (ninguém luta contra si mesmo).
Ao vencer a última luta, `BFC.StoryScene` conta o final (`story/<pasta>/historia.txt`) do personagem
**escolhido pelo jogador** — é o final dele, não dos adversários.
**DESAFIO SOLO** (novo item do menu) é a mesma campanha, mas sorteia o personagem do jogador
automaticamente, sem passar pela tela de seleção — para jogar sozinho sem escolher.

## 1. Adicionar um personagem
`python3 tools/novo_personagem.py "Nome" --arte arte.png` cria a pasta `chars/nome/` (com `character.json` completo) e registra no jogo.
Depois coloque os frames (`idle_01.png`, `attack_01.png`...) dentro da pasta. Tudo (stats, stage, música, voz, animações, efeitos)
fica no `character.json` — guia completo em `docs/PERSONAGENS.md`. A tela de seleção se ajusta sozinha (`BFC.SELECT_LAYOUT` em `config.js`).

## 2. Stage específico e trilha por personagem
No `character.json` do personagem: `"stage": "porto"` (id do stage) e `"music": "tema_krayller"` (id da trilha em `audio.js`, ou um arquivo na pasta do personagem).
Regras (em `js/systems/MatchRules.js`):
- **Arcade:** usa o stage "da casa" do adversário (CPU). Se ele não tiver, o do jogador; se nenhum, sorteia.
- **Stage AUTOMÁTICO** (Versus/Trainer): igual ao Arcade (dono = P2, depois P1). Toca a trilha do dono do stage, se tiver `music`; senão a música do stage; senão `battle`.
- **Stage escolhido na mão:** toca a música desse stage (ou `battle`).
Para mudar essas regras, edite só `MatchRules.js`.

## 3. Adicionar um stage
1. Coloque a imagem em `assets/images/stages/` (qualquer tamanho; ela cobre a tela sem distorcer).
2. Em `js/data/stages.js`: `{ id: 'favela', name: 'Morro ao Amanhecer', image: 'assets/images/stages/stage5.png', music: 'tema_morro', ground: 470 }`.
3. `ground` = altura (0–540) onde os pés pisam. Ligue **Mostrar hitboxes** nas Configurações e ajuste até casar com o chão da arte.

## 4. Música de fundo por stage
Coloque o arquivo em `assets/audio/music/`, registre em `audio.js > music` (`tema_morro: 'assets/audio/music/morro.mp3'`) e use esse id no `music` do stage. As trilhas `tema_*.wav` que vieram são exemplos gerados por `tools/gerar_temas_exemplo.py`; substitua pelas suas.

## 5. Vozes ("Lute!", "Você ganhou!", "Você perdeu!"...)
Em `audio.js > voices`. Eventos: `round1`, `round2`, `round3`, `fight`, `ko`, `timeup`, `win`, `lose`, `draw` (e `pick`, opcional, ao escolher o personagem).
- Sem arquivo, o navegador **fala o texto** (síntese de voz em pt-BR) — funciona na hora, mas a voz depende do sistema.
- Com arquivo: `fight: { text: 'Lute!', files: ['assets/audio/voice/default/fight.mp3'] }` (vários arquivos = sorteia um).
- Voz de um personagem: crie o pacote com o nome da pasta dele (ex.: `amanda: { win: {...} }`), ou defina `voices` direto no `character.json`. Evento ausente cai no narrador `default`.
- Quem fala: Arcade/Trainer → narrador ("Você ganhou/perdeu!"); Versus → o personagem vencedor (`win`).
O volume das vozes fica em Configurações.

## 6. Efeitos sonoros
`audio.js > sfx`. Cada golpe usa o `sfx` definido em `BFC.MOVES` (`config.js`); para um som novo, registre-o e use o nome no golpe.

## 7. Escolha de stage no Versus e no Trainer
Depois de escolher os personagens aparece a tela de stage: **Automático**, **Aleatório** e cada stage. No Versus, P1 (WASD+F) e P2 (setas+J) escolhem; se forem diferentes, o jogo sorteia entre os dois. No Trainer só o P1 escolhe.

## 8. Mapa do código (uma classe = uma responsabilidade)
`Game` (loop) · `Settings` · `Input` · `AudioManager` · `AssetLoader` · `Menu`/`HUD` · `Fighter` (física e estados) · `SpriteAnimator`/`RigAnimator` (desenho e animação) · `CombatSystem` · `ParticleSystem` · `MatchRules` (stage/música) · `Controllers` (humano/CPU/boneco) · `ArcadeMode` · cenas em `js/scenes/` (Menu, Select, StageSelect, Fight, Trainer, Settings, Message).
Para rodar o jogo: `iniciar.bat` / `iniciar.sh` (servidor local; necessário para os frames com fundo verde).
Para deixar outro arquivo compacto legível: `python3 tools/formatar_js.py caminho.js`.
