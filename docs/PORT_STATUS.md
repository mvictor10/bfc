# Estado do port Rust

## Recursos incluídos e convertidos

- Elenco registrado original: 21 IDs em `chars/chars.json`. Modelos adicionais `_modelo`/`_exemplo` e materiais não registrados `agente-lya`, `prof-jeffy` e `robsom` também estão incluídos sem entrar automaticamente na seleção.
- Configurações JSON dos personagens, atributos, corpo, rig e sobrescrita de golpes.
- 26 cenários e suas referências de imagens em `data/stages.ini`.
- Quadros individuais PNG/JPG/JPEG disponíveis; arte de seleção/corpo inteiro e rig como fallback.
- Efeitos de impacto partilhados, partículas e visualização de hitboxes.
- Modos Versus, CPU, Treinamento, campanha Arcade e IA-vs-IA; vitória, rounds e cronômetro.
- Leitura de histórias de finais e navegação por páginas.
- Música MP3/WAV, efeitos WAV e fala de vitória gravada ou sintetizada quando há texto configurado e o Windows oferece System.Speech.
- Scripts de build, testes Cargo e dashboard BFC Studio nativo em Rust/Win32.

## Limitações conhecidas

- O port ainda não reproduz toda a interface HTML5 (grade visual completa de seleção, várias telas de loading e todos os detalhes de layout).
- A animação via rig foi portada como aproximação; o formato de atlas legado `sheet.png`/`atlas.json` não é decodificado diretamente.
- A síntese de texto usa o fallback System.Speech do Windows, acionado por PowerShell. Disponibilidade depende da instalação local do Windows.
- Dificuldade, tempo, rounds e quantidade de adversários são persistidos em `data/engine.ini`; controles de volume individuais não foram portados.
- Não foi possível validar compilação/execução real aqui, pois não há toolchain `rustc`/`cargo` disponível no ambiente de preparação.

## Verificações executadas nesta preparação

- Auditoria estática anterior ao dashboard registrou 21 personagens, 26 cenários, 65 quadros em sequências configuradas e nenhuma referência obrigatória ausente. O dashboard inclui auditoria ampliada de JSON, artes, vozes, trilhas e cenários, a ser executada no Windows após compilar o binário.
- Auditoria estática atual: 21 personagens registrados, 26 estágios, seis trilhas catalogadas e 71 arquivos PNG com nomes de quadros reconhecidos nas pastas do elenco registrado.
- Validação JSON: 29 arquivos JSON analisados, zero erros de sintaxe.
- Validação de imagens via Pillow: 260 imagens verificadas, zero erros de decodificação.
- Integridade de assets: zero caminhos ausentes para JSON, artes/face dos personagens registrados, imagens de cenários e arquivos listados na biblioteca de músicas.
- Validação do ZIP de entrega: a integridade precisa ser confirmada após o empacotamento nesta sessão.

Essas verificações confirmam arquivos e referências de assets, não substituem `cargo build --release`, `cargo test` ou o teste visual no Windows.
