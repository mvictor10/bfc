CONFIGURACAO DO PORT RUST DO BRAZIL FIGHT COMBAT

engine.ini  - gravidade, salto, chão, tempo, rounds e dificuldade inicial.
moves.ini   - startup, active, recovery, damage, reach, y_offset, stun e knockback.
stages.ini  - id/nome/caminho de imagem/música/altura do chão para os estágios.

O elenco e os personagens são lidos de chars/chars.json e chars/<id>/character.json.
Os valores numéricos são limitados na leitura para impedir parâmetros absurdos.
Se um valor opcional não existir, o motor utiliza valores-padrão seguros.
