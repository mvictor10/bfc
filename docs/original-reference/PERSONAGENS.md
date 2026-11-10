# Personagens por pasta (frames soltos + character.json)

Cada personagem é uma **pasta** em `chars/`. Dentro dela ficam os **frames** (um PNG por quadro) e **um `character.json`
com toda a configuração**. O jogo remove o fundo verde sozinho.

```
chars/
  chars.json                 ← lista (e ordem) dos personagens da seleção
  amanda/
    character.json           ← TUDO do personagem (stats, stage, música, voz, animações, efeitos, golpes...)
    idle_01.png idle_02.png ...        (animações: idle, attack, kick, jump, crouch, walk, run, defense)
    fx_punch_01.png fx_hit_01.png fx_dizzy_01.png ...   (efeitos)
    face.png  sprite.png     ← (opcional) rosto 140x140 e arte grande para a tela de seleção
  _modelo/                   ← modelo copiado ao criar um personagem novo
  _exemplo/                  ← exemplo funcionando (frames de teste); veja "Exemplo"
```

## Criar um personagem
```
python3 tools/novo_personagem.py "Nome do Personagem" [--arte corpo_inteiro.png] [--velocidade 4.5 --poder 1.0 --vida 200]
```
Cria `chars/<nome>/` com um `character.json` completo, registra em `chars.json` e atualiza `chars/bundle.js`.
Depois é só colocar os frames dentro da pasta. (Sem `--arte`, o rosto da seleção sai do `idle_01.png`.)
À mão: copie `chars/_modelo`, renomeie, edite o `character.json` e acrescente o nome da pasta em `chars/chars.json`.

## Frames
- Nome: `<animação>_01.png`, `<animação>_02.png`, ... **dois dígitos, sem buracos** (o jogo lê enquanto existir o próximo).
- **Todos os frames do personagem com o mesmo tamanho** (ex.: 600x1067), fundo **verde neon #00FF00 liso**, personagem de perfil
  **olhando para a direita** (se olhar para a esquerda: `"facing": "left"`), pés sempre na mesma altura do quadro.
- Os pés ficam no ponto `anchor` do quadro (padrão: centro horizontal, 92% da altura) — não há recortes nem atlas, então
  nada "anda" ou desalinha: o quadro é desenhado inteiro, sempre no mesmo lugar.
- Tamanho na tela: `"size": 300` = altura (px) do **quadro inteiro**. Aumente/diminua para o personagem ficar maior/menor.
- Frames muito grandes são reduzidos ao carregar (`chroma.maxHeight`, padrão 560) para economizar memória.

### As 8 animações
| Animação | Quando toca | Como toca |
|---|---|---|
| `idle` | parado | repete |
| `walk` | andando | repete |
| `run` | andando por mais de ½ segundo (fica 35% mais rápido) | repete |
| `jump` | no ar | 1º quadro subindo, do meio no topo, último descendo |
| `crouch` | agachado | toca uma vez e segura o último quadro |
| `defense` | defendendo (R / I) | toca uma vez e segura o último quadro |
| `attack` | soco (F / J) | os quadros se distribuem pela duração do golpe |
| `kick` | chute (G / K) | idem |

Opcionais (basta criar os arquivos): `special` (H / L), `sweep` (rasteira: agachado + chute), `hit` (apanhando), `ko`.
**Animação que não existir continua como está** (a animação antiga por recortes/sheet) — dá para ir fazendo uma de cada vez.
A ordem de preferência de cada estado fica em `BFC.FRAME_USE` (`js/data/animations.js`): por exemplo, sem `walk` usa `run`, sem `defense` usa `crouch`.

### Derrotado (KO)
Quem perde a vida **fica agachado** (animação `ko`, ou o último quadro de `crouch`) com o efeito **zonzo** em cima da cabeça
(`fx_dizzy_01.png...` em loop; sem esses arquivos, o jogo desenha estrelinhas girando).

