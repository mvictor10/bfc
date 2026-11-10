# Arquitetura — uma classe, uma responsabilidade
Tudo vive no namespace global `BFC` (scripts clássicos, funciona abrindo o index.html direto). A ordem dos `<script>` no index.html importa.

| Classe | Arquivo | Responsabilidade |
|---|---|---|
| Game | js/Game.js | loop 60 ticks/s, cena atual, serviços compartilhados |
| Settings | core/Settings.js | configurações + localStorage |
| Input | core/Input.js | teclado (down / hit) |
| AudioManager | core/AudioManager.js | SFX e músicas |
| AssetLoader | core/AssetLoader.js | carrega imagens |
| Menu, HUD | ui/ | lista navegável; barras/timer/combo |
| Fighter | entities/Fighter.js | física + estados + hitbox/hurtbox |
| SpriteAnimator | entities/SpriteAnimator.js | ordem de preferência: frames > sheet > rig > arte única |
| FrameAnimator | entities/FrameAnimator.js | escolhe o quadro dos frames soltos (loop/once/progress/vy) |
| EffectSystem | systems/EffectSystem.js | efeitos com sprites do personagem e o zonzo do KO |
| CharLoader / ChromaKey | core/ | lê chars/, monta os personagens, carrega frames e remove o verde |
| RigAnimator | entities/RigAnimator.js | anima a arte única recortada em cabeça/tronco/pernas (correr, pular, agachar, soco, chute, especial, dano, KO) + rastro e aura |
| CombatSystem | systems/ | dano, bloqueio, combo |
| ParticleSystem | systems/ | faíscas |
| Human/Dummy/AIController | ai/Controllers.js | quem "aperta os botões" (mesma interface `act()`) |
| MatchRules | systems/MatchRules.js | escolhe stage e música de cada luta |
| ArcadeMode | modes/ | sequência de lutas |
| Scene (base), Menu/Select/StageSelect/Fight/Trainer/Settings/Message | scenes/ | telas |

## Como atualizar
- **Novo personagem**: coloque `cNN.png` e `cNN_face.png` em assets/sprites/characters e adicione uma linha em `js/data/characters.js` (e ajuste o loop de `AssetLoader`/`SelectScene` se passar de 18: grade 6x3).
- **Nome/atributos**: `js/data/characters.js` (os nomes são provisórios — troque pelos reais).
- **Novo golpe**: `BFC.MOVES` em `js/data/config.js` + tecla em `BFC.KEYS` + estado em `Fighter.update` + quadro em `SpriteAnimator`.
- **Sprite sheet para outro personagem**: gere o atlas (`tools/gerar_assets.py` mostra o formato: nome da animação → lista de {x,y,w,h,ax,ay}) e generalize `SpriteAnimator.atlas` para ler o atlas de `d.atlas` em vez de `BFC.AMANDA_ATLAS`.
- **Novo cenário**: `assets/images/stages/stageN.png` (960x540) e ajuste o sorteio em `FightScene`.
- **Novo modo**: crie uma classe em `modes/` e um item em `MenuScene`.
- **Sons/músicas**: troque os .wav (mesmos nomes) ou regenere com `tools/gerar_assets.py`.

## Animação por rig (todos os personagens)
`js/data/rig.js` guarda, por personagem, pescoço (`neck`), quadril (`hip`), divisão das pernas (`split`), `flip` (arte olhando p/ esquerda) e `skirt` (vestido/túnica). Se uma parte aparecer cortada ou torta, ajuste esses números. Os parâmetros de cada pose estão em `RigAnimator.pose()`. Para animação quadro a quadro de verdade, gere um atlas e use o caminho da Amanda.

> Guia de edição atualizado (personagens, stages, músicas, vozes): `docs/GUIA_EDICAO.md`. Sheets e rigs agora são ligados pelo nome da imagem (ex.: `c07`), não pelo número.

> **Atualização:** personagens agora vivem em `chars/<pasta>/` (veja `docs/PERSONAGENS.md`). `js/data/characters.js`, `rig.js` e `sheets.js` foram substituídos pelo `character.json` de cada pasta.
