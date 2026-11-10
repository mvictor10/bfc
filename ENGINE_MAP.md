# Mapa de arquitetura — Brazil Fight Combat Rust

```text
Win32/GDI + WinMM
   |  teclado normalizado / relógio / apresentação / áudio
   v
Game (cenas, modos e campanha)
   |-- HumanController / AiController (trait Controller)
   |-- Fighter x 2
   |    |-- FighterState (enum de estado)
   |    |-- física, movimento, gravidade e timers
   |    |-- body_rect / attack_rect
   |-- CombatSystem
   |    |-- hitbox x hurtbox
   |    |-- defesa frontal / dano / stun / knockback / combo
   |-- SpriteSet x 2 (PNG/JPEG/BMP + frames + rig)
   |-- cenário selecionado (960 x 540)
   v
Canvas por software -> HUD / personagens / FX / partículas -> GDI
```

## Um tick da luta

1. O backend lê o teclado e constrói `FrameInput`.
2. `Game` seleciona o controlador humano ou CPU conforme o modo atual.
3. Cada `Fighter` atualiza entrada, estado, timers e física.
4. `CombatSystem` resolve **primeiro o ataque de P1 contra P2 e depois P2 contra P1**. Assim, um primeiro acerto pode mudar o estado do oponente para `Hit` e cancelar o ataque dele ainda naquele tick, reproduzindo a ordem sequencial da cena JavaScript original.
5. O resolver aplica dano e stun em golpe comum; ao bloquear de frente, mantém o estado de defesa, reduz o dano e reduz o empurrão.
6. Os eventos gerados ativam FX e partículas, e são enviados ao sistema de áudio.
7. O renderizador desenha o cenário atual, os rigs/quadros, o HUD, os efeitos e as hitboxes quando a depuração está ligada.
8. O backend apresenta o framebuffer e limita a apresentação a aproximadamente 60 FPS.

## Módulos

- `config.rs`: INI, `chars/chars.json`, `character.json`, atributos, rig, overrides de golpes e estágios.
- `fighter.rs`: entidade de combate, estados, movimento e caixas de colisão.
- `combat.rs`: resolução determinística de impactos e eventos.
- `controller.rs` / `ai.rs`: Strategy para entrada humana ou CPU.
- `animation.rs`: decodificação de imagens, frames por prefixo, chroma key e FX compartilhados.
- `game.rs`: cenas/menu/configuração/luta/história, Versus/CPU/Treino/Arcade e rounds.
- `render.rs`: framebuffer por software, rig, sprites, efeitos, texto, HUD e debug.
- `platform/win32.rs`: janela, teclado, apresentação, busca da raiz e temporização.
- `platform/audio.rs`: música, SFX e vozes por APIs nativas Windows.

## Fontes de dados

`chars/chars.json` é a fonte de verdade da ordem do elenco. A configuração de cada lutador fica em `chars/<id>/character.json`. Cenários e golpes globais são materializados em `data/stages.ini` e `data/moves.ini` para serem editáveis sem alterar o fonte. Os assets registrados foram copiados do ZIP do Brazil Fight Combat original; a ausência de recurso opcional resulta em fallback quando implementado.

## Fronteira de dependência

O domínio de combate não utiliza HWND/HDC ou códigos de tecla Win32. Somente `platform/` depende da plataforma. Essa fronteira permite testar geometria, estados e colisões independentemente da janela, e facilita substituir o renderizador no futuro.