## Efeitos (`fx_<nome>_01.png ...`, fundo verde ou transparente)
| Efeito | Onde aparece |
|---|---|
| `punch`, `kick`, `special`, `sweep` | na frente do atacante quando o golpe passa a valer |
| `hit` (ou `hit_punch`, `hit_kick`, `hit_special`) | no oponente quando o golpe acerta |
| `block` | no ponto da defesa |
| `dizzy` | sobre a cabeça de quem foi derrotado (loop) |
Sem sprites de efeito, valem os efeitos desenhados por código (faíscas e arco do golpe).
Cada efeito aceita: `prefix`, `fps`, `loop`, `scale`, `blend` (`"lighter"` = brilho), `anchor` (`attacker`, `target`, `head`) e `offset: [x, altura]`
(em px a partir dos pés; x para a frente do personagem).

## character.json — todos os campos
| Campo | Para que serve |
|---|---|
| `name` | nome mostrado no jogo |
| `stats` | `speed` (3 lento … 7 rápido), `power` (1.0 = normal), `hp` (vida) |
| `stage` | stage "da casa" (id de `js/data/stages.js`) ou `null` |
| `music` | trilha-tema: id de `audio.js` **ou** um arquivo dentro da pasta (`"tema.mp3"`) ou `null` |
| `voice` / `voices` | pacote de voz; `voices` define falas na própria pasta: `{"win":{"text":"Foi fácil!","files":["win.mp3"]}}` |
| `art` | `sprite` (arte grande) e `face` (rosto) da seleção |
| `size`, `anchor`, `facing` | tamanho na tela, posição dos pés no quadro, para onde os frames olham |
| `chroma` | `color` (padrão `#00ff00`), `tolerance` (90), `softness` (60), `maxHeight` (560) |
| `maxFrames` | limite de frames por animação (24) |
| `animations` | por animação: `prefix`, `fps`, `mode` (`loop`/`once`/`progress`/`vy`), `frames` (força a quantidade) |
| `effects` | veja acima |
| `moves` | muda golpes só deste personagem: `{"kick":{"dmg":12,"reach":110}}` (campos de `BFC.MOVES`) |
| `body` | caixa de colisão `{ "w": 70, "h": 230 }` |
| `rig`, `sheet` | animação antiga (recortes / sprite sheet), usada para o que não tiver frames |

## Rodar o jogo
- **Recomendado:** `iniciar.bat` (Windows) ou `./iniciar.sh` (Mac/Linux) abre o jogo por um servidor local (`http://localhost:8000`).
  O navegador só deixa o jogo **ler os pixels** dos frames (para tirar o verde) por servidor. Editou um frame ou o JSON? Basta recarregar a página.
- **Duplo clique no index.html** também abre, usando `chars/bundle.js` (rode `python3 tools/atualizar_chars.py` depois de editar um
  `character.json`), mas os **frames com fundo verde não são processados** nesse modo (aparece um aviso e o jogo usa a animação antiga).
- `python3 tools/atualizar_chars.py` também mostra quantos frames/efeitos foram encontrados em cada pasta e avisa de buracos na sequência.

## Exemplo
`chars/_exemplo/` tem frames de teste (feitos com uma única pose, só para mostrar a estrutura, os efeitos e o KO zonzo).
Para vê-lo no jogo: acrescente `"_exemplo"` em `chars/chars.json` e recarregue.

## Estendendo o sistema
- **Nova animação** (ex.: `taunt`): crie `taunt_01.png...`; em `BFC.ANIM_DEFAULTS` (ou no `animations` do JSON) defina fps/mode; em
  `BFC.FRAME_USE` ligue-a a um estado (`'state:taunt': ['taunt','idle']`); crie o estado em `Fighter.freeMove`.
- **Novo golpe** (ex.: `dash`): `BFC.MOVES.dash` em `config.js`, o botão em `Fighter.freeMove`, `'attack:dash'` em `FRAME_USE`,
  e `fx_dash_*.png` + entrada em `BFC.FX_DEFAULTS`.
- **Novo efeito**: `fx_<nome>_01.png` + entrada em `BFC.FX_DEFAULTS` e uma chamada `effects.spawn(fighter, '<nome>')` onde ele deve aparecer.
- Cada peça é uma classe pequena: `CharLoader` (ler pastas/JSON), `ChromaKey` (tirar o verde), `FrameAnimator` (escolher o quadro),
  `EffectSystem` (efeitos), `SpriteAnimator` (ordem de preferência de desenho).
