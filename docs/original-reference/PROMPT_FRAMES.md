# Prompt para gerar os frames (um PNG por quadro)

Gere **uma animação por vez**, **um quadro por imagem**, sempre com o **mesmo tamanho** (ex.: 600x1067, 9:16) e fundo
**verde neon #00FF00 liso**. Nomeie `idle_01.png`, `idle_02.png`, ... e coloque em `chars/<personagem>/`.
Dica: gere `idle_01` primeiro e anexe-o como referência em todos os outros, para manter o personagem consistente.

## Prompt (troque {AÇÃO} e {QUADRO})
> Usando a personagem da imagem anexada (mantenha rosto, cabelo, roupa e proporções idênticos), crie **um único frame** de
> animação de jogo de luta 2D lateral, estilo ilustração HD inspirada nos jogos de luta 2D do Super Nintendo (Mortal Kombat).
> Ação: **{AÇÃO}**, frame **{QUADRO} de 6**. Imagem 600x1067 px (9:16), fundo verde neon #00FF00 liso, sem sombra no chão, sem texto.
> Personagem de perfil, olhando para a direita, corpo inteiro, centralizado, pés a 92% da altura da imagem e altura total do
> corpo de cerca de 85% da imagem; nenhum membro encosta na borda.

## O que desenhar em cada quadro
| Ação | 1 | 2 | 3 | 4 | 5 | 6 |
|---|---|---|---|---|---|---|
| idle (loop) | guarda neutra | peito sobe | ponto alto | começa a descer | quase neutra | volta ao 1 |
| attack (soco) | guarda | recua o punho | acelera | **extensão total (impacto)** | recolhe | guarda |
| kick | guarda | joelho levantado | perna estica | **chute estendido (impacto)** | recolhe | guarda |
| jump | agacha | decola | sobe, joelhos recolhidos | **topo** | cai, pernas estendem | aterrissa |
| crouch | guarda | desce | **agachado fundo (mantido)** | | | |
| defense | guarda | levanta os braços | **braços cruzados na frente do rosto (mantido)** | | | |
| walk (loop) | contato | desce | passagem | sobe | contato do outro pé | passagem |
| run (loop) | corpo inclinado, contato | impulso | voo | contato do outro pé | impulso | voo |
Opcional: `ko` (agachado, cabeça baixa), `hit` (apanhando), `special`.
Efeitos: `fx_punch`, `fx_kick`, `fx_hit`, `fx_block`, `fx_dizzy` (estrelinhas girando, 4–6 quadros em loop), 3–6 quadros cada, também em verde.
Mais ou menos quadros são aceitos (4 a 8 por animação funciona bem).
